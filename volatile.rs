use core::{
   ptr,
};

use alloc::boxed::Box;

pub struct Volatile<R, T, A>
   where
      R: Deref<Target=T>,
      T: Sized,
      A: access::ReadWrite, 
{
   inner_: Box<T>,
   reference: R,
   access_: PhantomData<A>,
}

/// # Traits for privalege specification
///
/// Here, we define and implement `Read`, `Write`, and `ReadWrite`.
pub mod access;

/// # Reading from volatile memory
/// 
/// Implementing read functionality for volatile memory access.
#[cfg(feature="mem_read")]
pub mod read;

/// # Writing to volatile memory
///
/// Implementing write functionality for manipulating volatile memory.
#[cfg(feature="mem_write")]
pub mod write;
