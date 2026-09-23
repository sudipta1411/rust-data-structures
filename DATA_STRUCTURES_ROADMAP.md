# Data structures roadmap

After finishing the AVL tree, implement a **ring buffer** next. It builds on the queue implementation while introducing manual storage management and wrapping indices.

The TigerBeetle files below are references. The order and implementation scope are recommendations for this Rust learning project, not a description of TigerBeetle's exact APIs.

## Suggested progression

| Order | Structure | What to learn | TigerBeetle reference |
|---|---|---|---|
| 1 | Ring buffer | Fixed capacity, wrapping indices, full/empty handling | [ring_buffer.zig](https://github.com/tigerbeetle/tigerbeetle/blob/main/src/stdx/ring_buffer.zig) |
| 2 | Bit set | Pack membership into bits; implement union, intersection, and iteration | [bit_set.zig](https://github.com/tigerbeetle/tigerbeetle/blob/main/src/stdx/bit_set.zig) |
| 3 | Segmented array | Store elements in chunks and manage growth without one contiguous allocation | [segmented_array.zig](https://github.com/tigerbeetle/tigerbeetle/blob/main/src/lsm/segmented_array.zig) |
| 4 | Set-associative cache | Hash keys into sets, handle collisions, and choose eviction policies | [set_associative_cache.zig](https://github.com/tigerbeetle/tigerbeetle/blob/main/src/lsm/set_associative_cache.zig) |
| 5 | Compressed bitmap | Compress runs of bits; compare memory usage against the bit set | [ewah.zig](https://github.com/tigerbeetle/tigerbeetle/blob/main/src/ewah.zig) |
| 6 | Small LSM tree | Combine sorted in-memory data, immutable files, merging, and compaction | [lsm directory](https://github.com/tigerbeetle/tigerbeetle/tree/main/src/lsm) |

## First implementation: ring buffer

Start with a safe Rust representation:

```rust
pub struct RingBuffer<T> {
    slots: Vec<Option<T>>,
    head: usize,
    len: usize,
}
```

Suggested API signatures (implement these inside an `impl<T> RingBuffer<T>` block):

```rust
pub fn new(capacity: usize) -> Self;
pub fn enqueue(&mut self, value: T) -> Result<(), T>;
pub fn dequeue(&mut self) -> Option<T>;
pub fn peek(&self) -> Option<&T>;
pub fn capacity(&self) -> usize;
pub fn is_full(&self) -> bool;
```

Reuse the existing `Collection` trait for `len` and `is_empty`. Return `Err(value)` when full so the caller retains ownership of the rejected item.

Use TigerBeetle to study algorithms and invariants, then write a smaller Rust version. `Vec<Option<T>>` allows a straightforward first implementation entirely in safe Rust.

## Choosing a direction

- **Mastering trees:** finish AVL, then implement a red-black tree.
- **Broader data-structure experience:** follow ring buffer → bit set → segmented array, then continue with caches and storage structures.
