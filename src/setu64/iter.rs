use std::borrow::Borrow;

use super::{mask, unsplit_u64, Internal, SetU64};

impl SetU64 {
    /// Iterate over
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u64> + 'a + std::fmt::Debug {
        self.inner_iter()
    }
}

impl SetU64 {
    fn inner_iter(&self) -> Inner<&SetU64> {
        match self.internal() {
            Internal::Empty => Inner::empty(self),
            Internal::Stack(t) => Inner {
                sz: t.sz as usize,
                sz_left: t.sz as usize,
                bits: t.bits as u64,
                ..Inner::empty(self)
            },
            Internal::Heap { s, .. } => Inner {
                sz: s.sz,
                sz_left: s.sz,
                bits: s.bits,
                ..Inner::empty(self)
            },
            Internal::Big { s, .. } => Inner {
                sz: s.sz,
                sz_left: s.sz,
                bits: s.bits,
                ..Inner::empty(self)
            },
            Internal::Dense { sz, .. } => Inner {
                sz: sz,
                sz_left: sz,
                ..Inner::empty(self)
            },
        }
    }
}

impl IntoIterator for SetU64 {
    type Item = u64;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        let x = self.inner_iter();
        let inner = Inner {
            sz: x.sz,
            sz_left: x.sz_left,
            bits: x.bits,
            cur: x.cur,
            base: x.base,
            last: x.last,
            index: x.index,
            set: self,
        };
        IntoIter { inner }
    }
}

/// An iterator over a set of `u64`
#[derive(Debug, Clone)]
pub struct IntoIter {
    inner: Inner<SetU64>,
}

impl Iterator for IntoIter {
    type Item = u64;
    fn next(&mut self) -> Option<u64> {
        self.inner.next()
    }
    fn min(self) -> Option<u64> {
        self.inner.min()
    }
    fn max(self) -> Option<u64> {
        self.inner.max()
    }
    fn last(self) -> Option<u64> {
        self.inner.last()
    }
    fn count(self) -> usize {
        self.inner.count()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Debug, Clone)]
struct Inner<T: Borrow<SetU64>> {
    sz: usize,
    sz_left: usize,
    bits: u64,
    /// The elements of the current word or bucket that have not been returned
    /// yet, as a bitmap.  Only the `Dense` and `Heap` formats use this.
    cur: u64,
    /// What to add to the number of a bit in `cur` to get its element.
    base: u64,
    /// The next word or bucket to load into `cur`, or the next slot of a `Big` set.
    index: usize,
    last: usize,
    set: T,
}

impl<T: Borrow<SetU64>> Inner<T> {
    fn empty(set: T) -> Self {
        Inner {
            sz: 0,
            sz_left: 0,
            bits: 0,
            cur: 0,
            base: 0,
            index: 0,
            last: 0,
            set,
        }
    }
}

impl<T: Borrow<SetU64>> Inner<T> {
    /// The rest of `next`: moves on to the next word or bucket, or returns the
    /// next element of one of the other formats.  Kept out of line so that
    /// `next` itself is small enough to be inlined into the loop that calls it.
    #[inline(never)]
    fn next_slow(&mut self) -> Option<u64> {
        match self.set.borrow().internal() {
            Internal::Empty => None,
            Internal::Stack(_) => {
                let bitsplits = super::BITSPLITS[self.sz];
                if self.sz_left > 0 {
                    let nbits = bitsplits[self.sz - self.sz_left];
                    let difference = self.bits as usize & super::mask(nbits as usize) as usize;
                    if self.sz_left == self.sz {
                        self.last = difference;
                    } else {
                        self.last = self.last + 1 + difference
                    }
                    self.bits = self.bits >> nbits;
                    self.sz_left -= 1;
                    Some(self.last as u64)
                } else {
                    None
                }
            }
            Internal::Heap { a, .. } => {
                if self.bits > 0 {
                    // The bitmap is in the low `bits` bits and the key above it.
                    let m = mask(self.bits as usize);
                    while let Some(&x) = a.get(self.index) {
                        self.index += 1;
                        if x & m != 0 {
                            self.cur = x & m;
                            self.base = unsplit_u64(x >> self.bits, 0, self.bits);
                            return self.next();
                        }
                    }
                } else {
                    if let Some(&first) = a.get(self.index) {
                        self.index += 1;
                        self.sz_left -= 1;
                        return Some(first);
                    }
                }
                None
            }
            Internal::Big { a, .. } => {
                while let Some(&x) = a.get(self.index) {
                    self.index += 1;
                    if x != 0 {
                        self.sz_left -= 1;
                        return Some(if x == self.bits { 0 } else { x });
                    }
                }
                None
            }
            Internal::Dense { a, .. } => {
                while let Some(&word) = a.get(self.index) {
                    self.index += 1;
                    if word != 0 {
                        self.cur = word;
                        self.base = (self.index as u64 - 1) << 6;
                        return self.next();
                    }
                }
                None
            }
        }
    }
}

