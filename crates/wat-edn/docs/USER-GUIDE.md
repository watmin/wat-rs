# wat-edn — User Guide

You're building an application that needs to read or write EDN
in Rust, and you've decided to use `wat-edn`. This guide shows
you how.

**Who this is for.** Application authors — Rust developers using
`wat-edn` as a dependency. For internals (lexer state machine,
parser recursion shape, performance methodology), read the source
under `src/`. For the language wat-edn ships alongside, see
`wat-rs/docs/USER-GUIDE.md`.

**What this guide covers.** Every public API surface, the wire
conventions for EDN and the JSON bridge, the cross-language story
with the companion Clojure library, performance characteristics
and how to measure them, common gotchas, and where to go when you
hit something this guide doesn't cover.

**What this guide does NOT cover.** Internals (how the lexer
dispatches, how `Value<'a>`'s lifetime threads through the parser,
how `CompactString` interacts with the variant size). The source
is small (~2000 LOC) and well-commented; read it directly.

**This guide is alive.** It evolves as `wat-edn` grows. Where the
guide lies, the test suite tells us; the guide gets updated. If
you hit something the guide didn't prepare you for, the gap is
worth reporting.

---

## Table of contents

1. [Setup](#1-setup)
2. [The Value type](#2-the-value-type)
3. [Parsing](#3-parsing)
4. [Writing](#4-writing)
5. [Constructing values](#5-constructing-values)
6. [Built-in tags](#6-built-in-tags)
7. [JSON conversion](#7-json-conversion)
8. [Pretty-print](#8-pretty-print)
9. [Spec extensions](#9-spec-extensions)
10. [The Clojure side](#10-the-clojure-side)
11. [Cross-language interop](#11-cross-language-interop)
12. [Performance](#12-performance)
13. [Spec coverage and conformance](#13-spec-coverage-and-conformance)
14. [Error handling](#14-error-handling)
15. [Common gotchas](#15-common-gotchas)
16. [Where to go next](#16-where-to-go-next)

---

## 1. Setup

`wat-edn` is a workspace member of `wat-rs`. From a sibling crate:

```toml
[dependencies]
wat-edn = { path = "../wat-rs/crates/wat-edn" }
```

That's it — no feature flags. Dependencies pulled transitively:
`serde_json` (JSON conversion), `chrono` (`#inst`), `uuid`
(`#uuid`), `num-bigint` and `bigdecimal` (`42N` / `3.14M` literals),
`memchr` (writer fast path), `compact_str` (Symbol/Keyword/Tag
inline storage), `thiserror` (error variants).

For Clojure consumers, the companion library lives at
`wat-edn-clj/` inside this crate. See [§10](#10-the-clojure-side).

---

## 2. The Value type

EDN's closed sum type lives at `wat_edn::Value<'a>`. The lifetime
parameter `'a` exists because the lexer's fast path returns
`Cow::Borrowed` slices into the input buffer for unescaped
strings — zero-copy parse for the common case.

```rust
pub enum Value<'a> {
    Nil,
    Bool(bool),
    Integer(i64),
    BigInt(Box<BigInt>),
    Float(f64),
    BigDec(Box<BigDecimal>),
    String(Cow<'a, str>),     // ← borrows from input when no escapes
    Char(char),
    Symbol(Symbol),            // ← CompactString-backed
    Keyword(Keyword),
    List(Vec<Value<'a>>),
    Vector(Vec<Value<'a>>),
    Map(Vec<(Value<'a>, Value<'a>)>),
    Set(Vec<Value<'a>>),
    Tagged(Tag, Box<Value<'a>>),
    Inst(DateTime<Utc>),
    Uuid(Uuid),
}
```

### When to use Value<'a> vs OwnedValue

The library exposes a `'static` alias:

```rust
pub type OwnedValue = Value<'static>;
```

Use `Value<'_>` (returned by `parse`) when:
- You're consuming the value within the lifetime of `input`
- You don't need to store, return, or thread the value across
  function boundaries
- You want zero-copy on string bodies

Use `OwnedValue` when:
- The value must outlive `input`'s borrow scope
- You're storing it in a struct, returning it from a function,
  putting it through a channel
- You don't care about the marginal cost of copying string bodies

Lift via `Value::into_owned`:

```rust
let v: Value<'_> = parse(input)?;
let owned: OwnedValue = v.into_owned();
```

Or use `parse_owned` directly:

```rust
let owned: OwnedValue = parse_owned(input)?;
```

`into_owned` recurses through containers, copying every borrowed
string into an owned `String`. Already-owned data passes through
unchanged.

---

## 3. Parsing

Three entry points:

```rust
use wat_edn::{parse, parse_owned, parse_all, Parser};

// Single top-level form, zero-copy where possible
let v: Value<'_> = parse("#myapp/Order {:id 1}")?;

// Single top-level form, materialize to 'static
let v: OwnedValue = parse_owned("#myapp/Order {:id 1}")?;

// All top-level forms (whitespace + comments between them)
let vs: Vec<Value<'_>> = parse_all("1 2 3")?;

// For streaming consumption, drive Parser directly
let mut p = Parser::new(input);
loop {
    match p.parse_next()? {
        None => break,
        Some(v) => /* process v */,
    }
}
```

### Parse rules

The parser is spec-strict. All of these errors:

```rust
parse("01")             // leading zero on non-zero int
parse("+0123")
parse(":/")             // :/ is not a legal keyword
parse("\\ ")            // backslash followed by whitespace
parse(".5")             // leading-dot-then-digit
parse("ns/123")         // numeric first char in name
parse("[#myapp/Foo]")   // dangling tag without element
parse("#bareTag 42")    // user tag without namespace
```

Parsed values come back through `Value`'s closed enum; you
pattern-match or use the `as_*` accessors:

```rust
let v = parse(":foo")?;
match v {
    Value::Keyword(k) => println!("ns: {:?}, name: {:?}",
                                  k.namespace(), k.name()),
    _ => unreachable!(),
}

// Or via accessor (returns Option)
if let Some(k) = v.as_keyword() {
    println!("{}", k);
}
```

Available accessors: `as_bool`, `as_i64`, `as_f64`, `as_str`,
`as_char`, `as_symbol`, `as_keyword`, `as_list`, `as_vector`,
`as_map`, `as_set`, `as_tagged`, `as_inst`, `as_uuid`, plus
`is_nil` and `type_name` for diagnostics.

---

## 4. Writing

Two entry points:

```rust
use wat_edn::{write, write_to};

let v = Value::Vector(vec![Value::Integer(1), Value::Integer(2)]);

// Returns a fresh String
let s: String = write(&v);

// Appends to caller-owned buffer (reuse across iterations)
let mut buf = String::with_capacity(1024);
write_to(&v, &mut buf);
write_to(&v, &mut buf);  // appends; caller clears or truncates
```

### Output style

Compact, no-extra-whitespace:

```text
[1 2]
{:asset :BTC, :side :Buy}
#myapp/Order {:id 1}
```

Maps emit `key value` pairs separated by `, ` (commas are
whitespace per spec; the comma is purely visual). Tagged values
emit `#ns/name <body>` with one space between tag and body. For
indented multi-line output, use [`write_pretty`](#8-pretty-print).

The writer's identifier path bypasses `fmt::Formatter` and
`push_str`'s directly to the caller's `String` — measurably
faster on identifier-heavy payloads. Identical bytes to what
the `Display` impls produce (locked by
`tests/display_equivalence.rs`).

---

## 5. Constructing values

### Primitives

```rust
Value::Nil
Value::Bool(true)
Value::Integer(42)
Value::Float(3.14)
Value::String("hello".into())                  // Cow::Owned
Value::Char('a')
```

### Symbol / Keyword / Tag

Constructors validate per the EDN spec (first character must be
non-numeric; `+`/`-`/`.` first character cannot be followed by a
digit). Two flavors:

```rust
use wat_edn::{Symbol, Keyword, Tag};

// Panic on invalid input — for compile-time-known names
let s = Symbol::new("foo");
let k = Keyword::ns("enterprise.config", "asset");
let t = Tag::ns("myapp", "Order");

// Returns Result for caller-supplied input
let s = Symbol::try_new(user_input)?;
let k = Keyword::try_ns(ns_input, name_input)?;
let t = Tag::try_ns(ns_input, name_input)?;
```

`Tag::namespace` is REQUIRED at the type level — no `Option`.
The EDN spec says user tags MUST be namespaced; the type enforces
it. There is no `Tag::new(name)` for that reason.

### Tagged values

```rust
let order = Value::Tagged(
    Tag::ns("myapp", "Order"),
    Box::new(Value::Map(vec![
        (Value::Keyword(Keyword::new("id")), Value::Integer(1)),
        (Value::Keyword(Keyword::new("name")), Value::String("Alice".into())),
    ])),
);

assert_eq!(write(&order), r#"#myapp/Order {:id 1, :name "Alice"}"#);
```

### Collections

```rust
Value::Vector(vec![Value::Integer(1), Value::Integer(2)])
Value::List(vec![/* ... */])
Value::Set(vec![/* multiset semantics — see Map/Set equality */])
Value::Map(vec![(k1, v1), (k2, v2)])
```

Maps preserve insertion order. Equality (per spec) is unordered
for Maps and Sets — see the
[gotchas](#15-common-gotchas) for the full table.

---

## 6. Built-in tags

The two spec-defined built-ins are canonicalized to typed
variants on parse:

```rust
let v = parse(r#"#inst "2026-04-28T16:00:00Z""#)?;
match v {
    Value::Inst(dt) => /* dt: DateTime<Utc> */,
    _ => unreachable!(),
}

let v = parse(r#"#uuid "550e8400-e29b-41d4-a716-446655440000""#)?;
match v {
    Value::Uuid(u) => /* u: uuid::Uuid */,
    _ => unreachable!(),
}
```

### `#inst` rules

- Body must be an EDN string in RFC 3339 form
- Output via `write` uses `chrono::SecondsFormat::AutoSi` (preserves
  fractional seconds when present)

### `#uuid` rules

- Body must be the canonical 8-4-4-4-12 hyphenated form
- The simple form (no hyphens) and URN form are REJECTED — stricter
  than `uuid::Uuid::parse_str`'s default

---

## 7. JSON conversion

JSON has fewer types than EDN. wat-edn uses sentinel-key tagged
objects on the JSON side to preserve EDN type fidelity through
the round-trip.

### Wire convention

```text
EDN value          JSON shape
─────────────────  ───────────────────────────────────────────────
nil                null
true / false       true / false
i64 (in range)     number
i64 (> 2^53)       string  "9007199254740993"
bigint             {"#bigint": "123N"}
f64                number
NaN / ±Inf         {"#float": "nan" | "inf" | "neg-inf"}
bigdec             {"#bigdec": "3.14M"}
string             string
char               {"#char": "X"}
keyword            ":foo" / ":ns/foo"  ← colon-prefix discriminator
symbol             {"#symbol": "foo"}
list / vector      array  (round-trips as Vector)
map (string keys)  object {"k": v, ...}
map (other keys)   object — non-string keys serialized as EDN
set                {"#set": [...]}
tagged             {"#tag": "ns/name", "body": ...}
inst               {"#inst": "2026-04-28T16:00:00Z"}
uuid               {"#uuid": "550e8400-..."}
```

### API

```rust
use wat_edn::{to_json_string, to_json_string_pretty,
              from_json_string, edn_to_json, json_to_edn};

let v = parse(r#"#myapp/Order {:id 1 :tags #{:vip}}"#)?;

// To JSON string (compact)
let s: String = to_json_string(&v);
// → {"#tag":"myapp/Order","body":{":id":1,":tags":{"#set":[":vip"]}}}

// To pretty JSON string
let s: String = to_json_string_pretty(&v);

// JSON string → EDN OwnedValue
let v2: OwnedValue = from_json_string(&s)?;

// Or work with serde_json::Value directly
let jv: serde_json::Value = edn_to_json(&v);
let back: OwnedValue = json_to_edn(&jv)?;
```

### Lossy conversions to be aware of

- **Lists collapse to vectors.** EDN `(1 2 3)` and `[1 2 3]` both
  serialize as JSON `[1, 2, 3]`; round-trip back yields `Vector`.
  If list-vs-vector distinction matters for your data, structure
  it differently.
- **Negative-zero floats.** JSON's `-0.0` round-trips as `0.0`.
- **Map insertion order.** JSON objects preserve insertion order
  in serde_json (and in cheshire 5+); equality semantics on the
  EDN side are unordered for Maps regardless.

### Use cases

This is built for emitting EDN-typed data to JSON-only sinks:
CloudWatch logs, HTTP APIs, JavaScript front-ends, Kafka topics
with JSON encoders. JSON consumers see clean JSON for the parts
they care about; type-aware downstream readers (the same
`wat-edn` binary, or `wat-edn-clj` on the Clojure side) recover
the original EDN types byte-for-byte.

---

## 8. Pretty-print

```rust
use wat_edn::write_pretty;

let v = parse(r#"{:asset :BTC :tags #{:vip} :nested [1 [2 [3]]]}"#)?;
println!("{}", write_pretty(&v));
```

Output:

```text
{:asset :BTC
 :tags #{:vip}
 :nested [1
          [2
           [3]]]}
```

Layout rules:
- Scalars stay on one line
- Small all-scalar collections (≤ 8 elements) inline
- Larger or nested collections break per element
- Maps always break per entry (more readable for keyed data)
- Tagged values keep tag-and-body cohesion: `#ns/name <body>`

The contract: pretty-printed output PARSES BACK to the same
`Value`. Round-trip identity is locked by `tests/pretty.rs`. The
visual layout is taste; the byte-equivalence after pretty is a
guarantee.

---

## 9. Spec extensions

`wat-edn` accepts and emits a few forms beyond what the EDN spec
strictly defines, all Clojure-aligned and round-trip-symmetric:

### String escapes

| Escape | Meaning | Spec? |
|---|---|---|
| `\n \t \r \\ \"` | named control chars + quote + backslash | YES |
| `\b` | U+0008 backspace | wat-edn extension (JSON parity) |
| `\f` | U+000C form feed | wat-edn extension (JSON parity) |
| `\/` | literal `/` | wat-edn extension (read-only, JSON parity) |
| `\uXXXX` | Unicode scalar value | wat-edn extension |

### Character names

| Form | Meaning | Spec? |
|---|---|---|
| `\c` | single char `c` | YES |
| `\newline \return \space \tab` | named whitespace | YES |
| `\uXXXX` | Unicode scalar value | YES |
| `\formfeed` | U+000C | wat-edn extension |
| `\backspace` | U+0008 | wat-edn extension |

### Non-finite floats

EDN doesn't define NaN or ±Infinity. wat-edn emits namespaced
sentinel tags so `f64` round-trips losslessly:

```text
NaN          → #wat-edn.float/nan nil
+Infinity    → #wat-edn.float/inf nil
-Infinity    → #wat-edn.float/neg-inf nil
```

Other EDN readers see them as ordinary user tags and may pass
through, ignore, or install handlers. wat-edn's parser
recognizes them and reconstructs `f64::NAN` / `INFINITY` /
`NEG_INFINITY`.

### Why these are documented up-front

A future strict-mode flag will gate them off for spec-pure output.
Until then, every extension is intentional, symmetric on both
read and write paths, and called out so consumers know what to
expect. See `src/lib.rs` for the canonical list (also enforced by
`tests/spec_strict.rs`).

---

## 10. The Clojure side

`wat-edn-clj/` is the companion Clojure library. Same wire
convention as Rust; reads and writes the same EDN bytes.

### Add to deps.edn

```clojure
{:deps {wat-edn-clj/wat-edn-clj
        {:local/root "wat-rs/crates/wat-edn/wat-edn-clj"}
        cheshire/cheshire {:mvn/version "5.13.0"}}}
```

### Header-file pattern (the killer feature)

`.wat` files declare types ONCE — the same artifact wat-rs's type
checker consumes (as code) is the artifact wat-edn-clj consumes
(as schema):

```clojure
(require '[wat-edn.core :as wat])

(wat/load-types! "shared.wat")
(wat/list-types)
;; => [enterprise.config/SizeAdjust
;;     enterprise.observer.market/TradeSignal ...]

;; Build a typed value (validates against the schema):
(wat/gen 'enterprise.config/SizeAdjust
         {:asset :BTC :factor 1.5 :reason "vol spike"})
;; => #enterprise.config/SizeAdjust {:asset :BTC, :factor 1.5,
;;                                    :reason "vol spike"}

;; Or get the EDN string directly:
(wat/emit 'enterprise.config/SizeAdjust {...})
;; => "#enterprise.config/SizeAdjust {:asset :BTC, ...}"

;; Validation throws BEFORE bytes leave Clojure:
(wat/gen 'enterprise.config/SizeAdjust
         {:asset "BTC"   ; ← schema says :Keyword, got String
          :factor 1.5
          :reason "x"})
;; => throws ex-info {:tag enterprise.config/SizeAdjust
;;                    :errors [{:field :asset
;;                              :expected "Keyword"
;;                              :got "BTC"}]}
```

### Untyped reading (any wat.* tag)

```clojure
(wat/read-str "#wat.core/Vec<i64> [1 2 3]")        ;; => [1 2 3]
(wat/read-str "#wat.core/Some<f64> 3.14")          ;; => [::wat-edn.core/some 3.14]
(wat/some-variant? *1)                              ;; => true
(wat/unwrap-some  *2)                              ;; => 3.14
```

Default reader handles Vec/HashMap/HashSet/HolonAST variants,
Some/None/Ok/Err sums, and falls through to `tagged-literal` for
anything not in the wat.* namespace.

### JSON conversion

```clojure
(require '[wat-edn.json :as wj])

(wj/to-json-string {:asset :BTC :tags #{:vip}})
;; => {":asset":":BTC",":tags":{"#set":[":vip"]}}

(wj/from-json-string ...)
;; => recovers EDN types from sentinel keys
```

Same wire convention as the Rust side. Tested cross-language by
`interop-tests/` (see [§11](#11-cross-language-interop)).

### Pretty-print

```clojure
(wat/pretty-edn {:asset :BTC :nested [1 [2 [3]]]})
;; → uses clojure.pprint with the wat-edn print-method extensions
```

### Variant constructors

```clojure
(wat/some-of 42)        ;; → #wat.core/Some 42
(wat/none-of)           ;; → #wat.core/None nil
(wat/ok-of "fine")      ;; → #wat.core/Ok "fine"
(wat/err-of "boom")     ;; → #wat.core/Err "boom"
```

These survive both EDN write/read and JSON conversion. See
`wat-edn-clj/README.md` for the standalone Clojure-only docs.

---

## 11. Cross-language interop

Empirical proof that bytes flow both directions across the
language boundary. Lives at `interop-tests/` (separate Cargo
project, not a workspace member).

Four handshakes verified end-to-end:

```text
1. wat-edn (Rust) → pure clojure.edn
   Rust emits EDN; Clojure's reference reader parses natively.
   No helper library required.

2. Pure Clojure pr-str → wat-edn (Rust)
   Clojure emits via tagged-literal + pr-str; wat-edn parses
   with full canonicalization (#inst → DateTime, #uuid → Uuid).

3. wat-edn-clj (schema-driven) → wat-edn
   Clojure loads shared.wat as schema, validates via gen,
   emits typed EDN; wat-edn parses cleanly.

4. EDN ↔ JSON ↔ EDN cross-language
   Rust EDN → wat-edn::to_json_string → cheshire/parse →
   wat-edn-clj/edn->json → cheshire/generate →
   wat-edn::from_json_string → Rust EDN
   Round-trip identity at every leg.
```

Run them:

```sh
cd interop-tests
cargo build --release

# Handshake 1: wat-edn → pure Clojure
cargo run --release | clojure -M clj/consume.clj

# Handshake 4: EDN ↔ JSON cross-language
echo '#myapp/Order {:id 1}' \
  | cargo run --release --bin json_producer \
  | clojure -Sdeps '{:paths ["../wat-edn-clj/src"] :deps {cheshire/cheshire {:mvn/version "5.13.0"}}}' \
            -M clj/json_passthrough.clj \
  | cargo run --release --bin json_consumer
```

See `interop-tests/README.md` for the full pipeline matrix.

---

## 12. Performance

### Measured throughput

```text
parse small  [1 2 3 4 5]              56 MB/s     0.19 µs/op
parse realistic blob (416B)          271 MB/s     1.46 µs/op
parse string-heavy (395B)            510 MB/s     0.74 µs/op
parse identifier-heavy (300B)        149 MB/s     1.91 µs/op
parse large flat (50-map vec, 1.7KB) 115 MB/s    14.7  µs/op
parse deeply nested (30 levels, 62B)  30 MB/s     1.94 µs/op

write small  [1 2 3 4 5]              111 MB/s    0.09 µs/op
write realistic blob                  996 MB/s    0.40 µs/op
write string-heavy                    858 MB/s    0.44 µs/op
write identifier-heavy                605 MB/s    0.47 µs/op
write large flat                      308 MB/s    5.4  µs/op
write deeply nested                   246 MB/s    0.25 µs/op
```

For comparison: serde_json on similar JSON measures ~500-1000 MB/s
parse on the same hardware. wat-edn is competitive while doing
strictly more work per token (typed tag dispatch, namespaced
symbols, big-number suffix recognition, spec-strict rejection).

### How to reproduce

```sh
cargo run --release --example bench -p wat-edn
```

The harness uses `std::time::Instant` over a million iterations
(or 50k–200k for larger inputs) on six fixtures: small, realistic
blob, identifier-heavy, string-heavy, large flat, deeply nested.
Output reports both MB/s and µs/op.

### Performance principles applied

- Hand-rolled byte-level lexer; no regex
- Single-pass recursive descent
- `Cow<'a, str>` for zero-copy strings on the lexer fast path
- `CompactString` (24-byte inline) for Symbol/Keyword/Tag names
- `Box<BigInt>` and `Box<BigDecimal>` to keep `Value` enum small
  for cache-friendly `Vec<Value>`
- `memchr3` for the writer's escape-byte search
- `#[inline]` on escape-codec helpers
- Direct `push_str` writers for symbols/keywords/tags (bypasses
  `fmt::Formatter` machinery — measured ~2× speedup vs `Display`)
- `Vec::with_capacity(8)` on container open (measured against 4
  and 16; 8 is the balanced choice)

### Performance non-decisions

- `lexical-core` for `f64` parsing was tried and reverted —
  per-call overhead exceeds savings on our workload (small numeric
  tokens). std::str::parse stays.
- `serde` integration (Serialize/Deserialize for `Value`) is
  available behind no flag yet; v0.2 candidate.
- SIMD-accelerated whitespace scanning beyond what `memchr` already
  provides is a v0.3+ candidate (the deeply-nested case at 30 MB/s
  is the only place it'd matter measurably).

---

## 13. Spec coverage and conformance

`wat-edn` is a second conforming implementation of the EDN spec —
peer to Clojure's reference reader.

### Spec types implemented

Every literal type defined by [edn-format/edn][edn-spec]:

- `nil`, `true`, `false`
- integers (`i64`), big integers (`42N`)
- floats (`f64`), big decimals (`3.14M`)
- strings with full escape support
- characters (named + `\uXXXX`)
- symbols, namespaced symbols
- keywords, namespaced keywords
- lists, vectors, maps, sets
- tagged elements with arbitrary nesting
- built-in `#inst` (RFC 3339) and `#uuid` (canonical)
- comments (`;`) and discard (`#_`)

### Spec rejections enforced

`tests/spec_strict.rs` locks every spec-mandated `MUST NOT`:
leading zeros, `:/` keyword, backslash + whitespace, numeric
first-char in name, leading-dot-then-digit, dangling tag in
collections, user tag without namespace, `#_` at top-level alone,
keyword starting with `::`, etc.

### Conformance verified by 5 ignorant ward casts

The `/ignorant` ward (a project-internal review process) ran
five times against the spec; the residual converged to:

- 0 critical divergences
- 4 documented inventions (the Clojure-aligned extensions in §9)
- 2 minor rough paths (set/map duplicate uniqueness — spec uses
  `should`; discard suppressing built-in validation — implemented)

### Implementation status

[edn-spec]: https://github.com/edn-format/edn

```text
Tests:  313 Rust + 39 Clojure (96 assertions)
        Zero failures, zero ignored.

Suites: 26 lib unit + 16 accessors + 4 display_equivalence
        + 8 pretty + 16 json + 176 comprehensive + 7 round_trip
        + 23 spec_conformance + 36 spec_strict + 1 doctest
```

---

## 14. Error handling

Two error types, both `thiserror`-derived:

```rust
pub enum Error {            // EDN parse errors
    Parse { pos: usize, kind: ErrorKind },
}

pub enum ErrorKind {
    UnexpectedEof,
    UnexpectedByte(u8),
    InvalidEscape(u8),
    InvalidUnicode(String),
    InvalidNumber(String),
    InvalidKeyword(String),
    InvalidSymbol(String),
    InvalidTag(String),
    InvalidChar(String),
    InvalidInst(String),
    InvalidUuid(String),
    UnclosedString / List / Vector / Map / Set,
    OddMapElements,
    TagWithoutElement(String),
    UserTagMissingNamespace(String),
    Other(String),
}

pub enum JsonError {        // JSON conversion errors
    Parse(String),
    NumberOutOfRange(String),
    InvalidTag(String),
    InvalidInst(String),
    InvalidUuid(String),
    InvalidBigInt(String),
    InvalidBigDec(String),
    InvalidChar(String),
    InvalidSymbol(String),
    InvalidFloat(String),
    InvalidKeyword(String),
    InvalidMap(String),
}
```

Errors carry a byte-position `pos` for parse failures (counted from
the start of the input), making it easy to surface a precise
location. `ErrorKind` variants carry context strings for
diagnostics; programmatic dispatch should match on the variant
name, not the string.

```rust
match parse(input) {
    Ok(v) => /* ... */,
    Err(Error::Parse { pos, kind }) => match kind {
        ErrorKind::UnexpectedEof => /* truncated input */,
        ErrorKind::InvalidNumber(s) => /* bad numeric token */,
        ErrorKind::TagWithoutElement(t) => /* dangling tag t */,
        _ => /* generic */,
    },
}
```

---

## 15. Common gotchas

### Cow lifetime across function boundaries

`parse(input)` returns `Value<'_>` borrowing from `input`. If you
need to return a parsed `Value` from a function that takes `input`
as a parameter, lift to `OwnedValue`:

```rust
fn read_config(input: &str) -> Result<OwnedValue> {
    parse(input).map(Value::into_owned)
    // OR: parse_owned(input)
}
```

If you forget, the borrow checker stops you with a lifetime error.

### List vs Vector through JSON

EDN distinguishes `(1 2 3)` (list) from `[1 2 3]` (vector); JSON
has only arrays. List → JSON loses the list type; round-trip back
yields a Vector. If list-vs-vector distinction matters, use a
tagged value or restructure.

### Map insertion order

`Vec<(Value, Value)>` preserves insertion order. Equality is
unordered (per spec). If you need ordered comparison, compare the
inner `Vec` directly via `.iter().zip(other.iter())` instead of
`==`.

### Cheshire returns Integer, not Long

The Clojure JSON path: when round-tripping integers through
`cheshire/parse-string`, the result is `Integer`, not `Long`.
Clojure's `=` compares them as equal but Java `.equals` (used by
`TaggedLiteral`) does not. `wat-edn-clj/json` coerces to `Long`
on parse — this is documented in case you build your own bridge.

### Big numbers > 2^53 in JSON

JSON's safe-integer range is ±2^53. wat-edn emits values outside
that range as JSON strings (`"9007199254740993"`) and the parser
recovers them. Be aware that other JSON tools may not — if you
exchange data with consumers that require numeric form, use
`#bigint` / `#bigdec` sentinels explicitly.

### Set with duplicate elements

The EDN spec says sets contain "unique values" but uses the soft
verb. `wat-edn` permits duplicates on parse for graceful interop;
equality is multiset-based (sets compare equal iff they have the
same elements with the same multiplicities). If you need strict
uniqueness, post-process with a deduplication pass.

### `#_` discard semantics under built-ins

Per spec, handlers should not run during a discard. wat-edn
implements this — `[1 #_#inst "bad-date" 2]` parses cleanly to
`[1 2]` even though `"bad-date"` would normally fail
`#inst` validation. Documented and tested.

---

## 16. Where to go next

- **API reference:** `cargo doc --open -p wat-edn`
- **Source:** `src/` (~2000 LOC, well-commented; lexer.rs and
  parser.rs are the worth-reading core)
- **Test suites:** `tests/` (every public behavior locked)
- **Bench harness:** `examples/bench.rs`
- **The Clojure side:** [`wat-edn-clj/`](../wat-edn-clj/) — its
  own README and tests
- **Cross-language proof:** [`interop-tests/`](../interop-tests/)
  — the four-handshake verification matrix
- **Spec extension rationale and ward-driven design history:**
  `holon/scratch/2026/04/003-edn-typed-wire/` — the design arc
  that produced this crate, beat-by-beat
- **wat-rs language guide:** `wat-rs/docs/USER-GUIDE.md` — for
  the surrounding language wat-edn ships with

When you hit something this guide didn't prepare you for, the
gap is worth reporting. The fastest fix path is: write the
failing test, file the issue, point at the spec section if one
applies. The library's discipline relies on tests being the
documentation of intent.
