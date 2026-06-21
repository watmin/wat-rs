# Stone 255.1b-iv-c — `metadata-of` emits plain values + the closed-domain enum flip

**STRIKE-READY.** Probe RED-verified at HEAD (`69890b62`-descendant working tree).

## Why (the defect, dogfounded)

iv-c was scoped as "flip `:kind`/`:defined-in`/`:layer` keyword→enum." Dogfooding the
reflection surface as EDN (`wat-scripts/intrinsic-metadata.wat`) surfaced a deeper defect the
flip rides on: **`eval_metadata_of`'s intrinsic branch wraps EVERY value in
`Value::holon__HolonAST`** (`runtime.rs` ~10111, the `put` closure). The probe proves it —
`:kind` is literally:

```
holon__HolonAST(Bind(Atom("Keyword"), Atom("intrinsic")))
```

— a holon VSA *bind* composition. So the metadata map EDN-serializes with
`#wat-edn.holon/Keyword`, `#wat-edn.holon/Bool`, … tags: the holon algebra-AST encoder
leaking into reflection. This is the same `HolonAST`-as-`EdnRepresentable` crutch the codebase
has been rooting out — `impl EdnRepresentable for Value` (comms/mod.rs:794) means plain values
already serialize cleanly; `value_to_edn_with` already has a clean `Value::Enum` arm
(edn_shim.rs:1671).

## What it delivers

`(metadata-of <intrinsic>)` returns a map of **plain wat values**, the three closed-domain
fields as **`Value::Enum`** (locked-record-model §5):

| key | HEAD (holon-wrapped) | iv-c (plain) |
|---|---|---|
| `:name` | `holon__HolonAST` | `Value::wat__core__keyword` |
| `:arity` | `holon__HolonAST` | `Value::i64` |
| `:pure` / `:deterministic` | `holon__HolonAST` | `Value::bool` |
| `:doc` / `:added` / `:ret` | `holon__HolonAST` | `Value::String` |
| `:kind` | `holon__HolonAST(:intrinsic)` | `Value::Enum :wat::runtime::Kind / Intrinsic` |
| `:defined-in` | `holon__HolonAST(:rust)` | `Value::Enum :wat::runtime::DefinedIn / Rust` |
| `:layer` | `holon__HolonAST(:substrate)` | `Value::Enum :wat::runtime::Layer / Substrate` |

EDN then renders cleanly: `{:name :wat::core::Bytes::to-hex :arity 1 :pure true … :kind
:wat.runtime.Kind/Intrinsic …}` — no `#wat-edn.holon/` tag anywhere.

## The one contract decision (pinned)

The closed-domain values are `Value::Enum` (option **b**, builder-approved), backed by **§5's full
enum mirror**:
- **Rust enum mirrors** (`Kind`/`DefinedIn`/`Layer`) — the derivation site uses the enum, so the
  compiler rejects a typo'd variant. Each has a method → `Value::Enum { type_path:
  ":wat::runtime::Kind", variant_name: "Intrinsic", fields: vec![] }`.
- **wat `defenum` ×3** so a wat consumer matches exhaustively + the type resolves/round-trips:
  ```
  (:wat::core::defenum :wat::runtime::Kind       :Macro :Fn :Intrinsic)
  (:wat::core::defenum :wat::runtime::DefinedIn  :Wat :Rust)
  (:wat::core::defenum :wat::runtime::Layer      :Substrate :Userland)
  ```
  **Capitalized variants** per §5 (also dodges any `:fn` keyword-legality question).
- The intrinsic branch only ever emits `Kind::Intrinsic`, `DefinedIn::Rust`, `Layer::Substrate`
  (all intrinsics are intrinsic/rust/substrate); the other variants exist for the future
  user-form branch + exhaustiveness.

## Out of scope (affirmative cuts)

- **The user-form branch** (`runtime.rs` ~10136-10148) — it dumps `binding_metadata` verbatim
  (arbitrary user-attached quoted forms, legitimately AST) and does NOT emit the baseline. User-form
  baseline-parity is a separate locked-record-model item. Do NOT touch it.
- **No `show-source`/`:source`, no `@see` check, no `(doc …)` accessor** — those are iv-b-v.

## Rooms (read in order)

1. `tests/nursery/probe_arc255_ivc_metadata_plain_values.rs` — the RED probe = the contract.
   Copy its expected shape EXACTLY.
2. `src/runtime.rs` ~10104-10134 — `eval_metadata_of` intrinsic branch. The `put` closure
   (~10111) wraps in `Value::holon__HolonAST`; the seven `put(...)` calls (~10119-10133) are the
   sites. Replace with plain-value inserts; `:kind`/`:defined-in`/`:layer` → `Value::Enum` via the
   new mirrors. (Keys stay `Value::wat__core__keyword(":name")` etc — unchanged.)
3. `src/value/value.rs:978` — `EnumValue { type_path, variant_name, fields }`, the construction
   shape. Cf. existing `Value::Enum(Arc::new(EnumValue { … }))` at runtime.rs:21267.
4. **Rust enum mirrors home:** `src/intrinsic/mod.rs` (the registry home) — define `Kind`/
   `DefinedIn`/`Layer` + a `fn to_enum_value(&self) -> Value`. (They classify the baseline; the
   registry home is the natural seat. Weigh may relocate.)
5. **wat defenums:** new file `wat/runtime-meta.wat` (the 3 defenums above; no deps beyond core),
   wired into `STDLIB_FILES` (`src/stdlib.rs:34`) anywhere after `wat/core.wat`. Mirror the
   `WatSource { path, source: include_str!(…) }` entries.
6. `src/intrinsic/reflect.rs` — the `eval_intrinsic_examples` `///` doc still says the examples
   are a `Value::Struct` and *"tuples"*; iv-b2-b made them `Value::wat__Record` records. Fix the
   doc (the doc-cannot-lie seam lying about itself). Update the `@ret` line too (`tuples`→`records`).

## STOP triggers (surface, do not improvise)

1. If a capitalized variant keyword (`:Fn`, `:Macro`, …) is rejected by `defenum` — STOP and
   surface; do NOT silently down/re-case (the casing is a pinned §5 decision).
2. If `metadata-of` turns out to have a checker scheme asserting the map's value type as
   `HolonAST` (grep found none in `check.rs`) so that plain values break the check — STOP and
   surface; do not weaken the scheme blindly.
3. If making the floor green seems to require editing the user-form branch or any out-of-scope
   file — STOP; it means the scope is wrong, surface it.

## Expectations (independent scorecard — fixed before the strike)

| # | what | command | expected |
|---|---|---|---|
| 1 | the probe goes green | `cargo test --release -p wat --test nursery metadata_of_emits_plain_values` | 1 passed |
| 2 | EDN is clean (no holon tags) | `./target/release/wat ./wat-scripts/intrinsic-metadata.wat` | output contains `:wat.runtime.Kind/Intrinsic`; NO `#wat-edn.holon/` substring |
| 3 | lib floor holds | `cargo test --release -p wat --lib` | 953 pass / 36 ign / 1 (pre-existing baseline) |
| 4 | nursery: only the iv-c probe flips green | `cargo test --release -p wat --test nursery` | the 4 pre-existing fails remain; iv-c probe now passes |
| 5 | clippy clean on touched files | `cargo clippy --release -p wat` | no new warnings on runtime.rs / intrinsic/ |

Runtime prediction: 25–40 min (one new wat file + STDLIB wiring + ~3 Rust enums + the put-closure
rewrite + 2 doc lines). Trap-door: the `defenum` load-order in `deporder.wat` may need the new
file declared; if a load-order error appears, that's the fix site (not a STOP).
