* 0.5.0 - Sep. 14, 2024

    - Implement `Clone` for `IntoIter`.  This change in implementation also
      removes a use of `unsafe`.

    - Bump MSRV to 1.61 due to dependencies.

    - Fixed panic when removing a nonexistent value from a full hash table.

