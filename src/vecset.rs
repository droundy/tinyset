use std;
use std::hash::Hash;
use std::borrow::Borrow;

/// A set that is stored in a Vec
#[derive(Debug, Clone)]
pub struct VecSet<T> { v: Vec<T> }
impl<T: Eq> VecSet<T> {
    /// Creates an empty set..
    pub fn new() -> VecSet<T> {
        VecSet { v: Vec::new() }
    }
    /// Returns the number of elements in the set.
    pub fn len(&self) -> usize {
        self.v.len()
    }
    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned.
    pub fn insert(&mut self, elem: T) -> bool {
        if self.v.contains(&elem) {
            false
        } else {
            self.v.push(elem);
            true
        }
    }
    /// Returns true if the set contains a value.
    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
        where
        T: Borrow<Q>, Q: Hash + Eq,
    {
        for v in self.v.iter() {
            if v.borrow() == value {
                return true;
            }
        }
        false
    }
    /// Removes an element, and returns true if that element was present.
    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
        where
        T: Borrow<Q>, Q: Hash + Eq,
    {
        for i in 0..self.v.len() {
            if self.v[i].borrow() == value {
                self.v.swap_remove(i);
                return true
            }
        }
        false
    }
    /// Returns an iterator over the set.
    pub fn iter(&self) -> Iter<T> {
        Iter( self.v.iter() )
    }
}

pub struct Iter<'a,T: 'a>(std::slice::Iter<'a,T>);
impl<'a, T: 'a+Eq> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    #[test]
    fn it_works() {
        let mut ss: VecSet<usize> = VecSet::new();
        ss.insert(5);
        assert!(ss.contains(&5));
        assert!(!ss.contains(&4));
        ss.insert(3);
        println!("now {:?}", &ss);
        assert!(ss.contains(&3));
        assert!(ss.contains(&5));
        assert!(ss.len() == 2);
        for num in ss.iter() {
            assert!(ss.contains(num));
        }
    }
    #[test]
    fn size_unwasted() {
        println!("small size: {}", std::mem::size_of::<VecSet<usize>>());
        println!(" hash size: {}", std::mem::size_of::<HashSet<usize>>());
        assert!(std::mem::size_of::<VecSet<usize>>() <=
                2*std::mem::size_of::<HashSet<usize>>());
    }
}
