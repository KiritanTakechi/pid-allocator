#![no_std]

extern crate alloc;

pub use allocator::{Pid, PidAllocator};

pub mod allocator;

#[cfg(test)]
mod tests;
