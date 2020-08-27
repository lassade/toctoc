Knocknoc
=========

*Simpler serde alternative with about the same feature set,
does less but is also way less intimidating to modify*

[![](https://github.com/lassade/knocknoc/workflows/Build/badge.svg)](https://github.com/lassade/knocknoc/blob/main/.github/workflows/rust.yml)

## About

`knocknoc` tries to be a simple alternative to `serde` by only using
trait objects. It also supports contextual (de)serialization, have a similar
but simplified api (less methods with most of the functionality).
It's also fast and lightweight;

The `derive` can be used for quick implementations of both
`Serialize` and `Deserialize` traits, you probably want and shouldn't be
afraid of implement your own versions tailoring to your needs.

Is not a one size fits all kind of solution is more like make
it your own size type of one. Fork and modify as you wish.

## Features

### No monomorphization

This comes directly from `miniserde` it makes the code bit slower
but a lot simpler to work with.

There are no nontrivial generic methods. All serialization and deserialization
happens in terms of trait objects. Thus no code is compiled more than once
across different generic parameters. In contrast, `serde_json` needs to stamp out
a fair amount of generic code for each choice of data structure being serialized
or deserialized.

Without monomorphization, the derived impls compile lightning fast and occupy
very little size in the executable.

### Simplified (de)serialization API

Almost the same api as `serde` but with less methods and most of the functionality.
If you need something just add it, fast and painless.

### Support for u8, i8, u32, i32 and f32

The added support will allow for a smaller binary representation, `f32`
is very commonly used on games, thus was added to avoid unnecessary
conversions between and from `f64`, that will take a bit of cpu cycles.

### Basic enumeration support

Formats like `ron` that have a notation for enum variants will need modifications to work.

On other formats like JSON the derive support externally tagged enumerations, adjacent and internal
formats may be added in the future (but probably won't). 

### Included formats JSON + BSON

You can write new formats just like in `serde`.

By default this crates ships with JSON as is the most commonly
used textual format and have a nice SIMD implementation for it;

And BSON is like json, but better suited for binary data.

### Data alignment

Both JSON though `bintext` and BSON supports (de)serialization of binary
aligned data. By default BSON only supports alignments of `4` but you can enable
the `higher-rank-alignment` to allow for higher alignments.

### Zero Copy

Like serde this lib supports zero copy load, and it also provides a simple
encoding formats for aligned binary data on both json and bson.

For json we have some thing like this `{ "binary": "#----01000000" }`, `#`
tells the parser this is binary data, the amount of `-` tells the alignment
requirement for this bytes; With this format `bintext` is able to decode
the string into a memory aligned byte slice!

For bson the start buffer must be aligned with `4` and thats it all the
necessary padding;

### Sateful or contextual (de)serialization

Made primarily for load/save assets and game entities references, but
using the `any-context` feature the context will became an alias for
`std::any::Any`.

### SIMD support

Json (de)serialization is be done using the [simd_json](https://crates.io/crates/simd-json)
crate, currently fastest (pure rust) json parsing crate available.
The `simd` feature is enabled by default;

### Similar crates

- `nanoserde` or `makepad-tinyserde`, it's designed to compile faster and
be lightweight (take less binary space), but is limited to a few formats
and by design will be hard to work with new ones;

- `miniserde` The stack free feature makes the implementation much more hard that it needs to be,
`knocknoc` achieves the same result by dynamically increasing the stack, whenever needed;

Miniserde (original)
=========

*This crate is a fork of miniserde*

*Prototype of a data structure serialization library with several opposite
design goals from [Serde](https://serde.rs).*

As a prototype, this library is not a production quality engineering artifact
the way Serde is. At the same time, it is more than a proof of concept and
should be totally usable for the range of use cases that it targets, which is
qualified below.

```toml
[dependencies]
knocknoc = "0.1"
```

Version requirement: rustc 1.31+

### Example

```rust
use knocknoc::{json, Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Example {
    code: u32,
    message: String,
}

fn main() -> knocknoc::Result<()> {
    let example = Example {
        code: 200,
        message: "reminiscent of Serde".to_owned(),
    };

    let j = json::to_string(&example, &());
    println!("{}", j);

    let out: Example = json::from_str(&j, &mut ())?;
    println!("{:?}", out);

    Ok(())
}
```

Here are some similarities and differences compared to Serde.

### Similar: Stupidly good performance

Seriously this library is way faster than it deserves to be. With very little
profiling and optimization so far and opportunities for improvement, this
library is on par with serde\_json for some use cases, slower by a factor of 1.5
for most, and slower by a factor of 2 for some. That is remarkable considering
the other advantages below.

### Similar: Strongly typed data

Just like Serde, we provide a derive macro for a Serialize and Deserialize
trait. You derive these traits on your own data structures and use
`json::to_string` to convert any Serialize type to JSON and `json::from_str` to
parse JSON into any Deserialize type. Like serde\_json there is a `Value` enum
for embedding untyped components.

### Different: Minimal design

This library does not tackle as expansive of a range of use cases as Serde does.
Feature requests are practically guaranteed to be rejected. If your use case is
not already covered, please use Serde.

The implementation is less code by a factor of 12 compared to serde +
serde\_derive + serde\_json, and less code even than the `json` crate which
provides no derive macro and cannot manipulate strongly typed data.

### Different: Infallible serialization

Serialization always succeeds. This means we cannot serialize some data types
that Serde can serialize, such as `Mutex` which may fail to serialize due to
poisoning. Also we only serialize to `String`, not to something like an i/o
stream which may be fallible.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
