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

/// Allocate a [`Page`] or multiple `pages`.
///
/// `pages`: the number of PAGE_SIZE pages to allocate
///
/// [`Page`]: crate::allocations::paging::Page
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

/// Allocate and zero a [`Page`] or multiple `pages`
///
/// `pages`: the number of pages to allocate
///
/// Each page is PAGE_SIZE which is calculated as 1 << PAGE_ORDER
/// On RISC-V, this typically will be 4,096 bytes.
///
/// [`Page`]: crate::allocations::paging::Page
#[cfg(feature="paging")]
pub unsafe extern "Rust" fn allocate_zeroed(pages: usize) -> *mut u8
{
   let ret: *mut u8 = self::allocate(pages);
   if !ret.is_null() {
      let size: usize = (PAGE_SIZE * pages) / 8;
      let wide: *mut u64 = ret as *mut u64;

      for i in 0..size {
         // We use wide so we can force an "sd" (store doubleword)
         // instruction rather than the "sb".
         //
         // This means eight times fewer stores.
         //
         // Typically, we have to be concerned about the remaining
         // bytes, but fortunately 4096 % 8 is equal to zero, and as
         // such we will not have any remaining bytes.
         (*wide.add(i)) = 0;
      }
   }

   return ret;
}

/// Deallocate a page by its [`pointer`].
///
/// The way we've structured this, it will automatically coalesce
/// contiguous pages.
///
/// [`pointer`]: https://doc.rust-lang.org/stable/std/primitive.u8.html
#[cfg(feature="paging")]
pub unsafe extern "Rust" fn deallocate(pointer: *mut u8)
{
   debug_assert!(!pointer.is_null());
   let address: usize =
      HEAP_START + (pointer as usize - ALLOC_START) / PAGE_SIZE;

   // Make sure that the address makes sense.
   //
   // The address we calculate here is the page structure,
   // and NOT the HEAP ADDRESS.
   debug_assert!(address >= HEAP_START && address < ALLOC_START);
   let mut page: *mut Page = address as *mut Page;

   debug_assert!((*page).is_taken(), "Freeing a non-taken page?");
   while (*page).is_taken() && !(*page).is_last() {
      (*page).clear();
      page = page.add(1);
   }

   // If the following assertion fails, it is most likely
   // caused by a double-free.
   debug_assert!(
      (*page).is_last() == true,
      "Possible double-free detected! (Not taken, found \
      before last)"
   );

   // If we get here, we have taken care of all previous pages
   // and we are on the last page.
   (*page).clear();
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

#[cfg(feature="paging")]
impl Page
{
   /// If this page has been marked as the "final" allocation,
   /// this function returns true, else it returns false.
   #[inline(always)]
   pub fn is_last(&self) -> bool
   {
      return self.flags & PageFlags::Last.value() != 0;
   }

   /// If this page is marked as "taken",
   /// this function returns true, else it returns false.
   #[inline(always)]
   pub fn is_taken(&self) -> bool
   {
      return self.flags & PageFlags::Taken.value() != 0;
   }

   /// If this page is free, this function returns true,
   /// else it returns false.
   #[inline(always)]
   pub fn is_free(&self) -> bool
   {
      return !self.is_taken();
   }

   /// Clear the Page structure and all associated allocations.
   #[inline(always)]
   pub fn clear(&mut self)
   {
      self.flags = PageFlags::Empty.value();
   }

   /// Set a certain flag.
   ///
   /// ## Note
   ///
   /// We run into trouble here since PageFlags is an enum
   /// and we haven't implemented BitOr on it.
   #[inline(always)]
   pub fn set_flag(&mut self, flag: PageFlags)
   {
      self.flags |= flag.value();
   }

   /// Clear a certain flag.
   ///
   /// ## Note
   ///
   /// We run into trouble here since PageFlags is an enum
   /// and we haven't implemented BitAnd on it.
   #[inline(always)]
   pub fn clear_flag(&mut self, flag: PageFlags)
   {
      self.flags &= !(flag.value());
   }
}

/// # Page Flags
///
/// These are the page flags. We use [`u8`] to represent this
/// as the [`Page`] stores this flag.
///
/// [`u8`]: https://doc.rust-lang.org/stable/std/primitive.u8.html
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
      return self.entry() & EntryFlags::Valid.value() != 0;
   }

   /// The first bit (index #0) is the V-bit for valid.
   #[cfg(feature="paging")]
   #[inline]
   pub fn is_invalid(&self) -> bool
   {
      return !self.is_valid();
   }

   /// A leaf has one or more RWX bits set.
   #[cfg(feature="paging")]
   #[inline]
   pub fn is_leaf(&self) -> bool
   {
      return self.entry() & 0xe != 0;
   }

   #[cfg(feature="paging")]
   #[doc(hidden)]
   #[inline]
   pub fn is_branch(&self) -> bool
   {
      return !self.is_leaf();
   }

   /// Sets the value of `entry`.
   #[cfg(feature="paging")]
   #[inline]
   pub fn set_entry(&mut self, entry: usize)
   {
      self.entry = entry;
   }

   /// Gets the current value of `entry`.
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

