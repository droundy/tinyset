//! Accurate comparisons of tinyset's current sets against the previous release
//! (built from tinyset 0.5.4 as `tinyset_old`), at 1% precision, for checking
//! whether a change made things faster, slower or larger.
//!
//! Every group has two candidates: the previous release's type (`old_*`, the
//! baseline) and the current one, so each comparison is the pair that matters,
//! measured directly. There is one group per operation, density and type, named
//! like `contains_sparse_setu64`. Density is the fraction of `0..max` that is in
//! the set: dense is 80%, sparse 5% and very sparse 0.1%. The groups that build
//! sets (`collect_*`, `insert_all_*`) also report how much memory that took. The
//! comparisons against other crates are in `libraries.rs`.

// The table layout of a tinyset set depends on random choices made when it
// grows, so two types holding the same elements can be laid out differently,
// and then differ in speed without any difference in code. With
// `deterministic_iteration` those choices depend only on the history of
// inserts and removes, which the old and current types share, so run this
// bench with
//
//     cargo bench --bench versions --no-default-features --features deterministic_iteration
#[cfg(feature = "rand")]
compile_error!(
    "versions.rs compares tables that must be laid out alike: run it with \
     --no-default-features --features deterministic_iteration"
);

use rand::Rng;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::rc::Rc;
use tinyset::{Fits64, Set64, SetU32, SetU64};

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

/// The same elements, held in each type being compared.
struct Sets {
    max: u64,
    setu32: SetU32,
    setu64: SetU64,
    set64: Set64<u64>,
    old_setu32: tinyset_old::SetU32,
    old_setu64: tinyset_old::SetU64,
    old_set64: tinyset_old::Set64<u64>,
}

impl Sets {
    fn random(density: f64, n: usize) -> Self {
        let v = random_elements(density, n);
        Sets {
            max: max_element(density, n),
            setu32: v.iter().map(|&x| x as u32).collect(),
            setu64: v.iter().copied().collect(),
            set64: v.iter().copied().collect(),
            old_setu32: v.iter().map(|&x| x as u32).collect(),
            old_setu64: v.iter().copied().collect(),
            old_set64: v.iter().copied().collect(),
        }
    }
}

/// A set of each type, and a random value to look up in them.
///
/// `scaling` makes a new input for every iteration and clones it for every
/// candidate. Building six fresh 1000-element sets per iteration would take
/// far longer than the operations being timed, so each input takes one of a
/// small pool of pre-built sets (shared, not copied) and pairs it with a new
/// random probe.
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

// Set inputs at three densities. Each input serves every operation's groups
// at that density, for every type.
#[scaling::input(
    group(
        "contains_dense_setu32",
        "contains_dense_setu64",
        "contains_dense_set64",
        "insert_remove_dense_setu32",
        "insert_remove_dense_setu64",
        "insert_remove_dense_set64",
        "min_dense_setu32",
        "min_dense_setu64",
        "min_dense_set64",
        "max_dense_setu32",
        "max_dense_setu64",
        "max_dense_set64",
        "last_dense_setu32",
        "last_dense_setu64",
        "last_dense_set64",
        "sum_dense_setu32",
        "sum_dense_setu64",
        "sum_dense_set64"
    ),
    name = "sets",
    sizes(0, 1, 2, 3, 4, 10, 100, 1000)
)]
fn dense_sets(n: usize) -> Fixture {
    fixture(0.8, n)
}

#[scaling::input(
    group(
        "contains_sparse_setu32",
        "contains_sparse_setu64",
        "contains_sparse_set64",
        "insert_remove_sparse_setu32",
        "insert_remove_sparse_setu64",
        "insert_remove_sparse_set64",
        "min_sparse_setu32",
        "min_sparse_setu64",
        "min_sparse_set64",
        "max_sparse_setu32",
        "max_sparse_setu64",
        "max_sparse_set64",
        "last_sparse_setu32",
        "last_sparse_setu64",
        "last_sparse_set64",
        "sum_sparse_setu32",
        "sum_sparse_setu64",
        "sum_sparse_set64"
    ),
    name = "sets",
    sizes(0, 1, 2, 3, 4, 10, 100, 1000)
)]
fn sparse_sets(n: usize) -> Fixture {
    fixture(0.05, n)
}

