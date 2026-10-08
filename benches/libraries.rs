//! Rough comparisons between tinyset's sets and other set types (`HashSet`,
//! `roaring` and `id-set`), at 20% precision, since most differ by several times.
//!
//! The comparisons that need accuracy, tinyset against its previous release, are
//! in `versions.rs`. Everything else is as there: each operation is compared at
//! three densities (the fraction of `0..max` that is in the set), with one
//! `scaling` group per operation and density and one candidate per set type
//! (`std64`, a `HashSet<u64>`, is the baseline). The groups that build sets also
//! report how much memory that took.

use rand::Rng;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::rc::Rc;
use tinyset::{Set64, SetU32, SetU64};

#[global_allocator]
static ALLOC: scaling::Allocator = scaling::Allocator::new();

/// Random distinct elements drawn from `0..max`, where `max` is chosen so
/// that `n` elements fill a fraction `density` of that range.
fn random_elements(density: f64, n: usize) -> Vec<u64> {
    assert!(density <= 1.0);
    let max = max_element(density, n);
    let mut rng = rand::rng();
    let mut set = HashSet::with_capacity(n);
    while set.len() < n {
        set.insert(rng.random_range(0..max));
    }
    set.into_iter().collect()
}

fn max_element(density: f64, n: usize) -> u64 {
    (n as f64 / density) as u64 + 1
}

/// The same elements, held in every set type being compared.
struct Sets {
    max: u64,
    setu32: SetU32,
    roaring: roaring::RoaringBitmap,
    std32: HashSet<u32>,
    setu64: SetU64,
    set64: Set64<u64>,
    std64: HashSet<u64>,
    idset: id_set::IdSet,
}

impl Sets {
    fn random(density: f64, n: usize) -> Self {
        let v = random_elements(density, n);
        Sets {
            max: max_element(density, n),
            setu32: v.iter().map(|&x| x as u32).collect(),
            roaring: v.iter().map(|&x| x as u32).collect(),
            std32: v.iter().map(|&x| x as u32).collect(),
            setu64: v.iter().copied().collect(),
            set64: v.iter().copied().collect(),
            std64: v.iter().copied().collect(),
            idset: v.iter().map(|&x| x as usize).collect(),
        }
    }
}

/// A set of each type, and a random value to look up in them.
///
/// `scaling` makes a new input for every iteration and clones it for every
/// candidate. Building seven fresh 1000-element sets per iteration would take
/// far longer than the operations being timed, and would need hundreds of
/// megabytes per batch. Instead, each input takes one of a small pool of
/// pre-built sets (shared, not copied) and pairs it with a new random probe.
#[derive(Clone)]
struct Fixture {
    probe: u64,
    sets: Rc<RefCell<Sets>>,
}

const POOL_SIZE: usize = 16;

thread_local! {
    static POOL: RefCell<HashMap<(u64, usize), Vec<Rc<RefCell<Sets>>>>> =
        RefCell::new(HashMap::new());
}

fn fixture(density: f64, n: usize) -> Fixture {
    POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        let sets = pool.entry((density.to_bits(), n)).or_insert_with(|| {
            (0..POOL_SIZE)
                .map(|_| Rc::new(RefCell::new(Sets::random(density, n))))
                .collect()
        });
        let mut rng = rand::rng();
        let sets = sets[rng.random_range(0..POOL_SIZE)].clone();
        let probe = rng.random_range(0..sets.borrow().max);
        Fixture { probe, sets }
    })
}

// Set inputs at three densities: 80%, 5% and 0.1% of the range. Each
// operation has a separate group per density, so that each grid is one
// density across all sizes.

#[scaling::input(
    group(
        "contains_dense",
        "insert_remove_dense",
        "min_dense",
        "max_dense",
        "last_dense",
        "sum_dense"
    ),
    name = "sets",
    sizes(0, 1, 2, 3, 4, 10, 100, 1000)
)]
fn dense_sets(n: usize) -> Fixture {
    fixture(0.8, n)
}

#[scaling::input(
    group(
        "contains_sparse",
        "insert_remove_sparse",
        "min_sparse",
        "max_sparse",
        "last_sparse",
        "sum_sparse"
    ),
    name = "sets",
    sizes(0, 1, 2, 3, 4, 10, 100, 1000)
)]
fn sparse_sets(n: usize) -> Fixture {
    fixture(0.05, n)
}

