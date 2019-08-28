use futures::task::spawn;
use futures::{task, AndThen, Async, Future, Poll, Map};

/// Our own type `MyFut`
#[derive(Debug)]
struct MyFut {
    n: i32,
}

impl MyFut {
    fn new() -> Self {
        MyFut { n: 0 }
    }
}

/// Implement `Future` for `MyFut`
impl Future for MyFut {
    /// Return type of `future` is `i32`
    type Item = i32;
    /// Error is of unit type
    type Error = ();

    /// Poll `MyFut` 3 times until it's ready
    /// Increment internal counter to track number of polls
    ///
    /// Call `task::current().notify()` to tell the executor
    /// that future can be polled again immediately (executor will `poll` it again).
    /// In `tokio` for this purpose `mio` crate is used
    /// that uses OS specific polling features `epoll(linux)/kqueue(darwin)/iocp(windows)`
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.n += 1;
        if self.n == 3 {
            println!("ready n = {}", self.n);
            return Ok(Async::Ready(self.n));
        } else {
            println!("not ready n = {}", self.n);
            task::current().notify();
            return Ok(Async::NotReady);
        }
    }
}

fn main() {
    /// Implicit types declarations in this file can be omitted
    let my_fut: MyFut = MyFut::new();

    /// Function to use in `AndThen` combinator
    let f: fn(i32) -> Result<i32, ()> = |val| {
        println!("done");
        Ok(val - 100)
    };

    /// `AndThen` type returned by `.and_then(closure)` combinator
    /// implements `Future` too so it can be used as regular future
    ///
    /// First generic type parameter is `MyFut` - future that will be polled first
    ///
    /// Second is a type that will be returned from the applied function
    /// It should realize `IntoFuture` trait so that it can be used in
    /// next possible combinator like `.map()` or one more `.and_then()`
    ///
    /// Third is a function that will be applied to the `MyFut` result
    /// It's return type should be the same as second type parameter
    let my_fut: AndThen<MyFut, Result<i32, ()>, fn(i32) -> Result<i32, ()>> = my_fut.and_then(f);

    /// `Map` type parameterized with 2 args
    ///
    /// First one `AndThen<MyFut, Result<i32, ()>, fn(i32) -> Result<i32, ()>>` is a `Future` to map
    ///
    /// Second one is a function that wll be applied to `v`
    /// once a `Future` will be ready (it's `poll` returns `Ok(Async::Ready(v))`)
    ///
    /// Note that instead of `fn(i32) -> Result<i32, ()>` for `AndThen` we use `fn(i32)-> i32`
    /// for map because semantics of `AndThen` is to run another future once the previous one completes
    /// and the semantics of `Map` is to map the result of the future from one value to another
    let my_fut : Map<AndThen<MyFut, Result<i32, ()>, fn(i32) -> Result<i32, ()>>, fn(i32)-> i32>= my_fut.map(|x| {
        x + 1000
    });

    /// Futures should be run onto an executor
    ///
    /// Later we will consider tokio executor instead of
    /// one provided by the future trait
    let mut s = spawn(my_fut);

    /// Wait for the outer future `Map` which will poll `AndThen`  which in turn will poll `MyFut`
    let r = s.wait_future();
    println!("{:?}", r);
}
