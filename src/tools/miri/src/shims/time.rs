use std::ffi::{OsStr, OsString};
use std::fmt::Write;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Datelike, Offset, Timelike, Utc};
use chrono_tz::Tz;

use crate::*;

/// Returns the time elapsed between the provided time and the unix epoch as a `Duration`.
pub fn system_time_to_duration<'tcx>(time: &SystemTime) -> InterpResult<'tcx, Duration> {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| err_unsup_format!("times before the Unix epoch are not supported"))
        .into()
}

impl<'tcx> EvalContextExt<'tcx> for crate::MiriInterpCx<'tcx> {}
pub trait EvalContextExt<'tcx>: crate::MiriInterpCxExt<'tcx> {
    fn parse_clockid(&self, clk_id: Scalar) -> Option<TimeoutClock> {
        // This clock support is deliberately minimal because a lot of clock types have fiddly
        // properties (is it possible for Miri to be suspended independently of the host?). If you
        // have a use for another clock type, please open an issue.
        let this = self.eval_context_ref();

        // Portable names that exist everywhere.
        if clk_id == this.eval_libc("CLOCK_REALTIME") {
            return Some(TimeoutClock::RealTime);
        } else if clk_id == this.eval_libc("CLOCK_MONOTONIC") {
            return Some(TimeoutClock::Monotonic);
        }

        // Some further platform-specific names we support.
        match this.tcx.sess.target.os.as_ref() {
            "linux" | "freebsd" | "android" => {
                // Linux further distinguishes regular and "coarse" clocks, but the "coarse" version
                // is just specified to be "faster and less precise", so we treat it like normal
                // clocks.
                if clk_id == this.eval_libc("CLOCK_REALTIME_COARSE") {
                    return Some(TimeoutClock::RealTime);
                } else if clk_id == this.eval_libc("CLOCK_MONOTONIC_COARSE") {
                    return Some(TimeoutClock::Monotonic);
                }
            }
            "macos" => {
                // `CLOCK_UPTIME_RAW` supposed to not increment while the system is asleep... but
                // that's not really something a program running inside Miri can tell, anyway.
                // We need to support it because std uses it.
                if clk_id == this.eval_libc("CLOCK_UPTIME_RAW") {
                    return Some(TimeoutClock::Monotonic);
                }
            }
            _ => {}
        }

        None
    }

    fn clock_gettime(
        &mut self,
        clk_id_op: &OpTy<'tcx>,
        tp_op: &OpTy<'tcx>,
        dest: &MPlaceTy<'tcx>,
    ) -> InterpResult<'tcx> {
        let this = self.eval_context_mut();

        this.assert_target_os_is_unix("clock_gettime");

        let clk_id = this.read_scalar(clk_id_op)?;
        let tp = this.deref_pointer_as(tp_op, this.libc_ty_layout("timespec"))?;

        let duration = match this.parse_clockid(clk_id) {
            Some(TimeoutClock::RealTime) => {
                this.check_no_isolation("`clock_gettime` with `REALTIME` clocks")?;
                system_time_to_duration(&SystemTime::now())?
            }
            Some(TimeoutClock::Monotonic) =>
                this.machine
                    .monotonic_clock
                    .now()
                    .duration_since(this.machine.monotonic_clock.epoch()),
            None => {
                return this.set_last_error_and_return(LibcError("EINVAL"), dest);
            }
        };

        let tv_sec = duration.as_secs();
        let tv_nsec = duration.subsec_nanos();

        this.write_int_fields(&[tv_sec.into(), tv_nsec.into()], &tp)?;
        this.write_int(0, dest)?;

