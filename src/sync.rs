use crate::spin::{Mutex, MutexGuard};

/// # "Spinning" Mutex wrapper
///
/// This struct behaves as a thin wrapper around `spin::Mutex`.
pub struct Locked<T: ?Sized>
{
   inner: Mutex<T>
}

impl<T> Locked<T>
{
   pub fn new(ref_: T) -> Locked<T>
   {
      Locked {
         inner: Mutex::new(ref_),
      }
   }

   pub fn lock(&self) -> MutexGuard<T>
   {
      self.inner.lock()
   }
}
