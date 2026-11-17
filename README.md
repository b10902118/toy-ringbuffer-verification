# Learning verification with a Rust ring buffer

An eight-slot FIFO queue of `u64` values, with constant-time push and pop,
no heap allocation, and no unsafe code. A full queue rejects a push with
`false`; an empty queue returns `None` from pop. All eight slots are usable.
Capacity and element type are deliberately fixed to keep the first proofs small.

## Run it

```sh
cargo run --example demo     # Some(10), Some(20), None
cargo test
bash scripts/verify-flux.sh
bash scripts/verify-verus.sh
bash scripts/check.sh        # formatting, tests, Clippy, both verifiers
```

Ordinary Cargo commands need only stable Rust and have no dependencies.
The verification commands require the tools described below. They fail when
a verifier is missing or a proof fails; they do not skip verification.
The Verus script verifies a library (`--crate-type=lib`), so it needs no
`main` function and does not compile or run a demonstration executable.

## Start reading here

1. `src/lib.rs`: executable Rust plus conditional Flux annotations.
2. `tests/behavior.rs`: boundary/wraparound tests and 100,000 deterministic
   operations compared with `std::collections::VecDeque`.
3. `verification/verus/ring_buffer.rs`: a separate executable Verus version,
   with a mathematical sequence specification and FIFO proofs.

These are two implementations of the same algorithm. Flux checks the actual
Cargo library. Verus checks its own implementation; there is **no formal
equivalence proof** connecting the two files. Keep their runtime operations
in sync when experimenting. Passing `cargo test` alone runs neither verifier.

## Understand the representation

`head` is the physical index of the oldest element; `len` is the number of
live elements. Logical element `i` lives at `(head + i) % 8`.

For example, with `head = 6` and `len = 4`, the queue occupies physical slots
`6, 7, 0, 1`, in that order. The next push writes slot `2`. A pop reads slot
`6` and advances the head to `7`. Old values in unused slots are irrelevant;
they are not erased, and `len` prevents reading them as queue elements.

The representation invariant is:

```text
0 <= head < 8
0 <= len <= 8
```

The constructor establishes it. Every public operation preserves it. On a
successful push, `head + len <= 14`; on pop, `head + 1 <= 8`. These bounds
also prove machine-integer arithmetic cannot overflow.

## What Flux proves

Flux now verifies **FIFO contents as well as state and arithmetic safety**.
The implementation uses eight named `u64` fields in a private `Slots` struct
instead of a Rust array. This lets each physical slot have an exact refinement,
without trusting an array wrapper. The public API, eight-slot capacity, circular
indexing, and constant-time operations are unchanged. Verus still uses an array;
there is no formal equivalence proof between these storage representations.

Read the specification-only definitions at the top of `src/lib.rs`:

- `slot(s, i)` selects a physical slot's value.
- `store(s, i, v)` describes replacing that slot while retaining every other value.
- `item(s, h, i)` selects logical queue element `i` using circular indexing.
- `pushed(before, after, value)` specifies length, head, exact storage changes,
  preservation of every old logical element, and the appended value.
- `popped(before, after)` specifies the head/length change, unchanged physical
  storage, and that each remaining element equals the next old element.

`forall i in 0 .. 8` checks all eight logical positions, guarded by the live
length. This finite quantification covers the entire fixed-capacity queue;
values and operation sequences are not restricted to test samples.

`Slots` has eight logical indices, each connected to a real field using
`field(u64[s0])`, etc. Its `get` and `set` bodies are checked against exact
read/write contracts. These are **not trusted functions**.

`RingBuffer` now has indices `head`, `len`, and `contents: Slots`.
`RingBuffer[@before]` binds the entire incoming refinement record;
`before.contents` accesses its physical storage model. `&strg` permits a strong
update, and `ensures self: RingBuffer{after: ...}` constrains the outgoing record.

Pop's return type is:

```text
Option<u64[item(before.contents, before.head, 0)]>[before.len > 0]
```

The inner refinement specifies the returned value when present; the outer
Boolean specifies that `Some` occurs exactly when the old queue was nonempty.
Thus returning `Some(0)`, returning `None` for a nonempty queue, or writing the
wrong value on push violates the checked contracts.

Run `bash scripts/verify-flux.sh`. It enables **strict overflow checking**,
**no-panic checking**, and Flux's standard-library contracts (including
`Option`'s presence index). It uses `$HOME/.flux` unless `FLUX_SYSROOT` is set.
A fresh run reports **10 checked, 0 trusted, 0 ignored**: seven library methods
plus three storage helpers. This count describes our code, not the trusted
standard-library models or verifier internals.

