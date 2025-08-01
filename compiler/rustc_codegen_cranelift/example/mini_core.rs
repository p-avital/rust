#![feature(
    no_core,
    lang_items,
    intrinsics,
    unboxed_closures,
    extern_types,
    decl_macro,
    rustc_attrs,
    transparent_unions,
    auto_traits,
    freeze_impls,
    thread_local
)]
#![no_core]
#![allow(dead_code, internal_features, ambiguous_wide_pointer_comparisons)]

#[lang = "pointee_sized"]
pub trait PointeeSized {}

#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}

#[lang = "sized"]
pub trait Sized: MetaSized {}

#[lang = "destruct"]
pub trait Destruct {}

#[lang = "tuple_trait"]
pub trait Tuple {}

#[lang = "unsize"]
pub trait Unsize<T: PointeeSized>: PointeeSized {}

#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T> {}

impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a mut U> for &'a mut T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<*const U> for *const T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<*mut U> for *mut T {}

#[lang = "dispatch_from_dyn"]
pub trait DispatchFromDyn<T> {}

// &T -> &U
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}
// &mut T -> &mut U
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a mut U> for &'a mut T {}
// *const T -> *const U
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<*const U> for *const T {}
// *mut T -> *mut U
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<*mut U> for *mut T {}
impl<T: MetaSized + Unsize<U>, U: MetaSized> DispatchFromDyn<Box<U>> for Box<T> {}

#[lang = "legacy_receiver"]
pub trait LegacyReceiver {}

impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
impl<T: MetaSized> LegacyReceiver for Box<T> {}

#[lang = "copy"]
pub trait Copy {}

#[lang = "bikeshed_guaranteed_no_drop"]
pub trait BikeshedGuaranteedNoDrop {}

impl Copy for bool {}
impl Copy for u8 {}
impl Copy for u16 {}
impl Copy for u32 {}
impl Copy for u64 {}
impl Copy for u128 {}
impl Copy for usize {}
impl Copy for i8 {}
impl Copy for i16 {}
impl Copy for i32 {}
impl Copy for isize {}
impl Copy for f32 {}
impl Copy for f64 {}
impl Copy for char {}
impl<'a, T: PointeeSized> Copy for &'a T {}
impl<T: PointeeSized> Copy for *const T {}
impl<T: PointeeSized> Copy for *mut T {}
impl<T: Copy> Copy for Option<T> {}

#[lang = "sync"]
pub unsafe trait Sync {}

unsafe impl Sync for bool {}
unsafe impl Sync for u8 {}
unsafe impl Sync for u16 {}
unsafe impl Sync for u32 {}
unsafe impl Sync for u64 {}
unsafe impl Sync for usize {}
unsafe impl Sync for i8 {}
unsafe impl Sync for i16 {}
unsafe impl Sync for i32 {}
unsafe impl Sync for isize {}
unsafe impl Sync for char {}
unsafe impl Sync for f32 {}
unsafe impl<'a, T: PointeeSized> Sync for &'a T {}
unsafe impl<T: Sync, const N: usize> Sync for [T; N] {}

#[lang = "freeze"]
unsafe auto trait Freeze {}

unsafe impl<T: PointeeSized> Freeze for PhantomData<T> {}
unsafe impl<T: PointeeSized> Freeze for *const T {}
unsafe impl<T: PointeeSized> Freeze for *mut T {}
unsafe impl<T: PointeeSized> Freeze for &T {}
unsafe impl<T: PointeeSized> Freeze for &mut T {}

#[lang = "structural_peq"]
pub trait StructuralPartialEq {}

#[lang = "not"]
pub trait Not {
    type Output;

    fn not(self) -> Self::Output;
}

impl Not for bool {
    type Output = bool;

    fn not(self) -> bool {
        !self
    }
}

#[lang = "mul"]
pub trait Mul<RHS = Self> {
    type Output;

    #[must_use]
    fn mul(self, rhs: RHS) -> Self::Output;
}

impl Mul for u8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self * rhs
    }
}

impl Mul for usize {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self * rhs
    }
}

#[lang = "add"]
pub trait Add<RHS = Self> {
    type Output;

    fn add(self, rhs: RHS) -> Self::Output;
}

