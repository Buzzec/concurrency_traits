### concurrency_traits v0.5.0
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
