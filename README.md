# Lake

> A fast, zero-cost, and deeply controllable memory system — not just an allocator, but a memory lake.

**Lake** is a linear memory management system designed for extreme performance, zero-cost abstractions, and full control. Inspired by bump allocators — but not limited by them — Lake offers a rich API for precise and predictable memory handling.

Unlike traditional arenas or bump allocators, Lake is built for high-performance, real-time, and system-level workloads where memory layout and allocation offsets matter. It enables:

- 🧠 **Manual control over allocation offsets**
- 🧵 **Thread-local memory lakes**
- ⚡ **Zero-cost Droplets with size, offset, and safety guards**
- 🧩 **Dynamic and static views into memory**
- 📦 **Zero-copy deserialization**
- 🔥 **Unsafe freedom when you need it**

Whether you're building a high-frequency trading engine, a zero-copy deserialization layer, or a custom HTTP server handling millions of requests per second — **Lake gives you the tools without getting in your way**.

This is **not** a GC, not a toy arena, and not another abstraction.  
This is your memory — and you’re in charge.

### ⚠️ Disclaimer: Power Comes With Responsibility ###

```Lake aims to be developer-friendly, but at its core, it is a low-level memory management tool.```

While most APIs are safe and ergonomic, incorrect usage can lead to panics or undefined behavior (UB) — especially when bypassing safe abstractions or misusing droplets and snapshots.

If you treat `Lake` with care — using the API correctly and understanding its semantics — you'll be rewarded with:
* 🧨 Blazing performance
* 🧠 Full control over memory layout
* 😎 Surprisingly fun developer experience

But misuse it — and your service might panic, crash, or worse.

```⚠️ Respect the lake. Know where you're swimming.```

## **⚡ Benchmark**
| Crate            | Avg time (ns)     | 95% interval                  |
|------------------|-------------------|-------------------------------|
| **lake**         | **2_229.48**      | 2_220.35 – 2_241.00           |
| **mimalloc**     | **5_660.63**      | 5_610.67 – 5_739.64           |
| **box_heap**     | **2_371_303.85**  | 2_364_276.32 – 2_378_224.65   |
| **heapless_vec** | **5_104_258.91**  | 5_074_069.63 – 5_136_300.93   |
| **bump-scope**   | **8_064_223.05**  | 7_936_244.90 – 8_233_482.12   |
| **bumpalo**      | **8_424_838.02**  | 8_392_666.31 – 8_459_892.26   |
| **scoped-arena** | **19_889_320.26** | 19_716_379.07 – 20_112_506.23 |
| **typed-arena**  | **20_121_930.44** | 19_938_222.96 – 20_332_554.67 |
| **vec_box**      | **36_881_416.34** | 36_304_472.75 – 37_555_579.94 |

## **🚀 Key Features**

### **🧠 Zero-Cost Arena Allocation with Rewind Semantics**

* **High-speed linear allocator** backed by a fixed-size preallocated buffer.
* **No deallocation** — just rewind: perfect for transient workloads, parsers, or encoders.
* **Scoped memory** control via:
    * `.mark() `, `.reset_to_mark()`, `.move_mark()` for checkpoint-style rewinding.
    * `.reset()` or `Drop` for full rewind of the arena.
* **Droplets**:
    * Typed or dynamically sized memory chunks with safe lifetime & generation tracking.
    * Auto-rewinds the lake when they're the last allocation.
* **Thread-local support**:  
```rust  
thread_lake_init();  

with_lake!(|lake| {  
    let d = lake.alloc::<256>().unwrap();  
    // ...  
});  
```

Lightweight, global-free access to arena memory per thread.

### **📌 Markers – Rewind Stack for Nested Scopes** ###
* **Push/pop-based rewind system for managing temporary allocations:**
  * `.mark()` — save current offset (push)
  * `.reset_to_mark()` — rewind to last mark (pop)
  * `.move_mark()` — update last mark to current position
* **Fully nestable:** suitable for recursive parsers, alloc-backed decision trees, or scope-local scratch buffers.
* **No memory is copied** — only the offset changes.

### **🕰️ Snapshots – Save/Restore Lake State** ###
* **Value-based rewind mechanism (vs. stack-style markers):**
  * `.snapshot()` **→** returns a lightweight LakeSnapshot
  * `.rewind(snapshot)` **→** restores exact state

### 💧 Droplets - Smartly Unsafe Memory Access
* **Droplet<N>:**
  * Fixed-size memory chunk, type-safe access, minimal overhead.
  * Usable like `[u8; N]`, optionally leakable into `'static` slice.
* **DropletDyn<SIZE>:**
  * Runtime-sized memory slice returned from `.process()` closures.
    * `process(f)` for dynamic data: Create a `DropletDyn` from a closure-generated buffer (e.g. serialize-once, write-once patterns).
  * Ideal for intermediate buffers, JSON payloads, or transformed data.
* **Safety-first under the hood:**
  * Generation + offset guards prevent use-after-free or reuse bugs.
  * `.is_valid()` to check droplet liveness.
* **Optional zero-copy leak (unsafe):**
```rust
let static_ref: &'static [u8; 128] = unsafe { droplet.leak() };
```

### 🌊 LakeView – Forkable Sub-Allocators
* **Lightweight**, non-owning slice of a lake — a "tributary" for local, scoped allocations.
* **Supports:**
  * `.alloc<N>()`, `.process()`, `.alloc_struct<T>()`, `.alloc_slice<T>()`
* **Recursive forkable views** via `.split(len)` — ideal for:
  * Recursive descent parsers
  * AST node-local arenas
  * Streaming transformations
* **Independent rewinding:**
  * `.reset()`, `.mark()`, and `.clear()` work per-view.
  * Droplets allocated from a view never affect the parent lake.
* Optional `.set_zeroing(true)` to wipe memory on reuse — useful for:
  * Cryptography
  * Sandbox isolation
  * Sensitive parsing

### 🔒 Sandbox Mode – Commit or Revert Scoped Allocations ###
* **Create temporary allocation scopes** inside any Lake or LakeView.
* Use `sandbox()` to enter a scope — all allocations are virtual until committed.
* Explicit control flow:
  * `.commit()` — keep the changes (offset += delta)
  * `Drop` — rollback automatically if not committed
* **Allows:**
  * Speculative encoders/parsers with no memory leak
  * Branch-local memory usage (like match arms or error-prone transforms)
  * Clean undo logic with zero allocations
  * (Think of it as a lightweight, memory-only transactional scope.)

### 🌐 Thread-local Lake — Zero-Config Per-Thread Arena ###
* One-line setup with `thread_lake_init()`
* Use the `with_lake! { ... }` macro to access the thread's private Lake
* All features available: `alloc`, `droplets`, `sandbox`, `mark`, etc.
* Avoids global contention and enables scoped high-speed parsing/processing per thread

### 🛠️ Utilities & Safety ###
* `align_up(offset, align)` — Minimal overhead alignment helper for struct and slice placement
  * **→** Used in `alloc_struct<T>` and `alloc_slice<T>` for correct in-place layout
* FBC! **macro** — Forget-but-Controlled:
  * Safely promotes values to `'static` lifetime by leaking them in a `Box`, wrapped in a transparent type to preserve `Send`/`Sync` correctness.
  * Ideal for one-time config, string interning, or static singletons without global mutability:
```rust
static CONFIG: &mut MyConfig = FBC!(MyConfig::new());
```
* `FBCWrap<T>` — Transparent wrapper used by `FBC!`, safe for multithreaded use via Send + Sync
* **These utilities exist to support high-performance and controlled unsafe memory usage patterns in Lake without sacrificing correctness or safety gates.**