impl Add for u8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl Add for i8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}

impl Add for usize {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self + rhs
    }
}

#[lang = "sub"]
pub trait Sub<RHS = Self> {
    type Output;

    fn sub(self, rhs: RHS) -> Self::Output;
}

impl Sub for usize {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
}

impl Sub for u8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
}

impl Sub for i8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
}

impl Sub for i16 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self - rhs
    }
}

#[lang = "rem"]
pub trait Rem<RHS = Self> {
    type Output;

    fn rem(self, rhs: RHS) -> Self::Output;
}

impl Rem for usize {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        self % rhs
    }
}

#[lang = "bitor"]
pub trait BitOr<RHS = Self> {
    type Output;

    #[must_use]
    fn bitor(self, rhs: RHS) -> Self::Output;
}

impl BitOr for bool {
    type Output = bool;

    fn bitor(self, rhs: bool) -> bool {
        self | rhs
    }
}

impl<'a> BitOr<bool> for &'a bool {
    type Output = bool;

    fn bitor(self, rhs: bool) -> bool {
        *self | rhs
    }
}

#[lang = "eq"]
pub trait PartialEq<Rhs: ?Sized = Self> {
    fn eq(&self, other: &Rhs) -> bool;
    fn ne(&self, other: &Rhs) -> bool;
}

impl PartialEq for u8 {
    fn eq(&self, other: &u8) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &u8) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for u16 {
    fn eq(&self, other: &u16) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &u16) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for u32 {
    fn eq(&self, other: &u32) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &u32) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for u64 {
    fn eq(&self, other: &u64) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &u64) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for u128 {
    fn eq(&self, other: &u128) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &u128) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for usize {
    fn eq(&self, other: &usize) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &usize) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for i8 {
    fn eq(&self, other: &i8) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &i8) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for i32 {
    fn eq(&self, other: &i32) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &i32) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for isize {
    fn eq(&self, other: &isize) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &isize) -> bool {
        (*self) != (*other)
    }
}

impl PartialEq for char {
    fn eq(&self, other: &char) -> bool {
        (*self) == (*other)
    }
    fn ne(&self, other: &char) -> bool {
        (*self) != (*other)
    }
}

impl<T: ?Sized> PartialEq for *const T {
    fn eq(&self, other: &*const T) -> bool {
        *self == *other
    }
    fn ne(&self, other: &*const T) -> bool {
        *self != *other
    }
}

impl<T: PartialEq> PartialEq for Option<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(lhs), Some(rhs)) => *lhs == *rhs,
            (None, None) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(lhs), Some(rhs)) => *lhs != *rhs,
            (None, None) => false,
            _ => true,
        }
    }
}

#[lang = "shl"]
pub trait Shl<RHS = Self> {
    type Output;

    #[must_use]
    fn shl(self, rhs: RHS) -> Self::Output;
}

impl Shl for u128 {
    type Output = u128;

    fn shl(self, rhs: u128) -> u128 {
        self << rhs
    }
}

#[lang = "neg"]
pub trait Neg {
    type Output;

    fn neg(self) -> Self::Output;
}

impl Neg for i8 {
    type Output = i8;

    fn neg(self) -> i8 {
        -self
    }
}

impl Neg for i16 {
    type Output = i16;

    fn neg(self) -> i16 {
        self
    }
}

impl Neg for isize {
    type Output = isize;

    fn neg(self) -> isize {
        -self
    }
}

impl Neg for f32 {
    type Output = f32;

    fn neg(self) -> f32 {
        -self
    }
}

pub enum Option<T> {
    Some(T),
    None,
}

pub use Option::*;

#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;

#[lang = "fn_once"]
#[rustc_paren_sugar]
pub trait FnOnce<Args: Tuple> {
    #[lang = "fn_once_output"]
    type Output;

    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}

#[lang = "fn_mut"]
#[rustc_paren_sugar]
pub trait FnMut<Args: Tuple>: FnOnce<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}

#[lang = "panic"]
#[track_caller]
pub fn panic(_msg: &'static str) -> ! {
    unsafe {
        libc::puts("Panicking\n\0" as *const str as *const i8);
        intrinsics::abort();
    }
}