// //////////////////////////////////////////////////////////////////////////////////////////////////////////////
// //  MMU ROUTINES  ////////////////////////////////////////////////////////////////////////////////////////////
// //////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents our entry flags as unsigned 64-bit integers.
#[cfg(feature="paging")]
#[repr(usize)]
pub enum EntryFlags
{
   None = 0,
   Valid = 1 << 0,
   Read = 1 << 1,
   Write = 1 << 2,
   Execute = 1 << 3,
   User = 1 << 4,
   Global = 1 << 5,
   Access = 1 << 6,
   Dirty = 1 << 7,

   // Convenience combinations
   ReadWrite = 1 << 1 | 1 << 2,
   ReadExecute = 1 << 1 | 1 << 3,
   ReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3,

   // User convenience combinations
   UserReadWrite = 1 << 1 | 1 << 2 | 1 << 4,
   UserReadExecute = 1 << 1 | 1 << 3 | 1 << 4,
   UserReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3 | 1 << 4,
}

/// Helper functions to convert the enum into a usize,
/// which is what our table entries will be.
impl EntryFlags
{
   /// Gets the flag as a [`usize`].
   ///
   /// [`usize`]: https://doc.rust-lang.org/stable/std/primitive.usize.html
   #[inline(always)]
   pub fn value(self) -> usize
   {
      self as usize
   }
}

/// Map a virtual address to a physical address using 4096-byte page
/// size.
///
/// ## Params
///
/// root: a mutable reference to the root Table
/// vaddr: The virtual address to map
/// paddr: The physical address to map
/// bits: An OR'd bitset containing the bits the leaf should have.
///       The bits should contain only the following:
///          Read, Write, Execute, User, and/or Global
///       The bits MUST include one or more of the following:
///          Read, Write, Execute
///       The valid bit automatically gets added.
#[cfg(feature="paging")]
pub fn map(root:  &mut Table,
           vaddr: usize,
           paddr: usize,
           bits:  usize,
           level: usize)
{
   debug_assert!(bits & 0xe != 0);
   // Extract each VPN from the virtual address.
   //
   // On the virtual address, each VPN is precisely nine bits,
   // which is why we use the mask 0x1ff = 0b1_1111_1111 (nine bits).
   let vpn: [usize; 3] = [
      // PPN[0] = paddr[20:12]
      (paddr >> 12) & 0x1ff,
      // PPN[1] = paddr[29:21]
      (paddr >> 21) & 0x1ff,
      // PPN[2] = paddr[55:30]
      (paddr >> 30) & 0x3ff_ffff,
   ];

   // We will use this as a floating-point reference so we can set
   // each individual entry as we traverse the table.
   let mut v: &mut [Entry; 512] = &mut root.entries[vpn[2]];

   // Now we're going to traverse the page table and set the bits
   // to their proper values. We expect that the root is valid,
   // however we are required to create anything beyond the root.
   //
   // In Rust, we create a range iterator using the '..' operator.
   // The `.rev()` will reverse the iteration since we need to start
   // with VPN[2].
   //
   // The '..' operator is inclusive on start but exclusive on end,
   // so (0..2) will iterate 0 and 1.
   for i in (level..2).rev() {
      if !v.is_valid() {
         // Allocate a page.
         let page: *mut u8 = allocate_zeroed(1);

         // The page is aligned by 4096, so store it directly.
         //
         // The page is stored in the entry shifted right
         // by two places.
         v.set_entry(
            (page as usize >> 2)
            | EntryFlags::Valid.value(),
         );
      }

      let entry: *mut Entry = ((v.entry() & !0x3ff) << 2) as *mut Entry;
      v = unsafe { entry.add(vpn[i]).as_mut().unwrap() };
   }

   // When we get here, we should be at VPN[0] and v should be pointing
   // to our entry.
   //
   // The entry structure is Figure 4.18 in the RISC-V Privileged Spec.
   let entry: usize = (ppn[2] << 28)               |
                      (ppn[1] << 19)               |
                      (ppn[0] << 10)               |
                      bits                         |
                      EntryFlags::Valid.value()    |
                      EntryFlags::Dirty.value()    |
                      EntryFlags::Access.value()   ;

   // Set the entry.
   //
   // V should be set to the correct pointer by the loop above.
   v.set_entry(entry);
}

