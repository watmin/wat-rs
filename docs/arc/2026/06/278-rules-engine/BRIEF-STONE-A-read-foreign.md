# BRIEF — Stone A: `:wat::edn::read-foreign` (the dynamic EDN decode)

**Read first:** `DESIGN-STONE-A-read-foreign.md` (the contract, pinned) + this brief. Grounded against HEAD
`a5a48aa1` (A.0 floor clean). Prior comparable to copy for SHAPE: `eval_edn_read` and its wiring (the sibling
verb you mirror). All `file:line` below verified this session.

## The work (one paragraph)
Add `:wat::edn::read-foreign` — a sibling of `:wat::edn::read` that, on an **unknown** tag, reconstructs a
**self-describing dynamic value** instead of raising `UnknownTag`: a map body → `ForeignRecord` (fqdn class +
its own ordered key→value fields), a vector body → `ForeignVariant` (enum-class + variant + positional fields);
recursive (a foreign record containing a foreign variant field decodes all the way down); re-serializes faithfully
back to the same `#tag {…}` / `#tag [...]`. Strict `read` is **untouched** (unknown tag still errors — the
no-hidden-failures floor, R41). Add the two dynamic `Value` variants + a baked `:wat::edn::` accessor surface
(`ForeignRecord/get`, `ForeignRecord/class`, `ForeignVariant/variant`, `ForeignVariant/enum-class`,
`ForeignVariant/fields`). Drive the RED gate `tests/value/probe_arc278_read_foreign.{rs,wat}` to GREEN without
weakening it.

## Rooms — read in order (why each)
1. `src/value/value.rs:42` (the `Value` enum), `:962` (`AggregateValue`), `:1004` (`EnumValue`), `:122`
   (`RustOpaque` — the ONLY existing dynamic variant; yours are new). **Add** `Value::ForeignRecord(Arc<…>)` +
   `Value::ForeignVariant(Arc<…>)` and their structs here. **ForeignRecord must self-carry its keys** (ordered
   `Vec<(String, Value)>` — NOT reuse `Value::Aggregate`, whose writer looks field names up in the registry at
   `edn_shim.rs:2971` and would fall to `field-{i}`, losing the foreign keys). ForeignVariant carries
   `{ enum_class: String, variant: String, fields: Vec<Value> }`.
2. `src/edn_shim.rs:183` `eval_edn_read` — the verb to mirror. **Add** `eval_edn_read_foreign` beside it: same
   String→`parse_owned` path, but decode via the foreign entry (room 3), same `TrackedValue::new(…,
   Provenance::RuntimeBuilt { producer: ":wat::edn::read-foreign", … })` shape (`:214`).
3. The decode chain — **thread a `foreign: bool` mode** (peer to the existing `allow_caps: bool`, keep
   `allow_caps=false` in the foreign path — foreign decode is untrusted): `edn_to_value` (`:1214`, thin wrapper) →
   add `edn_to_value_foreign` (thin wrapper setting `foreign=true`) → `edn_to_value_caps` (`:1226`) →
   `tagged_to_value` (`:2237`, `match body` at `:2359`) → `reconstruct_struct` (`:2457`) / `reconstruct_record`
   (`:2515`) / `reconstruct_enum_tagged` (`:2685`). Forward the flag through **every** recursive
   `edn_to_value_caps` call site (`:1262,:1269,:1281-1282,:1295,:2323-2324,:2346,:2501,:2555,:2721`) and the
   `edn_to_value` calls inside `tagged_to_value` (`:2293,:2303,:2304`) — inconsistent forwarding = broken nesting.
4. The three reroute hooks (foreign-mode ONLY; strict path unchanged): `src/edn_shim.rs:2470` (struct miss, map
   body → `ForeignRecord`), `:2529` (record miss, map body → `ForeignRecord`), `:2697` (enum miss, vector body →
   `ForeignVariant`). At each: `ns`/`name` + `entries`(or `items`) are in scope; class = `ns_to_wat_path(ns,name)`
   (`:2411`) for records / `ns_to_enum_path(ns)` (`:2692`) + `name` for variants; recursively decode fields
   **with `foreign=true`**.
5. The writer — `value_to_edn_with` (`src/edn_shim.rs:2878`). **Add two arms**: `Value::ForeignRecord` →
   `OwnedValue::Tagged(tag_from_type_path(class), Map(entries))` from the SELF-CARRIED keys (mirror the Aggregate
   arm `:2967-2992` but read keys off the ForeignRecord, NOT the registry); `Value::ForeignVariant` →
   `Tagged(tag_from_type_path("{enum_class}::{variant}"), Vector([fields]))` (mirror the Enum arm `:2994-3011`).