macro_rules! panic_const {
    ($($lang:ident = $message:expr,)+) => {
        pub mod panic_const {
            use super::*;

            $(
                #[track_caller]
                #[lang = stringify!($lang)]
                pub fn $lang() -> ! {
                    panic($message);
                }
            )+
        }
    }
}

panic_const! {
    panic_const_add_overflow = "attempt to add with overflow",
    panic_const_sub_overflow = "attempt to subtract with overflow",
    panic_const_mul_overflow = "attempt to multiply with overflow",
    panic_const_div_overflow = "attempt to divide with overflow",
    panic_const_rem_overflow = "attempt to calculate the remainder with overflow",
    panic_const_neg_overflow = "attempt to negate with overflow",
    panic_const_shr_overflow = "attempt to shift right with overflow",
    panic_const_shl_overflow = "attempt to shift left with overflow",
    panic_const_div_by_zero = "attempt to divide by zero",
    panic_const_rem_by_zero = "attempt to calculate the remainder with a divisor of zero",
}

#[lang = "panic_bounds_check"]
#[track_caller]
fn panic_bounds_check(index: usize, len: usize) -> ! {
    unsafe {
        libc::printf(
            "index out of bounds: the len is %d but the index is %d\n\0" as *const str as *const i8,
            len,
            index,
        );
        intrinsics::abort();
    }
}

#[lang = "panic_cannot_unwind"]
#[track_caller]
fn panic_cannot_unwind() -> ! {
    unsafe {
        libc::puts("panic in a function that cannot unwind\n\0" as *const str as *const i8);
        intrinsics::abort();
    }
}

#[lang = "eh_personality"]
// FIXME personality signature depends on target
fn eh_personality(
    _version: i32,
    _actions: i32,
    _exception_class: u64,
    _exception_object: *mut (),
    _context: *mut (),
) -> i32 {
    loop {}
}

#[lang = "panic_in_cleanup"]
fn panic_in_cleanup() -> ! {
    loop {}
}

#[cfg(all(unix, not(target_vendor = "apple")))]
#[link(name = "gcc_s")]
extern "C" {
    fn _Unwind_Resume(exc: *mut ()) -> !;
}

#[lang = "drop_in_place"]
#[allow(unconditional_recursion)]
pub unsafe fn drop_in_place<T: ?Sized>(to_drop: *mut T) {
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
    drop_in_place(to_drop);
}

#[lang = "unpin"]
pub auto trait Unpin {}

#[lang = "deref"]
pub trait Deref {
    type Target: ?Sized;

    fn deref(&self) -> &Self::Target;
}

#[repr(transparent)]
#[rustc_layout_scalar_valid_range_start(1)]
#[rustc_nonnull_optimization_guaranteed]
pub struct NonNull<T: PointeeSized>(pub *const T);

impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<NonNull<U>> for NonNull<T> where T: Unsize<U> {}
impl<T: PointeeSized, U: PointeeSized> DispatchFromDyn<NonNull<U>> for NonNull<T> where T: Unsize<U> {}

pub struct Unique<T: PointeeSized> {
    pub pointer: NonNull<T>,
    pub _marker: PhantomData<T>,
}

impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<Unique<U>> for Unique<T> where T: Unsize<U> {}
impl<T: PointeeSized, U: PointeeSized> DispatchFromDyn<Unique<U>> for Unique<T> where T: Unsize<U> {}

#[lang = "global_alloc_ty"]
pub struct Global;

#[lang = "owned_box"]
pub struct Box<T: ?Sized, A = Global>(Unique<T>, A);

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Box<U>> for Box<T> {}

impl<T> Box<T> {
    pub fn new(val: T) -> Box<T> {
        unsafe {
            let size = intrinsics::size_of::<T>();
            let ptr = libc::malloc(size);
            intrinsics::copy(&val as *const T as *const u8, ptr, size);
            Box(Unique { pointer: NonNull(ptr as *const T), _marker: PhantomData }, Global)
        }
    }
}

impl<T: ?Sized, A> Drop for Box<T, A> {
    fn drop(&mut self) {
        // inner value is dropped by compiler
        unsafe {
            libc::free(self.0.pointer.0 as *mut u8);
        }
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &**self
    }
}

