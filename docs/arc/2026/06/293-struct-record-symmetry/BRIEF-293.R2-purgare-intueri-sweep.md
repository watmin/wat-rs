# BRIEF — 293.R2 purgare + intueri sweep: the residue the aggregate collapse left behind

**The work.** The R2 trilogy (one `Value::Aggregate`; `register_record_methods` deleted; `/new` dropped) was fast and
green, but a purgare + intueri cast (weighed against the disk by the orchestrator) found **one real regression**, dead
code, an invariant-dead branch, and ~30 stale dead-world names. Fix all of it. The headline is **B1 — a correctness
regression the gate could not see** (its only covering test is `#[ignore]`'d for an arc-170 reason).

## PHASE 1 — correctness (do first; gated)

### B1 (REGRESSION) — `struct->form` emits the dead `:T/new` ctor
`src/runtime.rs:9511` — `let constructor = format!(":{}/new", s.class);` → **change to** `format!(":{}", s.class)`.
After R2.3, `:T/new` is unregistered (verified: `(:my::Pair/new 7 9)` → "not a registered function"); so
`(:wat::eval-ast! (:wat::core::struct->form some-struct))` fails with `UnknownFunction :T/new`. Also fix the two
stale strings IN THE SAME FN: `:9474` doc ("`(:my::Foo/new a b)` evaluates to…") and `:9505` error message
("struct value (e.g. `:my::Foo/new`'s output)") → bare-ctor wording.
**ADD a clean regression probe** `tests/types/probe_arc293_struct_to_form_roundtrip.{rs,wat}` (un-ignored): a
`defstruct`, then `(:wat::eval-ast! (:wat::core::struct->form (:T a b)))` reconstructs the struct and a field reads
back the original value — **NON-CONCURRENT** (no `run-thread`; that is what got `wat-tests/core/struct-to-form.wat`
ignored). Model the roundtrip on `struct-to-form.wat:24` but drop the `run-thread` wrapper; handle `eval-ast!`'s
`Result` return (it returns `Result<Value, EvalError>` — unwrap it). Verify it is RED before your fix, GREEN after.

### B2 — invariant-dead branch in `PartialEq for Value`
`src/value/value.rs:635` — the `_ => false` arm (same holder, cross `HolonForm`) is unreachable: the constructors
(`struct_`/`record` → `Empty`; `holon_record` → `Hologram`) make same-holder-different-holon impossible. Replace
`_ => false` with `unreachable!("AggregateValue holder/holon invariant violated: same holder {:?}, mismatched HolonForm", a.holder)` — honest (surfaces a real invariant break) and matches the illegal-states doctrine.

## PHASE 2 — dead code (purgare A1–A5; grep-confirmed no consumers, delete)
- `src/runtime.rs:49` — `use std::fmt;` (0 `fmt::` uses) — delete.
- `src/runtime.rs:53` — drop `wat_value` from `use wat_macros::{restricted_to, wat_value}` (keep `restricted_to`).
- `src/runtime.rs:5620` — `fn value_matches_type_pattern` (dead; live replacement is `value_matches_type_by_name` @6681) — delete the fn.
- `src/runtime.rs:19345` — `fn wrap_stream_as_socket_peer` (dead, no callers) — delete.
- `crates/wat-reader/src/parser.rs:503` — `fn ast_variant_label` (dead, no callers) — delete.
(Re-confirm each `grep -rn <name> src/ crates/` shows definition-only before deleting.)

## PHASE 3 — stale dead-world names (intueri L1/L2 + purgare B4–B7)
The three-variant world is gone; its names linger in comments, **live identifiers**, and **user-facing error
strings**. Rename per this mapping, updating EVERY occurrence (grep each):
- `Value::Struct` (in docs/comments) → `Value::Aggregate(… holder == Holder::Struct …)`
- `Value::wat__Record` → `… holder == Holder::Record …` ; `Value::wat__holon__Record` → `… holder == Holder::HolonRecord …`
- `struct_form` → `fields` ; `holon_form` (field refs/comments) → `holon` ; `type_name` (StructValue refs) → `class`
- the `register_record_methods` mention in `collect_all_record_fields` doc (`runtime.rs:1485`) → `register_aggregate_methods`

**Highest-value (live identifiers / user-facing — fix these for sure):**
- `src/runtime.rs:13759-13768` — `eval_record_field_at` **panic/error strings** say `struct_form.len()` / "within bounds of struct_form" → users see a field that doesn't exist; change to `fields`.
- `src/runtime.rs:5534/5560` — `fn keyword_accessor_record(…, struct_form: …)` param + `struct_form[i]` indexing → `fields`.
- `src/runtime.rs:13621/13635/13737` — `eval_record_of` / `eval_record_field_at` locals named `struct_form` → `fields`.
- `src/runtime.rs:1114` — the NEW R2 ctor-fallback local `struct_form_elems` → `field_asts`.
- `src/value/value.rs:1007` — `AggregateValue::holon_record(…, holon_form: Arc<HolonAST>)` param → `hologram` (it's the inner content, not a `HolonForm`).

**Comment-only (sweep, lower stakes but Level-1 lies):** `value.rs:562-569/713-714/824-825/1177-1179/988`,
`runtime.rs:1105/1385/1398/4035/4043/5467/5524-5528/6900/6985/7166/8993-8995/9467/11038/12008/12790-12795/13642/13779`,
`check.rs:19548`, `collection/eval.rs:1217`, `collection/map_container.rs:79-81`, `edn_shim.rs:2402/2457`. (The intueri
cast's site list — re-verify each names a dead variant/field before editing; some impls right below the comment are
already correct, so it's the COMMENT that lies.)

## PHASE 4 — stale `#[ignore]` (purgare B3)
`tests/types/probe_arc293_ctor_parity.rs:19,33` — ignored "until `:T` is the ctor"; R2.3 met that. Un-ignore + verify
GREEN. If it duplicates `probe_arc293_r23_construction_parity`, note that and keep whichever is the better gate (don't
delete coverage without the other being green).

## STOP triggers
- STOP-1: if B1's bare-ctor fix breaks any OTHER `struct->form` consumer (something that *wanted* the `/new` string) — report it; recon says `struct->form`'s only job is the eval-ast! roundtrip.
- STOP-2: if a "dead" item (Phase 2) has a consumer you find on re-grep — do NOT delete; report.
- STOP-3: if a rename (Phase 3) touches a LIVE keyword/string the runtime depends on (not just a name) — STOP; the renames are Rust identifiers + comments + error *text*, never wat-level keywords.

## Gate (orchestrator re-runs forced-clean)
- `cargo build --release -p wat` clean (the dead-code deletions + renames cascade — fix to zero).
- B1 probe GREEN (un-ignored); the named dead fns/imports gone (`grep` shows nothing); `grep -rn 'struct_form\|holon_form\|wat__Record\|wat__holon__Record\|Value::Struct\b' src/ crates/` returns only legitimate hits (if any remain, justify).
- `cargo nextest run --release` → floor 0; report exact pass/fail/skip (expect ~4100/0 + the un-ignored probes). Oracle: `-E 'test(core_record_def) + test(defstruct) + test(holon) + binary(types) + binary(services)'`.

## You are a LEAF
Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. No subagents. No commit. Build
incrementally; read every diff; trust only forced-clean builds. STOP + report if a STOP fires.
