# david-set

This is a set that is optimized to be space and time efficient for
small or large numbers of elements that implement the `Copy` trait.
The `Copy` constraint is convenient, but with more work this set could
work for arbitrary types (supporing `Eq` and `Hash`, of course).

Eventually it will need a better name.
