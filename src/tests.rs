use alloc::vec::Vec;

use crate::PidAllocator;

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

        assert_eq!(
            pids.len(),
            ORDER * usize::BITS as usize,
            "Not all PIDs were successfully re-allocated"
        );
    }

    #[test]
    fn test_contains_allocated_pid() {
        let allocator = PidAllocator::<ORDER>::new();
        let pid = allocator.allocate().expect("Failed to allocate PID");
        assert!(
            allocator.contains(*pid),
            "Allocated PID should be recognized as allocated"
        );
    }

    #[test]
    fn test_recycle_pid() {
        let allocator = PidAllocator::<ORDER>::new();
        let pid = allocator.allocate().expect("Failed to allocate PID");
        let pid_value = *pid;
        core::mem::drop(pid); // Drop to trigger recycle
        assert!(
            !allocator.contains(pid_value),
            "Recycled PID should not be recognized as allocated"
        );
    }

    #[test]
    fn test_contains_unallocated_pid() {
        let allocator = PidAllocator::<ORDER>::new();
        // 假设每个ORDER位都可以分配usize::BITS个PID，取一个足够大的数字以超过可能分配的PID范围
        let unallocated_pid = ORDER * usize::BITS as usize * 2;
        assert!(
            !allocator.contains(unallocated_pid),
            "Unallocated PID should not be recognized as allocated"
        );
    }