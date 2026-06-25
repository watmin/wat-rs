# BRIEF — arc 293 base-struct unification, strike 2a: align `RecordDef` field shape to `StructDef`

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents. Do NOT use git worktrees. Do NOT commit.** If the work exceeds these rooms or hits a STOP trigger,
STOP and report — do not improvise.

Build/test: `cargo build --release -p wat`; `cargo test --release -p wat …`. After editing any `wat/*.wat`,
`touch tests/test.rs`. Trust forced clean builds (`cargo clean -p wat && cargo build --release -p wat`) if stale.

## Decisions pinned (four-questioned + builder-confirmed — do NOT re-litigate)

- **D2 — fields are ALWAYS typed.** `RecordDef`'s split `field_names: Vec<String>` + `field_types:
  Option<Vec<TypeExpr>>` → a single **`fields: Vec<(String, TypeExpr)>`** (mirrors `StructDef.fields`). The
  `Option` is annihilated: production records are always typed (`Record::def` emits the typed `recordtype` form
  since 293.3-records); the only `None` sites are 2 TEST helpers, fixed below.
- **D3 — this is strike 2a only** (field-shape align). The `StructDef`+`RecordDef` → `AggregateDef{kind}` merge
  is strike 2b (later). Do NOT introduce `AggregateDef`/`AggregateKind` or touch `TypeDef::Struct`/`Record` here.

## The work, in one paragraph

Behavior-preserving refactor: change `RecordDef` from `{ name, parent, field_names: Vec<String>, field_types:
Option<Vec<TypeExpr>> }` to `{ name, parent, fields: Vec<(String, TypeExpr)> }`. Annihilate the dead
string-literal `recordtype` parse branch (no callers — verified). Migrate every `.field_names` / `.field_types`
read site (~66, across 10 files) to the new shape (add `field_names()` / `field_types()` accessor methods on
`RecordDef` to keep most read sites a mechanical change). Fix the 2 test helpers. **No behavior changes** — the
SET-diff ∅ + the green deterministic record/surface tests are the oracle. Ride the compile cascade to zero.

## Rooms — read in order (re-ground before editing)

1. **`src/types.rs:199-211`** — the `RecordDef` struct. Replace the two fields with `pub fields: Vec<(String,
   TypeExpr)>`. **Add accessor methods** (minimize churn at read sites):
   ```rust
   impl RecordDef {
       pub fn field_names(&self) -> impl Iterator<Item = &str> { self.fields.iter().map(|(n, _)| n.as_str()) }
       pub fn field_types(&self) -> impl Iterator<Item = &TypeExpr> { self.fields.iter().map(|(_, t)| t) }
   }
   ```
   (If a read site indexes by position or needs a `Vec`, give it `self.fields[i].0` / `.1` or `.collect()` —
   pick the minimal change per site.)
2. **`src/types.rs:2057-2246`** — `parse_recordtype`. **Annihilate the string-literal branch** (`:2136-2160`,
   the `matches!(elems[0], StringLit)` arm that yields `(names, None)`) — it has NO callers (all `recordtype`
   emission is the typed form since 293.3-records). Keep the typed branch (`:2161-2229`); its output is already
   `(names, Some(types))` — zip them into `fields: Vec<(String, TypeExpr)>`. Empty `[]` → `fields: vec![]`.
   Final: `Ok(TypeDef::Record(RecordDef { name, parent, fields }))`. Update the `///` doc (drop the two-forms note).
3. **`src/edn_shim.rs:2439-2447`** — the reconstruct-record Option branch `if let Some(ftys) = &def.field_types`.
   Now unconditional: iterate `def.fields` / `def.field_types()`; `rewrap_option_field(fty, inner)` always has
   the type. Collapse the `else { inner }` dead arm.
4. **`src/edn_shim.rs:2748-2753`** (TEST helper `make_types`) — `field_names: vec![...], field_types: None` →
   `fields: vec![("minter-pid".to_string(), <i64 TypeExpr>), ("name".to_string(), <Vector<i64> TypeExpr>)]`.
   The real types: `minter-pid <- :wat::core::i64`, `name <- :wat::core::Vector<wat::core::i64>` (see
   `wat/spawn.wat:33-35`). Build the `TypeExpr` via the same path `parse_type_node` / the existing `TypeExpr`
   constructors use (grep how other Rust-side `TypeExpr` are built for built-in records).
