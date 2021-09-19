use core::{
   cmp,
   ptr::{self, NonNull},
};

use crate::{
   allocations::{
      Allocator,
      ecs::{AllocResult,AllocError},
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
   next: *mut FreeBlock,
}

impl FreeBlock
{
   /// Construct a `FreeBlock` header pointing at `next`.
   pub fn new(next: *mut FreeBlock) -> FreeBlock
   {
      return FreeBlock{next};
   }

   /// Return the next available free block.
   #[inline]
   pub fn next(&self) -> *mut FreeBlock
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
   free_lists: &'a mut [*mut FreeBlock],

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
                     free_lists: &mut [*mut FreeBlock],
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
      let candidate: *mut FreeBlock = self.free_lists[order];
      if candidate != ptr::null_mut() {
         self.free_lists[order] = (*candidate).next();
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
      self.free_lists[order] = free_block;
   }

   // TODO: Finish Heap implementation.

   /// Attempt to remove a block from our free array, returning true
   /// success, and false if the block wasn't on our free array.  This is
   /// the slowest part of a primitive buddy allocator, because it runs in
   /// O(log N) time where N is the number of blocks of a given size.
   ///
   /// We could perhaps improve this by keeping our free lists sorted,
   /// because then "nursery generation" allocations would probably tend
   /// to occur at lower addresses and then be faster to find / rule out
   /// finding.
   unsafe fn free_list_remove(&mut self, order: usize, block: NonNull<u8>) -> bool
   {
      let block_pointer: *mut FreeBlock = block as *mut FreeBlock;

      // Yuck, array traversals are gross without recursion.
      //
      // Here, `*checking` is the pointer we want to check,
      // and `checking` is the memory location we found it at,
      // which we'll need if we want to replace the value
      // `*checking` with a new value.
      let mut checking: *mut *mut FreeBlock = &mut self.free_lists[order];

      // Loop until we run out of free blocks.
      while *checking != ptr::null_mut() {
         // Is this the block we need to remove?
         if *checking == block_pointer {
            // This is the block we must remove.
            // Overwrite the value we used to get here with
            // the next block in the sequence.
            *checking = (*(*checking)).next();
            return true;
         }

         // Haven't found it yet, so point `checking` at the address
         // containing our `next` field.  (Once again, this is so we'll
         // be able to reach back and overwrite it later if necessary.)
         checking = &mut ((*(*checking)).next());
      }

      return false;
   }

   unsafe fn split_free_block(&mut self, block: NonNull<u8>, mut order: usize, order_needed: usize)
   {
      // Get the size of our starting block.
      let mut size_to_split: usize = self.order_size(order);

      // Progressively cut our block down to size.
      while order > order_needed {
         // Update our loop counters to describe a block half the size.
         size_to_split >>= 1;
         order -= 1;

         // Insert the "upper half" of the block into the free array.
         let split: usize = block.offset(size_to_split as isize);
         self.free_list_insert(order, split);
      }
   }

   /// Allocate a block of memory.
   ///
   /// Must be large enough to contain `size` bytes and aligned to `align`.
   /// This will return `Err` if the `align` is greater than the `MIN_HEAP_ALIGN`,
   /// if `align` is not a power-of-two, or if we cannot find enough memory
   /// to allocate.
   ///
   /// All allocated memory must be passed to `deallocate` with the same [`Layout`]
   /// or else terrible things will happen.
   pub unsafe fn allocate(&mut self, layout: Layout) -> AllocResult<NonNull<[u8]>>
   {
      let mut align: usize = layout.align();
      let mut size: usize = layout.size();

      // Figure out which order block we will need.
      if let Some(order_needed) = self.allocation_order(size, align) {
         // Start with the smallest acceptable block size and search upward
         // until we reach blocks the size of the entire heap.
         for order in order_needed..self.free_lists.len() {
            // Do we have a block this size?
            if let Some(block) = self.free_list_pop(order) {
               // If the block is too big, break it up.
               // This leaves the address unchanged because we always
               // allocate at the head of a block.
               if order > order_needed {
                  self.split_free_block(block, order, order_needed);
               }

               let nonnull: NonNull<[u8]> = NonNull::new(block).unwrap();

               // We have an allocation
               return Ok(nonnull);
            }
         }

         // We could not find a large enough block for this allocation.
         return Err(AllocError);
      } else {
         // We cannot allocate a block with the specified size and alignment.
         return Err(AllocError);
      }
   }

   /// Given a `block` with the specified `order`, find the "buddy" block.
   ///
   /// The "buddy" is the other half of the block that we initially split
   /// from and also the block that we could potentially merge with.
   pub unsafe fn buddy(&self, order: usize, block: NonNull<u8>) -> Option<NonNull<u8>>
   {
      let relative: usize = (block as usize) - (self.heap_base as usize);
      let size: usize = self.order_size(order);

      if size >= self.heap_size {
         return None;
      } else {
         // Fun fact: we can find our buddy by xoring the right bit in our
         // offset from the base of the heap.
         return Some(self.heap_base.offset((relative ^ size) as isize));
      }
   }


}

/// Initializes the heap.
pub unsafe fn init(heap_base: NonNull<u8>,
                   heap_size: usize,
                   free_lists: &'static mut [Option<FreeBlock>])
{
   let mut heap = HEAP.lock();
   *heap = Some(Heap::new(heap_base, heap_size, free_lists));
}
