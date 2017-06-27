extern crate david_allocator;
extern crate david_set;
extern crate rand;
extern crate smallset;

use std::time::{Instant, Duration};

use rand::{XorShiftRng, SeedableRng, Rand};

use std::collections::HashSet;
use std::collections::BTreeSet;
use david_set::Set;
use david_set::VecSet;
use david_set::CastSet;

type SmallSet<T> = smallset::SmallSet<[T; 8]>;

macro_rules! initialize {
    ($set: ident, $item: ident, $num: expr) => {{
        let mut rng = XorShiftRng::from_seed([$num as u32,$num as u32,3,4]);
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
        (set, std::mem::size_of::<$set<$item>>(), net_alloced, total_alloced)
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

macro_rules! bench_contains {
    ($set: ident, $item: ident, $size: expr, $iters: expr) => {{
        let (set, my_stack, my_size, my_alloc) = initialize!($set, $item, $size);
        let mut total = 0;
        let mut i = 0;
        let my_time = time_me!({
            i = (i+1)%(2*$size as $item);
            if set.contains(&i) { total += 1; }
        }, $iters);
        (total, my_time, my_stack, my_size, my_alloc)
    }};
}

macro_rules! bench_all_contains {
    ($item: ident, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "contains", "size");
        print!("{:^8}(stac/heap/allo)", "set");
        print!("{:^8}(stac/heap/allo)", "vecset");
        print!("{:^8}(stac/heap/allo)", "btree");
        print!("{:^8}(stac/heap/allo)", "smallset");
        print!("{:^8}(stac/heap/allo)", "castset");
        println!();
        for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)
                                  .filter(|&x|x<$maxsz)) {
            print!("{:5}",size);

            let (total_true, hash_time, hash_stack, hash_size, hash_total)
                = bench_contains!(HashSet, $item, size, $iters);
            let (total, my_time, my_stack, my_size, my_total)
                = bench_contains!(Set, $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            let (total, my_time, my_stack, my_size, my_total)
                = bench_contains!(VecSet, $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            let (total, my_time, my_stack, my_size, my_total)
                = bench_contains!(BTreeSet, $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            let (total, my_time, my_stack, my_size, my_total)
                = bench_contains!(SmallSet, $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            let (total, my_time, my_stack, my_size, my_total)
                = bench_contains!(CastSet, $item, size, $iters);
            if total != total_true {
                println!("serious problem!");
            }
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            println!();
        }
    }};
}

macro_rules! bench_remove_insert {
    ($set: ident, $item: ident, $size: expr, $iters: expr) => {{
        let (mut set, my_stack, my_size, my_alloc) = initialize!($set, $item, $size);
        let mut i = 0;
        let my_time = time_me!({
            i = (i+1)%(2*$size as $item);
            if set.remove(&i) { set.insert(i); }
        }, $iters);
        (my_time, my_stack, my_size, my_alloc)
    }};
}

macro_rules! bench_all_remove_insert {
    ($item: ident, $iters: expr, $maxsz: expr) => {{
        print!("{:10}\n---------\n{:>5}", "remove/ins", "size");
        print!("{:^8}(stac/heap/allo)", "set");
        print!("{:^8}(stac/heap/allo)", "vecset");
        print!("{:^8}(stac/heap/allo)", "btree");
        print!("{:^8}(stac/heap/allo)", "smallset");
        print!("{:^8}(stac/heap/allo)", "castset");
        println!();
        for size in (1..15).chain([20,30,50,100,1000,10000].iter().map(|&x|x)
                                  .filter(|&x|x<$maxsz)) {
            print!("{:5}",size);
            let (hash_time, hash_stack, hash_size, hash_total)
                = bench_remove_insert!(HashSet, $item, size, $iters);

            let (my_time, my_stack, my_size, my_total)
                = bench_remove_insert!(Set, $item, size, $iters);
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            let (my_time, my_stack, my_size, my_total)
                = bench_remove_insert!(VecSet, $item, size, $iters);
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            let (my_time, my_stack, my_size, my_total)
                = bench_remove_insert!(BTreeSet, $item, size, $iters);
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            let (my_time, my_stack, my_size, my_total)
                = bench_remove_insert!(SmallSet, $item, size, $iters);
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));

            let (my_time, my_stack, my_size, my_total)
                = bench_remove_insert!(CastSet, $item, size, $iters);
            print!(" {:6.3} ({:4.2}/{:4.2}/{:4.2})",
                   my_time/hash_time,
                   ((my_stack+my_size) as f64/(hash_stack + hash_size) as f64),
                   (my_size as f64/hash_size as f64),
                   (my_total as f64/hash_total as f64));
            println!();
        }
    }};
}

fn main() {
    let iters = 10000000;

    println!("\n==============\n    u8\n==============");
    let maxsz = 250;
    bench_all_contains!(u8, iters, maxsz);
    bench_all_remove_insert!(u8, iters, maxsz);

    let maxsz = 10*iters;
    println!("\n==============\n    u32\n==============");
    bench_all_contains!(u32, iters, maxsz);
    bench_all_remove_insert!(u32, iters, maxsz);

    println!("\n==============\n    usize\n==============");
    bench_all_contains!(usize, iters, maxsz);
    bench_all_remove_insert!(usize, iters, maxsz);
}

fn duration_to_f64(t: Duration) -> f64 {
    t.as_secs() as f64 + (t.subsec_nanos() as f64)*1e-9
}
