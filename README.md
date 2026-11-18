# A ring buffer verified with Flux

A FIFO queue of `u64` values with any positive compile-time capacity, constant-time
push and pop, inline array storage, and no unsafe code. A full queue rejects a
push with `false`; an empty queue returns `None` from pop. All slots are usable.

```rust
use verified_ringbuffer::RingBuffer;

let mut queue = RingBuffer::<13>::new();
assert!(queue.push(42));
assert_eq!(queue.pop(), Some(42));
```

Capacity must be explicit: use `RingBuffer::<N>::new()`.
Zero capacity violates the Flux constructor precondition and panics in ordinary
Rust builds.

## Run it

```sh
cargo run --example demo
cargo test
bash scripts/verify-flux.sh
python3 scripts/check-flux-mutations.py
bash scripts/check.sh
```

Ordinary Cargo commands need only stable Rust and have no dependencies.
`check.sh` runs formatting, tests, Clippy, Flux, and the separate Verus example.
Verification scripts fail when a verifier is missing or a proof fails.

## The Flux implementation

Read `src/lib.rs`. The private `Slots<N>` wrapper contains a real `[u64; N]`.
Its refinement index is `Map<int, int>`, a mathematical map from physical array
indices to values. The map exists only in specifications; no map is allocated
at runtime. The wrapper uses Flux's
[opaque-struct pattern](https://flux-rs.github.io/flux/guide/specifications.html#opaque-structs).

Three small methods form the trusted boundary:

- `Slots::new`: initializes the array to zero, modeled by `map_default(0)`.
- `Slots::get`: reads index `i < N`, returning `map_select(m, i)`.
- `Slots::set`: writes index `i < N`, updating the model to `map_store(m, i, v)`.

Flux assumes those three contracts rather than verifying their bodies. Their
bodies are ordinary safe Rust array operations. Queue correctness is conditional
on these contracts being accurate; the behavioral tests also exercise storage.
The queue's constructor, accessors, push, pop, and default constructor are
checked, not trusted.

`RingBuffer<N>` tracks three refinement indices: `head`, `len`, and
`contents: Map<int, int>`. Its fields connect the runtime array and counters to
those indices. The invariants state:

```text
0 < N <= usize::MAX
0 <= head < N
0 <= len <= N
```

The constructor contract explicitly includes the upper bound because this Flux
version needs it to establish machine-integer overflow bounds for symbolic `N`.
Every Rust `usize` capacity already satisfies that upper bound.

Logical position `i` ranges over `[0, len)`. Its physical slot is
`(head + i) % N`. For example, capacity five with head three and length three
occupies slots `3, 4, 0`, in that order.

The specification function `index(h, i, capacity)` expresses wraparound using
one conditional subtraction. On valid logical positions this equals modulo:
`h + i < 2 * capacity`. This form lets the solver use linear arithmetic.
`item(m, h, i, capacity)` selects the value at that logical position.

Executable push computes the tail without adding potentially large counters:

```rust
let remaining = N - self.head;
let tail = if self.len < remaining {
    self.head + self.len
} else {
    self.len - remaining
};
```

Pop advances the head to zero at the last slot, otherwise adding one. Flux
checks these operations with strict overflow and no-panic checking.

## Contents and FIFO proofs

There are two distinct layers in `src/lib.rs`.

`view(buffer, i, capacity)` maps a logical queue position to the value in its
physical array model. Only positions `0 <= i < buffer.len` are live. It is a
function of the actual modeled storage, not an independent ghost queue that
could drift away from it.

The abstract predicates describe queue behavior at an arbitrary position `i`:

- `fifo_push`: increase length when space exists, preserve every old logical
  value, and append the supplied value. When full, preserve logical contents
  and length.
- `fifo_pop`: reduce length when nonempty and shift logical values left by one.
  An empty queue stays logically empty.

Neither predicate specifies a head update, physical write location, or equality
of physical storage maps. Pop's return contract separately specifies the old
logical first value and whether `Some` is returned:

```text
Option<u64[view(before, 0, N)]>[before.len > 0]
```

`push_step` and `pop_step` describe this implementation's concrete transitions.
Flux checks the method bodies against these postconditions. Then the checked
lemmas `push_refines_fifo` and `pop_refines_fifo` prove that the concrete steps
imply the abstract predicates for an arbitrary `i` and capacity `N`. Indices
outside the live range impose no contents requirement.

`clearing_pop_refines_fifo` demonstrates why the abstract contract is weaker:
it proves that clearing the removed physical slot to zero also satisfies the
same `fifo_pop` predicate. The executable pop currently leaves storage unchanged;
this alternative is a checked transition lemma, not another executable method.

The generic `fifo_client` checks that pushing arbitrary values `x` and `y`, then
popping three times, returns `x`, `y`, and `None`, for every capacity `N >= 2`.
This client illustrates composition; the general FIFO argument comes from the
constructor, invariants, and the arbitrary-state step proofs, rather than from
this one example history.

### Backend limitation

The desired abstract public postconditions would quantify over all positions:

```text
forall i { fifo_push(before, after, value, N, i) }
forall i { fifo_pop(before, after, N, i) }
```

The installed Flux SMT backend rejects unbounded quantifiers with:
`unbounded quantifiers are only supported with the lean backend`.
Consequently the actual public signatures still expose the concrete steps, and
their abstract meaning is proved separately, pointwise. This is not a fully
abstract public interface. No new trusted queue methods or axioms are introduced
to work around the limitation. The three storage helpers remain trusted.

Run `bash scripts/verify-flux.sh`: **10 checked, 3 trusted, 0 ignored**.
The four proof/client functions are included only during Flux verification;
they add no runtime code. The script loads the standard-library contracts,
enables strict overflow and no-panic checks, and forces rechecking.

Correctness after arbitrary finite histories follows inductively: construction
establishes a valid empty queue; each operation preserves validity and has the
proved abstract effect for any incoming valid state. This argument remains
conditional on the trusted storage contracts and verifier/library models.

## Separate Verus example

`verification/verus/ring_buffer.rs` remains an independent eight-slot executable
Verus implementation, with a mathematical sequence specification. Run it with
`bash scripts/verify-verus.sh`. It is not the Cargo library and has no formal
equivalence proof connecting it to the generic Flux implementation.

Both examples are single-threaded queues. Their proofs rely on their respective
verifier, solver, and library models.

## Tool setup

This workspace uses `cargo-flux 942a386 (2026-10-20)` with nightly-2026-10-11.
The official [Flux installation guide](https://flux-rs.github.io/flux/guide/install.html)
describes installation and solver prerequisites. The source revision is
`942a386cf93b00636f8a73c57c1a5f068114ebad` in
[flux-rs/flux](https://github.com/flux-rs/flux).
Ensure `cargo-flux`, `flux`, and `z3` are on PATH. The verification script uses
`$HOME/.flux` unless `FLUX_SYSROOT` is set.

The separate Verus example uses `0.2026.10.18.7d82994` with Rust 1.99.1,
installed under `.tools/verus-x86-linux` (ignored by Git). To install on Linux
x86-64:

```sh
mkdir -p .tools
curl -fL 'https://github.com/verus-lang/verus/releases/download/release/0.2026.10.18.7d82994/verus-0.2026.10.18.7d82994-x86-linux.zip' -o .tools/verus.zip
echo 'e9af03732d4bdd32b93a8fcf471706739c24c1473fa5086618d96c47237845dd  .tools/verus.zip' | sha256sum -c -
unzip .tools/verus.zip -d .tools
rustup toolchain install 1.99.1 --profile minimal
bash scripts/verify-verus.sh
```

For other platforms, follow the official
[Verus installation guide](https://github.com/verus-lang/verus/blob/main/INSTALL.md)
and set `VERUS_BIN` to the executable's absolute path. Keep its companion files
alongside it. Upgrading either verifier may require annotation changes.
