use alloc::boxed::Box;

use core::{
   marker::PhantomData,
   ops::{Deref,DerefMut},
   ptr,
};

use self::error::{ReadError,WriteError,Access};

/// # Volatile memory access/manipulation
///
/// Wraps a reference and makes accesses of the referenced value volatile.
///
/// Allows volatile reads and writes on the referenced value. The referenced value needs to
/// be `Copy` for reading and writing, as volatile reads and writes take and return copies
/// of the value.
///
/// Since not all volatile resources (e.g. memory mapped device registers) are both readable
/// and writable, this type supports limiting the allowed access types through an optional second
/// generic parameter `A` that can be one of `ReadWriteImpl`, `ReadImpl`, or `WriteImpl`. It defaults
/// to `ReadWriteImpl`, which allows all operations.
///
/// The size of this struct is the same as the size of the contained reference.
#[derive(Clone)]
#[repr(transparent)]
pub struct Volatile<R, A = access::ReadWriteImpl>
{
   ref_: R,
   access_: PhantomData<A>,
}

impl<R> Volatile<R, access::ReadWriteImpl>
{
   /// # Constructor (read and write access)
   ///
   /// Constructs a new volatile instance wrapping the given reference.
   ///
   /// While it is possible to construct `Volatile` instances from arbitrary values (including
   /// non-reference values), most of the methods are only available when the wrapped type is
   /// a reference. The only reason that we don't forbid non-reference types in the constructor
   /// functions is that the Rust compiler does not support trait bounds on generic `const`
   /// functions yet. When this becomes possible, we will release a new version of this library
   /// with removed support for non-references. For these reasons it is recommended to use
   /// the `Volatile` type only with references.
   ///
   /// ## Example
   ///
   /// ```rust
   /// use pmem::volatile::Volatile;
   ///
   /// let mut value = 0u32;
   ///
   /// let mut volatile = Volatile::new(&mut value);
   /// volatile.write(1);
   /// assert_eq!(volatile.read(), 1);
   /// ```
   pub const fn new(ref_: R) -> Volatile<R>
   {
      return Volatile{
         ref_,
         access_: PhantomData,
      };
   }

   /// # Constructor (read-only access)
   ///
   /// Constructs a new read-only volatile instance wrapping the given reference.
   ///
   /// This is equivalent to the `new` function with the difference that the returned
   /// `Volatile` instance does not permit write operations. This is for example useful
   /// with memory-mapped hardware registers that are defined as read-only by the hardware.
   ///
   /// ## Example
   ///
   /// Reading is allowed:
   ///
   /// ```rust
   /// use volatile::Volatile;
   ///
   /// let value = 0u32;
   ///
   /// let volatile = Volatile::new_read_only(&value);
   /// assert_eq!(volatile.read(), 0);
   /// ```
   ///
   ///But writing is not:
   ///
   /// ```compile_fail
   /// use volatile::Volatile;
   ///
   /// let mut value = 0u32;
   ///
   /// let mut volatile = Volatile::new_read_only(&mut value);
   /// volatile.write(1);
   /// //ERROR: ^^^^^ the trait `volatile::access::Writable` is not implemented
   /// //             for `volatile::access::ReadOnly`
   /// ```
   pub const fn new_read_only(ref_: R) -> Volatile<R, access::ReadImpl>
   {
      return Volatile{
         ref_,
         access_: PhantomData,
      };
   }

   /// # Constructor (write-only access)
   ///
   /// Constructs a new write-only volatile instance wrapping the given reference.
   ///
   /// This is equivalent to the `new` function with the difference that the returned
   /// `Volatile` instance does not permit read operations. This is for example useful
   /// with memory-mapped hardware registers that are defined as write-only by the hardware.
   ///
   /// ## Example
   ///
   /// Writing is allowed:
   ///
   /// ```rust
   /// use volatile::Volatile;
   ///
   /// let mut value = 0u32;
   ///
   /// let mut volatile = Volatile::new_write_only(&mut value);
   /// volatile.write(1);
   /// ```
   ///
   /// But reading is not:
   ///
   /// ```compile_fail
   /// use volatile::Volatile;
   ///
   /// let value = 0u32;
   ///
   /// let volatile = Volatile::new_write_only(&value);
   /// volatile.read();
   /// //ERROR: ^^^^ the trait `volatile::access::Readable` is not implemented
   /// //            for `volatile::access::WriteOnly`
   /// ```
   pub const fn new_write_only(ref_: R) -> Volatile<R, access::WriteImpl>
   {
      return Volatile{
         ref_,
         access_: PhantomData,
      };
   }
}

impl<R, T, A> Volatile<R, A>
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
   fn read(&self) -> access::Result<*mut T>
      where
         A: access::Read,
   {
      // Get our return value.
      let ret = unsafe { ptr::read_volatile(&*self.ref_) };

      // Check if our return value is null or not.
      match ret.is_null() {
         true => Err(error::READ),
         false => Ok(*mut ret)
      }
   }

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
         A: access::Write,
         R: DerefMut,
   {
      match src.is_null() {
         true => Err(error::WRITE),
         false => {
            // SAFETY: we know our value is not null and as such, this call is safe.
            unsafe { ptr::write_volatile(&mut *self.ref_, *src) };

            Ok(())
         }
      }
   }

   /// # Read and update contained value
   ///
   /// Updates the contained value using the given closure and volatile instructions.
   ///
   /// Performs a volatile read of the contained value, passes a mutable reference to it to the
   /// function `f`, and then performs a volatile write of the (potentially updated) value back to
   /// the contained value.
   ///
   /// ```rust
   /// use volatile::Volatile;
   ///
   /// let mut value = 42;
   /// let mut volatile = Volatile::new(&mut value);
   /// volatile.update(|val| *val += 1);
   ///
   /// assert_eq!(volatile.read(), 43);
   /// ```
   pub fn update<F>(&mut self, f: F) -> access::Result<()>
      where
         A: access::ReadWrite,
         R: DerefMut,
         F: FnOnce(*mut T),
   {
      let mut value = self.read().unwrap();
      
      f(value);
      
      if let Err(e) = self.write(value) {
         return Err(e);
      } else {
         return Ok(());
      }
   }
}

/// # Traits for privalege specification
///
/// Here, we define and implement `Read`, `Write`, and `ReadWrite`.
pub mod access;

/// # Error handling
pub mod error;

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
