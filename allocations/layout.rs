use core::{
   fmt,
   mem,
   num::NonZeroUsize,
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
   pub fn size(&self) -> usize { self.size_ }

   /// The minimum byte alignment for a memory block of this layout.
   #[cfg(feature="allocator")]
   #[inline]
   pub fn align(&self) -> usize { self.align_.get() }

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
