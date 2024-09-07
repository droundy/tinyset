use std::ops::Deref;

use super::{mask, unsplit_u64, Internal, SetU64};

impl SetU64 {
    /// Iterate over
    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u64> + 'a + std::fmt::Debug {
        self.private_iter()
    }

    fn private_iter<'a>(&'a self) -> Iter<'a> {
        match self.internal() {
            Internal::Empty => Iter::Empty,
            Internal::Stack(t) => Iter::Stack(t),
            Internal::Heap { s, a } => Iter::Heap(HeapIter {
                sz_left: s.sz,
                bits: s.bits,
                whichbit: 0,
                index: 0,
                array: a,
            }),
            Internal::Big { s, a } => Iter::Big(BigIter {
                sz_left: s.sz,
                bits: s.bits,
                index: 0,
                a,
            }),
            Internal::Dense { a, sz } => Iter::Dense(DenseIter {
                sz_left: sz,
                whichword: 0,
                whichbit: 0,
                a,
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum Iter<'a> {
    Empty,
    Stack(super::Tiny),
    Heap(HeapIter<&'a [u64]>),
    Big(BigIter<&'a [u64]>),
    Dense(DenseIter<&'a [u64]>),
}

impl<'a> Iterator for Iter<'a> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<u64> {
        match self {
            Iter::Empty => None,
            Iter::Stack(ref mut t) => t.next(),
            Iter::Dense(it) => it.next(),
            Iter::Big(it) => it.next(),
            Iter::Heap(it) => it.next(),
        }
    }
    #[inline]
    fn count(self) -> usize {
        match self {
            Iter::Empty => 0,
            Iter::Stack(t) => t.count(),
            Iter::Dense(it) => it.count(),
            Iter::Big(it) => it.count(),
            Iter::Heap(it) => it.count(),
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Iter::Empty => (0, Some(0)),
            Iter::Stack(t) => t.size_hint(),
            Iter::Dense(it) => it.size_hint(),
            Iter::Big(it) => it.size_hint(),
            Iter::Heap(it) => it.size_hint(),
        }
    }
    #[inline]
    fn min(self) -> Option<u64> {
        match self {
            Iter::Empty => None,
            Iter::Stack(t) => t.min(),
            Iter::Dense(it) => it.min(),
            Iter::Big(it) => it.min(),
            Iter::Heap(it) => it.min(),
        }
    }
    #[inline]
    fn max(self) -> Option<u64> {
        match self {
            Iter::Empty => None,
            Iter::Stack(t) => t.max(),
            Iter::Dense(it) => it.max(),
            Iter::Big(it) => it.max(),
            Iter::Heap(it) => it.max(),
        }
    }
}

#[derive(Debug, Clone)]
struct BigIter<V> {
    sz_left: usize,
    bits: u64,
    index: usize,
    a: V,
}

impl<V: Deref<Target = [u64]>> Iterator for BigIter<V> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<u64> {
        while let Some(&x) = self.a.get(self.index) {
            self.index += 1;
            if x != 0 {
                self.sz_left -= 1;
                return Some(if x == self.bits { 0 } else { x });
            }
        }
        None
    }
    #[inline]
    fn last(self) -> Option<u64> {
        self.a
            .into_iter()
            .rev()
            .cloned()
            .filter(|&x| x != 0)
            .map(|x| if x == self.bits { 0 } else { x })
            .next()
    }
    #[inline]
    fn count(self) -> usize {
        self.sz_left
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz_left, Some(self.sz_left))
    }
    #[inline]
    fn min(self) -> Option<u64> {
        if self.sz_left == 0 {
            None
        } else {
            self.a
                .into_iter()
                .cloned()
                .filter(|x| *x != 0)
                .map(|x| if x == self.bits { 0 } else { x })
                .min()
        }
    }
}

#[derive(Debug, Clone)]
struct HeapIter<a> {
    sz_left: usize,
    bits: u64,
    whichbit: u64,
    index: usize,
    array: a,
}

impl<V: Deref<Target = [u64]>> Iterator for HeapIter<V> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bits > 0 {
            while let Some(&x) = self.array.get(self.index) {
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
            if let Some(&first) = self.array.get(self.index) {
                self.index += 1;
                self.sz_left -= 1;
                return Some(first);
            }
        }
        None
    }
    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.array
            .into_iter()
            .rev()
            .cloned()
            .filter(|&x| x != 0)
            .map(|x| x >> self.bits + (x & mask(self.bits as usize)).leading_zeros() as u64 - 63)
            .next()
    }
    #[inline]
    fn count(self) -> usize {
        self.sz_left
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz_left, Some(self.sz_left))
    }
    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        if self.sz_left == 0 {
            None
        } else if self.whichbit == 0 {
            let x = self
                .array
                .into_iter()
                .cloned()
                .filter(|x| *x != 0)
                .min()
                .unwrap();
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
    #[inline]
    fn max(mut self) -> Option<Self::Item> {
        if self.sz_left == 0 {
            None
        } else if self.whichbit == 0 {
            let x = self
                .array
                .into_iter()
                .cloned()
                .filter(|x| *x != 0)
                .max()
                .unwrap();
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
}

#[derive(Debug, Clone)]
struct DenseIter<V> {
    sz_left: usize,
    whichword: usize,
    whichbit: u64,
    a: V,
}

impl<V: Deref<Target = [u64]>> Iterator for DenseIter<V> {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(word) = self.a.get(self.whichword) {
                while self.whichbit < 64 {
                    let bit = self.whichbit;
                    self.whichbit = 1 + bit;
                    if word & (1 << bit) != 0 {
                        self.sz_left -= 1;
                        return Some(((self.whichword as u64) << 6) + bit as u64);
                    }
                }
                self.whichbit = 0;
                self.whichword += 1;
            } else {
                return None;
            }
        }
    }
    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.sz_left == 0 {
            return None;
        }
        let zero_words = self.a.iter().rev().cloned().take_while(|&x| x == 0).count() as u64;
        let zero_bits = self.a[self.a.len() - 1 - zero_words as usize].leading_zeros() as u64;
        Some(self.a.len() as u64 * 64 - zero_bits - 1 - zero_words * 64)
    }
    #[inline]
    fn max(self) -> Option<Self::Item> {
        self.last()
    }
    #[inline]
    fn count(self) -> usize {
        self.sz_left
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.sz_left, Some(self.sz_left))
    }
    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl IntoIterator for SetU64 {
    type Item = u64;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        let iter = unsafe { std::mem::transmute(self.private_iter()) };
        IntoIter { iter, _set: self }
    }
}
/// An iterator over a set of `u64`.
#[derive(Debug, Clone)]
pub struct IntoIter {
    iter: Iter<'static>,
    _set: SetU64,
}

impl Iterator for IntoIter {
    type Item = u64;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
    #[inline]
    fn last(self) -> Option<Self::Item> {
        self.iter.last()
    }
    #[inline]
    fn min(self) -> Option<Self::Item> {
        self.iter.min()
    }
    #[inline]
    fn max(self) -> Option<Self::Item> {
        self.iter.max()
    }
    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
