use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pid_allocator::PidAllocator;

fn pid_allocator_benchmark(c: &mut Criterion) {
    let allocator = PidAllocator::<32>::new();
    c.bench_function("PidAllocator::allocate", |b| {
        b.iter(|| {
            let pid = allocator.allocate();
            black_box(pid);
        })
    });
}

criterion_group!(benches, pid_allocator_benchmark);
criterion_main!(benches);
