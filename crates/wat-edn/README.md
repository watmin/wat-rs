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

## JSON conversion

```rust
use wat_edn::{parse, to_json_string, to_json_string_pretty,
              from_json_string, write, write_pretty};

// EDN → JSON (sentinel-key tagged objects preserve type fidelity)
let v = parse(r#"#myapp/Order {:id 1 :tags #{:vip}}"#).unwrap();
let json = to_json_string(&v);
// → {"#tag":"myapp/Order","body":{":id":1,":tags":{"#set":[":vip"]}}}

// JSON → EDN (round-trips back to the same Value)
let v2 = from_json_string(&json).unwrap();

// Pretty-print EDN
println!("{}", write_pretty(&v));
```

Wire convention: `{"#tag":..., "body":...}` for tagged values,
`{"#set":[...]}` for sets, `{"#bigint":"123N"}` for big integers,
`":foo"` colon-prefix string for keywords. See `src/json.rs` for
the full table; the Clojure side at `wat-edn-clj/src/wat_edn/json.clj`
agrees on the same wire convention.

Useful for: emitting EDN-typed data to JSON-only sinks
(CloudWatch logs, HTTP APIs, JavaScript front-ends), then reading
back without losing type information.

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
