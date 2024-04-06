# PidAllocator

## Overview

The `PidAllocator` crate provides a thread-safe and efficient PID (Process Identifier) allocation system, suitable for systems where PIDs need to be dynamically allocated and recycled. This crate is designed to work in `no_std` environments, making it suitable for use in embedded systems, operating systems, and other contexts where standard library facilities are not available.

## Features

- **Thread-Safe Allocation**: Uses `Arc` and `SpinMutex` to ensure that PIDs can be safely allocated and recycled across multiple threads.
- **Efficient Recycling**: Implements a fast allocation strategy that efficiently recycles PIDs, ensuring minimal wastage of the PID space.
- **`no_std` Compatibility**: Designed to work in `no_std` environments, making it ideal for low-level system programming.

## Usage

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
pid_allocator = "0.1.3"
```

And then in your Rust code:

```rust
#![no_std]

extern crate alloc;
use pid_allocator::{PidAllocator, Pid};

const ORDER: usize = 32; // Customize based on your requirements

fn main() {
    let allocator = PidAllocator::<ORDER>::new();
    
    // Attempt to allocate a PID
    if let Some(pid) = allocator.allocate() {
        // Use the PID for your purposes
        println!("Allocated PID: {}", *pid);
        
        // PID will be automatically recycled when `pid` goes out of scope
    }
}
```

## Structures

### `PidAllocator`

The main structure that manages PID allocation and recycling.

#### Methods

- `new() -> Self`: Creates a new instance of the PID allocator.
- `allocate() -> Option<Pid>`: Allocates a new PID, if available, and wraps it in a `Pid` structure.
- `contains(usize) -> bool`: Checks whether a given PID is currently allocated.

### `Pid`

A handle to an allocated PID. Automatically recycles the PID when dropped.

## How It Works

The `PidAllocator` crate utilizes a layered approach to manage the allocation and recycling of PIDs. Each layer represents a group of PIDs, with the state of each PID (allocated or free) tracked using a bit in a `usize` value. The allocator scans these layers to quickly find free PIDs and to recycle them when no longer in use.

## Contribution

Contributions are welcome! Please feel free to submit pull requests, report bugs, and suggest features through the issue tracker.

## License

This crate is licensed under [MIT license](LICENSE).

---