`[package.metadata.flux] enabled = true` is essential: without it, a successful
`cargo flux` can mean only compilation. The script clears Flux build artifacts
to make rechecking visible. `cfg_attr(flux, ...)` keeps ordinary stable Rust
builds free of verification dependencies.

Run `python3 scripts/check-flux-mutations.py` to check that deliberately broken
copies are rejected. It also verifies an unchanged baseline and leaves the
working source untouched.

## The Verus specification

`valid()` expresses the representation invariant. `view()` constructs an
abstract `Seq<u64>` from the live physical slots. This sequence exists only
in specifications; it allocates nothing at runtime.

The contracts say:

| Operation | Required observable behavior |
| --- | --- |
| `new` | Valid queue with empty sequence |
| Successful `push(x)` | New sequence equals old sequence with `x` appended |
| Rejected `push(x)` | Sequence unchanged; rejection occurs exactly when full |
| Nonempty `pop` | Returns the old first element; leaves the old sequence's suffix |
| Empty `pop` | Returns `None`; sequence unchanged |

`old(self)` means the incoming state; `final(self)` means the outgoing
state in a mutable-reference postcondition. `requires` states a caller's
obligation and `ensures` states what the implementation must establish.

The optional proof blocks are currently commented out: Verus verifies these
contracts automatically. In those explanatory blocks,
`let ghost before = self.view()` remembers the mathematical queue without
runtime storage. The quantified assertions show each existing logical
element is preserved or shifted as appropriate. `=~=` asks for sequence
equality by equal length and equal elements. The verifier checks these
assertions; they are not assumptions. The optional client example (currently
commented out) demonstrates that a client can
enqueue 10 and 20 and dequeue them in that order using the contracts.

The Verus source contains no `assume`, `admit`, or `external_body`. Both
proofs still rely on their verifier, solver, and standard-library models.
They do not establish concurrency safety for a lock-free algorithm,
compiler correctness, or hardware behavior. This queue is single-threaded.

## Tool setup and versions

Verified in this workspace with:

- Stable Rust 1.98.1 for Cargo tests and Clippy.
- `cargo-flux 942a386 (2026-10-20)`, using nightly-2026-10-11.
- Verus `0.2026.10.18.7d82994`, using Rust 1.99.1.

Flux was already installed here. Its official [installation guide](https://flux-rs.github.io/flux/guide/install.html)
documents installation and solver prerequisites. To reproduce the source
revision, clone [flux-rs/flux](https://github.com/flux-rs/flux), check out
`942a386cf93b00636f8a73c57c1a5f068114ebad`, and follow that revision's
installation instructions. Ensure `cargo-flux`, `flux`, and `z3` are on
your PATH. This run used Z3 4.12.2; the upstream guide recommends newer Z3.

Verus is installed locally under `.tools/verus-x86-linux` (ignored by Git).
For another Linux x86-64 checkout:

```sh
mkdir -p .tools
curl -fL 'https://github.com/verus-lang/verus/releases/download/release/0.2026.10.18.7d82994/verus-0.2026.10.18.7d82994-x86-linux.zip' -o .tools/verus.zip
echo 'e9af03732d4bdd32b93a8fcf471706739c24c1473fa5086618d96c47237845dd  .tools/verus.zip' | sha256sum -c -
unzip .tools/verus.zip -d .tools
rustup toolchain install 1.99.1 --profile minimal
bash scripts/verify-verus.sh
```

For other platforms, follow the official [Verus installation guide](https://github.com/verus-lang/verus/blob/main/INSTALL.md)
and set `VERUS_BIN` to the absolute path of the installed executable. Keep
the release's companion files alongside it. Tool versions matter because
both verifiers evolve; upgrading may require annotation changes.

## Exercises: break a proof, then repair it

Work on one change at a time, then revert it before the next exercise.

1. Remove `% CAPACITY` from the Rust push index. Run Flux: it should reject
   the violated storage index precondition after wraparound.
2. Change Rust's `self.len += 1` to `self.len += 2`. Flux should reject
   the state transition or invariant.
3. Change either implementation's pop to return `Some(0)`. Its verifier should reject FIFO correctness
   even though returning zero cannot cause an out-of-bounds access.
4. Add a `peek` method: it must leave the queue unchanged and return its
   first element when nonempty. Write the contract before the body.
5. Generalize the capacity (Flux's named slots and finite quantifiers must also change). Decide what capacity zero means, and revisit
   overflow: `(head + len) % capacity` is not automatically safe for
   arbitrarily large capacities. A branch-based index calculation can
   avoid that addition overflow.

Tests exercise particular executions; the proofs cover every state allowed
by their invariants and contracts. Comparing their failures is a useful way
to learn the difference between safety and functional correctness.
