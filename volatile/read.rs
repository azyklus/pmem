use super::{
   access,
   error::ReadError,
   Volatile,
};

use core::{
   ops::{
      Deref, DerefMut,
      Index, IndexMut,
   },
};

/// # `Copy` type reads 
///
/// Read references to `Copy` types.
impl<R, T> access::Read for Volatile<R>
   where
      R: Deref<Target = T>,
      T: Copy,
{
   /// # Read functionality
   ///
   /// Performs a volatile read of the contained value.
   ///
   /// Returns a copy of the read value. Volatile reads are guaranteed not to be optimized
   /// away by the compiler, but by themselves do not have atomic ordering
   /// guarantees. To also get atomicity, consider looking at the `Atomic` wrapper types of
   /// the standard/`core` library.
   ///
   /// ## Examples
   ///
   /// ```rust
   /// use pmem::volatile::Volatile;
   ///
   /// let value = 42;
   /// let shared_reference = Volatile::new(&value);
   /// assert_eq!(shared_reference.read(), 42);
   ///
   /// let mut value = 50;
   /// let mut_reference = Volatile::new(&mut value);
   /// assert_eq!(mut_reference.read(), 50);
   /// ```
   fn read(&self) -> access::Result<T>
   {
      // Get our return value.
      let ret = unsafe { ptr::read_volatile(&*self.ref_) };

      // Check if our return value is null or not.
      match ret.is_null() {
         true => Err(ReadError),
         false => Ok(ret)
      }
   }
}
