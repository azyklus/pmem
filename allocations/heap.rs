use core::{
   ptr::{self, NonNull},
};

/// The alignment of the heap.
pub const MIN_HEAP_ALIGN: usize = 4096;

/// # The Heap Interface
///
/// This data structure is stored _outside_ the heap somewhere,
/// because every single byte of our heap is potentially
/// available for allocation.
pub struct Heap<'a>
{
   /// The base address of our heap.
   ///
   /// Must be aligned along the boundary of [`MIN_HEAP_ALIGN`].
   ///
   /// [`MIN_HEAP_ALIGN`]: crate::allocations::heap::MIN_HEAP_ALIGN
   heap_base: NonNull<u8>,

   /// The size of our heap.
   ///
   /// This value must be a power of two.
   heap_size: usize,

   /// The free lists for our heap.
   ///
   /// The block at `free_lists[0]` contains the smallest block that
   /// we may allocate, and the array at the end can only contain a
   /// single free block size of the entire heap, and only when no
   /// memory is allocated.
   free_lists: &'a mut [NonNull<FreeBlock>],

   /// Our minimum block size.
   ///
   ///
   min_block_size: usize,

   ///
   min_block_size_log2: u8,
}

