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
#![feature(llvm_asm)]

/// # Memory allocation facilities
pub mod allocations;

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