impl<T: Borrow<SetU64>> Iterator for Inner<T> {
    type Item = u64;
    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cur != 0 {
            // Return the lowest element left in the current word or bucket.
            let bit = self.cur.trailing_zeros() as u64;
            self.cur &= self.cur - 1;
            self.sz_left -= 1;
            Some(self.base + bit)
        } else {
            self.next_slow()
        }
    }
    #[inline]
    fn last(self) -> Option<Self::Item> {
        match self.set.borrow().internal() {
            Internal::Empty => None,
            Internal::Stack(t) => t.max(),
            // The last element is the highest bit of the last occupied bucket, which
            // is where iterating over the set ends.
            Internal::Heap { a, .. } => a
                .into_iter()
                .rev()
                .cloned()
                .filter(|&x| x != 0)
                .map(|x| {
                    let highest = 63 - (x & mask(self.bits as usize)).leading_zeros() as u64;
                    unsplit_u64(x >> self.bits, highest, self.bits)
                })
                .next(),
            Internal::Big { a, .. } => a
                .into_iter()
                .rev()
                .cloned()
                .filter(|&x| x != 0)
                .map(|x| if x == self.bits { 0 } else { x })
                .next(),
            Internal::Dense { a, .. } => {
                if self.sz_left == 0 {
                    return None;
                }
                let zero_words = a.iter().rev().cloned().take_while(|&x| x == 0).count() as u64;
                let zero_bits = a[a.len() - 1 - zero_words as usize].leading_zeros() as u64;
                Some(a.len() as u64 * 64 - zero_bits - 1 - zero_words * 64)
            }
        }
    }
    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        if self.sz_left == 0 {
            return None;
        }
        match self.set.borrow().internal() {
            Internal::Empty => None,
            Internal::Stack(t) => t.min(),
            Internal::Heap { a, .. } => {
                if self.index == 0 && self.cur == 0 {
                    let x = a.into_iter().cloned().filter(|x| *x != 0).min().unwrap();
                    Some((x >> self.bits) * self.bits + x.trailing_zeros() as u64)
                } else {
                    let mut min = self.next().unwrap();
                    while let Some(x) = self.next() {
                        if x < min {
                            min = x;
                        }
                    }
                    Some(min)
                }
            }
            Internal::Big { a, .. } => a
                .into_iter()
                .cloned()
                .filter(|x| *x != 0)
                .map(|x| if x == self.bits { 0 } else { x })
                .min(),
            Internal::Dense { .. } => self.next(),
        }
    }
    #[inline]
    fn max(mut self) -> Option<Self::Item> {
        if self.sz_left == 0 {
            return None;
        }
        match self.set.borrow().internal() {
            Internal::Empty => None,
            Internal::Stack(t) => t.max(),
            Internal::Heap { a, .. } => {
                if self.index == 0 && self.cur == 0 {
                    let x = a.iter().cloned().filter(|x| *x != 0).max().unwrap();
                    let reference = (x >> self.bits) * self.bits;
                    let m = mask(self.bits as usize);
                    let extra = 63 - (x & m).leading_zeros() as u64;
                    Some(reference + extra)
                } else {
                    let mut max = self.next().unwrap();
                    while let Some(x) = self.next() {
                        if x > max {
                            max = x;
                        }
                    }
                    Some(max)
                }
            }
            Internal::Big { a, .. } => {
                let mut biggest = 0;
                for &x in a {
                    if x != 0 {
                        biggest = biggest.max(if x == self.bits { 0 } else { x });
                    }
                }
                Some(biggest)
            }
            Internal::Dense { .. } => self.last(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        self.sz_left
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz_left, Some(self.sz_left))
    }
}
