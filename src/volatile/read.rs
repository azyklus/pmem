use super::{
   access::{self,Result},
   Volatile,
};

use core::{
   ops::{
      Deref, DerefMut,
   },
};

#[cfg(feature="mem_read")]
pub trait Read
{
   #[cfg(feature="mem_read")]
   fn read_unaligned<T>(&self) -> Result<T>{ unimplemented!("implement function") }
}
