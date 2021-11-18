//! This crate implements the virtual machine.

#![feature(as_array_of_cells)]
#![feature(extern_types)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_write_slice)]
#![feature(never_type)]
#![feature(unwrap_infallible)]
#![no_std]
#![warn(missing_docs)]

extern crate alloc;
extern crate core;

pub mod heap;
pub mod object;
