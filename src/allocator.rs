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
    ///
    /// This constructor initializes the PID allocator, setting up its internal
    /// structure to keep track of allocated and available PIDs. The allocator
    /// supports up to `ORDER * usize::BITS` unique PIDs, where `ORDER` is a
    /// compile-time constant defining the number of layers in the allocator.
    ///
    /// # Examples
    ///
    /// ```
    /// use pid_allocator::PidAllocator;
    /// 
    /// let allocator = PidAllocator::<8>::new();
    /// ```
    ///
    /// This creates a PID allocator with 8 layers, capable of managing
    /// a total of `8 * usize::BITS` PIDs, which depends on the architecture
    /// (e.g., 256 PIDs for a 32-bit architecture or 512 PIDs for a 64-bit architecture).
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SpinMutex::new(PidAllocatorInner::new())),
        }
    }

    /// Attempts to allocate a new PID. Returns `Some(Pid)` if successful, or `None` if all PIDs are currently allocated.
    /// The allocated PID is wrapped in a `Pid` object, which will automatically recycle the PID when dropped.
    ///
    /// This method locks the internal state, searches for a free PID, marks it as used,
    /// and returns a `Pid` instance representing the allocated PID. If no PIDs are available,
    /// it returns `None`.
    ///
    /// # Returns
    ///
    /// * `Some(Pid<ORDER>)` containing the allocated PID if allocation is successful.
    /// * `None` if all PIDs are already allocated.
    ///
    /// # Examples
    ///
    /// Successful allocation:
    ///
    /// ```
    /// use pid_allocator::PidAllocator;
    /// 
    /// let allocator = PidAllocator::<8>::new();
    /// if let Some(pid) = allocator.allocate() {
    ///     println!("Allocated PID: {}", *pid);
    /// }
    /// ```
    ///
    /// Handling failure to allocate a PID:
    ///
    /// ```
    /// use pid_allocator::PidAllocator;
    /// 
    /// let allocator = PidAllocator::<8>::new();
    /// let mut pids = Vec::new();
    /// while let Some(pid) = allocator.allocate() {
    ///     pids.push(pid);
    /// }
    /// // At this point, no more PIDs can be allocated.
    /// assert!(allocator.allocate().is_none());
    /// ```
    ///
    /// In this example, PIDs are continuously allocated until no more are available,
    /// at which point `allocate()` returns `None`.
    pub fn allocate(&self) -> Option<Pid<ORDER>> {
        let mut inner = self.inner.lock();
        inner.allocate().map(|number| Pid {
            number,
            allocator: self.inner.clone(),
        })
    }

    /// Checks whether a given PID is currently allocated.
    ///
    /// # Parameters
    ///
    /// * `number`: The PID number to check for allocation.
    ///
    /// # Returns
    ///
    /// * `true` if the PID is currently allocated.
    /// * `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use pid_allocator::PidAllocator;
    /// 
    /// let allocator = PidAllocator::<32>::new();
    /// let pid = allocator.allocate().expect("Failed to allocate PID");
    ///
    /// assert!(allocator.contains(*pid), "The PID should be marked as allocated.");
    /// ```
    ///
    /// # Note
    ///
    /// This method performs a read-only operation on the allocator's state and is thread-safe,
    /// thanks to the internal use of `Arc<SpinMutex<...>>`. However, because the state of the allocator
    /// can change in concurrent environments, the returned allocation state might not remain valid
    /// immediately after this method is called.
    pub fn contains(&self, number: usize) -> bool {
        self.inner.lock().contains(number)
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
        const BITS_PER_LAYER_SHIFT: usize = usize::BITS.trailing_zeros() as usize;

        let layer_index = number >> BITS_PER_LAYER_SHIFT;
        let bit_index = number & (usize::BITS - 1) as usize;

        self.bottom_layers[layer_index] &= !(1 << bit_index);
        self.top_layer &= !(1 << layer_index);
    }

    /// Checks whether a given PID is currently allocated.
    pub fn contains(&self, number: usize) -> bool {
        const BITS_PER_LAYER_SHIFT: usize = usize::BITS.trailing_zeros() as usize;
        let layer_index = number >> BITS_PER_LAYER_SHIFT;
        let bit_index = number & ((1 << BITS_PER_LAYER_SHIFT) - 1);

        if layer_index < self.bottom_layers.len() {
            (self.bottom_layers[layer_index] & (1 << bit_index)) != 0
        } else {
            false
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