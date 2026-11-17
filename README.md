0. Edit [src/lib.rs](src/lib.rs)
1. Run `./scripts/verify.sh`


The buffer supports any positive compile-time capacity, including capacities
that are not powers of two. For example, inside `verus!`:

```rust
let mut buffer = RingBuffer::<13>::new();
let accepted = buffer.push(42);
let value = buffer.pop();
assert(accepted);
assert(value == Some(42));
```

`RingBuffer::<N>::new()` requires `N > 0`. Storage is an inline `[u64; N]`;
capacity stays fixed for the lifetime of the buffer. Push returns `false` when
full, and pop returns `None` when empty. Index calculations avoid `usize`
overflow even for large capacities. Verification covers the generic
implementation for every positive `N`.
