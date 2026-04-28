# interop-tests — wat-edn ↔ Clojure proof artifacts

Empirical verification that bytes flow correctly between
**wat-edn (Rust)** and **clojure.edn (the reference reader)** plus
**wat-edn-clj** (the schema-driven Clojure side).

This is a separate Cargo project (NOT in the wat-rs workspace).
Build standalone: `cd interop-tests && cargo build --release`.

Run requires `clojure` 1.11+ on `$PATH`.

## What's here

```
src/main.rs            Rust binary: builds a TradeSignal-shaped
                       Value and writes it as EDN to stdout.

src/bin/reader.rs      Rust binary: reads EDN from stdin
                       (typically Clojure's pr-str output) and
                       asserts the parsed structure.

src/bin/typed_reader.rs Rust binary: reads schema-validated EDN
                       (Clojure consumer used wat-edn-clj/load-types!
                       + gen) and asserts the typed body.

clj/consume.clj        Clojure consumer using PURE clojure.edn —
                       no wat-edn-clj dep required. Proves wat-edn's
                       output is spec-conforming EDN.

clj/produce.clj        Clojure producer using only tagged-literal +
                       pr-str (no wat-edn-clj dep). Proves wat-edn
                       reads Clojure's pr-str output cleanly.

clj/produce_typed.clj  Clojure producer using wat-edn-clj. Loads
                       schema from ../wat-edn-clj/wat/shared.wat,
                       validates fields via wat/gen, emits typed
                       EDN. Proves the schema-driven pipeline.
```

## Run the four handshakes

### 1. wat-edn → pure Clojure

```sh
cd interop-tests
cargo run --release | clojure -M clj/consume.clj
```

Confirms wat-edn's output parses with stock `clojure.edn/read`.
Built-in `#inst` / `#uuid` canonicalize to `Date` / `UUID`. User
tags surface as `tagged-literal` pairs.

### 2. Pure Clojure → wat-edn

```sh
clojure -M clj/produce.clj | cargo run --release --bin reader
```

Confirms wat-edn parses Clojure's `pr-str` output. UTF-8 strings
(em-dash) survive. `#inst` → `chrono::DateTime<Utc>`. `#uuid` →
`uuid::Uuid`.

### 3. wat-edn-clj → wat-edn (schema-driven)

```sh
clojure \
  -Sdeps '{:paths ["../wat-edn-clj/src"]}' \
  -M clj/produce_typed.clj \
  | cargo run --release --bin typed_reader
```

Confirms the **header-file architecture**: Clojure loads
`shared.wat`, validates a value via `wat/gen`, emits typed EDN;
wat-edn parses with full structural assertion. ONE schema, two
language readers.

### 4. EDN ↔ JSON cross-language

```sh
echo '#myapp/Order {:id 1 :name "x"}' \
  | cargo run --release --bin json_producer \
  | clojure \
      -Sdeps '{:paths ["../wat-edn-clj/src"] :deps {cheshire/cheshire {:mvn/version "5.13.0"}}}' \
      -M clj/json_passthrough.clj \
  | cargo run --release --bin json_consumer
```

The full chain:
- Rust wat-edn parses the EDN string
- Rust wat-edn::to_json_string converts to JSON
- Clojure wat-edn.json/from-json-string parses (same wire convention)
- Clojure wat-edn.json/to-json-string re-emits
- Rust wat-edn::from_json_string parses back to EDN
- All steps verified by structural assertion in json_consumer

Wire format documented at `crates/wat-edn/src/json.rs`. Both
sides agree on sentinel-key tagged objects (`#tag`, `#set`,
`#bigint`, `#bigdec`, `#float`, `#char`, `#symbol`, `#inst`,
`#uuid`); keywords use `":foo"` colon-prefix discriminator.

## What this proves

```
wat-edn produces EDN bytes that:
  ✓ pure clojure.edn reads natively
  ✓ wat-edn-clj reads with type validation

wat-edn reads EDN bytes that:
  ✓ Clojure pr-str produces (untyped tagged-literal)
  ✓ wat-edn-clj/emit produces (schema-validated)

shared.wat is the SINGLE SOURCE OF TRUTH:
  ✓ same file reads as code (wat-rs type checker)
  ✓ same file reads as schema (wat-edn-clj load-types!)
  ✓ no codegen step, no separate IDL
```

The architecture wat-edn ships with: peer-to-peer EDN
implementation in Rust, alongside Clojure's reference reader,
with `.wat` files as the cross-language schema header.
