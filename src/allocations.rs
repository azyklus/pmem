use core::{
   fmt,
   ptr::{
      self,
      NonNull
   },
   result,
};

use self::ecs::AllocResult;
use self::layout::Layout;

extern "C"
{
   static HEAP_SIZE: usize;
   static HEAP_START: usize;

   static MEMORY_END: usize;

   static STACK_END: usize;
   static STACK_START: usize;
}

#[cfg(feature="allocator")]
pub unsafe trait Allocator
{
   fn allocate(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>;
   unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout);

   /// # Zero-initialized allocation
   ///
   /// Behaves similarly to `allocate` but also makes sure that the allocated
   /// memory is zero-initialized.
   ///
   /// ## Errors
   ///
   /// Returning `Err` indicates that either memory is exhausted or `layout` does
   /// not meet the allocator's size or alignment constraints.
   ///
   /// Implementations are encouraged to return `Err` on memory exhaustion rather
   /// than panicking or aborting, but this is not a requirement.
   ///
   /// Clients wishing to abort in response to an allocation error are encouraged
   /// to call the [`handle_alloc_error`] function rather than directly invoking
   /// the `panic!` or similar macro.
   ///
   /// [`handle_alloc_error`]: crate::allocations::handle_alloc_error
   fn allocate_zeroed(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      let ptr = self.allocate(layout)?;
      unsafe {
         ptr.as_non_null_ptr().as_ptr().write_bytes(0, ptr.len())
      };

      return Ok(ptr);
   }