#[scaling::input(
    group(
        "contains_very_sparse_setu32",
        "contains_very_sparse_setu64",
        "contains_very_sparse_set64",
        "insert_remove_very_sparse_setu32",
        "insert_remove_very_sparse_setu64",
        "insert_remove_very_sparse_set64",
        "min_very_sparse_setu32",
        "min_very_sparse_setu64",
        "min_very_sparse_set64",
        "max_very_sparse_setu32",
        "max_very_sparse_setu64",
        "max_very_sparse_set64",
        "last_very_sparse_setu32",
        "last_very_sparse_setu64",
        "last_very_sparse_set64",
        "sum_very_sparse_setu32",
        "sum_very_sparse_setu64",
        "sum_very_sparse_set64"
    ),
    name = "sets",
    sizes(0, 1, 2, 3, 4, 10, 100, 1000)
)]
fn very_sparse_sets(n: usize) -> Fixture {
    fixture(0.001, n)
}

/// Registers the previous release's `$old` (the baseline) and the current `$new`
/// in `$groups`, each evaluating the expression with `$x` bound to the probe
/// and `$s` to its own set.
macro_rules! pair {
    ($groups:tt -> $ret:ty, $old:ident, $new:ident, |$x:ident, $s:ident| $e:expr) => {
        #[scaling::bench(group $groups, baseline)]
        fn $old(f: &mut Fixture) -> $ret {
            let $x = f.probe;
            let $s = &mut f.sets.borrow_mut().$old;
            $e
        }
        #[scaling::bench(group $groups)]
        fn $new(f: &mut Fixture) -> $ret {
            let $x = f.probe;
            let $s = &mut f.sets.borrow_mut().$new;
            $e
        }
    };
}

mod contains {
    use super::*;
    pair!(("contains_dense_setu32", "contains_sparse_setu32", "contains_very_sparse_setu32") -> bool, old_setu32, setu32, |x, s| s.contains(x as u32));
    pair!(("contains_dense_setu64", "contains_sparse_setu64", "contains_very_sparse_setu64") -> bool, old_setu64, setu64, |x, s| s.contains(x));
    pair!(("contains_dense_set64", "contains_sparse_set64", "contains_very_sparse_set64") -> bool, old_set64, set64, |x, s| s.contains(&x));
}

/// Inserts the probe and then removes it, or the reverse if it was already
/// present, so that every iteration leaves the shared set as it found it.
mod insert_remove {
    use super::*;
    pair!(("insert_remove_dense_setu32", "insert_remove_sparse_setu32", "insert_remove_very_sparse_setu32") -> bool, old_setu32, setu32, |x, s| { let x = x as u32; if s.insert(x) { s.remove(x) } else { s.remove(x) && s.insert(x) } });
    pair!(("insert_remove_dense_setu64", "insert_remove_sparse_setu64", "insert_remove_very_sparse_setu64") -> bool, old_setu64, setu64, |x, s| if s.insert(x) { s.remove(x) } else { s.remove(x) && s.insert(x) });
    pair!(("insert_remove_dense_set64", "insert_remove_sparse_set64", "insert_remove_very_sparse_set64") -> bool, old_set64, set64, |x, s| if s.insert(x) { s.remove(&x) } else { s.remove(&x) && s.insert(x) });
}

mod min {
    use super::*;
    pair!(("min_dense_setu32", "min_sparse_setu32", "min_very_sparse_setu32") -> Option<u64>, old_setu32, setu32, |_x, s| s.iter().min().map(|x| x as u64));
    pair!(("min_dense_setu64", "min_sparse_setu64", "min_very_sparse_setu64") -> Option<u64>, old_setu64, setu64, |_x, s| s.iter().min());
    pair!(("min_dense_set64", "min_sparse_set64", "min_very_sparse_set64") -> Option<u64>, old_set64, set64, |_x, s| s.iter().min());
}

mod max {
    use super::*;
    pair!(("max_dense_setu32", "max_sparse_setu32", "max_very_sparse_setu32") -> Option<u64>, old_setu32, setu32, |_x, s| s.iter().max().map(|x| x as u64));
    pair!(("max_dense_setu64", "max_sparse_setu64", "max_very_sparse_setu64") -> Option<u64>, old_setu64, setu64, |_x, s| s.iter().max());
    pair!(("max_dense_set64", "max_sparse_set64", "max_very_sparse_set64") -> Option<u64>, old_set64, set64, |_x, s| s.iter().max());
}

