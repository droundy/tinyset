pub trait AnyMap : Default + Clone {
    type Key: Copy + Eq + Ord + std::fmt::Display + std::fmt::Debug;
    type Elem: Clone + Eq + Ord + std::fmt::Display + std::fmt::Debug;
    fn ins(&mut self, k: Self::Key, v: Self::Elem) -> Option<Self::Elem>;
    fn rem(&mut self, k: Self::Key) -> Option<Self::Elem>;
    fn con(&self, k: Self::Key) -> bool;
    fn vec(&self) -> Vec<(Self::Key, Self::Elem)>;
    fn ln(&self) -> usize;
    fn ge(&self, k: Self::Key) -> Option<&Self::Elem>;
}

impl<K: Copy + Eq + Ord + std::fmt::Display + std::fmt::Debug + std::hash::Hash,
     V: Clone + Eq + Ord + std::fmt::Display + std::fmt::Debug> AnyMap for std::collections::HashMap<K, V> {
    type Key = K;
    type Elem = V;
    fn ins(&mut self, k: Self::Key, v: Self::Elem) -> Option<Self::Elem> {
        self.insert(k, v)
    }
    fn rem(&mut self, k: Self::Key) -> Option<Self::Elem> {
        self.remove(&k)
    }
    fn ge(&self, k: Self::Key) -> Option<&Self::Elem> {
        self.get(&k)
    }
    fn con(&self, k: Self::Key) -> bool {
        self.contains_key(&k)
    }
    fn vec(&self) -> Vec<(Self::Key, Self::Elem)> {
        self.iter().map(|(k,v)| (*k, v.clone())).collect()
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
    fn check_string_maps(slice: Vec<(u64,String)>) {
        check_map::<std::collections::HashMap<u64,String>>(&slice);
    }
    #[test]
    fn check_u8_maps(slice: Vec<(u8,i8)>) {
        check_map::<std::collections::HashMap<u8,i8>>(&slice);
    }
    #[test]
    fn check_i8_maps(slice: Vec<(i8,u8)>) {
        check_map::<std::collections::HashMap<i8,u8>>(&slice);
    }
}

#[cfg(test)]
pub fn check_map<T: AnyMap>(elems: &[(T::Key, T::Elem)]) {
    println!("\n\ncheck_map {:?}\n", elems);
    let mut s = T::default();
    let mut count = 0;
    for (k,v) in elems.iter().cloned() {
        let has_k = s.con(k);
        let changed_something = s.ins(k,v.clone());
        if changed_something.is_none() {
            count += 1;
            println!("    {} is new now count {}", k, count);
        }
        assert_eq!(has_k, changed_something.is_some());
        println!("what is this? count {} does it have {}?", count, k);
        assert_eq!(Some(&v), s.ge(k));
        assert!(s.con(k));
        assert_eq!(s.ln(), count);
        assert_eq!(s.vec().into_iter()
                   .count(), count);
    }
    assert!(elems.len() >= s.ln());
    println!("map {:?} with length {}", elems, s.ln());
    for (k,v) in s.vec().into_iter() {
        println!("    {}: {}", k, v);
    }
    assert_eq!(s.vec().into_iter().count(), s.ln());
    for (k,_) in s.vec().into_iter() {
        println!("found {}", k);
        assert!(elems.iter().any(|(kk,_)| kk == &k));
    }
    for (k,_) in elems.iter().cloned() {
        println!("YYYY looking for {}", k);
        assert!(s.con(k));
    }
    for (k,_) in elems.iter().cloned() {
        println!("removing {}", k);
        s.rem(k);
    }
    for (k,_) in elems.iter().cloned() {
        println!("XXXX looking for {}", k);
        assert!(!s.con(k));
    }
    assert_eq!(s.ln(), 0);
}