   /// # Expand memory block
   ///
   /// Attempts to expand the memory block.
   ///
   /// Returns a new [`NonNull<[u8]>`][core::ptr::NonNull] containing a pointer and the actual
   /// size of the allocated memory. The pointer is suitable for holding data described by `new_layout`.
   /// To accomplish this, the allocator may extend the allocation referenced by `ptr` to fit the new layout.
   ///
   /// If this returns `Ok`, ownership of the memory block referenced by `ptr` has been transferred to this
   /// allocator. The memory may or may not have been freed and should be considered unusable unless it was
   /// transferred back to the caller again by this method.
   ///
   /// If this returns `Err`, then ownership of the memory block referenced by `ptr` has not been transferred
   /// to this allocator and the contents of the memory block remain unchanged.
   ///
   /// ## Safety
   ///
   /// - `ptr` must denote a block of memory [*currently allocated*] via this allocator.
   /// - `old_layout` must [*fit*] that block of memory (The `new_layout` argument need not fit it.).
   /// - `new_layout.size()` must be greater than or equal to `old_layout.size()`.
   ///
   /// [*currently allocated*]: #currently-allocated-memory
   /// [*fit*]: #memory-fitting
   ///
   /// ## Errors
   ///
   /// Returns `Err` if the new layout does not meet the allocator's size and alignment requirements or if
   /// growing the block of memory otherwise fails.
   ///
   /// Implementations are encouraged to return `Err` rather than invoke the `panic!` or similar macro
   /// directly, but this is not a requirement.
   ///
   /// Clients wishing to abort computation in response to an allocation failure should call the [`handle_alloc_error`]
   /// function rather than directly invoke `panic!` or similar macro.
   ///
   /// [`handle_alloc_error`]: crate::allocations::handle_alloc_error
   unsafe fn grow(&self,
      ptr: NonNull<u8>,
      old_layout: Layout,
      new_layout: Layout,
   ) -> AllocResult<NonNull<[u8]>>
   {
      debug_assert!(
         new_layout.size() >= old_layout.size(),
         "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
      );

      let new_ptr = self.allocate(new_layout)?;

      // SAFETY: because `new_layout.size()` must be greater than or equal to
      // `old_layout.size()`, both the old and new memory allocation are valid for reads and
      // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
      // deallocated, it cannot overlap `new_ptr`. Thus, the call to `copy_nonoverlapping` is
      // safe. The safety contract for `dealloc` must be upheld by the caller.
      unsafe {
         ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), old_layout.size());
         self.deallocate(ptr, old_layout);
      }

      return Ok(new_ptr);
   }

   /// # Shrink the memory block
   ///
   /// Attempts to shrink the memory block.
   /// Returns a new [`NonNull<[u8]>`][core::ptr::NonNull] containing a pointer and the actual
   /// size of the allocated memory. The pointer is suitable for holding data described by `new_layout`.
   /// To accomplish this, the allocator may extend the allocation referenced by `ptr` to fit the new layout.
   ///
   /// If this returns `Ok`, ownership of the memory block referenced by `ptr` has been transferred to this
   /// allocator. The memory may or may not have been freed and should be considered unusable unless it was
   /// transferred back to the caller again by this method.
   ///
   /// If this returns `Err`, then ownership of the memory block referenced by `ptr` has not been transferred
   /// to this allocator and the contents of the memory block remain unchanged.
   ///
   /// ## Safety
   ///
   /// - `ptr` must denote a block of memory [*currently allocated*] via this allocator.
   /// - `old_layout` must [*fit*] that block of memory (The `new_layout` argument need not fit it.).
   /// - `new_layout.size()` must be less than or equal to `old_layout.size()`.
   ///
   /// [*currently allocated*]: #currently-allocated-memory
   /// [*fit*]: #memory-fitting
   ///
   /// ## Errors
   ///
   /// Returns `Err` if the new layout does not meet the allocator's size and alignment requirements or if
   /// shrinking the block of memory otherwise fails.
   ///
   /// Implementations are encouraged to return `Err` rather than invoke the `panic!` or similar macro
   /// directly, but this is not a requirement.
   ///
   /// Clients wishing to abort computation in response to an allocation failure should call the [`handle_alloc_error`]
   /// function rather than directly invoke `panic!` or similar macro.
   ///
   /// [`handle_alloc_error`]: crate::allocations::handle_alloc_error
   unsafe fn shrink(&self,
      ptr: NonNull<u8>,
      old_layout: Layout,
      new_layout: Layout,
   ) -> AllocResult<NonNull<[u8]>>
   {
      debug_assert!(
         new_layout.size() <= old_layout.size(),
         "`new_layout.size()` must be less than or equal to `old_layout.size()`"
      );

      let new_ptr = self.allocate(new_layout)?;

      unsafe {
         ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_mut_ptr(), new_layout.size());
         self.deallocate(ptr, old_layout);
      }

      return Ok(new_ptr);
   }

   /// # Create a "by reference" adaptor
   ///
   /// This function creates a by-reference adaptor for this `Allocator` instance.
   /// The returned adaptor also implements `Allocator` and will simply borrow this.
   #[inline(always)]
   fn by_ref(&self) -> &Self
   where
      Self: Sized,
   {
      self
   }
}

unsafe impl<A> Allocator for &A
   where
      A: Allocator + ?Sized,
{
   #[inline]
   fn allocate(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      (**self).allocate(layout)
   }

   #[inline]
   fn allocate_zeroed(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      (**self).allocate_zeroed(layout)
   }

   #[inline]
   unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout)
   {
      // SAFETY: safety contract must be upheld by caller.
      unsafe { (**self).deallocate(ptr, layout) }
   }
}

use crate::sync::Locked;

unsafe impl<A> Allocator for Locked<A>
   where
      A: Allocator + Sized,
{
   #[inline]
   fn allocate(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      self.lock().allocate(layout)
   }

   #[inline]
   fn allocate_zeroed(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      self.lock().allocate_zeroed(layout)
   }

   #[inline]
   unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout)
   {
      // SAFETY: the safety contract must be upheld by the caller.
      unsafe { self.lock().deallocate(ptr, layout) }
   }
}

#[cfg(feature = "allocator")]
pub fn handle_alloc_error(layout: Layout) -> !
{
   loop{}
}

/// # Implements an ECS-style allocator
pub mod ecs;

/// # Global memory allocator implementation
pub mod global;

/// # Heap allocator implementation
pub mod heap;

/// # Defines memory layout structure
pub mod layout;

/// # Implements a simple page allocator
pub mod paging;