        interp_ok(())
    }

    fn gettimeofday(
        &mut self,
        tv_op: &OpTy<'tcx>,
        tz_op: &OpTy<'tcx>,
    ) -> InterpResult<'tcx, Scalar> {
        let this = self.eval_context_mut();

        this.assert_target_os_is_unix("gettimeofday");
        this.check_no_isolation("`gettimeofday`")?;

        let tv = this.deref_pointer_as(tv_op, this.libc_ty_layout("timeval"))?;

        // Using tz is obsolete and should always be null
        let tz = this.read_pointer(tz_op)?;
        if !this.ptr_is_null(tz)? {
            return this.set_last_error_and_return_i32(LibcError("EINVAL"));
        }

        let duration = system_time_to_duration(&SystemTime::now())?;
        let tv_sec = duration.as_secs();
        let tv_usec = duration.subsec_micros();

        this.write_int_fields(&[tv_sec.into(), tv_usec.into()], &tv)?;

        interp_ok(Scalar::from_i32(0))
    }

    // The localtime() function shall convert the time in seconds since the Epoch pointed to by
    // timer into a broken-down time, expressed as a local time.
    // https://linux.die.net/man/3/localtime_r
    fn localtime_r(
        &mut self,
        timep: &OpTy<'tcx>,
        result_op: &OpTy<'tcx>,
    ) -> InterpResult<'tcx, Pointer> {
        let this = self.eval_context_mut();

        this.assert_target_os_is_unix("localtime_r");
        this.check_no_isolation("`localtime_r`")?;

        let time_layout = this.libc_ty_layout("time_t");
        let timep = this.deref_pointer_as(timep, time_layout)?;
        let result = this.deref_pointer_as(result_op, this.libc_ty_layout("tm"))?;

        // The input "represents the number of seconds elapsed since the Epoch,
        // 1970-01-01 00:00:00 +0000 (UTC)".
        let sec_since_epoch: i64 =
            this.read_scalar(&timep)?.to_int(time_layout.size)?.try_into().unwrap();
        let dt_utc: DateTime<Utc> =
            DateTime::from_timestamp(sec_since_epoch, 0).expect("Invalid timestamp");

        // Figure out what time zone is in use
        let tz = this.get_env_var(OsStr::new("TZ"))?.unwrap_or_else(|| OsString::from("UTC"));
        let tz = match tz.into_string() {
            Ok(tz) => Tz::from_str(&tz).unwrap_or(Tz::UTC),
            _ => Tz::UTC,
        };

        // Convert that to local time, then return the broken-down time value.
        let dt: DateTime<Tz> = dt_utc.with_timezone(&tz);

        // This value is always set to -1, because there is no way to know if dst is in effect with
        // chrono crate yet.
        // This may not be consistent with libc::localtime_r's result.
        let tm_isdst = -1;
        this.write_int_fields_named(
            &[
                ("tm_sec", dt.second().into()),
                ("tm_min", dt.minute().into()),
                ("tm_hour", dt.hour().into()),
                ("tm_mday", dt.day().into()),
                ("tm_mon", dt.month0().into()),
                ("tm_year", dt.year().strict_sub(1900).into()),
                ("tm_wday", dt.weekday().num_days_from_sunday().into()),
                ("tm_yday", dt.ordinal0().into()),
                ("tm_isdst", tm_isdst),
            ],
            &result,
        )?;

        // solaris/illumos system tm struct does not have
        // the additional tm_zone/tm_gmtoff fields.
        // https://docs.oracle.com/cd/E36784_01/html/E36874/localtime-r-3c.html
        if !matches!(&*this.tcx.sess.target.os, "solaris" | "illumos") {
            // tm_zone represents the timezone value in the form of: +0730, +08, -0730 or -08.
            // This may not be consistent with libc::localtime_r's result.

            let offset_in_seconds = dt.offset().fix().local_minus_utc();
            let tm_gmtoff = offset_in_seconds;
            let mut tm_zone = String::new();
            if offset_in_seconds < 0 {
                tm_zone.push('-');
            } else {
                tm_zone.push('+');
            }
            let offset_hour = offset_in_seconds.abs() / 3600;
            write!(tm_zone, "{offset_hour:02}").unwrap();
            let offset_min = (offset_in_seconds.abs() % 3600) / 60;
            if offset_min != 0 {
                write!(tm_zone, "{offset_min:02}").unwrap();
            }

            // Add null terminator for C string compatibility.
            tm_zone.push('\0');

            // Deduplicate and allocate the string.
            let tm_zone_ptr = this.allocate_bytes_dedup(tm_zone.as_bytes())?;

            // Write the timezone pointer and offset into the result structure.
            this.write_pointer(tm_zone_ptr, &this.project_field_named(&result, "tm_zone")?)?;
            this.write_int_fields_named(&[("tm_gmtoff", tm_gmtoff.into())], &result)?;
        }
        interp_ok(result.ptr())
    }
    #[allow(non_snake_case, clippy::arithmetic_side_effects)]
    fn GetSystemTimeAsFileTime(
        &mut self,
        shim_name: &str,
        LPFILETIME_op: &OpTy<'tcx>,
    ) -> InterpResult<'tcx> {
        let this = self.eval_context_mut();

        this.assert_target_os("windows", shim_name);
        this.check_no_isolation(shim_name)?;

        let filetime = this.deref_pointer_as(LPFILETIME_op, this.windows_ty_layout("FILETIME"))?;

        let duration = this.system_time_since_windows_epoch(&SystemTime::now())?;
        let duration_ticks = this.windows_ticks_for(duration)?;

        let dwLowDateTime = u32::try_from(duration_ticks & 0x00000000FFFFFFFF).unwrap();
        let dwHighDateTime = u32::try_from((duration_ticks & 0xFFFFFFFF00000000) >> 32).unwrap();
        this.write_int_fields(&[dwLowDateTime.into(), dwHighDateTime.into()], &filetime)?;

        interp_ok(())
    }

    #[allow(non_snake_case)]
    fn QueryPerformanceCounter(
        &mut self,
        lpPerformanceCount_op: &OpTy<'tcx>,
    ) -> InterpResult<'tcx, Scalar> {
        let this = self.eval_context_mut();

        this.assert_target_os("windows", "QueryPerformanceCounter");

        // QueryPerformanceCounter uses a hardware counter as its basis.
        // Miri will emulate a counter with a resolution of 1 nanosecond.
        let duration =
            this.machine.monotonic_clock.now().duration_since(this.machine.monotonic_clock.epoch());
        let qpc = i64::try_from(duration.as_nanos()).map_err(|_| {
            err_unsup_format!("programs running longer than 2^63 nanoseconds are not supported")
        })?;
        this.write_scalar(
            Scalar::from_i64(qpc),
            &this.deref_pointer_as(lpPerformanceCount_op, this.machine.layouts.i64)?,
        )?;
        interp_ok(Scalar::from_i32(-1)) // return non-zero on success
    }

    #[allow(non_snake_case)]
    fn QueryPerformanceFrequency(
        &mut self,
        lpFrequency_op: &OpTy<'tcx>,
    ) -> InterpResult<'tcx, Scalar> {
        let this = self.eval_context_mut();

        this.assert_target_os("windows", "QueryPerformanceFrequency");

        // Retrieves the frequency of the hardware performance counter.
        // The frequency of the performance counter is fixed at system boot and
        // is consistent across all processors.
        // Miri emulates a "hardware" performance counter with a resolution of 1ns,
        // and thus 10^9 counts per second.
        this.write_scalar(
            Scalar::from_i64(1_000_000_000),
            &this.deref_pointer_as(lpFrequency_op, this.machine.layouts.u64)?,
        )?;
        interp_ok(Scalar::from_i32(-1)) // Return non-zero on success
    }

    #[allow(non_snake_case, clippy::arithmetic_side_effects)]
    fn system_time_since_windows_epoch(&self, time: &SystemTime) -> InterpResult<'tcx, Duration> {
        let this = self.eval_context_ref();

        let INTERVALS_PER_SEC = this.eval_windows_u64("time", "INTERVALS_PER_SEC");
        let INTERVALS_TO_UNIX_EPOCH = this.eval_windows_u64("time", "INTERVALS_TO_UNIX_EPOCH");
        let SECONDS_TO_UNIX_EPOCH = INTERVALS_TO_UNIX_EPOCH / INTERVALS_PER_SEC;

        interp_ok(system_time_to_duration(time)? + Duration::from_secs(SECONDS_TO_UNIX_EPOCH))
    }

    #[allow(non_snake_case, clippy::arithmetic_side_effects)]
    fn windows_ticks_for(&self, duration: Duration) -> InterpResult<'tcx, u64> {
        let this = self.eval_context_ref();

        let NANOS_PER_SEC = this.eval_windows_u64("time", "NANOS_PER_SEC");
        let INTERVALS_PER_SEC = this.eval_windows_u64("time", "INTERVALS_PER_SEC");
        let NANOS_PER_INTERVAL = NANOS_PER_SEC / INTERVALS_PER_SEC;

        let ticks = u64::try_from(duration.as_nanos() / u128::from(NANOS_PER_INTERVAL))
            .map_err(|_| err_unsup_format!("programs running more than 2^64 Windows ticks after the Windows epoch are not supported"))?;
        interp_ok(ticks)
    }

    fn mach_absolute_time(&self) -> InterpResult<'tcx, Scalar> {
        let this = self.eval_context_ref();

        this.assert_target_os("macos", "mach_absolute_time");

        // This returns a u64, with time units determined dynamically by `mach_timebase_info`.
        // We return plain nanoseconds.
        let duration =
            this.machine.monotonic_clock.now().duration_since(this.machine.monotonic_clock.epoch());
        let res = u64::try_from(duration.as_nanos()).map_err(|_| {
            err_unsup_format!("programs running longer than 2^64 nanoseconds are not supported")
        })?;
        interp_ok(Scalar::from_u64(res))
    }

    fn mach_timebase_info(&mut self, info_op: &OpTy<'tcx>) -> InterpResult<'tcx, Scalar> {
        let this = self.eval_context_mut();

        this.assert_target_os("macos", "mach_timebase_info");

        let info = this.deref_pointer_as(info_op, this.libc_ty_layout("mach_timebase_info"))?;

        // Since our emulated ticks in `mach_absolute_time` *are* nanoseconds,
        // no scaling needs to happen.
        let (numer, denom) = (1, 1);
        this.write_int_fields(&[numer.into(), denom.into()], &info)?;

        interp_ok(Scalar::from_i32(0)) // KERN_SUCCESS
    }

    fn nanosleep(&mut self, duration: &OpTy<'tcx>, rem: &OpTy<'tcx>) -> InterpResult<'tcx, Scalar> {
        let this = self.eval_context_mut();

        this.assert_target_os_is_unix("nanosleep");

        let duration = this.deref_pointer_as(duration, this.libc_ty_layout("timespec"))?;
        let _rem = this.read_pointer(rem)?; // Signal handlers are not supported, so rem will never be written to.

        let duration = match this.read_timespec(&duration)? {
            Some(duration) => duration,
            None => {
                return this.set_last_error_and_return_i32(LibcError("EINVAL"));
            }
        };

        this.block_thread(
            BlockReason::Sleep,
            Some((TimeoutClock::Monotonic, TimeoutAnchor::Relative, duration)),
            callback!(
                @capture<'tcx> {}
                |_this, unblock: UnblockKind| {
                    assert_eq!(unblock, UnblockKind::TimedOut);
                    interp_ok(())
                }
            ),
        );
        interp_ok(Scalar::from_i32(0))
    }

    fn clock_nanosleep(
        &mut self,
        clock_id: &OpTy<'tcx>,
        flags: &OpTy<'tcx>,
        timespec: &OpTy<'tcx>,
        rem: &OpTy<'tcx>,
    ) -> InterpResult<'tcx, Scalar> {
        let this = self.eval_context_mut();
        let clockid_t_size = this.libc_ty_layout("clockid_t").size;

        let clock_id = this.read_scalar(clock_id)?.to_int(clockid_t_size)?;
        let timespec = this.deref_pointer_as(timespec, this.libc_ty_layout("timespec"))?;
        let flags = this.read_scalar(flags)?.to_i32()?;
        let _rem = this.read_pointer(rem)?; // Signal handlers are not supported, so rem will never be written to.

        // The standard lib through sleep_until only needs CLOCK_MONOTONIC
        if clock_id != this.eval_libc("CLOCK_MONOTONIC").to_int(clockid_t_size)? {
            throw_unsup_format!("clock_nanosleep: only CLOCK_MONOTONIC is supported");
        }

        let duration = match this.read_timespec(&timespec)? {
            Some(duration) => duration,
            None => {
                return this.set_last_error_and_return_i32(LibcError("EINVAL"));
            }
        };

        let timeout_anchor = if flags == 0 {
            // No flags set, the timespec should be interperted as a duration
            // to sleep for
            TimeoutAnchor::Relative
        } else if flags == this.eval_libc_i32("TIMER_ABSTIME") {
            // Only flag TIMER_ABSTIME set, the timespec should be interperted as
            // an absolute time.
            TimeoutAnchor::Absolute
        } else {
            // The standard lib (through `sleep_until`) only needs TIMER_ABSTIME
            throw_unsup_format!(
                "`clock_nanosleep` unsupported flags {flags}, only no flags or \
                TIMER_ABSTIME is supported"
            );
        };

        this.block_thread(
            BlockReason::Sleep,
            Some((TimeoutClock::Monotonic, timeout_anchor, duration)),
            callback!(
                @capture<'tcx> {}
                |_this, unblock: UnblockKind| {
                    assert_eq!(unblock, UnblockKind::TimedOut);
                    interp_ok(())
                }
            ),
        );
        interp_ok(Scalar::from_i32(0))
    }

    #[allow(non_snake_case)]
    fn Sleep(&mut self, timeout: &OpTy<'tcx>) -> InterpResult<'tcx> {
        let this = self.eval_context_mut();

        this.assert_target_os("windows", "Sleep");

        let timeout_ms = this.read_scalar(timeout)?.to_u32()?;

        let duration = Duration::from_millis(timeout_ms.into());

        this.block_thread(
            BlockReason::Sleep,
            Some((TimeoutClock::Monotonic, TimeoutAnchor::Relative, duration)),
            callback!(
                @capture<'tcx> {}
                |_this, unblock: UnblockKind| {
                    assert_eq!(unblock, UnblockKind::TimedOut);
                    interp_ok(())
                }
            ),
        );
        interp_ok(())
    }
}
