# Stybulate - Tabulate with Style!

A Rust library to create ASCII tables with styled borders.
Inspired by the PyPi package <https://pypi.org/project/tabulate/>.

[![Build status](https://github.com/guigui64/stybulate/workflows/CI/badge.svg)](https://github.com/guigui64/stybulate/actions)
[![Crates.io](https://meritbadge.herokuapp.com/stybulate)](https://crates.io/crates/stybulate)
[![Rust](https://img.shields.io/badge/rust-1.38.0%2B-blue.svg?maxAge=3600)](https://github.com/guigui64/stybulate)
[![Docs.rs](https://docs.rs/stybulate/badge.svg)](https://docs.rs/stybulate)
[![License](https://img.shields.io/crates/l/stybulate)](https://github.com/guigui64/stybulate/blob/master/LICENSE-MIT)

### Example

```rust
use stybulate::{tabulate, Style, Cell};
let headers = vec!["strings", "numbers"];
let contents = vec![
    vec![Cell::Text("answer"), Cell::Int(42)],
    vec![Cell::Text("pi"), Cell::Float(3.1415)],
];
let expected = vec![
    "╒═══════════╤═══════════╕",
    "│ strings   │   numbers │",
    "╞═══════════╪═══════════╡",
    "│ answer    │   42      │",
    "├───────────┼───────────┤",
    "│ pi        │    3.1415 │",
    "╘═══════════╧═══════════╛",
].join("\n");
let table = tabulate(Style::Fancy, contents, headers);
assert_eq!(expected, table);
```

### License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
