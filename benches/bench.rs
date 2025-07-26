#![feature(allocator_api)]
#![feature(array_ptr_get)]

use std::alloc::Layout;
use bumpalo::Bump;
use criterion::{criterion_group, criterion_main, Criterion};
use lake::droplet::Droplet;
use lake::Lake;
use scoped_arena::Scope;
use std::hint::black_box;
use bump_scope::BumpBox;
use typed_arena::Arena;

const SIZE: usize = 128_000;
const ITERS: usize = 1000;

fn bench_bumpalo(c: &mut Criterion) {
    c.bench_function("bumpalo", |b| {
        let mut bump = Bump::with_capacity(SIZE * ITERS);
        b.iter(|| {
            bump.reset();
            for i in 0..ITERS {
                let ptr: &mut [u8] = bump.alloc_slice_fill_default(SIZE);
                ptr[0] = (i % 256) as u8;
                black_box(ptr[0]);
                black_box(ptr[SIZE - 1]);
            }
        });
    });
}

fn bench_lake(c: &mut Criterion) {
    c.bench_function("lake", |b| {
        let mut lake: Lake<{ SIZE * ITERS }> = Lake::new();
        b.iter(|| {
            lake.clear();
            for i in 0..ITERS {
                let mut ptr: Droplet<SIZE, Lake<{ SIZE * ITERS }>> = lake.alloc::<SIZE>().unwrap();
                ptr[0] = (i % 256) as u8;
                black_box(ptr[0]);
                black_box(ptr[SIZE - 1]);
            }
        });
    });
}

fn bench_typed_arena(c: &mut Criterion) {
    c.bench_function("typed-arena", |b| {
        let ta: Arena<[u8; SIZE]> = Arena::new();
        b.iter(|| {
            // Typed arena.. em.... clean();    :)))))))
            for i in 0..ITERS {
                let ptr: &mut [u8; 128000] = ta.alloc([0u8; SIZE]);
                ptr[0] = (i % 256) as u8;
                black_box(ptr[0]);
                black_box(ptr[SIZE - 1]);
            }
        });
    });
}

fn bench_scoped_arena(c: &mut Criterion) {
    c.bench_function("scoped-arena", |b| {
        let mut scope = Scope::new();
        b.iter(|| {
            scope.reset();
            for i in 0..ITERS {
                let data: &mut [u8; 128000] = scope.to_scope([0u8; SIZE]);
                data[0] = (i % 256) as u8;
                black_box(data[0]);
                black_box(data[SIZE - 1]);
            }
        });
    });
}

fn bench_box(c: &mut Criterion) {
    c.bench_function("box_heap", |b| {
        b.iter(|| {
            for i in 0..ITERS {
                let mut ptr: Box<[u8; 128000]> = Box::new([0u8; SIZE]);
                ptr[0] = (i % 256) as u8;
                black_box(ptr[0]);
                black_box(ptr[SIZE - 1]);
            }
        });
    });
}

fn bench_vec_box(c: &mut Criterion) {
    c.bench_function("vec_box", |b| {
        let mut vec: Vec<Box<[u8; 128000]>> = Vec::with_capacity(ITERS);
        b.iter(|| {
            for i in 0..ITERS {
                let mut item = Box::new([0u8; SIZE]);
                item[0] = (i % 256) as u8;
                vec.push(item);
            }
            black_box(&vec);
        });
    });
}

fn bench_heapless(c: &mut Criterion) {
    use heapless::Vec as StackVec;
    c.bench_function("heapless_vec", |b| {
        let mut vec: StackVec<[u8; SIZE], ITERS> = StackVec::new();
        b.iter(|| {
            vec.clear();
            for i in 0..ITERS {
                let mut buf: [u8; 128000] = [0u8; SIZE];
                buf[0] = (i % 256) as u8;
                vec.push(buf).unwrap();
            }
            black_box(&vec);
        });
    });
}

fn bench_mimalloc(c: &mut Criterion) {
    use mimalloc::MiMalloc;
    use std::alloc::{GlobalAlloc, Layout};
    c.bench_function("mimalloc", |b| {
        let alloc = MiMalloc;
        let layout: Layout = Layout::array::<u8>(SIZE * ITERS).unwrap();
        let base_ptr: *mut u8 = unsafe { alloc.alloc(layout).cast::<u8>() };

        b.iter(|| unsafe {
            for i in 0..ITERS {
                let ptr: *mut u8 = base_ptr.add(i * SIZE);
                *ptr = (i % 256) as u8;
                black_box(*ptr);
            }
        });
        unsafe { alloc.dealloc(base_ptr, layout) };
    });
}

fn bench_bump_scope(c: &mut Criterion) {
    use bump_scope::Bump;
    c.bench_function("bump-scope", |b| {
        let layout: Layout = Layout::array::<u8>(SIZE * ITERS).unwrap();
        let mut bump: Bump = Bump::with_capacity(layout);
        b.iter(|| {
            bump.reset();
            for i in 0..ITERS {
                let mut ptr: BumpBox<[u8; 128000]> = bump.alloc([0u8;SIZE]);
                ptr[0] = (i % 256) as u8;
                black_box(ptr[0]);
                black_box(ptr[SIZE - 1]);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_bumpalo,
    bench_lake,
    bench_typed_arena,
    bench_scoped_arena,
    bench_box,
    bench_vec_box,
    bench_heapless,
    bench_mimalloc,
    bench_bump_scope
);
criterion_main!(benches);
