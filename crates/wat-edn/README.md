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

# v4/v5 UUID generation (`new_uuid_v4()` / `new_uuid_v5()`) is built in — no
# feature flag (arc 296 removed the optional `mint` feature: uuid generation
# is core to wat, not an opt-in).
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
- 344 Rust tests + 39 Clojure tests, all green

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
`#wat.core.f64/{NaN,+Inf,-Inf}` sentinels for `f64` round-trip.
See [§9 Spec extensions](docs/USER-GUIDE.md#9-spec-extensions).

## Verification (cross-language)

Self round-trip is `cargo test -p wat-edn` (344/344 passing). The
spec-conformance claim above — *"peer to Clojure's reference reader"* —
is empirically verified by piping wat-edn output through stock
`clojure.edn/read` (no helpers, no extensions):

```sh
cd interop-tests
cargo build --release

# Handshake 1: wat-edn → pure Clojure (trade signal fixture)
cargo run --release --bin wat-edn-interop-tests | clojure -M clj/consume.clj

# Handshake 2: Pure Clojure → wat-edn (Clojure pr-str fixture)
clojure -M clj/produce.clj | cargo run --release --bin reader

# Handshake 3: Shape matrix — 23 named shapes, wat-edn → Clojure
cargo run --release --bin shape_matrix | clojure -M clj/consume_shapes.clj

# Handshake 4: Shape matrix — Clojure → wat-edn (reverse direction)
clojure -M clj/produce_shapes.clj | cargo run --release --bin shape_matrix_reader
```

The shape matrix exercises: primitives, collections (vec/set/map),
nested collections, EDN-spec built-in tags (`#inst`/`#uuid`), FQDN
tagged literals (`#wat.core/Some`, `#wat.core/None nil`, `#wat.core/Ok`,
`#wat.core/Err`, `#wat.time/Duration`), nested complex
(`#wat.core/Some #{{:foo "baz"}}`, `Ok<Vec<Map>>`, `Some<Some<i64>>`),
and composite keys (tagged values as map keys via arc 216's
`impl Hash for Value`).

**Discipline:** every wat-edn substrate touch must run these four
handshakes before INSCRIPTION. See `interop-tests/README.md` for the
full pipeline matrix.

## License

MIT OR Apache-2.0
