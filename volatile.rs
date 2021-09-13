use core::{
   ptr,
   marker::PhantomData,
};

use alloc::boxed::Box;

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