mod last {
    use super::*;
    pair!(("last_dense_setu32", "last_sparse_setu32", "last_very_sparse_setu32") -> Option<u64>, old_setu32, setu32, |_x, s| s.iter().last().map(|x| x as u64));
    pair!(("last_dense_setu64", "last_sparse_setu64", "last_very_sparse_setu64") -> Option<u64>, old_setu64, setu64, |_x, s| s.iter().last());
    pair!(("last_dense_set64", "last_sparse_set64", "last_very_sparse_set64") -> Option<u64>, old_set64, set64, |_x, s| s.iter().last());
}

mod sum {
    use super::*;
    pair!(("sum_dense_setu32", "sum_sparse_setu32", "sum_very_sparse_setu32") -> u64, old_setu32, setu32, |_x, s| s.iter().map(|x| x as u64).sum());
    pair!(("sum_dense_setu64", "sum_sparse_setu64", "sum_very_sparse_setu64") -> u64, old_setu64, setu64, |_x, s| s.iter().sum());
    pair!(("sum_dense_set64", "sum_sparse_set64", "sum_very_sparse_set64") -> u64, old_set64, set64, |_x, s| s.iter().sum());
}

/// Elements to build a set from, for the benchmarks that time construction.
#[derive(Clone)]
struct Elements(Vec<u64>);

#[scaling::input(
    group(
        "collect_dense_setu32",
        "collect_dense_setu64",
        "collect_dense_set64",
        "insert_all_dense_setu32",
        "insert_all_dense_setu64",
        "insert_all_dense_set64"
    ),
    name = "elements",
    sizes(1, 10, 100, 1000)
)]
fn dense_elements(n: usize) -> Elements {
    Elements(random_elements(0.8, n))
}

#[scaling::input(
    group(
        "collect_sparse_setu32",
        "collect_sparse_setu64",
        "collect_sparse_set64",
        "insert_all_sparse_setu32",
        "insert_all_sparse_setu64",
        "insert_all_sparse_set64"
    ),
    name = "elements",
    sizes(1, 10, 100, 1000)
)]
fn sparse_elements(n: usize) -> Elements {
    Elements(random_elements(0.05, n))
}

