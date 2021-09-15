use core::{
  fmt,
  ptr,
  marker::PhantomData,
  ops::{Deref,DerefMut},
};

/// Wraps a reference to make accesses to the referenced value volatile.
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
  access: PhantomData<A>,
}

impl<R> Volatile<R, access::ReadWriteImpl>
{
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
   #[inline]
   pub const fn new(ref_: R) -> Volatile<R, access::ReadWriteImpl>
   {
      return Volatile{
         ref_,
         access: PhantomData,
      };
   }

   /// Constructs a new read-only volatile instance wrapping the given reference.
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
   /// use pmem::volatile::Volatile;
   ///
   /// let mut value = 0u32;
   ///
   /// let mut volatile = Volatile::new_read_only(&mut value);
   /// volatile.read(1);
   /// ```
   ///
   /// But reading is not:
   ///
   /// ```compile_fail
   /// use pmem::volatile::Volatile;
   ///
   /// let value = 0u32;
   ///
   /// let volatile = Volatile::new_write_only(&value);
   /// volatile.write();
   /// //ERROR: ^^^^ the trait `pmem::volatile::access::Write` is not implemented
   /// //            for `pmem::volatile::access::ReadImpl`
   /// ```
   #[inline]
   pub const fn new_read_only(ref_: R) -> Volatile<R, access::ReadImpl>
   {
      return Volatile{
         ref_,
         access: PhantomData,
      };
   }

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
   /// use pmem::volatile::Volatile;
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
   /// use pmem::volatile::Volatile;
   ///
   /// let value = 0u32;
   ///
   /// let volatile = Volatile::new_write_only(&value);
   /// volatile.read();
   /// //ERROR: ^^^^ the trait `pmem::volatile::access::Read` is not implemented
   /// //            for `pmem::volatile::access::WriteImpl`
   /// ```
   #[inline]
   pub const fn new_write_only(ref_: R) -> Volatile<R, access::WriteImpl>
   {
      return Volatile{
         ref_,
         access: PhantomData,
      };
   }
}

/// Methods for references to `Copy` types.
impl<R, T, A> Volatile<R, A>
   where
      R: Deref<Target = T>,
      T: Copy,
{
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
   #[inline]
   pub fn read(&self) -> access::Result<T>
   {
      // UNSAFE: unsafe as we do not know if the value is valid.
      let mut ret = unsafe{(&mut ptr::read_volatile(&*self.ref_) as *mut T)};

      return match ret.is_null() {
         true => Err(error::Access::Read),
         // UNSAFE: dereferencing a raw pointer is unsafe by definition.
         false => Ok(unsafe {*ret}),
      };
   }

   /// Performs a volatile write, setting the contained value to the given `value`.
   ///
   /// Volatile writes are guaranteed to not be optimized away by the compiler, but by
   /// themselves do not have atomic ordering guarantees. To also get atomicity, consider
   /// looking at the `Atomic` wrapper types of the standard/`core` library.
   ///
   /// ## Example
   ///
   /// ```rust
   /// use pmem::volatile::Volatile;
   ///
   /// let mut value = 42;
   /// let mut volatile = Volatile::new(&mut value);
   /// volatile.write(50);
   ///
   /// assert_eq!(volatile.read(), 50);
   /// ```
   #[inline]
   pub fn write(&mut self, mut value: T) -> access::Result<()>
      where
         A: access::Read,
         R: DerefMut,
   {
      return match (&mut value as *mut T).is_null() {
         true => Err(error::Access::Write),
         false => {
            unsafe { ptr::write_volatile(&mut *self.ref_, value) };

            Ok(())
         },
      };
   }

   /// Updates the contained value using the given closure and volatile instructions.
   ///
   /// Performs a volatile read of the contained value, passes a mutable reference to it to the
   /// function `f`, and then performs a volatile write of the (potentially updated) value back to
   /// the contained value.
   ///
   /// ```rust
   /// use pmem::volatile::Volatile;
   ///
   /// let mut value = 42;
   /// let mut volatile = Volatile::new(&mut value);
   /// volatile.update(|val| *val += 1);
   ///
   /// assert_eq!(volatile.read(), 43);
   /// ```
   pub fn update<F>(&mut self, func: F) -> access::Result<()>
      where
         A: access::ReadWrite,
         R: DerefMut,
         F: FnOnce(&mut T),
   {
      let mut value = self.read()?;

      func(&mut value);

      self.write(value)
   }
}

