#![no_std]
#![feature(alloc_error_handler)]

extern crate x86_64;
extern crate alloc;

mod io;
mod heap;
mod vm;
mod pic;
#[macro_use]
mod output;
mod kernel;
mod page_alloc;
mod interrupt_controller;
