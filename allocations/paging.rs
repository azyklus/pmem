use alloc::vec::Vec;

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
   entries: Vec<Entry>,
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
