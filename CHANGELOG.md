* 0.6.0 - Mar. 9, 2025

    - Increase MRSV to 1.63 due to libc dependency.

    - New feature `deterministic_iteration`.

* 0.5.0 - Oct. 12, 2024

    - Implement `Clone` for `IntoIter`.  This change in implementation also
      removes a use of `unsafe`.

    - Bump MSRV to 1.61 due to dependencies.

    - Fixed panic when removing a nonexistent value from a full hash table.

    - Fix UB if an allocation exceeds `isize::MAX`.

