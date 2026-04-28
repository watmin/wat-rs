# wat-edn

Spec-conforming EDN parser and writer for Rust. Hand-rolled for
performance, designed for the wat language and useful anywhere Rust
code needs to read or write EDN.

## What is EDN?

[Extensible Data Notation](https://github.com/edn-format/edn) — Rich
Hickey's data interchange format for Clojure. Like JSON, but
typed-by-tag, immutable-by-default, and built to round-trip lossless
across processes.

## Coverage

Every literal type defined by the spec:

- `nil`, `true`, `false`
- integers (`i64`) and big integers (`42N`, `num_bigint::BigInt`)
- floats (`f64`) and big decimals (`3.14M`, `bigdecimal::BigDecimal`)
- strings with full escape support (`\n \t \r \b \f \" \\ \/ \uXXXX`)
- characters (`\c \newline \space \tab \return \formfeed \backspace \uXXXX`)
- symbols and namespaced symbols (`foo`, `ns/foo`)
- keywords and namespaced keywords (`:foo`, `:ns/foo`)
- lists `(1 2 3)`, vectors `[1 2 3]`, maps `{:k :v}`, sets `#{1 2 3}`
- tagged elements `#tag value` with arbitrary nesting
- built-in tags `#inst` (RFC 3339 → `chrono::DateTime<Utc>`) and
  `#uuid` (canonical → `uuid::Uuid`)
- comments (`;`) and discard (`#_`)

## Use

```rust
use wat_edn::{parse, write, Value};

let v = parse(r#"#myapp/Order {:id 42 :total 99.99}"#)?;
let s = write(&v);
```

## Performance

- Hand-rolled byte-level lexer; no regex, no parser-combinator
  framework.
- Single-pass recursive-descent parser.
- Borrowed string bodies via `Cow<str>` — escapes are the only path
  that allocates.
- Comma is whitespace per spec.
- Map preserves insertion order; consumers convert to their preferred
  hash structure after reading.

Run the benchmark:

```sh
cargo run --release --example bench -p wat-edn
```

## Clojure side

The companion Clojure library lives at [`wat-edn-clj/`](wat-edn-clj/).
It loads schema from the same `.wat` files wat-rs's type checker
consumes (header-file pattern):

```clojure
(require '[wat-edn.core :as wat])

(wat/load-types! "shared.wat")

(wat/gen 'enterprise.config/SizeAdjust
         {:asset :BTC :factor 1.5 :reason "vol spike"})
;; => #enterprise.config/SizeAdjust {:asset :BTC, :factor 1.5, ...}
;;    (validation throws on field-type mismatch BEFORE bytes leave Clojure)

(wat/read-typed 'enterprise.config/SizeAdjust edn-string)
;; => validated body map
```

One `.wat` file. Two readers. Same schema. EDN bytes flow either
direction (Clojure ↔ wat) without any helper-library handshake for
the standard EDN spec.

## License

MIT OR Apache-2.0
