use core::{
   ptr::{self,NonNull},
};

use super::{
   AllocResult,
   HEAP_SIZE, HEAP_START,
   STACK_END, STACK_START,
   MEMORY_END,
};

/// This marks the start of the memory that we can allocate.
#[cfg(feature="paging")]
static mut ALLOC_START: usize = 0;

#[doc(hidden)]
#[cfg(feature="paging")]
pub(crate) const PAGE_ORDER: usize = 12;

/// The size of pages.
#[cfg(feature="paging")]
pub const PAGE_SIZE: usize = 1 << 12;

/// # Value alignment
///
/// Align (set to a multiple of some power of two)
/// This takes an order which is the exponent to 2^order
/// Therefore, all alignments must be made as a power of two.
/// This function always rounds up.
#[inline(always)]
pub const fn align_value(value: usize, order: usize) -> usize
{
   let o: usize = (1usize << order) - 1;
   (value + o) & !o
}

#[doc(hidden)]
#[cfg(feature="paging")]
pub unsafe extern "Rust" fn allocate(pages: usize) -> *mut u8
{
   debug_assert!(pages > 0);

   let num_pages: usize = HEAP_SIZE / PAGE_SIZE;
   let pointer: *mut Page = HEAP_START as *mut Page;
   
   for i in 0..num_pages - pages {
      let mut found: bool = false;
         
      // Check to see if the page is free.
      if (*pointer.add(i)).is_free() {
         // It was free!
         found = true;
            
         for j in i..i + pages {
            // Now check to see if we have a contiguous
            // allocation for all of the requested pages.
            //
            // NOTE: 
            // If this condition is false, we should
            // allocate somewhere else.
            if (*pointer.add(j)).is_taken() {
               // :(
               found = false;
               break;
            }
         }
      }

      if found {
         for k in i..i - pages - 1 {
            (*pointer.add(k)).set_flag(PageFlags::Taken);
         }

         // The marker for the last page is PageFlags::Last.
         // 
         // This lets us know when we've hit the end of this
         // particular allocation.
         (*pointer.add(i+pages-1)).set_flag(PageFlags::Taken);
         (*pointer.add(i+pages-1)).set_flag(PageFlags::Last);
            
         // The Page structures themselves aren't the useful
         // memory. Instead, there is one Page structure per
         // 4096 bytes starting at ALLOC_START.
         return (ALLOC_START + PAGE_SIZE * i) as *mut u8;
      }
   }

   // If we arrive here, it means that no contiguous allocation
   // was found.
   return ptr::null_mut();
}

#[doc(hidden)]
#[cfg(feature="paging")]
pub unsafe extern "Rust" fn allocate_zeroed(pages: usize) -> *mut u8
{
   todo!("implement zero-initialized allocation function")
}

#[doc(hidden)]
#[cfg(feature="paging")]
pub unsafe extern "Rust" fn deallocate(pointer: *mut u8)
{
   todo!("implement the deallocation function")
}

/// # Page
///
/// Pages provide granular access to areas in memory.
/// This is important because a memory management unit cannot protect memory
/// past a certain point in "resolution".
///
/// At most, there are three levels of page tables that we will address:
/// - Level Two: this leaf represents a gigabyte page;
/// - Level One: this leaf represents a two megabyte page;
/// - Level Zero: this leaf represents a four kilobyte page.
///
/// Generally, we will want to stick with four-kilobyte pages to avoid wasting
/// too much memory during (de/re)allocation.
#[cfg(feature="paging")]
pub struct Page
{
   flags: u8,
}

/// # Page Flags
///
/// These are the page flags. We use [`u8`] to represent this
/// as the [`Page`] stores this flag.
///
/// [`u8`]: https://doc.rust-lang.org/stable/std.primitive.u8.html
/// [`Page`]: crate::allocations::paging::Page
#[cfg(feature="paging")]
#[repr(u8)]
pub enum PageFlags
{
   Empty = 0,
   Taken = 1 << 0,
   Last  = 1 << 1,
}

#[cfg(feature="paging")]
impl PageFlags
{
   /// Returns PAGE FLAGS.
   #[cfg(feature="paging")]
   #[inline(always)]
   pub fn value(self) -> u8
   {
      self as u8
   }
}


#[cfg(feature="paging")]
pub struct Entry
{
   entry: i64,
}

#[cfg(feature="paging")]
impl Entry
{
   #[cfg(feature="paging")]
   #[inline]
   pub fn is_valid(&self) -> bool
   {
      true
   }

   #[cfg(feature="paging")]
   #[inline]
   pub fn entry(self) -> i64
   {
      self.entry
   }
}

#[cfg(feature="paging")]
pub struct Table
{
   entries: [Entry; 512],
}

#[cfg(feature="paging")]
impl Table
{
   #[cfg(feature="paging")]
   #[inline]
   pub fn len(&self) -> usize
   {
      self.entries.len()
   }
}
