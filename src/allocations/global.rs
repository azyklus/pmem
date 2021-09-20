use super::{
   layout::{size_align, Layout},
   paging,
   ptr::NonNull,
   AllocResult, Allocator,
};

use core::intrinsics;

/// Allocate memory with the global allocator.
///
/// This function forwards calls to the [`Heap::allocate`] or [`paging::allocate`] method
/// or function, respectively.
///
/// # Safety
/// - See [`Heap::allocate`]
/// - See [`paging::allocate`]
///
/// [`Heap::allocate`]: crate::allocations::heap::Heap::allocate
/// [`paging::allocate`]: crate::allocations::paging::allocate
#[inline]
pub unsafe fn allocate(layout: Layout) -> *mut u8
{
   unsafe { super::__rustc_allocate(layout.size(), layout.align()) }
}

/// Deallocate memory with the global allocator.
///
/// This function forwards calls to [`Heap::deallocate`] or [`paging::deallocate`] method
/// or function, respectively.
///
/// # Safety
/// - See [`Heap::deallocate`]
/// - See [`paging::deallocate`]
///
/// [`Heap::deallocate`]: crate::allocations::heap::Heap::deallocate
/// [`paging::deallocate`]: crate::allocations::paging::deallocate
#[inline]
pub unsafe fn deallocate(pointer: *mut u8, layout: Layout)
{
   unsafe { super::__rustc_deallocate(pointer, layout.size(), layout.align()) }
}

/// Reallocate memory with the global allocator.
#[inline]
pub unsafe fn reallocate(pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8
{
   unsafe { super::__rustc_reallocate(pointer, layout.size(), layout.align(), new_size) }
}

/// Allocate zero-initialized memory with the global allocator.
#[inline]
pub unsafe fn allocate_zeroed(layout: Layout) -> *mut u8
{
   let pointer: NonNull<u8> = NonNull::new(self::alloc(layout)).expect("value is null");
   unsafe {
      let unchecked: NonNull<[u8]> = NonNull::slice_from_raw_parts(pointer, layout.size());
      unchecked
         .as_non_null_ptr()
         .as_ptr()
         .write_bytes(0, unchecked.len());
   }

   return pointer.as_ptr();
}

/// # The Global memory allocator
#[cfg(all(feature = "allocator", not(feature = "paging")))]
pub struct Global;

impl Global
{
   #[inline]
   fn alloc_impl(&self, layout: Layout, zeroed: bool) -> AllocResult<NonNull<[u8]>>
   {
      let _align: usize = layout.align();
      let _size: usize = layout.size();

      match layout.size() {
         0 => Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0)),
         // SAFETY: `layout` is non-zero in size.
         size => unsafe {
            let raw: *mut u8 = if zeroed {
               allocate_zeroed(layout)
            } else {
               allocate(layout)
            };
            let pointer: NonNull<u8> = NonNull::new(raw).ok_or(AllocError).unwrap();
            let slice: NonNull<[u8]> = NonNull::slice_from_raw_parts(pointer, size);

            return Ok(slice);
         },
      }
   }

   #[inline]
   unsafe fn grow_impl(
      &self,
      pointer: NonNull<u8>,
      old_layout: Layout,
      new_layout: Layout,
      zeroed: bool,
   ) -> AllocResult<NonNull<[u8]>>
   {
      debug_assert!(new_layout.size() >= old_layout.size());

      match old_layout.size() {
         0 => self.alloc_impl(new_layout, zeroed),

         // SAFETY: `new_size` is non-zero as `old_size` is greater than or equal to `new_size`
         // as required by safety conditions. Other conditions must be upheld by caller.
         old_size if old_layout.align() == new_layout.align() => unsafe {
            let new_size = new_layout.size();

            // `realloc` probably checks for `new_size >= old_layout.size()` or something similar.
            intrinsics::assume(new_size >= old_layout.size());

            let raw: *mut u8 = reallocate(pointer.as_ptr(), old_layout, new_size);
            let ptr: NonNull<u8> = NonNull::new(raw).ok_or(AllocError).expect("value is null");
            if zeroed {
               raw.add(old_size).write_bytes(0, new_size - old_size);
            }

            let parts: NonNull<[u8]> = NonNull::slice_from_raw_parts(ptr, new_size);

            return Ok(parts);
         },

         // SAFETY: because `new_layout.size()` must be greater than or equal to `old_size`,
         // both the old and new memory allocation are valid for reads and writes for `old_size`
         // bytes. Also, because the old allocation wasn't yet deallocated, it cannot overlap
         // `new_ptr`. Thus, the call to `copy_nonoverlapping` is safe. The safety contract
         // for `dealloc` must be upheld by the caller.
         old_size => unsafe {
            let new: NonNull<[u8]> = self.alloc_impl(new_layout, zeroed);
            ptr::copy_nonoverlapping(pointer.as_ptr(), new.as_mut_ptr(), old_size);
            self.deallocate(pointer, old_layout);

            return Ok(new);
         },
      }
   }
}

/// Implement our heap allocator here.
#[cfg(all(feature = "allocator", not(feature = "paging")))]
unsafe impl Allocator for Global
{
   #[inline]
   fn allocate(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      self.alloc_impl(layout, false)
   }

   #[inline]
   unsafe fn deallocate(&self, pointer: NonNull<u8>, layout: Layout)
   {
      if layout.size() != 0 {
         unsafe {
            deallocate(pointer.as_ptr(), layout);
         }
      }
   }

   #[inline]
   fn allocate_zeroed(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      self.alloc_impl(layout, true)
   }
}

/// # Global page allocator
#[cfg(all(feature = "paging", not(feature = "allocator")))]
pub struct Global;

#[cfg(all(feature = "paging", not(feature = "allocator")))]
unsafe impl Allocator for Global
{
   fn allocate(&self, layout: Layout) -> AllocResult<NonNull<u8>>
   {
      debug_assert!(layout.size() > 0);

      // SAFETY: this is a safe call as we know our values and their size beforehand.
      unsafe {
         return Ok(NonNull::new_unchecked(
            paging::allocate(layout.size()) as *mut u8
         ));
      }
   }

   #[allow(unused)]
   unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout)
   {
      paging::deallocate(ptr.as_mut_ptr());
   }
}
