use super::{
   Allocator,
   AllocResult,
   layout::{
      Layout,
      size_align,
   },
   ptr::NonNull,
};

/// # The Global memory allocator
#[cfg(feature="allocator")]
pub struct Global;

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
