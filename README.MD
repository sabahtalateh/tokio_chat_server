# Tokio Chat Server

На примере чата можно хорошо разобраться с асинхронным способом писать программы, так как асинхронное программирование в обшем виде означает что есть какое-то нечто `event loop` наподобие, чему можно отдать какой то код, это нечто его исполнит, и пнёт нас когда закончит, а вы в это время отправитесь грабить сарацинов

Ну и вот. Значит попробуем разобраться с футурами в расте на примере чат сервера, это пример из документации по [Tokio](https://tokio.rs/docs/going-deeper/chat/), однако мне было не особо понятно всё что я там видел до тех пор пока я не стал писать это сам, поочерёдно реализую необходимый функционал, вобщем можт кому будет интересно а кому и не будет. А всем остальным бог судья

###Футуры
Крэйт [`futures`](https://docs.rs/futures/0.1.28/futures/) определяет трэйт [`Future`](https://docs.rs/futures/0.1.28/futures/future/trait.Future.html), основным методом которого является `poll`. Обратимся же поскорее к его [сигнатуре](https://docs.rs/futures/0.1.28/futures/future/trait.Future.html#required-methods).
```rust
fn poll(&mut self) -> Poll<Self::Item, Self::Error>
```

Ну что-же неплохо для начала, поглядим на тип [Poll](https://docs.rs/futures/0.1.28/futures/type.Poll.html)

```rust
type Poll<T, E> = Result<Async<T>, E>
```
Это уже более знакомые слова
```rust
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}
```
и
```rust
pub enum Async<T> {
    Ready(T),
    NotReady,
}
```
Зачем нам всё это добро? Чтобы приблизить наступление всепланетного коммунизма.
Семантически `Poll` или в переводе на язык Пушкина и Чаадаева, то есть на 1С - `Опрос`, предстваляет ответ на дамское вопрошание вопрошание `Изволил ли господин сходить за рогаликами?`, прогрессивная система типов помогает нам выразить ответ на такой каверзный вопрос, `Господин в добром здравии(Идёт за рогаликами)/Result(NotReady)`, `Господин в добром здравии(Более того стоит возле вас с рогаликом в руке)/Result(Ready(T))` где `T` - тип который вернёт футура когда выполнится и `По дороге к пекарне господина взорвали революционеры/Err(E)` где `E` - тип ошибки, например `io::Error`.

Реализуем трэйт `Future` для нашего собственного типа. Ниже приведён код файла [`examples/future.rs`](examples/future.rs) для запуска `cargo run --example future`
```rust
use futures::task::spawn;
use futures::{task, AndThen, Async, Future, Poll};

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
    /// that used OS specific polling features `epoll(linux)/kqueue(darwin)/iocp(windows)`
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
        Ok(val)
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

    /// Futures should be run onto an executor
    ///
    /// Later we will consider tokio executor instead of
    /// one provided by the future trait
    let mut s = spawn(my_fut);

    /// Wait for the outer future `AndThen` which will
    /// first execute inner `MyFut` and then apply `f()` to it
    let r = s.wait_future();
    println!("{:?}", r);
}
```

 
