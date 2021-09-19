use core::{
   fmt,
   mem,
   num::NonZeroUsize,
   ptr::NonNull,
};

#[inline]
pub const fn size_align<T>() -> (usize, usize)
{
   return (mem::size_of::<T>(), mem::align_of::<T>());
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg(feature="allocator")]
pub struct Layout
{
   /// Size of the requested block measured in bytes.
   size_: usize,

   // alignment of the requested block of memory, measured in bytes.
   // we ensure that this is always a power-of-two, because API's
   // like `posix_memalign` require it and it is a reasonable
   // constraint to impose on Layout constructors.
   //
   // (However, we do not analogously require `align >= sizeof(void*)`,
   //  even though that is *also* a requirement of `posix_memalign`.)
   align_: NonZeroUsize,
}

impl Layout
{
   /// Constructs a `Layout` from a given `size` and `align`,
   /// or returns `LayoutError` if any of the following conditions
   /// are not met:
   ///
   /// * `align` must not be zero,
   ///
   /// * `align` must be a power of two,
   ///
   /// * `size`, when rounded up to the nearest multiple of `align`,
   ///    must not overflow (i.e., the rounded value must be less than
   ///    or equal to `usize::MAX`).
   #[cfg(feature="allocator")]
   #[inline]
   pub const fn from_size_align(size: usize, align: usize) -> Result<Self, LayoutError>
   {
      if !align.is_power_of_two() {
         return Err(LayoutError);
      }

      // (power-of-two implies align != zero).

      // Rounded up size is:
      //   size_rounded_up = (size + align - 1) & !(align - 1);
      //
      // We know from above that align != 0. If adding (align - 1)
      // does not overflow, then rounding up will be fine.
      //
      // Conversely, &-masking with !(align - 1) will subtract off
      // only low-order-bits. Thus if overflow occurs with the sum,
      // the &-mask cannot subtract enough to undo that overflow.
      //
      // Above implies that checking for summation overflow is both
      // necessary and sufficient.
      if size > usize::MAX {
         return Err(LayoutError);
      }

      // SAFETY: the conditions for `from_size_align_unchecked` have been
      // checked above.
      unsafe { Ok(Layout::from_size_align_unchecked(size, align)) }
   }

   /// Creates a layout, bypassing all checks.
   ///
   /// # Safety
   ///
   /// This function is unsafe as it does not verify the preconditions from
   /// [`Layout::from_size_align`].
   #[cfg(feature="allocator")]
   #[inline]
   pub const unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Self
   {
      // SAFETY: the caller must ensure that `align` is greater than zero.
      Layout { size_: size, align_: unsafe { NonZeroUsize::new_unchecked(align) } }
   }

   /// The minimum size in bytes for a memory block of this layout.
   #[cfg(feature="allocator")]
   #[inline]
   pub const fn size(&self) -> usize { self.size_ }

   /// The minimum byte alignment for a memory block of this layout.
   #[cfg(feature="allocator")]
   #[inline]
   pub const fn align(&self) -> usize { self.align_.get() }

   /// Constructs a `Layout` suitable for holding a value of type `T`.
   #[cfg(feature="allocator")]
   #[inline]
   pub const fn new<T>() -> Self
   {
      let (size, align) = size_align::<T>();

      // SAFETY: the align is guaranteed by Rust to be a power of two and
      // the size+align combo is guaranteed to fit in our address space. As a
      // result use the unchecked constructor here to avoid inserting code
      // that panics if it isn't optimized well enough.
      unsafe { Layout::from_size_align_unchecked(size, align) }
   }

   // TODO: Finish `Layout` implementation.

   /// Produces layout describing a record that could be used to
   /// allocate backing structure for `T` (which could be a trait
   /// or other unsized type like a slice).
   #[cfg(feature="allocator")]
   #[inline]
   pub fn for_value<T: ?Sized>(&self, t: &T) -> Self
   {
      let (size, align) = (mem::size_of_val(t), mem::align_of_val(t));
      debug_assert!(Layout::from_size_align(size, align).is_ok());
      // SAFETY: see rationale in `new` for why this is using the unsafe variation
      unsafe { Layout::from_size_align_unchecked(size, align) }
   }

   /// Produces layout describing a record that could be used to
   /// allocate backing structure for `T` (which could be a trait
   /// or other unsized type like a slice).
   ///
   /// # Safety
   ///
   /// This function is only safe to call if the following conditions hold:
   ///
   /// - If `T` is `Sized`, this function is always safe to call.
   /// - If the unsized tail of `T` is:
   ///     - a [slice], then the length of the slice tail must be an intialized
   ///       integer, and the size of the *entire value*
   ///       (dynamic tail length + statically sized prefix) must fit in `isize`.
   ///     - a [trait object], then the vtable part of the pointer must point
   ///       to a valid vtable for the type `T` acquired by an unsizing coersion,
   ///       and the size of the *entire value*
   ///       (dynamic tail length + statically sized prefix) must fit in `isize`.
   ///     - an (unstable) [extern type], then this function is always safe to
   ///       call, but may panic or otherwise return the wrong value, as the
   ///       extern type's layout is not known. This is the same behavior as
   ///       [`Layout::for_value`] on a reference to an extern type tail.
   ///     - otherwise, it is conservatively not allowed to call this function.
   ///
   /// [trait object]: ../../book/ch17-02-trait-objects.html
   /// [extern type]: ../../unstable-book/language-features/extern-types.html
   pub fn for_value_raw<T: ?Sized>(&self, t: &T) -> Self
   {
      // SAFETY: we pass along the prerequisites of these functions to the caller
      let (size, align) = unsafe { (mem::size_of_val_raw(t), mem::align_of_val_raw(t)) };
      debug_assert!(Layout::from_size_align(size, align).is_ok());
      // SAFETY: see rationale in `new` for why this is using the unsafe variant
      unsafe { Layout::from_size_align_unchecked(size, align) }
   }

   /// Creates a `NonNull` that is dangling, but well-aligned for this Layout.
   ///
   /// Note that the pointer value may potentially represent a valid pointer,
   /// which means this must not be used as a "not yet initialized"
   /// sentinel value. Types that lazily allocate must track initialization by
   /// some other means.
   pub const fn dangling(&self) -> NonNull<u8>
   {
      // SAFETY: align is guaranteed to be non-zero.
      unsafe { NonNull::new_unchecked(self.align() as *mut u8) }
   }
}

/// # Layout Error
///
/// The parameters given to `Layout::from_size_align`
/// or some other `Layout` constructor
/// do not satisfy its documented constraints.
#[cfg(feature="allocator")]
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LayoutError;

impl fmt::Display for LayoutError
{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
   {
      f.write_str("invalid paramters to Layout::from_size_align")
   }
}
