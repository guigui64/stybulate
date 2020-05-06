# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [next]

## [1.1.1] - 2020-05-06

* Fix README (install binary with *cli* feature)

## [1.1.0] - 2020-05-06

* Add a CLI binary
* Remove trailing zeros after dot in floating numbers

## [1.0.0] - 2020-04-29

* API Change: now a `Table` object needs to be created
* An `Unstyle` trait can be implemented for the text content. Default impl is
  given for `String`s
* Use `AsciiEscapedString`s when embedding ASCII escape sequences in a str.
  Using a `String` won't work anymore.
* New feature *ansi_term_style* (enable by default) for styled content
  (`ansi_term::ANSIStrings`) and borders (`ansi_term::Style`)
* `examples/` folder added with detailed examples

## [0.1.1] - 2020-04-20

* Regex dependency removed
* New styles added (FancyGithub and FancyPresto)
* Handle unicode strings (in width calculus)
* Colored text supported

## [0.1.0] - 2020-04-17

* First release

[next]: https://github.com/guigui64/stybulate/compare/1.1.1...HEAD
[1.1.1]: https://github.com/guigui64/stybulate/compare/1.1.0...1.1.1
[1.1.0]: https://github.com/guigui64/stybulate/compare/1.0.0...1.1.0
[1.0.0]: https://github.com/guigui64/stybulate/compare/0.1.1...1.0.0
[0.1.1]: https://github.com/guigui64/stybulate/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/guigui64/stybulate/releases/tag/0.1.0
