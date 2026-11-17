use vstd::prelude::*;

verus! {

// Verus syntax is checked with scripts/verify.sh rather than ordinary rustc.
// Capacity is fixed per type and must be positive.
pub struct RingBuffer<const N: usize> {
    slots: [u64; N],
    head: usize,
    len: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub closed spec fn valid(&self) -> bool {
        N > 0 && self.head < N && self.len <= N
    }

    // i is the i-th element to pop
    pub closed spec fn view(&self) -> Seq<u64> {
        Seq::new(self.len as nat, |i: int| self.slots@[
            if self.head + i < N { self.head + i } else { self.head + i - N }
        ])
    }

    pub fn new() -> (result: Self)
        requires N > 0,
        ensures result.valid(), result.view() == Seq::<u64>::empty(),
    {
        Self { slots: [0; N], head: 0, len: 0 }
    }

    pub fn len(&self) -> (result: usize)
        requires self.valid(),
        ensures result == self.view().len(),
    { self.len }

    pub fn is_empty(&self) -> (result: bool)
        requires self.valid(),
        ensures result == (self.view().len() == 0),
    { self.len == 0 }

    pub fn is_full(&self) -> (result: bool)
        requires self.valid(),
        ensures result == (self.view().len() == N),
    { self.len == N }

    pub fn push(&mut self, value: u64) -> (accepted: bool)
        requires old(self).valid(),
        ensures
            final(self).valid(),
            accepted == (old(self).view().len() < N),
            accepted ==> final(self).view() == old(self).view().push(value),
            !accepted ==> final(self).view() == old(self).view(),
    {
        if self.len == N { return false; }
        let ghost before = self.view();
        // Subtract before adding to avoid overflowing usize.
        let remaining = N - self.head;
        let tail = if self.len < remaining {
            self.head + self.len
        } else {
            self.len - remaining
        };
        self.slots[tail] = value;
        self.len += 1;
        true
    }

    pub fn pop(&mut self) -> (result: Option<u64>)
        requires old(self).valid(),
        ensures
            final(self).valid(),
            old(self).view().len() == 0 ==> result == None && final(self).view() == old(self).view(),
            old(self).view().len() > 0 ==> result == Some(old(self).view()[0])
                && final(self).view() == old(self).view().subrange(1, old(self).view().len() as int),
    {
        if self.len == 0 { return None; }
        let ghost before = self.view();
        let value = self.slots[self.head];
        self.head = if self.head == N - 1 { 0 } else { self.head + 1 };
        self.len -= 1;
        Some(value)
    }
}

} // verus!
