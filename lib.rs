/*!
*Memory management for P-systems*

This library provides facilities for managing memory within P-systems.
*/

#![cfg_attr(not(test), no_std)]
#![deny(clippy::all)]
#![warn(missing_docs)]
#![allow(unused)]
#![allow(dead_code)]
#![feature(asm)]
#![feature(decl_macro)]
#![feature(llvm_asm)]

/// # Memory allocation facilities
pub mod alloc;

#[cfg(test)]
mod tests 
{
   #[test]
   fn it_works() 
   {
      assert_eq!(2 + 2, 4);
   }
}