#[scaling::input(
    group(
        "contains_very_sparse",
        "insert_remove_very_sparse",
        "min_very_sparse",
        "max_very_sparse",
        "last_very_sparse",
        "sum_very_sparse"
    ),
    name = "sets",
    sizes(0, 1, 2, 3, 4, 10, 100, 1000)
)]
fn very_sparse_sets(n: usize) -> Fixture {
    fixture(0.001, n)
}

/// Registers one candidate per set type in each of `$groups`, each evaluating its
/// expression with `$x` bound to the probe and `$s` to its own set.
macro_rules! candidates {
    ($groups:tt -> $ret:ty, |$x:ident, $s:ident| {
        setu32: $setu32:expr,
        roaring: $roaring:expr,
        std32: $std32:expr,
        setu64: $setu64:expr,
        set64: $set64:expr,
        std64: $std64:expr,
        idset: $idset:expr $(,)?
    }) => {
        use super::*;
        candidate!($groups, setu32, $ret, $x, $s, $setu32);
        candidate!($groups, roaring, $ret, $x, $s, $roaring);
        candidate!($groups, std32, $ret, $x, $s, $std32);
        candidate!($groups, setu64, $ret, $x, $s, $setu64);
        candidate!($groups, set64, $ret, $x, $s, $set64);
        #[scaling::bench(group $groups, baseline)]
        fn std64(f: &mut Fixture) -> $ret {
            let $x = f.probe;
            let $s = &mut f.sets.borrow_mut().std64;
            $std64
        }
        candidate!($groups, idset, $ret, $x, $s, $idset);
    };
}

macro_rules! candidate {
    ($groups:tt, $field:ident, $ret:ty, $x:ident, $s:ident, $e:expr) => {
        #[scaling::bench(group $groups, uninteresting)]
        fn $field(f: &mut Fixture) -> $ret {
            let $x = f.probe;
            let $s = &mut f.sets.borrow_mut().$field;
            $e
        }
    };
}

mod contains {
    candidates!(("contains_dense", "contains_sparse", "contains_very_sparse") -> bool, |x, s| {
        setu32: s.contains(x as u32),
        roaring: s.contains(x as u32),
        std32: s.contains(&(x as u32)),
        setu64: s.contains(x),
        set64: s.contains(&x),
        std64: s.contains(&x),
        idset: s.contains(x as usize),
    });
}

/// Inserts the probe and then removes it, or the reverse if it was already
/// present, so that every iteration leaves the shared set as it found it.
mod insert_remove {
    candidates!(("insert_remove_dense", "insert_remove_sparse", "insert_remove_very_sparse") -> bool, |x, s| {
        setu32: { let x = x as u32; if s.insert(x) { s.remove(x) } else { s.remove(x) && s.insert(x) } },
        roaring: { let x = x as u32; if s.insert(x) { s.remove(x) } else { s.remove(x) && s.insert(x) } },
        std32: { let x = x as u32; if s.insert(x) { s.remove(&x) } else { s.remove(&x) && s.insert(x) } },
        setu64: if s.insert(x) { s.remove(x) } else { s.remove(x) && s.insert(x) },
        set64: if s.insert(x) { s.remove(&x) } else { s.remove(&x) && s.insert(x) },
        std64: if s.insert(x) { s.remove(&x) } else { s.remove(&x) && s.insert(x) },
        idset: { let x = x as usize; if s.insert(x) { s.remove(x) } else { s.remove(x) && s.insert(x) } },
    });
}

mod min {
    candidates!(("min_dense", "min_sparse", "min_very_sparse") -> Option<u64>, |_x, s| {
        setu32: s.iter().min().map(|x| x as u64),
        roaring: s.iter().min().map(|x| x as u64),
        std32: s.iter().min().map(|&x| x as u64),
        setu64: s.iter().min(),
        set64: s.iter().min(),
        std64: s.iter().copied().min(),
        idset: s.iter().min().map(|x| x as u64),
    });
}

mod max {
    candidates!(("max_dense", "max_sparse", "max_very_sparse") -> Option<u64>, |_x, s| {
        setu32: s.iter().max().map(|x| x as u64),
        roaring: s.iter().max().map(|x| x as u64),
        std32: s.iter().max().map(|&x| x as u64),
        setu64: s.iter().max(),
        set64: s.iter().max(),
        std64: s.iter().copied().max(),
        idset: s.iter().max().map(|x| x as u64),
    });
}

