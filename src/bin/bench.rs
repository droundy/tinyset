extern crate david_set;
extern crate rand;

use std::time::{Instant, Duration};

use rand::{XorShiftRng, SeedableRng, Rand};

use std::collections::HashSet;
use std::collections::BTreeSet;
use david_set::Set;

macro_rules! initialize {
    ($set: ident, $item: ident, $num: expr) => {{
        let mut set = $set::<$item>::new();
        let mut rng = XorShiftRng::from_seed([1,2,3,4]);
        if $num > 0 {
            while set.len() < $num {
                set.insert($item::rand(&mut rng) % (2*$num as $item));
                set.remove(& ($item::rand(&mut rng) % (2*$num as $item)));
            }
        }
        let mut unused = $item::rand(&mut rng);
        while set.contains(&unused) {
            unused = $item::rand(&mut rng);
        }
        let all_present: Vec<_> = set.iter().map(|&x| x).collect();
        let present = if $num > 0 {
            all_present[usize::rand(&mut rng) % all_present.len()]
        } else {
            $item::rand(&mut rng)
        };
        (set, unused, present)
    }};
}

macro_rules! time_me {
    ($fn: expr, $num: expr) => {{
        let now = Instant::now();
        for _ in 0..$num {
            $fn;
        }
        duration_to_f64(now.elapsed())
    }};
}

fn sorted<T: IntoIterator>(iter: T) -> Vec<T::Item>
    where T::Item: Ord + Eq
{
    let mut v: Vec<_> = iter.into_iter().collect();
    v.sort();
    v
}

fn main() {
    let iters = 10000000;
    println!("{:10} {:>5} {:>8} {:>8} {:>15} {:>15}",
             "contains", "size", "set/hash", "btree/hash", "set (s)", "hash (s)");
    for size in (0..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)) {
        let (set, mut unused, _) = initialize!(Set, usize, size);
        //println!("size {} {:?}", set.len(), sorted(&set));
        let set_time = time_me!({unused = unused+1; set.contains(&unused)}, iters);

        let (set, mut unused, _) = initialize!(HashSet, usize, size);
        let hash_time = time_me!({unused = unused+1; set.contains(&unused)}, iters);

        let (set, mut unused, _) = initialize!(BTreeSet, usize, size);
        let btree_time = time_me!({unused = unused+1; set.contains(&unused)}, iters);

        println!("{:10} {:5} {:8.5} {:8.5} {:15.6} {:15.6}",
                 "", size, set_time/hash_time, btree_time/hash_time, set_time, hash_time);
    }
    println!("{:10}{:>6}{:>9}{:>9}{:>16}{:>16}",
             "remove/insert", "size", "set/hash", "btree/hash", "set (s)", "hash (s)");
    for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)) {
        let (mut set, _, _) = initialize!(Set, usize, size);
        let mut next = 0;
        let set_time = time_me!({
            if set.remove(&next) {
                set.insert(next);
            }
            next += 1;
            next %= 2*size;
        }, iters);

        let (mut set, _, _) = initialize!(HashSet, usize, size);
        let hash_time = time_me!({
            if set.remove(&next) {
                set.insert(next);
            }
            next += 1;
            next %= 2*size;
        }, iters);

        let (mut set, _, _) = initialize!(BTreeSet, usize, size);
        let btree_time = time_me!({
            if set.remove(&next) {
                set.insert(next);
            }
            next += 1;
            next %= 2*size;
        }, iters);

        println!("{:10} {:5} {:8.5} {:8.5} {:15.6} {:15.6}",
                 "", size, set_time/hash_time, btree_time/hash_time, set_time, hash_time);
    }
}

fn duration_to_f64(t: Duration) -> f64 {
    t.as_secs() as f64 + (t.subsec_nanos() as f64)*1e-9
}

fn pretty_time(t: f64) -> String {
    if t < 1e-7 {
        format!("{:7.2} ns", t/1e-9)
    } else if t < 1e-4 {
        format!("{:7.2} us", t/1e-6)
    } else {
        format!("{:7.2} ms", t/1e-3)
    }
}
