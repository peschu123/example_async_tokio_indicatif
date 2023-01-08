# example_async_tokio_indicatif

Since async can be a bit confusing at the beginning, I made a little example to visualize async behavior with indicatif multi progress. 
Lots of comments (maybe too much?!) so that everybody can understand whats going on.
 
This is for beginners (like me) in rust/programming. :-)

It uses mainly [indicatif](https://github.com/console-rs/indicatif) (multi) progress bar and [tokio](https://tokio.rs/) JoinSet + spawn.

Based on these examples:

[indicatif examples/multi.rs](https://github.com/console-rs/indicatif/blob/main/examples/multi.rs)

[indicatif examples/tokio.rs](https://github.com/console-rs/indicatif/blob/main/examples/tokio.rs)

[tokio JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html#examples)



Thanks to [alice](https://github.com/Darksonn) for helping me with JoinSet

https://users.rust-lang.org/t/limited-concurrency-for-future-execution-tokio/87171/7