/// Unmaps and frees all memory associated with a [`Table`].
///
/// `root`: The root table to start freeing.
///
///
/// ## NOTE
///
/// This does NOT free root directly. This must be
/// freed manually.
/// The reason we don't free the root is because it is
/// usually embedded into the Process structure.
///
/// [`Table`]: crate::allocations::paging::Table
#[cfg(feature="paging")]
pub fn unmap(root: &mut Table)
{
   for lv2 in 0..Table::len() {
      let ref entry_lv2 = root.entries[lv2];
      if entry_lv2.is_valid() && entry_lv2.is_branch() {
         // This is a valid entry, so drill down and free.
         let memaddr_lv1: usize = (entry_lv2.entry() & !0x3ff) << 2;
         let table_lv1: &mut Table = unsafe {
            // Make table_lv1 a mutable reference instead of a pointer.
            (memaddr_lv1 as *mut Table).as_mut()
         };

         for lv1 in 0..Table::len() {
            let ref entry_lv1 = table_lv1[lv1];
            if entry_lv1.is_valid() && entry_lv1.is_branch() {
               let memaddr_lv0: usize = (entry_lv1.entry() & !0x3ff) << 2;
            }
         }

         // The next level is zero, which cannot have branches
         // and so we free here.
         self::deallocate(memaddr_lv1);
      }
   }
}

/// Walk the page table to convert a virtual address to a
/// physical address.
///
/// If a page fault would occur, this returns None
/// Otherwise, it returns Some with the physical address.
#[cfg(feature="paging")]
pub fn virt_to_phys(root: &Table, vaddr: usize) -> Option<usize>
{
   // Walk the page table pointed to by root.
   let vpn: [usize; 3] = [
      // VPN[0] = vaddr[20:12]
      (vaddr >> 12) & 0x1ff,
      // VPN[1] = vaddr[29:21]
      (vaddr >> 21) & 0x1ff,
      // VPN[2] = vaddr[38:30]
      (vaddr >> 30) & 0x1ff,
   ];

   let mut v = &root.entries[vpn[2]];

   for i in (0..=2).rev() {
      // This is an invalid entry; page fault here.
      if v.is_invalid(){ break; }
      else if v.is_leaf() {
         // In RISC-V, a leaf can be present at any level.

         // The offset mask masks off the PPN.
         // Each PPN is nine bits and they start at bit twelve,
         // so our formula is (12 + i * 9).
         let off_mask = (1 << (12 + i * 9)) - 1;
         let vaddr_pgoff = vaddr & off_mask;
         let address = ((v.entry() << 2) as usize) & !off_mask;

         return Some(address | vaddr_pgoff);
      }

      // Set `v` to the next entry which is pointed to
      // by this entry. However, the address was shifted right
      // by two places when stored in the page table entry,
      // so we shift it left to put it back in place.
      let entry = ((v.entry() & !0x3ff) << 2) as *const Entry;

      // We do 'i - 1' here, however we should get None or Some()
      // above before we do 0 - 1 = -1.
      v = unsafe { entry.add(vpn[i - 1]).as_ref() };
   }

   // If we get here, we have exhausted all valid tables
   // and have not found a leaf.
   return None;
}
