### concurrency_traits v0.7.2
- Fixed queue bug

### concurrency_traits v0.7.1
- Added semaphores
- Added `FullAsyncQueue`
  - Adjusted a few erroneous queue trait dependencies related

## concurrency_traits v0.7.0
- Major refactoring of file layout
- Used `async_trait` for async versions
- Removed blanket implementations

## concurrency_traits v0.6.0
- Added `AtomicMutex`
- Added `RawTimeoutMutex` to `RawSpinLock`
- Added `AtomicRwLock`
- Added `SpinRwLock`
- Added `TryUpgradeRwLock`, `UpgradeTimeoutRwLock`, `DowngradeRwLock`, and related raws
  - Implemented for multiple types

### concurrency_traits v0.5.3
- Fixed simple futures importing with std
  - Added CI lint to check this

### ~~concurrency_traits v0.5.2~~
- Added `FullAsyncMutex`

### ~~concurrency_traits v0.5.1~~
- Fixed overflow bug in `ParkQueue`
- Fixed overflow bug in `RawParkMutex`

## ~~concurrency_traits v0.5.0~~
- Added ThreadTimeoutParker to StdThreadFunctions
- Changed a lot of blanket impls
- Added more custom mutexes and queues
- Added `crossbeam` support through `impl_crossbeam` feature

## concurrency_traits v0.4.0
- Added general system traits

### concurrency_traits v0.3.1
- Added `TimeoutQueue` and `AsyncTimeoutQueue`

## concurrency_traits v0.3.0
- Made `ThreadSpawner` able to handle errors

## concurrency_traits v0.2.0
- Added implementations for std lib and parking_lot
- Changed blanked `Deref` impls to targeted impls

# concurrency_traits v0.1.0
- Initial Version!
