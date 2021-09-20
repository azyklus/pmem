use core::{
   fmt,
   result
};

/// Result type alias for allocation errors.
#[cfg(feature="allocator")]
pub type AllocResult<T> = result::Result<T, AllocError>;

/// # Allocation error
///
/// The `AllocError` indicates a failure in memory allocation.
/// This failure may be due to the exhaustion of memory or something
/// wrong when combining given input arguments with this allocator.
#[cfg(feature = "allocator")]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AllocError;

impl fmt::Display for AllocError
{
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
   {
      f.write_str("memory allocation failed")
   }
}