6. Registration: runtime dispatch `src/runtime.rs:3966` — add `":wat::edn::read-foreign" => …eval_edn_read_foreign
   (…).map_err(Into::into)` beside the `read` arm; type-checker `src/check.rs:19231` — add an
   `env.register(":wat::edn::read-foreign", …)` with the same `String -> T` scheme. Register the accessor verbs
   the same way (or as baked wat — your call; see room 7).
7. The accessor surface. `wat/edn.wat` (33 lines; `newtype` at `:32-33` is the WRONG idiom). The accessors are
   NOT plain field-accessors (`get` takes a KEY; the values are dynamic), so bake them as Rust verbs registered
   like `read-foreign` (room 6), OR declare minimal wat wrappers — your call, but they must type-check the probe.
   **Accessor contract (pinned):** foreign accessors traffic in `:wat::core::Value` at the dynamic boundaries
   (heterogeneous — R7 universal top): `ForeignRecord/get : (ForeignRecord, Keyword) -> Value`;
   `ForeignRecord/class : ForeignRecord -> String`; `ForeignVariant/variant : Value -> Keyword` (runtime-checks
   it is a ForeignVariant, raises a clean located error otherwise — no-hidden-failures);
   `ForeignVariant/enum-class : Value -> String`; `ForeignVariant/fields : Value -> Vector<Value>`. Register the
   type names `:wat::edn::ForeignRecord` / `:wat::edn::ForeignVariant` so annotations/returns resolve.

## Implementation sketch (fill it; do not invent the shape)
```
// value/value.rs
pub struct ForeignRecordValue  { pub class: String, pub fields: Vec<(String, Value)> } // ordered, self-carried keys
pub struct ForeignVariantValue { pub enum_class: String, pub variant: String, pub fields: Vec<Value> }
Value::ForeignRecord(Arc<ForeignRecordValue>) | Value::ForeignVariant(Arc<ForeignVariantValue>)

// edn_shim.rs — foreign entry (mirror edn_to_value :1214)
pub fn edn_to_value_foreign(edn, types) -> Result<Value, EdnReadError> { edn_to_value_caps(edn, types, /*allow_caps*/false, /*foreign*/true) }
// … thread `foreign` through the chain; at :2470/:2529 (map, foreign) build ForeignRecord from entries
//    (recursively decode each value with foreign=true); at :2697 (vector, foreign) build ForeignVariant from items.

// edn_shim.rs — writer arms (mirror :2967 / :2994) reading self-carried keys / enum_class+variant

// runtime.rs:3966 + check.rs:19231 — register read-foreign + the accessor verbs
```

## Blast radius
`src/value/value.rs`, `src/edn_shim.rs`, `src/runtime.rs` (one dispatch arm + accessor arms), `src/check.rs` (verb
registrations), possibly `wat/edn.wat` (accessor surface). NO change to strict `read`, to A.0's encoding, or to any
existing golden. New probe only: `tests/value/probe_arc278_read_foreign.{rs,wat}` (ON DISK, uncommitted — it is the
acceptance gate, confirmed RED at HEAD: `read_foreign_...` fails on `unknown function: :wat::edn::read-foreign`,
`strict_read_...` already passes. Do NOT edit the probe to pass; make the code satisfy it. It commits GREEN with your
implementation in one commit — no broken commits.)

## STOP triggers (halt + surface; do not improvise)
1. If making the probe green would require WEAKENING strict `read` (e.g. strict stops erroring on unknown tags) —
   STOP. Strict must stay strict (R41); foreign is a separate opt-in path.
2. If threading `foreign` cleanly through the recursive chain proves infeasible without a broader refactor — STOP
   and surface the exact site; do NOT half-thread it (broken nesting is a silent corruption).
3. If `ForeignRecord`/`ForeignVariant` cannot re-serialize to the EXACT `#tag {…}` / `#tag [...]` the reader
   consumed (round-trip identity) — STOP; the self-carried-keys representation exists precisely to guarantee this.
4. If the accessor type-flow can't make the probe's `(ForeignVariant/variant (ForeignRecord/get fr :kind))`
   type-check without loosening the checker — STOP and surface; the Value-at-the-boundary contract (room 7) is the
   intended shape.

## Acceptance (weighed by the ORCHESTRATOR's own re-run, not your report)
- `tests/value/probe_arc278_read_foreign` GREEN — both tests (the recursive foreign navigation; strict still errors).
- Whole floor: `cargo nextest run --release` back to the standing baseline (the known `no_inlined_wat` lint at 351
  + the known `wat-cli sigterm…polling_contract` flake that passes isolated) — ZERO new failures.
- `cargo clippy` clean on the touched files; content-integrity (the diff touches only the rooms above + the new
  Value variants/arms/registrations — nothing smuggled).
- Report: the load-bearing diff summary + which tests you ran + honest deltas. The orchestrator re-runs before commit.
