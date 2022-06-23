# rust-log-mdc

A mapped diagnostic context (MDC) for use with the `log` crate.

An MDC is a thread local map of strings used to make relevant information
from a system available in its log messages. Logging crates such as
[log4rs][log4rs] will retrieve values from the MDC for output.

For example, a web server may process many requests simultaneously on
different threads. Generating an ID for each request and storing it in the
MDC makes it easy to partition log messages on a per-request basis.

**This crate also provide [global](src/global.rs) MDC for all thread output.**

[log4rs]: https://crates.io/crates/log4rs

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