#[lang = "exchange_malloc"]
unsafe fn allocate(size: usize, _align: usize) -> *mut u8 {
    libc::malloc(size)
}

#[lang = "drop"]
pub trait Drop {
    fn drop(&mut self);
}

#[lang = "manually_drop"]
#[repr(transparent)]
pub struct ManuallyDrop<T: ?Sized> {
    pub value: T,
}

#[lang = "maybe_uninit"]
#[repr(transparent)]
pub union MaybeUninit<T> {
    pub uninit: (),
    pub value: ManuallyDrop<T>,
}

pub mod intrinsics {
    #[rustc_intrinsic]
    pub fn abort() -> !;
    #[rustc_intrinsic]
    pub fn size_of<T>() -> usize;
    #[rustc_intrinsic]
    pub unsafe fn size_of_val<T: ?::Sized>(val: *const T) -> usize;
    #[rustc_intrinsic]
    pub fn align_of<T>() -> usize;
    #[rustc_intrinsic]
    pub unsafe fn align_of_val<T: ?::Sized>(val: *const T) -> usize;
    #[rustc_intrinsic]
    pub unsafe fn copy<T>(src: *const T, dst: *mut T, count: usize);
    #[rustc_intrinsic]
    pub unsafe fn transmute<T, U>(e: T) -> U;
    #[rustc_intrinsic]
    pub unsafe fn ctlz_nonzero<T>(x: T) -> u32;
    #[rustc_intrinsic]
    pub const fn needs_drop<T: ?::Sized>() -> bool;
    #[rustc_intrinsic]
    pub fn bitreverse<T>(x: T) -> T;
    #[rustc_intrinsic]
    pub fn bswap<T>(x: T) -> T;
    #[rustc_intrinsic]
    pub unsafe fn write_bytes<T>(dst: *mut T, val: u8, count: usize);
    #[rustc_intrinsic]
    pub unsafe fn unreachable() -> !;
}

pub mod libc {
    // With the new Universal CRT, msvc has switched to all the printf functions being inline wrapper
    // functions. legacy_stdio_definitions.lib which provides the printf wrapper functions as normal
    // symbols to link against.
    #[cfg_attr(unix, link(name = "c"))]
    #[cfg_attr(target_env = "msvc", link(name = "legacy_stdio_definitions"))]
    extern "C" {
        pub fn printf(format: *const i8, ...) -> i32;
    }

    #[cfg_attr(unix, link(name = "c"))]
    #[cfg_attr(target_env = "msvc", link(name = "msvcrt"))]
    extern "C" {
        pub fn puts(s: *const i8) -> i32;
        pub fn malloc(size: usize) -> *mut u8;
        pub fn free(ptr: *mut u8);
        pub fn memcpy(dst: *mut u8, src: *const u8, size: usize);
        pub fn memmove(dst: *mut u8, src: *const u8, size: usize);
        pub fn strncpy(dst: *mut u8, src: *const u8, size: usize);
    }
}

#[lang = "index"]
pub trait Index<Idx: ?Sized> {
    type Output: ?Sized;
    fn index(&self, index: Idx) -> &Self::Output;
}

impl<T> Index<usize> for [T; 3] {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self[index]
    }
}

impl<T> Index<usize> for [T] {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self[index]
    }
}

extern "C" {
    type VaListImpl;
}

#[lang = "va_list"]
#[repr(transparent)]
pub struct VaList<'a>(&'a mut VaListImpl);

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semitransparent"]
pub macro stringify($($t:tt)*) {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semitransparent"]
pub macro file() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semitransparent"]
pub macro line() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semitransparent"]
pub macro cfg() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semitransparent"]
pub macro asm() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semitransparent"]
pub macro global_asm() {
    /* compiler built-in */
}

#[rustc_builtin_macro]
#[rustc_macro_transparency = "semitransparent"]
pub macro naked_asm() {
    /* compiler built-in */
}

pub static A_STATIC: u8 = 42;

#[lang = "panic_location"]
struct PanicLocation {
    file: &'static str,
    line: u32,
    column: u32,
}

#[no_mangle]
#[cfg(not(all(windows, target_env = "gnu")))]
pub fn get_tls() -> u8 {
    #[thread_local]
    static A: u8 = 42;

    A
}
