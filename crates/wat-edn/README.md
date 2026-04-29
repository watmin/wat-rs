# wat-edn

Spec-conforming EDN parser, writer, and JSON bridge for Rust.
A second conforming implementation of [EDN][edn], peer to
Clojure's reference reader, with companion Clojure library
([`wat-edn-clj/`](wat-edn-clj/)) sharing one wire convention.

[edn]: https://github.com/edn-format/edn

## Add to Cargo.toml

```toml
[dependencies]
wat-edn = { path = "../wat-rs/crates/wat-edn" }

# Optional: enable v4 UUID minting (`new_uuid_v4()`). Pulls `uuid`'s `v4`
# feature, which links `getrandom`. Off by default so parser-only consumers
# don't pay for entropy init they don't use.
# wat-edn = { path = "...", features = ["mint"] }
```

## Quickest example

```rust
use wat_edn::{parse, write, to_json_string, from_json_string};

// EDN ↔ Value
let v = parse(r#"#myapp/Order {:id 1 :tags #{:vip}}"#).unwrap();
let edn = write(&v);

// EDN ↔ JSON (sentinel-key tagged objects preserve type fidelity)
let json = to_json_string(&v);
let v2 = from_json_string(&json).unwrap();
assert_eq!(v.into_owned(), v2);
```

## What you get

- Hand-rolled byte-level lexer + recursive-descent parser
- `Value<'a>` with `Cow<'a, str>` zero-copy strings; `OwnedValue` alias for `'static`
- `CompactString`-inlined Symbol/Keyword/Tag (no heap alloc for short names)
- Direct `push_str` writers (no `Display` formatter overhead)
- Round-trip-safe JSON conversion with sentinel-key tagged objects
- Pretty-print with byte-equivalent round-trip identity
- 313 Rust tests + 39 Clojure tests, all green

## Performance (stable, M-class hardware)

```
parse small  [1 2 3 4 5]              56 MB/s     0.19 µs/op
parse realistic blob (416B)          271 MB/s     1.46 µs/op
parse string-heavy (395B)            510 MB/s     0.74 µs/op
parse identifier-heavy (300B)        149 MB/s     1.91 µs/op

write small  [1 2 3 4 5]              111 MB/s    0.09 µs/op
write realistic blob                  996 MB/s    0.40 µs/op
write string-heavy                    858 MB/s    0.44 µs/op
write identifier-heavy                605 MB/s    0.47 µs/op
```

Run `cargo run --release --example bench -p wat-edn` to reproduce.

## Deeper documentation

The short version lives here. The full user guide — every API,
concrete examples, wire conventions, cross-language interop,
performance methodology, common gotchas — lives at:

**[`docs/USER-GUIDE.md`](docs/USER-GUIDE.md)**

Quick links into it:

- [Setup and feature flags](docs/USER-GUIDE.md#1-setup)
- [The Value type — Value<'a> vs OwnedValue](docs/USER-GUIDE.md#2-the-value-type)
- [Parsing](docs/USER-GUIDE.md#3-parsing) /
  [Writing](docs/USER-GUIDE.md#4-writing) /
  [Pretty-print](docs/USER-GUIDE.md#8-pretty-print)
- [JSON conversion](docs/USER-GUIDE.md#7-json-conversion)
- [The Clojure side](docs/USER-GUIDE.md#10-the-clojure-side)
- [Cross-language interop](docs/USER-GUIDE.md#11-cross-language-interop)

## Spec coverage

Every literal type defined by the EDN spec, including built-in
`#inst` (RFC 3339 → `chrono::DateTime<Utc>`) and `#uuid`
(canonical → `uuid::Uuid`). Five `/ignorant` ward casts confirm
zero spec divergence; the strict-rejection test suite locks every
spec-mandated `must not` against regression.

Documented extensions (Clojure-aligned, all round-trip-symmetric):
`\b \f \/` string escapes, `\formfeed \backspace` char names,
`#wat-edn.float/{nan,inf,neg-inf}` sentinels for `f64` round-trip.
See [§9 Spec extensions](docs/USER-GUIDE.md#9-spec-extensions).

## License

MIT OR Apache-2.0
