use super::{
   Allocator,
   AllocResult,
   layout::{
      Layout,
      size_align,
   },
   paging,
   ptr::NonNull,
};

/// # The Global memory allocator
#[cfg(feature="allocator",not(feature="paging"))]
pub struct Global;

#[cfg(feature="allocator",not(feature="paging"))]
unsafe impl Allocator for Global
{
   fn allocate(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      debug_assert!(layout.size() > 0);

      unimplemented!("implement function")
   }

   unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout)
   {
      unimplemented!("implement function")
   }
}

/// # Global page allocator
#[cfg(feature="paging",not(feature="allocator"))]
pub struct Global;

#[cfg(feature="paging",not(feature="allocator"))]
unsafe impl Allocator for Global
{
   fn allocate(&self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      debug_assert!(layout.size() > 0);

      // SAFETY: this is a safe call as we know our values and their size beforehand.
      unsafe {
         return Ok(NonNull::new_unchecked(paging::allocate(layout.size()) as [u8]));
      }
   }

   #[allow(unused)]
   unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout)
   {
      paging::deallocate(ptr.as_mut_ptr());
   }
}
