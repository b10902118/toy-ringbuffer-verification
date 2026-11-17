//! An eight-slot FIFO queue. See README.md for the verification walkthrough.
#![forbid(unsafe_code)]
#![cfg_attr(flux, flux::defs {
    // Total mathematical functions; executable get/set require an index < 8.
    fn slot(s: Slots, i: int) -> int {
        if i == 0 { s.s0 }
        else if i == 1 { s.s1 }
        else if i == 2 { s.s2 }
        else if i == 3 { s.s3 }
        else if i == 4 { s.s4 }
        else if i == 5 { s.s5 }
        else if i == 6 { s.s6 }
        else { s.s7 }
    }
    fn store(s: Slots, i: int, v: int) -> Slots {
        {
            s0: if i == 0 { v } else { s.s0 },
            s1: if i == 1 { v } else { s.s1 },
            s2: if i == 2 { v } else { s.s2 },
            s3: if i == 3 { v } else { s.s3 },
            s4: if i == 4 { v } else { s.s4 },
            s5: if i == 5 { v } else { s.s5 },
            s6: if i == 6 { v } else { s.s6 },
            s7: if i == 7 { v } else { s.s7 },
        }
    }
    fn item(s: Slots, h: int, i: int) -> int { slot(s, (h + i) % 8) }
    // Logical queue element: item(contents, head, i), for 0 <= i < len.
    fn pushed(b: RingBuffer, a: RingBuffer, v: int) -> bool {
        a.head == b.head &&
        a.len == (if b.len < 8 { b.len + 1 } else { b.len }) &&
        a.contents == (if b.len < 8 { store(b.contents, (b.head + b.len) % 8, v) } else { b.contents }) &&
        (forall i in 0 .. 8 {
            i < b.len => item(a.contents, a.head, i) == item(b.contents, b.head, i)
        }) &&
        (b.len < 8 => item(a.contents, a.head, b.len) == v)
    }
    fn popped(b: RingBuffer, a: RingBuffer) -> bool {
        a.head == (if b.len > 0 { (b.head + 1) % 8 } else { b.head }) &&
        a.len == (if b.len > 0 { b.len - 1 } else { 0 }) &&
        a.contents == b.contents &&
        (forall i in 0 .. 8 {
            i < a.len => item(a.contents, a.head, i) == item(b.contents, b.head, i + 1)
        })
    }
})]

// Individually refined slots let Flux track each write without a trusted array wrapper.
#[cfg_attr(flux, flux::refined_by(s0: int, s1: int, s2: int, s3: int, s4: int, s5: int, s6: int, s7: int))]
struct Slots {
    #[cfg_attr(flux, flux::field(u64[s0]))]
    s0: u64,
    #[cfg_attr(flux, flux::field(u64[s1]))]
    s1: u64,
    #[cfg_attr(flux, flux::field(u64[s2]))]
    s2: u64,
    #[cfg_attr(flux, flux::field(u64[s3]))]
    s3: u64,
    #[cfg_attr(flux, flux::field(u64[s4]))]
    s4: u64,
    #[cfg_attr(flux, flux::field(u64[s5]))]
    s5: u64,
    #[cfg_attr(flux, flux::field(u64[s6]))]
    s6: u64,
    #[cfg_attr(flux, flux::field(u64[s7]))]
    s7: u64,
}

impl Slots {
    #[cfg_attr(flux, flux::sig(fn() -> Slots[{ s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0, s6: 0, s7: 0 }]))]
    fn new() -> Self {
        Self {
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
        }
    }

    #[cfg_attr(flux, flux::sig(fn(&Slots[@s], i: usize{i < 8}) -> u64[slot(s, i)]))]
    fn get(&self, i: usize) -> u64 {
        match i {
            0 => self.s0,
            1 => self.s1,
            2 => self.s2,
            3 => self.s3,
            4 => self.s4,
            5 => self.s5,
            6 => self.s6,
            _ => self.s7,
        }
    }

    #[cfg_attr(flux, flux::sig(fn(self: &strg Slots[@s], i: usize{i < 8}, v: u64) ensures self: Slots[store(s, i, v)]))]
    fn set(&mut self, i: usize, v: u64) {
        match i {
            0 => self.s0 = v,
            1 => self.s1 = v,
            2 => self.s2 = v,
            3 => self.s3 = v,
            4 => self.s4 = v,
            5 => self.s5 = v,
            6 => self.s6 = v,
            _ => self.s7 = v,
        }
    }
}

pub const CAPACITY: usize = 8;

/// A bounded queue: full pushes return false without overwriting unread values.
/// Fixed capacity and u64 elements keep the first verification exercise small.
#[cfg_attr(flux, flux::refined_by(head: int, len: int, contents: Slots))]
#[cfg_attr(flux, flux::invariant(0 <= head && head < 8))]
#[cfg_attr(flux, flux::invariant(0 <= len && len <= 8))]
pub struct RingBuffer {
    #[cfg_attr(flux, flux::field(Slots[contents]))]
    slots: Slots,
    #[cfg_attr(flux, flux::field(usize[head]))]
    head: usize,
    #[cfg_attr(flux, flux::field(usize[len]))]
    len: usize,
}

impl RingBuffer {
    #[cfg_attr(flux, flux::sig(fn() -> RingBuffer[0, 0, { s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0, s6: 0, s7: 0 }]))]
    pub fn new() -> Self {
        Self {
            slots: Slots::new(),
            head: 0,
            len: 0,
        }
    }

    #[cfg_attr(flux, flux::sig(fn(&RingBuffer[@h, @n, @s]) -> usize[n]))]
    pub fn len(&self) -> usize {
        self.len
    }

    #[cfg_attr(flux, flux::sig(fn(&RingBuffer[@h, @n, @s]) -> bool[n == 0]))]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[cfg_attr(flux, flux::sig(fn(&RingBuffer[@h, @n, @s]) -> bool[n == 8]))]
    pub fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    /// Appends a value, or returns false when full.
    #[cfg_attr(flux, flux::sig(fn(self: &strg RingBuffer[@before], value: u64) -> bool[before.len < 8]
        ensures self: RingBuffer{after: pushed(before, after, value)}))]
    pub fn push(&mut self, value: u64) -> bool {
        if self.len == CAPACITY {
            return false;
        }
        // Both operands are < 8, so the addition cannot overflow.
        let tail = (self.head + self.len) % CAPACITY;
        self.slots.set(tail, value);
        self.len += 1;
        true
    }

    /// Removes the oldest value, or returns None when empty.
    #[cfg_attr(flux, flux::sig(fn(self: &strg RingBuffer[@before])
        -> Option<u64[item(before.contents, before.head, 0)]>[before.len > 0]
        ensures self: RingBuffer{after: popped(before, after)}))]
    pub fn pop(&mut self) -> Option<u64> {
        if self.len == 0 {
            return None;
        }
        let value = self.slots.get(self.head);
        self.head = (self.head + 1) % CAPACITY;
        self.len -= 1;
        Some(value)
    }
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self::new()
    }
}
