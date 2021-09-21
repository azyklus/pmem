/*!
*Memory management for P-systems*

This library provides facilities for managing memory within P-systems.
*/

#![no_std]
#![deny(clippy::all)]
#![warn(missing_docs)]
#![allow(unused)]
#![allow(dead_code)]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(custom_test_frameworks)]
#![feature(decl_macro)]
#![feature(lang_items)]
#![feature(layout_for_ptr)]
#![feature(llvm_asm)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(slice_ptr_get)]
#![feature(slice_ptr_len)]
#![test_runner(ptest::runner)]

extern crate alloc;
extern crate core;

#[macro_use]
extern crate lazy_static;

extern crate pcore;
extern crate ptest;

#[cfg(feature="sync")]
pub extern crate spin;

/// # Memory allocation facilities
pub mod allocations;

/// # Synchronization primitives
#[cfg(feature="sync")]
pub mod sync;

/// # Special unique pointer type
pub mod unique;

/// # Volatile memory access and manipulation
pub mod volatile;

#[cfg(test)]
mod tests
{
   #[test_case]
   fn it_works()
   {
      assert_eq!(2 + 2, 4);
   }
}