#[scaling::input(
    group(
        "collect_very_sparse_setu32",
        "collect_very_sparse_setu64",
        "collect_very_sparse_set64",
        "insert_all_very_sparse_setu32",
        "insert_all_very_sparse_setu64",
        "insert_all_very_sparse_set64"
    ),
    name = "elements",
    sizes(1, 10, 100, 1000)
)]
fn very_sparse_elements(n: usize) -> Elements {
    Elements(random_elements(0.001, n))
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

/// Registers the previous release's `$old` set type (the baseline) and the
/// current `$new` in `$groups`, each building a set from the elements with
/// `$build` and returning it, so that the metrics functions below can report
/// how much memory the set holds when it is built.
macro_rules! build_pair {
    ($groups:tt, $build:ident, $old:ident, $new:ident, $oldty:ty, $newty:ty, $item:ty, $convert:expr) => {
        #[scaling::bench(group $groups, baseline)]
        fn $old(e: &mut Elements) -> $oldty {
            $build::<$oldty, $item>(&e.0, $convert, |s, x| s.insert(x))
        }
        #[scaling::bench(group $groups)]
        fn $new(e: &mut Elements) -> $newty {
            $build::<$newty, $item>(&e.0, $convert, |s, x| s.insert(x))
        }
    };
}

/// Builds each set with `collect()`.
mod collect {
    use super::*;
    build_pair!(
        (
            "collect_dense_setu32",
            "collect_sparse_setu32",
            "collect_very_sparse_setu32"
        ),
        collect_into,
        old_setu32,
        setu32,
        tinyset_old::SetU32,
        SetU32,
        u32,
        |x: u64| x as u32
    );
    build_pair!(
        (
            "collect_dense_setu64",
            "collect_sparse_setu64",
            "collect_very_sparse_setu64"
        ),
        collect_into,
        old_setu64,
        setu64,
        tinyset_old::SetU64,
        SetU64,
        u64,
        |x: u64| x
    );
    build_pair!(
        (
            "collect_dense_set64",
            "collect_sparse_set64",
            "collect_very_sparse_set64"
        ),
        collect_into,
        old_set64,
        set64,
        tinyset_old::Set64<u64>,
        Set64<u64>,
        u64,
        |x: u64| x
    );
}

/// Builds each set by inserting one element at a time.
mod insert_all {
    use super::*;
    build_pair!(
        (
            "insert_all_dense_setu32",
            "insert_all_sparse_setu32",
            "insert_all_very_sparse_setu32"
        ),
        insert_into,
        old_setu32,
        setu32,
        tinyset_old::SetU32,
        SetU32,
        u32,
        |x: u64| x as u32
    );
    build_pair!(
        (
            "insert_all_dense_setu64",
            "insert_all_sparse_setu64",
            "insert_all_very_sparse_setu64"
        ),
        insert_into,
        old_setu64,
        setu64,
        tinyset_old::SetU64,
        SetU64,
        u64,
        |x: u64| x
    );
    build_pair!(
        (
            "insert_all_dense_set64",
            "insert_all_sparse_set64",
            "insert_all_very_sparse_set64"
        ),
        insert_into,
        old_set64,
        set64,
        tinyset_old::Set64<u64>,
        Set64<u64>,
        u64,
        |x: u64| x
    );
}

/// How much memory each type holds once built. Metrics pair with candidates by
/// return type, so each type needs a function of its own. A set that fits in
/// its own pointer holds no heap memory.
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

mod memory {
    use super::*;
    memory!(
        setu32_memory,
        SetU32,
        (
            "collect_dense_setu32",
            "collect_sparse_setu32",
            "collect_very_sparse_setu32",
            "insert_all_dense_setu32",
            "insert_all_sparse_setu32",
            "insert_all_very_sparse_setu32"
        )
    );
    memory!(
        old_setu32_memory,
        tinyset_old::SetU32,
        (
            "collect_dense_setu32",
            "collect_sparse_setu32",
            "collect_very_sparse_setu32",
            "insert_all_dense_setu32",
            "insert_all_sparse_setu32",
            "insert_all_very_sparse_setu32"
        )
    );
    memory!(
        setu64_memory,
        SetU64,
        (
            "collect_dense_setu64",
            "collect_sparse_setu64",
            "collect_very_sparse_setu64",
            "insert_all_dense_setu64",
            "insert_all_sparse_setu64",
            "insert_all_very_sparse_setu64"
        )
    );
    memory!(
        old_setu64_memory,
        tinyset_old::SetU64,
        (
            "collect_dense_setu64",
            "collect_sparse_setu64",
            "collect_very_sparse_setu64",
            "insert_all_dense_setu64",
            "insert_all_sparse_setu64",
            "insert_all_very_sparse_setu64"
        )
    );
    memory!(
        set64_memory,
        Set64<u64>,
        (
            "collect_dense_set64",
            "collect_sparse_set64",
            "collect_very_sparse_set64",
            "insert_all_dense_set64",
            "insert_all_sparse_set64",
            "insert_all_very_sparse_set64"
        )
    );
    memory!(
        old_set64_memory,
        tinyset_old::Set64<u64>,
        (
            "collect_dense_set64",
            "collect_sparse_set64",
            "collect_very_sparse_set64",
            "insert_all_dense_set64",
            "insert_all_sparse_set64",
            "insert_all_very_sparse_set64"
        )
    );
}

// Round trips through the `Fits64` conversions that `Set64` stores with.
mod fits64 {
    use super::*;

    #[scaling::bench(make_input = || rand::random::<i64>())]
    fn i64_round_trip(x: &mut i64) -> i64 {
        unsafe { i64::from_u64(x.to_u64()) }
    }

    #[scaling::bench(make_input = || rand::random::<i32>())]
    fn i32_round_trip(x: &mut i32) -> i32 {
        unsafe { i32::from_u64(x.to_u64()) }
    }

    #[scaling::bench(make_input = || rand::random::<i16>())]
    fn i16_round_trip(x: &mut i16) -> i16 {
        unsafe { i16::from_u64(x.to_u64()) }
    }

    #[scaling::bench(make_input = || rand::random::<i8>())]
    fn i8_round_trip(x: &mut i8) -> i8 {
        unsafe { i8::from_u64(x.to_u64()) }
    }
}

/// Like `scaling::main!()`, but asking for 1% precision (the default, written
/// out so it stays that way). Placement still limits what the old and current
/// types can be told apart by to about 8-9%: see AGENTS.md.
///
/// A quicker, rougher run (5% takes a couple of minutes on a quiet machine) can be had by
/// setting `TINYSET_BENCH_PRECISION=0.05`.
fn main() -> Result<(), scaling::RegistrationError> {
    let precision = std::env::var("TINYSET_BENCH_PRECISION")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.01);
    scaling::Config::default()
        .with_relative_error(precision)
        .run_and_print()
}
