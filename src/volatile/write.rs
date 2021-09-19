use super::{
   access::{self,Result},
   Volatile,
};

use core::{
   ops::{
      Deref, DerefMut,
   },
   ptr,
};

#[cfg(feature="mem_write")]
pub trait Write
{
   #[cfg(feature="mem_write")]
   fn write_unaligned<R, T>(&mut self, mut value: R) -> Result<()>{ unimplemented!("implement function") }

   #[cfg(feature="mem_write")]
   unsafe fn write_unchecked<R, T>(&mut self, value: R){ unimplemented!("implement function") }
}

#[cfg(feature="mem_write")]
impl<R, T, A> Write for Volatile<R, A>
   where
      R: Deref<Target = T>,
      T: Copy,
{
   #[cfg(feature="mem_write")]
   fn write_unaligned<R, T>(&mut self, mut value: R) -> Result<()>
      where
         R: DerefMut,
         A: access::Write,
   {
      let ret = match (&mut value as *mut T).is_null() {
         true => Err(error::Access::Write),
         false => {
            unsafe {
               Self::write_unchecked(value);
            }

            Ok(())
         }
      }

      return ret;
   }

   #[cfg(feature="mem_write")]
   unsafe fn write_unchecked<R, T>(&mut self, value: R)
      where
         R: DerefMut,
         A: access::Write,
   {
      ptr::write_volatile(&mut *self.ref_, value);
   }
}
