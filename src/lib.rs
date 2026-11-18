//! A fixed-capacity FIFO queue, verified with Flux. See README.md.
#![forbid(unsafe_code)]
#![cfg_attr(flux, flux::defs {
    // Mathematical arithmetic: no machine-integer overflow in specifications.
    fn index(h: int, i: int, capacity: int) -> int {
        if h + i < capacity { h + i } else { h + i - capacity }
    }
    fn item(m: Map<int, int>, h: int, i: int, capacity: int) -> int {
        map_select(m, index(h, i, capacity))
    }
    // Representation function: logical position i reads the actual storage model.
    // Only positions 0 <= i < b.len belong to the logical queue.
    fn view(b: RingBuffer, i: int, capacity: int) -> int {
        item(b.contents, b.head, i, capacity)
    }

    fn push_spec(b: RingBuffer, a: RingBuffer, v: int, capacity: int) -> bool {
        a.head == b.head &&
        a.len == (if b.len < capacity { b.len + 1 } else { b.len }) &&
        a.contents == (if b.len < capacity {
            map_store(b.contents, index(b.head, b.len, capacity), v)
        } else { b.contents })
    }
    fn pop_spec(b: RingBuffer, a: RingBuffer, capacity: int) -> bool {
        a.head == (if b.len > 0 { index(b.head, 1, capacity) } else { b.head }) &&
        a.len == (if b.len > 0 { b.len - 1 } else { 0 }) &&
        a.contents == b.contents
    }
})]

// Somehow has to wrap an array in a struct and define the constructor, getter and setter.
// https://github.com/PLSysSec/tock/blob/master/flux_support/src/flux_arr.rs
// There ring buffer verification only covers indices
// https://github.com/PLSysSec/tock/blob/master/kernel/src/collections/ring_buffer.rs
#[cfg_attr(flux, flux::opaque)]
#[cfg_attr(flux, flux::refined_by(vals: Map<int, int>))]
struct Slots<const N: usize> {
    inner: [u64; N],
}

// The only trusted boundary: an actual array modeled by a mathematical map.
impl<const N: usize> Slots<N> {
    #[cfg_attr(flux, flux::trusted)]
    #[cfg_attr(flux, flux::sig(fn() -> Slots<N>[map_default(0)]))]
    fn new() -> Self {
        Self { inner: [0; N] }
    }

    #[cfg_attr(flux, flux::trusted)]
    #[cfg_attr(flux, flux::sig(fn(&Slots<N>[@m], i: usize{i < N}) -> u64[map_select(m, i)]))]
    fn get(&self, i: usize) -> u64 {
        self.inner[i]
    }

    #[cfg_attr(flux, flux::trusted)]
    #[cfg_attr(flux, flux::sig(fn(self: &strg Slots<N>[@m], i: usize{i < N}, v: u64)
        ensures self: Slots<N>[map_store(m, i, v)]))]
    fn set(&mut self, i: usize, v: u64) {
        self.inner[i] = v;
    }
}

/// A queue with positive, compile-time capacity N. All N slots are usable.
#[cfg_attr(flux, flux::refined_by(head: int, len: int, contents: Map<int, int>))]
#[cfg_attr(flux, flux::invariant(N > 0 && N <= usize::MAX))]
#[cfg_attr(flux, flux::invariant(0 <= head && head < N))]
#[cfg_attr(flux, flux::invariant(0 <= len && len <= N))]
pub struct RingBuffer<const N: usize> {
    #[cfg_attr(flux, flux::field(Slots<N>[contents]))]
    slots: Slots<N>,
    #[cfg_attr(flux, flux::field(usize[head]))]
    head: usize,
    #[cfg_attr(flux, flux::field(usize[len]))]
    len: usize,
}

impl<const N: usize> RingBuffer<N> {
    #[cfg_attr(flux, flux::sig(fn() -> RingBuffer<N>[0, 0, map_default(0)] requires N > 0 && N <= usize::MAX))]
    #[allow(clippy::new_without_default)] // Construction requires a positive explicit capacity.
    pub fn new() -> Self {
        assert!(N > 0, "ring buffer capacity must be positive");
        Self {
            slots: Slots::new(),
            head: 0,
            len: 0,
        }
    }

    #[cfg_attr(flux, flux::sig(fn(&RingBuffer<N>[@h, @n, @m]) -> usize[n]))]
    pub fn len(&self) -> usize {
        self.len
    }

    #[cfg_attr(flux, flux::sig(fn(&RingBuffer<N>[@h, @n, @m]) -> bool[n == 0]))]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[cfg_attr(flux, flux::sig(fn(&RingBuffer<N>[@h, @n, @m]) -> bool[n == N]))]
    pub fn is_full(&self) -> bool {
        self.len == N
    }

    /// Appends a value, or returns false without changing the queue when full.
    #[cfg_attr(flux, flux::sig(fn(self: &strg RingBuffer<N>[@before], value: u64)
        -> bool[before.len < N]
        ensures self: RingBuffer<N>{after: push_spec(before, after, value, N)}))]
    pub fn push(&mut self, value: u64) -> bool {
        if self.len == N {
            return false;
        }
        // Equivalent to (head + len) % N, without overflowing usize.
        let remaining = N - self.head;
        let tail = if self.len < remaining {
            self.head + self.len
        } else {
            self.len - remaining
        };
        self.slots.set(tail, value);
        self.len += 1;
        true
    }

    /// Removes the oldest value, or returns None without changing an empty queue.
    #[cfg_attr(flux, flux::sig(fn(self: &strg RingBuffer<N>[@before])
        -> Option<u64[view(before, 0, N)]>[before.len > 0]
        ensures self: RingBuffer<N>{after: pop_spec(before, after, N)}))]
    pub fn pop(&mut self) -> Option<u64> {
        if self.len == 0 {
            return None;
        }
        let value = self.slots.get(self.head);
        self.head = if self.head == N - 1 { 0 } else { self.head + 1 };
        self.len -= 1;
        Some(value)
    }
}