5. **`src/capability/registry.rs:249-253`** (TEST helper `make_types_with_wire`) — identical fix to room 4.
6. **`src/check.rs`** (the 293.3-records Record arm — grep `rd.field_names`/`rd.field_types`) — it currently
   clones `field_names` + `field_types` and zips them; now it is just `rd.fields.clone()` directly. **Simplifies.**
7. **`grep -rn "\.field_names\|\.field_types\|field_names:\|field_types:" src/`** — the remaining read/match sites
   across `value/value.rs`, `runtime.rs`, `collection/eval.rs`, `rete/matcher.rs`, `rete/kernel.rs`,
   `closure_extract.rs`. Migrate each to `.fields` / `.field_names()` / `.field_types()`. (Exclude `StructDef`
   `.fields` and `struct_form` — those are NOT this change.)

## STOP triggers (halt + report; do NOT improvise)

1. **STOP if a `field_types: None` / `field_types = None` construction is NOT one of the 2 named test helpers** —
   i.e. a production path genuinely builds a record without types. Recon says only the 2 helpers; if you find a
   third, STOP and report it (the always-typed decision assumed there are no others).
2. **STOP if annihilating the string-literal `recordtype` branch breaks any test** that passes `["name" …]` to
   `recordtype` — recon says there are none; if one surfaces, report it (do not keep the dead branch silently).
3. **STOP if the cascade spreads into `StructDef`/`struct_form`/`TypeDef::Struct`/`TypeDef::Record` LOGIC** (this
   is field-shape only; the variant merge is 2b). Report if a site forces a `TypeDef` variant change.
4. **STOP if a read site needs `field_types` to be an `Option`** (some code path that meaningfully distinguishes
   "typed vs untyped record") — that would contradict D2; report it.
5. You are a LEAF. Do NOT spawn subagents. If bigger than these rooms, STOP and report.

## Gate (the orchestrator re-runs every line against the disk)

| what | command | expected |
|---|---|---|
| build compiles (cascade to zero) | `cargo build --release -p wat` | clean |
| records still satisfy surfaces (the 293.3 path survives the shape change) | `cargo test --release -p wat --test probe_arc293_record_surface` | 3 passed |
| struct surfaces + structtype intact | `cargo test --release -p wat --test probe_arc293_structural_surface --test probe_arc293_structtype_primitive` | 2 + 1 passed |
| record construction/accessors/EDN unchanged | `cargo test --release -p wat --test test -- core_record_def` | 7 passed |
| the test helpers compile + their suites pass | `cargo test --release -p wat --test test -- general_decode_refuses_capability_tags` + the capability/registry codec tests | green |
| SocketAddressWire path intact | `cargo test --release -p wat --test probe_arc272_6a_capability_handoff --test probe_arc272_6c2_record_ipc_derisk` | green |
| no new regressions | `cargo test -p wat --no-fail-fast`, failing-test **SET** vs HEAD (`bd0935db` code, floor ≈ 201) | **∅** new (weigh by SET; the `arc234_stone2a` nursery fails are PRE-EXISTING — proven at `919f825e`) |

## Report back
Full `git diff --stat`; verbatim gate output for each row; the failing-test SET (sorted) for the SET-diff; any
honest deltas vs this BRIEF + which STOP triggers fired. Do NOT commit — leave it uncommitted for the weigh.

Runtime: 45–90 min. Trap-doors: (a) the `TypeExpr` construction for the 2 test helpers (build `Vector<wat::core::i64>`
the way the existing Rust-side built-in records do — grep for a `Parametric`/`TypeExpr` built-in example); (b) read
sites that index `field_names` by position (give them `self.fields[i].0`); (c) the `edn_shim` Option-branch collapse.
The whole strike is behavior-preserving — the SET-diff ∅ + green deterministic tests are your truth oracle.
