extern crate david_allocator;
extern crate david_set;
extern crate rand;

use std::time::{Instant, Duration};

use rand::{XorShiftRng, SeedableRng, Rand};

use std::collections::HashSet;
use std::collections::BTreeSet;
use david_set::Set;

macro_rules! initialize {
    ($set: ident, $item: ident, $num: expr) => {{
        let mut rng = XorShiftRng::from_seed([1,2,3,4]);
        let before = david_allocator::net_allocation();
        let before_total = david_allocator::total_allocation();
        let mut set = $set::<$item>::new();
        if $num > 0 {
            while set.len() < $num {
                set.insert($item::rand(&mut rng) % (2*$num as $item));
                set.remove(& ($item::rand(&mut rng) % (2*$num as $item)));
            }
        }
        let net_alloced = david_allocator::net_allocation() - before;
        let total_alloced = david_allocator::total_allocation() - before_total;
        let mut unused = $item::rand(&mut rng);
        while set.contains(&unused) {
            unused = $item::rand(&mut rng);
        }
        (set, unused, std::mem::size_of::<$set<$item>>(), net_alloced, total_alloced)
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
    println!("{:10} {:>5} {:>8} ({:>4}/{:>4}/{:>4}) {:>8} ({:4}/{:4}/{:4})",
             "contains", "size",
             "set/hash", "stac", "heap","allo",
             "btree/hash", "stac", "heap","allo");
    for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)) {
        let (set, mut unused, set_stack, set_size, set_total)
            = initialize!(Set, usize, size);
        //println!("size {} {:?}", set.len(), sorted(&set));
        let mut total = 0;
        let set_time = time_me!({
            unused = (unused+1)%(2*size);
            if set.contains(&unused) { total += 1; }
        }, iters);
        let set_total_true = total;

        let (set, mut unused, hash_stack, hash_size, hash_total)
            = initialize!(HashSet, usize, size);
        let mut total = 0;
        let hash_time = time_me!({
            unused = (unused+1)%(2*size);
            if set.contains(&unused) { total += 1; }
        }, iters);
        if total != set_total_true {
            println!("serious problem with hash!");
        }

        let (set, mut unused, btree_stack, btree_size, btree_total)
            = initialize!(BTreeSet, usize, size);
        let mut total = 0;
        let btree_time = time_me!({
            unused = (unused+1)%(2*size);
            if set.contains(&unused) { total += 1; }
        }, iters);
        if total != set_total_true {
            println!("serious problem with btree!");
        }

        println!("{:10} {:5} {:8.5} ({:4.2}/{:4.2}/{:4.2}) {:8.5} ({:4.2}/{:4.2}/{:4.2})",
                 "", size,
                 set_time/hash_time,
                 ((set_stack+set_size) as f64/(hash_stack + hash_size) as f64),
                 (set_size as f64/hash_size as f64),
                 (set_total as f64/hash_total as f64),
                 btree_time/hash_time,
                 ((btree_stack+btree_size) as f64/(hash_stack + hash_size) as f64),
                 (btree_size as f64/hash_size as f64),
                 (btree_total as f64/hash_total as f64));
    }
    println!("{:10} {:>5} {:>8} ({:>4}/{:>4}/{:>4}) {:>8} ({:4}/{:4}/{:4})",
             "remove/ins", "size",
             "set/hash", "stac", "heap","allo",
             "btree/hash", "stac", "heap","allo");
    for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)) {
        let (mut set, _, set_stack, set_size, set_total)
            = initialize!(Set, usize, size);
        let mut next = 0;
        let set_time = time_me!({
            if set.remove(&next) {
                set.insert(next);
            }
            next += 1;
            next %= 2*size;
        }, iters);

        let (mut set, _, hash_stack, hash_size, hash_total)
            = initialize!(HashSet, usize, size);
        let hash_time = time_me!({
            if set.remove(&next) {
                set.insert(next);
            }
            next += 1;
            next %= 2*size;
        }, iters);

        let (mut set, _, btree_stack, btree_size, btree_total)
            = initialize!(BTreeSet, usize, size);
        let btree_time = time_me!({
            if set.remove(&next) {
                set.insert(next);
            }
            next += 1;
            next %= 2*size;
        }, iters);

        println!("{:10} {:5} {:8.5} ({:4.2}/{:4.2}/{:4.2}) {:8.5} ({:4.2}/{:4.2}/{:4.2})",
                 "", size,
                 set_time/hash_time,
                 ((set_stack+set_size) as f64/(hash_stack + hash_size) as f64),
                 (set_size as f64/hash_size as f64),
                 (set_total as f64/hash_total as f64),
                 btree_time/hash_time,
                 ((btree_stack+btree_size) as f64/(hash_stack + hash_size) as f64),
                 (btree_size as f64/hash_size as f64),
                 (btree_total as f64/hash_total as f64));
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
