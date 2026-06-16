# DESIGN + BRIEF — 234.7b: holon records round-trip on the wire

## Frame (the corrected model)

A holon record's **data** has three encodings of the *same* thing: `holon_form` (the VSA `HolonAST`,
where identity lives — `Eq`/`Hash` delegate to it, `value.rs:660,884`), the EDN wire form, and the
positional `struct_form` (a Rust-side cache). **holon doesn't have a "struct" — `struct_form` is a
derived view of the holon data** (literally the leaf values of the holon_form's `Bundle`).

Since **the holon encoding is itself EDN-encodable** (`holon_ast_to_edn` ↔ `edn_to_holon_ast` round-trip
exactly — newtypes claim 1), the wire carries the **holon_form in its edn encoding**, and decode reads
it back *unchanged*, then **projects** `struct_form` out of it. No recompute, no macro replication.

**Why NOT recompute holon_form on decode** (the rejected alternative): `to_holon_inner` *errors* on base
records (`runtime.rs:31212-31224`) and the holon_form *assembly* (`Bind(Atom(class), Bundle[…])`) lives
only in the `Record.wat` macro (`wat/Record.wat:230-235`). Rebuilding it in Rust would replicate the
macro — a second construction algorithm that must stay bit-identical forever = the exact divergence-class
"no tolerance for latent bugs" forbids. Riding the holon_form as edn avoids it entirely.

This stone touches **only** `wat__holon__Record`. Base records (234.7a) are done and untouched.

## Contract (pinned)

1. **Encode** (`edn_shim.rs` — the holon arm split out in 234.7a, currently still field-N struct_form map):
   replace its body with the holon_form-as-edn wire:
   ```rust
   Value::wat__holon__Record { class_fqdn, holon_form, .. } => {
       let tag = tag_from_type_path(class_fqdn);
       OwnedValue::Tagged(tag, Box::new(holon_ast_to_edn(holon_form)))
   }
   ```
   (Drop the `struct_form` from this arm's match binding — it's no longer read here; the leaf values live
   in `holon_form`.) The body is a `#wat-edn.holon/Bind[…]` value, NOT a map — that's the decode signal.
2. **Decode — add `reconstruct_holon_record`** and route to it. In `tagged_to_value`'s body match
   (`edn_shim.rs:1899-1903`, currently `Map → struct/record`, `Vector → enum`, `Nil → enum-unit`), add:
   a `Edn::Tagged(inner, _)` body whose `inner.namespace() == "wat-edn.holon"` **and** whose class path
   resolves to `TypeDef::Record` → `reconstruct_holon_record`. (A holon-tagged body under a Record class
   is unambiguously a holon record; a `Map` body under a Record class is a base record — 234.7a.)
   ```rust
   fn reconstruct_holon_record(ns, name, body: &Edn, types) -> Result<Value, EdnReadError> {
       // 1. holon_form back, EXACT, via the proven round-trip:
       let holon_form = edn_to_holon_ast(body)?;          // HolonAST
       // 2. project struct_form from the Bundle leaves:
       //    holon_form == Bind(Atom(class), Bundle[ Bind(name, val_node), ... ])
       //    struct_form[i] = from_holon_item(val_node_i)   (pure; runtime.rs:11907)
       //    STOP if the shape isn't Bind(_, Bundle(_)) — that's a malformed holon record.
       // 3. class_fqdn from the wire tag path (strip leading ':').
       Ok(Value::wat__holon__Record { class_fqdn, struct_form: Arc::new(fields), holon_form: Arc::new(holon_form) })
   }
   ```
   `from_holon_item` returns `Result<Value, EvalBreak>` — bridge `EvalBreak → EdnReadError` (a
   spanless decode error, like the other reconstruct_* errors).
3. **struct_form projection**: walk `holon_form`: it must be `HolonAST::Bind(_class, bundle)` where
   `bundle` is `HolonAST::Bundle(children)`; each child is `HolonAST::Bind(_name, val_node)`; push
   `from_holon_item(val_node)` in order. Reuse the structure `record_assoc` walks (`runtime.rs:13207-13261`)
   as the worked reference for the exact shape.

## Disconfirming probe — `tests/probe_arc234_7b_holon_record_roundtrip.rs` (NEW)

A **holon** record (`:wat::holon::Record::def`) through `value → edn → value`, asserting THREE contracts
(equality alone is insufficient — `Eq` delegates only to holon_form, so it would pass even if the
struct_form projection were wrong):
1. **Identity**: `decoded == original` (holon_form round-tripped exactly).
2. **Projection**: a **field accessor** on the decoded record returns the correct field value (proves
   struct_form was projected correctly — the gap equality can't see).
3. **Shape**: the written edn string contains `#wat-edn.holon` (rode the holon encoding, not a field-N map).

**RED at HEAD**: holon record encodes as a field-N map (no `#wat-edn.holon` body) AND decode errors
`UnknownTag` (no holon-record decode path). GREEN after.

Find the holon-record constructor + edn write/read verbs in `wat-tests/core/record-def.wat`
(`:wat::holon::Record::def :test::rd::HPt [...]`) and `wat-tests/edn/newtypes.wat` (write/read) — do not
guess. Model the harness on `tests/probe_arc234_7a_base_record_roundtrip.rs` (just shipped).

## Blast radius

`src/edn_shim.rs` (holon encode arm + new `reconstruct_holon_record` + one dispatch arm) + the new probe.
Base-record path (234.7a) and struct/enum paths untouched. No holon record round-trips today, so nothing
asserts the old field-N holon shape (confirm with the full sweep).

## STOP triggers (halt + surface)

1. If `from_holon_item` cannot be called from `edn_shim` (needs eval context after all), STOP and report —
   do not stand up an eval context in the decoder.
2. If a round-tripped holon record's `holon_form` is NOT `Bind(_, Bundle(_))` (the shape the projection
   assumes), STOP and report the actual shape — do not guess a different walk.
3. If `edn_to_holon_ast` is not directly callable on the inner body `Edn` in that context, STOP and report
   the actual holon-decode entry point.

## Verify (run + report each)

Baseline at the 234.7a HEAD: lib `928 passed / 36 failed`.
1. `cargo build --release -p wat 2>&1 | tail -5` — clean.
2. `cargo test --release -p wat --test probe_arc234_7b_holon_record_roundtrip 2>&1 | grep "test result"` — passed (3 contracts).
3. `cargo test --release -p wat --test probe_arc234_7a_base_record_roundtrip 2>&1 | grep "test result"` — still passed (base unaffected).
4. `cargo test --release -p wat --lib -- --test-threads=1 2>&1 | grep "test result"` — passed ≥ 928; `failed == 36`.
5. `cargo test --release -p wat 2>&1 | grep -E "test result: FAILED" | head` — no NEW failing binaries.

Commit nothing — the orchestrator weighs the diff and commits on green.