mod last {
    candidates!(("last_dense", "last_sparse", "last_very_sparse") -> Option<u64>, |_x, s| {
        setu32: s.iter().last().map(|x| x as u64),
        roaring: s.iter().last().map(|x| x as u64),
        std32: s.iter().last().map(|&x| x as u64),
        setu64: s.iter().last(),
        set64: s.iter().last(),
        std64: s.iter().copied().last(),
        idset: s.iter().last().map(|x| x as u64),
    });
}

mod sum {
    candidates!(("sum_dense", "sum_sparse", "sum_very_sparse") -> u64, |_x, s| {
        setu32: s.iter().map(|x| x as u64).sum(),
        roaring: s.iter().map(|x| x as u64).sum(),
        std32: s.iter().map(|&x| x as u64).sum(),
        setu64: s.iter().sum(),
        set64: s.iter().sum(),
        std64: s.iter().sum(),
        idset: s.iter().map(|x| x as u64).sum(),
    });
}

/// Elements to build a set from, for the benchmarks that time construction.
#[derive(Clone)]
struct Elements(Vec<u64>);

#[scaling::input(
    group("collect_dense", "insert_all_dense"),
    name = "elements",
    sizes(1, 10, 100, 1000)
)]
fn dense_elements(n: usize) -> Elements {
    Elements(random_elements(0.8, n))
}

#[scaling::input(
    group("collect_sparse", "insert_all_sparse"),
    name = "elements",
    sizes(1, 10, 100, 1000)
)]
fn sparse_elements(n: usize) -> Elements {
    Elements(random_elements(0.05, n))
}

#[scaling::input(
    group("collect_very_sparse", "insert_all_very_sparse"),
    name = "elements",
    sizes(1, 10, 100, 1000)
)]
fn very_sparse_elements(n: usize) -> Elements {
    Elements(random_elements(0.001, n))
}

/// A metrics function reporting how much memory a set of type `$ty` holds once built.
/// Metrics pair with candidates by return type, so each set type needs a function of its
/// own. A set that fits in its own pointer holds no heap memory.
macro_rules! memory {
    ($name:ident, $ty:ty, $groups:tt) => {
        #[scaling::metrics(group $groups, allocation)]
        fn $name(_set: $ty) -> scaling::Metrics {
            scaling::Metrics::new()
                .net_allocated_bytes()
                .peak_allocated_bytes()
                .allocation_count()
        }
    };
}

/// Registers one candidate per set type in each of `$groups`, each building a set of
/// its type from the elements with `$build` and returning it, so that the metrics
/// function can report how much memory the set holds when it is built.
///
/// Metrics pair with candidates by return type, so each set type needs a metrics
/// function of its own. A set that fits in its own pointer holds no heap memory.
macro_rules! builders {
    ($groups:tt, $build:ident) => {
        use super::*;
        #[scaling::bench(group $groups, uninteresting)]
        fn setu32(e: &mut Elements) -> SetU32 {
            $build::<SetU32, u32>(&e.0, |x| x as u32, |s, x| s.insert(x))
        }
        memory!(setu32_memory, SetU32, $groups);
        #[scaling::bench(group $groups, uninteresting)]
        fn roaring(e: &mut Elements) -> roaring::RoaringBitmap {
            $build::<roaring::RoaringBitmap, u32>(&e.0, |x| x as u32, |s, x| s.insert(x))
        }
        memory!(roaring_memory, roaring::RoaringBitmap, $groups);
        #[scaling::bench(group $groups, uninteresting)]
        fn std32(e: &mut Elements) -> HashSet<u32> {
            $build::<HashSet<u32>, u32>(&e.0, |x| x as u32, |s, x| s.insert(x))
        }
        memory!(std32_memory, HashSet<u32>, $groups);
        #[scaling::bench(group $groups, uninteresting)]
        fn setu64(e: &mut Elements) -> SetU64 {
            $build::<SetU64, u64>(&e.0, |x| x, |s, x| s.insert(x))
        }
        memory!(setu64_memory, SetU64, $groups);
        #[scaling::bench(group $groups, uninteresting)]
        fn set64(e: &mut Elements) -> Set64<u64> {
            $build::<Set64<u64>, u64>(&e.0, |x| x, |s, x| s.insert(x))
        }
        memory!(set64_memory, Set64<u64>, $groups);
        #[scaling::bench(group $groups, baseline)]
        fn std64(e: &mut Elements) -> HashSet<u64> {
            $build::<HashSet<u64>, u64>(&e.0, |x| x, |s, x| s.insert(x))
        }
        memory!(std64_memory, HashSet<u64>, $groups);
        #[scaling::bench(group $groups, uninteresting)]
        fn idset(e: &mut Elements) -> id_set::IdSet {
            $build::<id_set::IdSet, usize>(&e.0, |x| x as usize, |s, x| s.insert(x))
        }
        memory!(idset_memory, id_set::IdSet, $groups);
    };
}

