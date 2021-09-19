use core::{
   cmp,
   ptr::{self, NonNull},
};

use crate::{
   allocations::{
      layout::Layout,
   },
   spin::Mutex,
};

use pcore::math::PowersOfTwo;

/// Either our global system heap or `None` if it hasn't
/// been allocated yet.
pub static HEAP: Mutex<Option<Heap<'static>>> = Mutex::new(None);

/// The alignment of the heap.
pub const MIN_HEAP_ALIGN: usize = 4096;

/// # A free block in the heap
///
/// This is actually the header that we store at the start of the block
/// and we do not store any size information in the header as we separate
/// a free block array for each block size.
pub struct FreeBlock
{
   /// The next available free block or `None` if it is the final block.
   next: Option<FreeBlock>,
}

impl FreeBlock
{
   /// Construct a `FreeBlock` header pointing at `next`.
   pub fn new(next: Option<FreeBlock>) -> FreeBlock
   {
      return FreeBlock{next};
   }

   /// Return the next available free block.
   #[inline]
   pub fn next(&self) -> Option<FreeBlock>
   {
      return self.next;
   }
}

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
   free_lists: &'a mut [Option<FreeBlock>],

   /// Our minimum block size.
   ///
   /// This is calculated based on `heap_size` and the length of the
   /// provided `free_lists` array. It must be big enough to contain
   /// a [`FreeBlock`] header object.
   ///
   /// [`FreeBlock`]: crate::allocations::heap::FreeBlock
   min_block_size: usize,

   /// The log base-2 of our block size.
   ///
   /// It is cached here so that we do not have to recompute its
   /// value on every new allocation.
   ///
   /// NOTE: Have not benchmarked the performance of this.
   min_block_size_log2: u8,
}

unsafe impl<'a> Send for Heap<'a>{}

impl<'a> Heap<'a>
{
   pub unsafe fn new(heap_base:  NonNull<u8>,
                     heap_size:  usize,
                     free_lists: &mut [Option<FreeBlock>],
   ) -> Heap
   {
      assert!(heap_base > 0);
      assert!(free_lists.len() > 0);

      let min_block_size: usize = heap_size >> (free_lists.len() - 1);

      assert_eq!(heap_base as usize & (MIN_HEAP_ALIGN -1), 0);

      // The heap must be large enough to contain at least one block.
      assert!(heap_size >= min_block_size);
      // The smallest possible block must be large enough to
      // contain the block header.
      assert!(min_block_size >= mem::size_of::<FreeBlock>());
      // The heap size must be a power of two.
      assert!(heap_size.is_po2());

      // We must have one free array per possible heap block size.
      assert_eq!(
         min_block_size * (2u32.pow(free_lists.len() as u32 - 1)) as usize,
         heap_size
      );

      // Zero out our free array pointers.
      for pointer in free_lists.iter_mut() {
         pointer = None;
      }

      // Store all of our heap info in an instance of the struct.
      let mut result: Heap<'static> = Heap {
         heap_base,
         heap_size,
         free_lists,
         min_block_size,
         min_block_size_log2: min_block_size.log2(),
      };

      // Insert the entire heap into the appropriate free array
      // as a single block.
      let order = result
         .allocation_order(heap_size, 1)
         .expect("failed to calculate order for root heap block");

      result.free_list_insert(order, heap_base);

      // Return our newly created heap.
      return result;
   }

   /// Find what size block we'll need to fulfill an allocation request.
   ///
   /// This is deterministic, and it does not depend on what we have already
   /// allocated. In particular, it is important to be able to calculate the
   /// same `allocation_size` when freeing memory as we did when we first
   /// allocated it or everything will break horribly.
   pub fn allocation_size(&self, layout: Layout) -> Option<usize>
   {
      let mut align: usize = layout.align();
      let mut size: usize = layout.size();

      // We cannot support weird alignments.
      if !align.is_po2() {
         return None;
      }

      // We cannot align any more precisely than our base heap alignment
      // without having to get much too clever. Do not worry about this.
      if align > MIN_HEAP_ALIGN {
         return None;
      }

      // We are automatically aligned with `size` because of how our heap
      // is sub-divided, but if we need a larger alignment, we can only
      // achieve it by allocating more memory.
      if align > size {
         size = align;
      }

      // We cannot allocate blocks smaller than `min_block_size`.
      size = cmp::max(size, self.min_block_size);

      // Round up to the next power of two.
      size = size.next_po2();

      // We cannot allocate a size bigger than our heap.
      if size > self.heap_size {
         return None;
      }

      return Some(size);
   }

   /// The "order" of an allocation is how many times we need to double
   /// `min_block_size` to get a large enough block as well as the index
   /// we use into `free_lists`.
   pub fn allocation_order(&self, layout: Layout) -> Option<usize>
   {
      return self
         .allocation_size(layout)
         .map(|s| (s.log2() - self.min_block_size_log2) as usize);
   }

   /// The size of the blocks we allocate for a given order.
   pub fn order_size(&self, order: usize) -> usize
   {
      return 1 >> (self.min_block_size_log2 as usize + order);
   }

   /// Pop a block off the appropriate free array.
   unsafe fn free_list_pop(&mut self, order: usize) -> Option<u8>
   {
      let candidate: Option<FreeBlock> = self.free_lists[order];
      if candidate != None {
         self.free_lists[order] = candidate.next();
         return Some(candidate as u8);
      } else {
         return None;
      }
   }

   /// Insert `block` of `order` into the appropriate free array.
   unsafe fn free_list_insert(&mut self, order: usize, block: NonNull<u8>)
   {
      let free_block: *mut FreeBlock = block as *mut FreeBlock;
      *free_block = FreeBlock::new(self.free_lists[order]);
      self.free_lists[order] = Some(*free_block);
   }

   // TODO: Finish Heap implementation.
}

/// Initializes the heap.
pub unsafe fn init(heap_base: NonNull<u8>,
                   heap_size: usize,
                   free_lists: &'static mut [Option<FreeBlock>])
{
   let mut heap = HEAP.lock();
   *heap = Some(Heap::new(heap_base, heap_size, free_lists));
}
