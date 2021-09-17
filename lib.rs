/*!
*Memory management for P-systems*

This library provides facilities for managing memory within P-systems.
*/

#![cfg_attr(not(test), no_std)]
#![deny(clippy::all)]
#![warn(missing_docs)]
#![allow(unused)]
#![allow(dead_code)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(lang_items)]
#![feature(layout_for_ptr)]
#![feature(llvm_asm)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]

extern crate alloc;
extern crate core;

#[macro_use]
extern crate lazy_static;

#[cfg(feature="sync")]
pub extern crate spin;

/// # Memory allocation facilities
pub mod allocations;

/// # Synchronization primitives
#[cfg(feature="sync")]
pub mod sync;

/// # Volatile memory access and manipulation
pub mod volatile;

#[cfg(test)]
mod tests
{
   #[test]
   fn it_works()
   {
      assert_eq!(2 + 2, 4);
   }
}
