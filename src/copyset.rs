pub trait CopySet : Default {
    type Item: Copy + Eq + Ord + std::fmt::Display + std::fmt::Debug;
    fn ins(&mut self, e: Self::Item) -> bool;
    fn rem(&mut self, e: Self::Item) -> bool;
    fn con(&self, e: Self::Item) -> bool;
    fn vec(&self) -> Vec<Self::Item>;
    fn ln(&self) -> usize;
}

impl CopySet for std::collections::HashSet<u64> {
    type Item = u64;
    fn ins(&mut self, e: u64) -> bool {
        self.insert(e)
    }
    fn rem(&mut self, e: u64) -> bool {
        self.remove(&e)
    }
    fn con(&self, e: u64) -> bool {
        self.contains(&e)
    }
    fn vec(&self) -> Vec<u64> {
        self.iter().cloned().collect()
    }
    fn ln(&self) -> usize {
        self.len()
    }
}

impl CopySet for std::collections::HashSet<u32> {
    type Item = u32;
    fn ins(&mut self, e: u32) -> bool {
        self.insert(e)
    }
    fn rem(&mut self, e: u32) -> bool {
        self.remove(&e)
    }
    fn con(&self, e: u32) -> bool {
        self.contains(&e)
    }
    fn vec(&self) -> Vec<u32> {
        self.iter().cloned().collect()
    }
    fn ln(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
use proptest::prelude::*;
#[cfg(test)]
proptest!{
    #[test]
    fn check_random_sets(slice in prop::collection::vec(1u64..5, 1usize..10)) {
        check_set::<std::collections::HashSet<u64>>(&slice);
    }
    #[test]
    fn check_medium_sets(slice in prop::collection::vec(1u64..255, 1usize..100)) {
        check_set::<std::collections::HashSet<u64>>(&slice);
    }
    #[test]
    fn check_big_sets(slice: Vec<u64>) {
        check_set::<std::collections::HashSet<u64>>(&slice);
    }
}

pub fn check_set<T: CopySet>(elems: &[T::Item]) {
    println!("\n\ncheck_set {:?}\n", elems);
    let mut s = T::default();
    let mut count = 0;
    for x in elems.iter().cloned() {
        let was_here = s.con(x);
        let changed_something = s.ins(x);
        if changed_something {
            count += 1;
            println!("    {} is new now count {}", x, count);
        }
        assert_eq!(!was_here, changed_something);
        println!("what is this? count {} does it have {}?", count, x);
        assert!(s.con(x));
        assert_eq!(s.ln(), count);
        assert_eq!(s.vec().into_iter()
.count(), count);
    }
    assert!(elems.len() >= s.ln());
    assert_eq!(elems.iter().cloned().min(),
               s.vec().into_iter().min());
    println!("set {:?} with length {}", elems, s.ln());
    for x in s.vec().into_iter() {
        println!("    {}", x);
    }
    assert_eq!(s.vec().into_iter().count(), s.ln());
    for x in s.vec().into_iter() {
        println!("looking for {}", x);
        assert!(elems.contains(&x));
    }
    for x in s.vec().into_iter() {
        println!("found {}", x);
        assert!(elems.contains(&x));
    }
    for x in elems.iter().cloned() {
        println!("YYYY looking for {}", x);
        assert!(s.con(x));
    }
    for x in elems.iter().cloned() {
        println!("removing {}", x);
        s.rem(x);
    }
    for x in elems.iter().cloned() {
        println!("XXXX looking for {}", x);
        assert!(!s.con(x));
    }
    assert_eq!(s.ln(), 0);
}

