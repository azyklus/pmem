use super::{
   access,
   error::WriteError,
   Volatile,
};

use core::{
   ops::{
      Deref, DerefMut,
      Index, IndexMut,
   },
};

/// # `Copy` type writes.
impl<R, T> access::Write for Volatile<R>
   where
      R: Deref<Target = T>,
      T: Copy,
{
   /// # Write functionality
   ///
   /// Performs a volatile write, setting the contained value to the given `value`.
   ///
   /// Volatile writes are guaranteed to not be optimized away by the compiler, but by
   /// themselves do not have atomic ordering guarantees. To also get atomicity, consider
   /// looking at the `Atomic` wrapper types of the standard/`core` library.
   ///
   /// ## Example
   ///
   /// ```rust
   /// use volatile::Volatile;
   ///
   /// let mut value = 42;
   /// let mut volatile = Volatile::new(&mut value);
   /// volatile.write(50);
   ///
   /// assert_eq!(volatile.read(), 50);
   /// ```
   fn write(&mut self, src: *mut T) -> access::Result<()>
      where
         R: DerefMut,
   {
      match src.is_null() {
         true => Err(WriteError),
         false => {
            // SAFETY: we know our value is not null and as such, this call is safe.
            unsafe { ptr::write_volatile(&mut *self.ref_, src) };

            Ok(())
         }
      }
   }
}
