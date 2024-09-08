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
            whichbit: x.whichbit,
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
    whichbit: u64,
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
            whichbit: 0,
            index: 0,
            last: 0,
            set,
        }
    }
}

impl<T: Borrow<SetU64>> Iterator for Inner<T> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
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
                    while let Some(&x) = a.get(self.index) {
                        while self.whichbit < self.bits {
                            let oldbit = self.whichbit;
                            self.whichbit += 1;
                            if (x & (1 << oldbit)) != 0 {
                                self.sz_left -= 1;
                                return Some(unsplit_u64(x >> self.bits, oldbit, self.bits));
                            }
                        }
                        self.index += 1;
                        self.whichbit = 0;
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
            Internal::Dense { a, .. } => loop {
                if let Some(word) = a.get(self.index) {
                    while self.whichbit < 64 {
                        let bit = self.whichbit;
                        self.whichbit = 1 + bit;
                        if word & (1 << bit) != 0 {
                            self.sz_left -= 1;
                            return Some(((self.index as u64) << 6) + bit as u64);
                        }
                    }
                    self.whichbit = 0;
                    self.index += 1;
                } else {
                    return None;
                }
            },
        }
    }
    #[inline]
    fn last(self) -> Option<Self::Item> {
        match self.set.borrow().internal() {
            Internal::Empty => None,
            Internal::Stack(t) => t.max(),
            Internal::Heap { a, .. } => a
                .into_iter()
                .rev()
                .cloned()
                .filter(|&x| x != 0)
                .map(|x| {
                    x >> self.bits + (x & mask(self.bits as usize)).leading_zeros() as u64 - 63
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
                if self.whichbit == 0 {
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
                if self.whichbit == 0 {
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
