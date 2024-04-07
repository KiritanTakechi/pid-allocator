#![no_std]

extern crate alloc;

pub use allocator::PidAllocator;

pub mod allocator;

#[cfg(test)]
mod tests;