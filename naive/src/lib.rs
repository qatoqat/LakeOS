#![feature(asm)]
#![feature(decl_macro)]
#![feature(alloc_error_handler)]
#![feature(const_in_array_repeat_expressions)]
#![feature(optin_builtin_traits)]
#![feature(const_fn)]
#![feature(allocator_api)]
#![feature(const_saturating_int_methods)]
#![feature(linked_list_cursors)]
#![feature(llvm_asm)]

#![no_std]

extern crate alloc;
extern crate rustyl4api;

#[macro_use] mod utils;
mod rt;
pub mod space_manager;
mod vm_allocator;
pub mod thread;
pub mod process;
mod panic;

extern "C" {
    static _end: [u8; 0];
}