#![no_std]

extern crate alloc;

use core::ops::Deref;

use alloc::sync::Arc;
use spin::mutex::SpinMutex;

/// A thread-safe PID allocator that can allocate and recycle PIDs efficiently.
/// It encapsulates the allocator's state within an `Arc<SpinMutex<...>>` to allow safe shared access across threads.
#[derive(Debug, Default)]
pub struct PidAllocator<const ORDER: usize> {
    inner: Arc<SpinMutex<PidAllocatorInner<ORDER>>>,
}

/// The internal state of the PID allocator, containing the layers of available PIDs.
#[derive(Debug)]
pub struct PidAllocatorInner<const ORDER: usize> {
    top_layer: usize,
    bottom_layers: [usize; ORDER],
}

impl<const ORDER: usize> PidAllocator<ORDER> {
    /// Creates a new instance of the PID allocator.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SpinMutex::new(PidAllocatorInner::new())),
        }
    }

    /// Attempts to allocate a new PID. Returns `Some(Pid)` if successful, or `None` if all PIDs are currently allocated.
    /// The allocated PID is wrapped in a `Pid` object, which will automatically recycle the PID when dropped.
    pub fn allocate(&self) -> Option<Pid<ORDER>> {
        let mut inner = self.inner.lock();
        inner.allocate().map(|number| Pid {
            number,
            allocator: self.inner.clone(),
        })
    }
}

impl<const ORDER: usize> PidAllocatorInner<ORDER> {
    fn new() -> Self {
        Self {
            top_layer: 0,
            bottom_layers: [0; ORDER],
        }
    }

    /// Allocates a PID from the internal state. This method should only be called with exclusive access to the state.
    pub fn allocate(&mut self) -> Option<usize> {
        for (index, &layer) in self.bottom_layers.iter().enumerate() {
            if layer != usize::MAX {
                let free_bit = (!layer).trailing_zeros() as usize;
                self.bottom_layers[index] |= 1 << free_bit;
                if self.bottom_layers[index] == usize::MAX {
                    self.top_layer |= 1 << index;
                }
                return Some(index * (usize::BITS as usize) + free_bit);
            }
        }
        None
    }

    /// Recycles the given PID, making it available for allocation again.
    pub fn recycle(&mut self, number: usize) {
        let bits_per_layer = usize::BITS as usize;
        let layer_index = number / bits_per_layer;
        let bit_index = number % bits_per_layer;
        self.bottom_layers[layer_index] &= !(1 << bit_index);
        if self.bottom_layers[layer_index] != usize::MAX {
            self.top_layer &= !(1 << layer_index);
        }
    }
}

impl<const ORDER: usize> Default for PidAllocatorInner<ORDER> {
    fn default() -> Self {
        Self::new()
    }
}

/// A handle to an allocated PID. When dropped, the PID is automatically recycled back into the allocator.
#[derive(Debug)]
pub struct Pid<const ORDER: usize> {
    number: usize,
    allocator: Arc<SpinMutex<PidAllocatorInner<ORDER>>>,
}

impl<const ORDER: usize> Deref for Pid<ORDER> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.number
    }
}

impl<const ORDER: usize> Drop for Pid<ORDER> {
    fn drop(&mut self) {
        self.allocator.lock().recycle(self.number);
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    const ORDER: usize = 32;

    #[test]
    fn pid_allocate_success() {
        let allocator = PidAllocator::<ORDER>::new();
        assert!(allocator.allocate().is_some());
    }

    #[test]
    fn pid_allocate_unique() {
        let allocator = PidAllocator::<ORDER>::new();
        let pid1 = allocator.allocate().expect("Failed to allocate PID 1");
        let pid2 = allocator.allocate().expect("Failed to allocate PID 2");
        assert_ne!(*pid1, *pid2, "Allocated PIDs should be unique");
    }

    #[test]
    fn pid_recycle_and_reallocate() {
        let allocator = PidAllocator::<ORDER>::new();

        {
            let _pid = allocator.allocate().expect("Failed to allocate PID");
        }

        let mut pids = Vec::new();
        for _ in 0..ORDER * usize::BITS as usize {
            if let Some(pid) = allocator.allocate() {
                pids.push(*pid);
            } else {
                panic!("Failed to allocate a new PID after recycling");
            }
        }

        assert_eq!(pids.len(), ORDER * usize::BITS as usize, "Not all PIDs were successfully re-allocated");
    }
}
