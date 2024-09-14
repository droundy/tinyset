use std::borrow::Borrow;

use super::{mask, unsplit_u32, Internal, SetU32};

impl SetU32 {
    /// Iterate over
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u32> + 'a + std::fmt::Debug {
        self.inner_iter()
    }
}

impl SetU32 {
    fn inner_iter(&self) -> Inner<&SetU32> {
        match self.internal() {
            Internal::Empty => Inner::empty(self),
            Internal::Stack(t) => Inner {
                sz: t.sz as u32,
                sz_left: t.sz as u32,
                stack_bits: t.bits,
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

impl IntoIterator for SetU32 {
    type Item = u32;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        let x = self.inner_iter();
        let inner = Inner {
            sz: x.sz,
            sz_left: x.sz_left,
            bits: x.bits,
            stack_bits: x.stack_bits,
            whichbit: x.whichbit,
            last: x.last,
            index: x.index,
            set: self,
        };
        IntoIter { inner }
    }
}

/// An iterator over a set of `u32`
#[derive(Debug, Clone)]
pub struct IntoIter {
    inner: Inner<SetU32>,
}

impl Iterator for IntoIter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.inner.next()
    }
    fn min(self) -> Option<u32> {
        self.inner.min()
    }
    fn max(self) -> Option<u32> {
        self.inner.max()
    }
    fn last(self) -> Option<u32> {
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
struct Inner<T: Borrow<SetU32>> {
    sz: u32,
    sz_left: u32,
    stack_bits: usize,
    bits: u32,
    whichbit: u32,
    index: usize,
    last: usize,
    set: T,
}

impl<T: Borrow<SetU32>> Inner<T> {
    fn empty(set: T) -> Self {
        Inner {
            sz: 0,
            sz_left: 0,
            stack_bits: 0,
            bits: 0,
            whichbit: 0,
            index: 0,
            last: 0,
            set,
        }
    }
}

impl<T: Borrow<SetU32>> Iterator for Inner<T> {
    type Item = u32;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.set.borrow().internal() {
            Internal::Empty => None,
            Internal::Stack(_) => {
                let bitsplits = super::BITSPLITS[self.sz as usize];
                if self.sz_left > 0 {
                    let nbits = bitsplits[(self.sz - self.sz_left) as usize];
                    let difference = self.stack_bits & mask(nbits as usize) as usize;
                    if self.sz_left == self.sz {
                        self.last = difference;
                    } else {
                        self.last = self.last + 1 + difference
                    }
                    self.stack_bits = self.stack_bits >> nbits;
                    self.sz_left -= 1;
                    Some(self.last as u32)
                } else {
                    None
                }
            }
            Internal::Heap { a, .. } => {
                if self.bits > 0 {
                    while let Some(&x) = a.get(self.index) {
                        while self.whichbit < self.bits as u32 {
                            let oldbit = self.whichbit;
                            self.whichbit += 1;
                            if (x & (1 << oldbit)) != 0 {
                                self.sz_left -= 1;
                                return Some(unsplit_u32(x >> self.bits, oldbit, self.bits as u32));
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
                        return Some(if x == self.bits as u32 { 0 } else { x });
                    }
                }
                None
            }
            Internal::Dense { a, .. } => loop {
                if let Some(word) = a.get(self.index) {
                    while self.whichbit < 32 {
                        let bit = self.whichbit;
                        self.whichbit = 1 + bit;
                        if word & (1 << bit) != 0 {
                            self.sz_left -= 1;
                            return Some(((self.index as u32) << 5) + bit as u32);
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
        if self.sz_left == 0 {
            return None;
        }
        match self.set.borrow().internal() {
            Internal::Empty => None,
            Internal::Stack(t) => t.max(),
            Internal::Heap { a, .. } => a
                .into_iter()
                .rev()
                .cloned()
                .filter(|&x| x != 0)
                .map(|x| x >> self.bits as u32 + (x & mask(self.bits as usize)).leading_zeros() - 31)
                .next(),
            Internal::Big { a, .. } => a
                .into_iter()
                .rev()
                .cloned()
                .filter(|&x| x != 0)
                .map(|x| if x == self.bits as u32 { 0 } else { x })
                .next(),
            Internal::Dense { a, .. } => {
                let zero_words = a.iter().rev().cloned().take_while(|&x| x == 0).count() as u32;
                let zero_bits = a[a.len() - 1 - zero_words as usize].leading_zeros() as u32;
                Some(a.len() as u32 * 32 - zero_bits - 1 - zero_words * 32)
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
                    Some((x >> self.bits as u32) * self.bits as u32 + x.trailing_zeros() as u32)
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
                .map(|x| if x == self.bits as u32 { 0 } else { x })
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
                    let x = a
                        .into_iter()
                        .cloned()
                        .filter(|x| *x != 0)
                        .max()
                        .unwrap();
                    let reference = (x >> self.bits) * self.bits as u32;
                    let m = mask(self.bits as usize);
                    let extra = 31 - (x & m).leading_zeros();
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
                        biggest = biggest.max(if x == self.bits as u32 { 0 } else { x });
                    }
                }
                Some(biggest)
            }
            Internal::Dense { .. } => self.last(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        self.sz_left as usize
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz_left as usize, Some(self.sz_left as usize))
    }
}