impl<R, A> Volatile<R, A>
{
   /// Extracts the inner value stored in the wrapper type.
   ///
   /// This method gives direct access to the wrapped reference and thus allows
   /// non-volatile access again. This is seldom what you want since there is usually
   /// a reason that a reference is wrapped in `Volatile`. However, in some cases it might
   /// be required or useful to use the `read_volatile`/`write_volatile` pointer methods of
   /// the standard library directly, which this method makes possible.
   ///
   /// Since no memory safety violation can occur when accessing the referenced value using
   /// non-volatile operations, this method is safe. However, it _can_ lead to bugs at the
   /// application level, so this method should be used with care.
   ///
   /// ## Example
   ///
   /// ```
   /// use volatile::Volatile;
   ///
   /// let mut value = 42;
   /// let mut volatile = Volatile::new(&mut value);
   /// volatile.write(50);
   /// let unwrapped: &mut i32 = volatile.extract_inner();
   ///
   /// assert_eq!(*unwrapped, 50); // non volatile access, be careful!
   /// ```
   pub fn inner(self) -> R
   {
      self.ref_
   }
}

impl<R, T, A> Volatile<R, A>
   where
      R: Deref<Target = T>,
      T: ?Sized,
{
   /// Constructs a new `Volatile` reference by mapping the wrapped value.
   ///
   /// This method is useful for accessing individual fields of volatile structs.
   ///
   /// Note that this method gives temporary access to the wrapped reference, which allows
   /// accessing the value in a non-volatile way. This is normally not what you want, so
   /// **this method should only be used for reference-to-reference transformations**.
   ///
   /// ## Examples
   ///
   /// Accessing a struct field:
   ///
   /// ```
   /// use pmem::volatile::Volatile;
   ///
   /// struct Example { field_1: u32, field_2: u8, }
   /// let mut value = Example { field_1: 15, field_2: 255 };
   /// let mut volatile = Volatile::new(&mut value);
   ///
   /// // construct a volatile reference to a field
   /// let field_2 = volatile.map(|example| &example.field_2);
   /// assert_eq!(field_2.read(), 255);
   /// ```
   ///
   /// Don't misuse this method to do a non-volatile read of the referenced value:
   ///
   /// ```
   /// use pmem::volatile::Volatile;
   ///
   /// let mut value = 5;
   /// let mut volatile = Volatile::new(&mut value);
   ///
   /// // DON'T DO THIS:
   /// let mut readout = 0;
   /// volatile.map(|value| {
   ///    readout = *value; // non-volatile read, might lead to bugs
   ///    value
   /// });
   /// ```
   pub fn map<'a, F, U>(&'a self, func: F) -> Volatile<&'a U, A>
      where
         F: FnOnce(&'a T) -> &'a U,
         U: ?Sized,
         T: 'a,
   {
      return Volatile{
         ref_: func(self.ref_.deref()),
         access: self.access,
      };
   }

   /// Constructs a new `Volatile` reference by mapping the wrapped value.
   ///
   /// This method is useful for accessing individual fields of volatile structs.
   ///
   /// Note that this method gives temporary access to the wrapped reference, which allows
   /// accessing the value in a non-volatile way. This is normally not what you want, so
   /// **this method should only be used for reference-to-reference transformations**.
   ///
   /// ## Examples
   ///
   /// Accessing a struct field:
   ///
   /// ```
   /// use pmem::volatile::Volatile;
   ///
   /// struct Example { field_1: u32, field_2: u8, }
   /// let mut value = Example { field_1: 15, field_2: 255 };
   /// let mut volatile = Volatile::new(&mut value);
   ///
   /// // construct a volatile reference to a field
   /// let field_2 = volatile.map(|example| &example.field_2);
   /// assert_eq!(field_2.read(), 255);
   /// ```
   ///
   /// Don't misuse this method to do a non-volatile read of the referenced value:
   ///
   /// ```
   /// use pmem::volatile::Volatile;
   ///
   /// let mut value = 5;
   /// let mut volatile = Volatile::new(&mut value);
   ///
   /// // DON'T DO THIS:
   /// let mut readout = 0;
   /// volatile.map(|value| {
   ///    readout = *value; // non-volatile read, might lead to bugs
   ///    value
   /// });
   /// ```
   pub fn map_mut<'a, F, U>(&'a mut self, func: F) -> Volatile<&'a mut U, A>
      where
         F: FnOnce(&mut T) -> &mut U,
         R: DerefMut,
         U: ?Sized,
         T: 'a,
   {
      return Volatile{
         ref_: func(self.ref_.deref_mut()),
         access: self.access,
      };
   }
}

// TODO: Add methods for handling slices and arrays.

/// # Memory access rules
pub mod access;

/// # Error handling for volatiles
///
/// In this module, we define functions to handle errors that we might encounter
/// while attempting to access/manipulate volatile memory.
pub mod error;
