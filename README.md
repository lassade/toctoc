Knocknoc
=========

*Knocknoc is a miniserde fork with more features guided towards gamedev*


### Extra: Support for u8, i8, u32, i32 and f32

The added support will allow for a smaller binary representation, `f32`
is very commonly used on games, thus was added to avoid unnecessary
conversions between and from `f64`, that will take a bit of cpu cycles.

### Extra: Enumeration support

This lib only use externally tagged enumerations. Adjacent and internal
formats may be added in the future (but probably won't).

### Extra: Bson support (binary json)

Bson is like a binary json format. so it's more suitable for binary
data and also self described, but it doest not guarantee memory alignment
for data structures that may require it.

### Extra: Stateful or contextual (de)serialization

Used for load assets or reference game entities

### Extra: SIMD support

By enabling the `simd` feature, json (de)serialization will be done by
the [simd_json](https://crates.io/crates/simd-json) crate,
currently fastest (pure rust) json parsing crate available.

### Zero Copy

Like serde this lib supports zero copy load, and it also provides a simple
encoding formats for aligned binary data on both json and bson.

For json we have some thing like this `{ "binary": "#----01000000" }`, `#`
tells the parser this is binary data, the amount of `-` tells the alignment
requirement for this bytes; With this format `bintext` is able to decode
the string into a memory aligned byte slice!

For bson the start buffer must be aligned with `4` and thats it all the
necessary padding;

Miniserde (original)
=========

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

### Different: No monomorphization

There are no nontrivial generic methods. All serialization and deserialization
happens in terms of trait objects. Thus no code is compiled more than once
across different generic parameters. In contrast, serde\_json needs to stamp out
a fair amount of generic code for each choice of data structure being serialized
or deserialized.

Without monomorphization, the derived impls compile lightning fast and occupy
very little size in the executable.

### Different: No recursion

Serde depends on recursion for serialization as well as deserialization. Every
level of nesting in your data means more stack usage until eventually you
overflow the stack. Some formats set a cap on nesting depth to prevent stack
overflows and just refuse to deserialize deeply nested data.

In knocknoc neither serialization nor deserialization involves recursion. You
can safely process arbitrarily nested data without being exposed to stack
overflows. Not even the Drop impl of our json `Value` type is recursive so you
can safely nest them arbitrarily.

### Different: No deserialization error messages

When deserialization fails, the error type is a unit struct containing no
information. This is a legit strategy and not just laziness. If your use case
does not require error messages, good, you save on compiling and having your
instruction cache polluted by error handling code. If you do need error
messages, then upon error you can pass the same input to serde\_json to receive
a line, column, and helpful description of the failure. This keeps error
handling logic out of caches along the performance-critical codepath.

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
