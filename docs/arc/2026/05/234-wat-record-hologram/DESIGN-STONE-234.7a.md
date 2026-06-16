# DESIGN + BRIEF — 234.7a: base records round-trip on the wire

## The bug (grounded against `552cd874`)

Records are **encode-only** on the EDN wire — the type *designated* for serialization can't survive a
round-trip, while structs (the *non-wire* type) round-trip fully and by-name. The inversion:

- **Encode** (`src/edn_shim.rs:2366-2380`): the record arm hardcodes `format!("field-{}", i)` for keys,
  ignoring `RecordDef.field_names` — which exists (`src/types.rs:2217`). Structs consult their def
  (`edn_shim.rs:2264-2272`).
- **Decode** (`src/edn_shim.rs:1899`): every tagged-map routes to `reconstruct_struct`, which matches
  **only** `TypeDef::Struct` and returns `UnknownTag` for a record (`edn_shim.rs:1971-1976`). There is
  **no `reconstruct_record`**.

Why it never bit: no record has crossed a real serialization boundary yet (defservice is thread-tier,
in-memory; holon/enum round-trip tests use enums). 6c.2's `AddressWire` is the first — which surfaced it.

This stone closes **base `wat__Record`** (no `holon_form`). Holon `wat__holon__Record` is **234.7b**
(it rides its `holon_form` as edn — a different mechanism; leave its arm untouched here).

## Contract (pinned)

1. **Encode — split the combined arm** (`edn_shim.rs:2366-2380`). Today:
   `Value::wat__holon__Record {..} | Value::wat__Record {..} => field-N map`. Split into two arms:
   - `Value::wat__Record { class_fqdn, struct_form }` → a **named** tagged-map: look up
     `types.and_then(|t| t.get(class_fqdn))` → `TypeDef::Record(def)` → use `def.field_names` as the map
     keys; **fallback** `field-{i}` when no def is found (mirror the struct arm at `edn_shim.rs:2264-2272`
     exactly, just `TypeDef::Record` instead of `TypeDef::Struct`).
   - `Value::wat__holon__Record { class_fqdn, struct_form, .. }` → **UNCHANGED** (keep the existing
     field-N map). 234.7b reworks this arm to ride `holon_form` as edn. Do not touch it beyond splitting.
2. **Decode — add `reconstruct_record`** (mirror `reconstruct_struct`, `edn_shim.rs:1964-2014`): resolve
   the path, match `TypeDef::Record(def)`, build a key→value map from the EDN entries, walk
   `def.field_names`/`def.field_types` in declaration order (by-name lookup; `UnknownStructField`-style
   error on a missing key), apply `rewrap_option_field` per field type, and build
   `Value::wat__Record { class_fqdn: Arc::new(path), struct_form: Arc::new(fields) }`.
3. **Dispatch — resolve the TypeDef** (`edn_shim.rs:1899`). Today: `Edn::Map(entries) => reconstruct_struct(...)`.
   Change so a tagged-map routes by the resolved def: look up the path's `TypeDef` →
   `Struct` → `reconstruct_struct`; `Record` → `reconstruct_record`; neither → the existing `UnknownTag`
   error. (Reuse `ns_to_wat_path` the way `reconstruct_struct` does it internally.)

## Disconfirming probe — `tests/probe_arc234_7a_base_record_roundtrip.rs` (NEW)

A **base** record through `value → edn → value`, asserting (1) round-trip **equality** and (2) the edn
string carries **named** keys, not `field-N`. **RED at HEAD** (encode emits `field-0`; decode errors
`UnknownTag`). GREEN after.

Model the harness on the existing wat-level round-trip: `wat-tests/core/record-def.wat` constructs base
records via `(:wat::Record::def :test::rd::Pt [x <- :wat::core::i64 y <- :wat::core::i64])`;
`wat-tests/edn/newtypes.wat` (claim 1) round-trips a value through `:wat::edn::write` → `:wat::edn::read`.
Compose: a Rust integration test using `startup_from_source` + `eval_in_frozen` that builds a `Pt`,
writes it to an edn string, reads it back, asserts equal **and** asserts the written string contains
`:x` / `:y` (not `field-0`). If the exact constructor verb or write/read verb differs, find it in those
two files — do not guess.

## Blast radius

`src/edn_shim.rs` (the three sites above) + the new probe. The holon arm stays field-N (234.7b).
No record asserts `field-N` anywhere today (grep: 3 `field-N` hits, all comments), and nothing
round-trips a record now — so existing tests are unaffected. Run the full lib + a broad test sweep to confirm.

## STOP triggers (halt + surface — do not improvise)

1. If `reconstruct_struct`'s path resolution (`ns_to_wat_path`) can't be reused for the record dispatch,
   STOP and report the actual resolution shape — do not hand-roll a parallel path resolver.
2. If a `TypeDef::Record`'s `def.field_names` is empty or absent where you expected names, STOP and report
   (the fallback handles no-def, but an *empty-named registered record* is a different, unexpected shape).
3. If splitting the encode arm shows holon records would lose their current behaviour, STOP — the holon
   arm must remain byte-for-byte as it is today (234.7b owns it).

## Verify (run + report each)

Baseline at `552cd874`: lib `928 passed / 36 failed` (the 36 pre-existing). After: pass count rises
(new probe + any unit tests), `failed == 36` unchanged.

1. `cargo build --release -p wat 2>&1 | tail -5` — clean.
2. `cargo test --release -p wat --test probe_arc234_7a_base_record_roundtrip 2>&1 | grep "test result"` — passed.
3. `cargo test --release -p wat --lib -- --test-threads=1 2>&1 | grep "test result"` — passed ≥ 928; `failed == 36`.
4. `cargo test --release -p wat --test probe_arc234_stone2b_defrecord_macro 2>&1 | grep "test result"` — passed (record macro regression).
5. `cargo test --release -p wat 2>&1 | grep -E "test result: FAILED" | head` — no NEW failing integration binaries vs baseline.

Commit nothing — the orchestrator weighs the diff and commits on green.
