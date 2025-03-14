# v1.3.1
* Optimise spin strategy handling.

# v1.3.0
* Add `sleep_until`, `SpinSleeper::sleep_until`.

# v1.2.1
* Windows: Update _windows-sys_ to 0.59.

# v1.2.0
* Deprecate `LoopHelper`. Instead use _spin_sleep_util_ crate.
* Windows: Use a high resolution waitable timer when available (>= Windows 10, version 1803).
* Windows: Replace _winapi_ with _windows-sys_ dependency.
* Windows: Remove _once_cell_ dependency.

# v1.1.1
* Fix LoopHelper increment overflow handling.

# v1.1.0
* Expose fn `native_sleep`.
* Add `SpinSleeper::with_spin_strategy` which allows specifying a `SpinStrategy`.
  Previously thread yielding was always used.
* Windows: Use `SpinStrategy::SpinLoopHint` by default (see #12).

# v1.0.0
* Use rust 1.38 _duration_float_ methods to replace manual implementations.
* Use edition 2018.
* Windows: Replace lazy_static dependency with once_cell.

# v0.3.7
* `report_rate()` no longer eagerly reports on first call, as this is often inaccurate & unexpected.

# v0.3.6
* Add `spin_sleep::sleep`
* Add `LoopHelper::set_target_rate` &  `LoopHelper::target_rate`
* Improve documentation

# v0.3.5
* Use `std::thread::yield_now` in spin wait loops for efficiency
* Add `Default` implementation for `SpinSleeper` using OS-specific accuracy defaults