fn collect_into<S: Default + FromIterator<T>, T>(
    v: &[u64],
    convert: impl Fn(u64) -> T,
    _insert: impl Fn(&mut S, T) -> bool,
) -> S {
    v.iter().map(|&x| convert(x)).collect()
}

fn insert_into<S: Default + FromIterator<T>, T>(
    v: &[u64],
    convert: impl Fn(u64) -> T,
    insert: impl Fn(&mut S, T) -> bool,
) -> S {
    let mut s = S::default();
    for &x in v {
        insert(&mut s, convert(x));
    }
    s
}

/// Builds each set with `collect()`.
mod collect {
    builders!(
        ("collect_dense", "collect_sparse", "collect_very_sparse"),
        collect_into
    );
}

/// Builds each set by inserting one element at a time.
mod insert_all {
    builders!(
        (
            "insert_all_dense",
            "insert_all_sparse",
            "insert_all_very_sparse"
        ),
        insert_into
    );
}

// How the cost of building and iterating over a set grows with its size, at
// 5% density.
macro_rules! scaling_benchmarks {
    ($($name:ident: $ty:ty, $elem:ty;)*) => {
        mod collect_scaling {
            use super::*;
            $(
                #[scaling::bench_scaling(nmin = 8, make_input = |n: usize| random_elements(0.05, n))]
                fn $name(v: &mut Vec<u64>) -> usize {
                    v.iter().map(|&x| x as $elem).collect::<$ty>().len() as usize
                }
            )*
        }
        mod sum_scaling {
            use super::*;
            $(
                #[scaling::bench_scaling(
                    nmin = 8,
                    make_input = |n: usize| random_elements(0.05, n).into_iter().map(|x| x as $elem).collect::<$ty>()
                )]
                fn $name(s: &mut $ty) -> u64 {
                    s.iter().map(|x| x as u64).sum()
                }
            )*
        }
    };
}

scaling_benchmarks! {
    setu32: SetU32, u32;
    roaring: roaring::RoaringBitmap, u32;
    setu64: SetU64, u64;
    set64: Set64<u64>, u64;
    idset: id_set::IdSet, usize;
}

// The `HashSet`s iterate over references, so they need their own.
mod std_scaling {
    use super::*;

    #[scaling::bench_scaling(nmin = 8, make_input = |n: usize| random_elements(0.05, n))]
    fn collect_std32(v: &mut Vec<u64>) -> usize {
        v.iter().map(|&x| x as u32).collect::<HashSet<u32>>().len()
    }

    #[scaling::bench_scaling(nmin = 8, make_input = |n: usize| random_elements(0.05, n))]
    fn collect_std64(v: &mut Vec<u64>) -> usize {
        v.iter().copied().collect::<HashSet<u64>>().len()
    }

    #[scaling::bench_scaling(
        nmin = 8,
        make_input = |n: usize| random_elements(0.05, n).into_iter().map(|x| x as u32).collect::<HashSet<u32>>()
    )]
    fn sum_std32(s: &mut HashSet<u32>) -> u64 {
        s.iter().map(|&x| x as u64).sum()
    }

    #[scaling::bench_scaling(
        nmin = 8,
        make_input = |n: usize| random_elements(0.05, n).into_iter().collect::<HashSet<u64>>()
    )]
    fn sum_std64(s: &mut HashSet<u64>) -> u64 {
        s.iter().sum()
    }
}

/// Like `scaling::main!()`, but aiming for 20% precision rather than the
/// default 1%: the comparisons here are mostly several times apart, so a rough
/// ratio is enough.  Setting `TINYSET_BENCH_PRECISION` (as a fraction, `0.05`
/// for 5%) asks for something else.
fn main() -> Result<(), scaling::RegistrationError> {
    let precision = std::env::var("TINYSET_BENCH_PRECISION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.2);
    scaling::Config::default()
        .with_relative_error(precision)
        .with_rough_error(precision.max(0.2))
        .run_and_print()
}
