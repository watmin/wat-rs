# BRIEF — Arc 226 Stone 226.1 — Type predicates for classifier-wrapped entities

**Stone scope:** Mint `:wat::holon::is?` polymorphic predicate verb + 9 convenience predicates for the classifier-wrapped types (Map/Set/Vector/List/Tuple/Symbol/Keyword/Tag/Nil). Uses arc 228's `extract_classifier` helper. **Structural classifier-name match for v1; VSA similarity threshold-tunable enhancement deferred to 226.2+.**

**Type:** Sonnet Mode A.
**Time budget:** 90-180 min target; 240 min STOP.
**Depends on:** Stone 230.1 SHIPPED (commit `9f70959`); uniform classifier-encoding across all typed entities; `extract_classifier` helper available.
**Calibration:** Closest precedents — Stone 228.1 (~36 min for 5 new verbs + classifier-wrap + 4 probe updates) + Stone 230.1 (~30 min for variant retirement). This stone is smaller scope (no encoding cascade; just new verbs + tests). Pattern locked.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/wat-rs/`**
- Branch: `arc-170-gap-j-v5-deadlock-state` (already current)
- Linux only; no `--no-verify`
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch holon-rs (substrate is settled post-arc-230)
- DO NOT touch wat-edn
- **HARD CUT** discipline if any rename surfaces (no aliases)

## BASH DISCIPLINE

- ONE cargo command at a time, foreground
- NO piping through `| grep` / `| tail`
- NO concurrent background cargo runs
- 5 known signal-handler test hangs (task #413) — skip per Verification

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Available from arc 228 (Stone 228.1)

- `extract_classifier(&HolonAST) -> Option<String>` — already in `src/runtime.rs` — returns classifier name if outermost form is `Bind(Atom(String(name)), _)`
- `extract_classifier_inner_bundle(&HolonAST) -> Option<&Vec<HolonAST>>` — already in `src/runtime.rs`
- The 5 collection constructor verbs (Map/Set/Vector/List/Tuple) — already minted by arc 228

### Available from arc 230 (Stone 230.1)

- Symbol/Keyword/Tag/Nil now use classifier-wrap encoding (no variant); `extract_classifier` recognizes them
- The 4 constructor helpers (`HolonAST::symbol/keyword/tag/nil`) produce classifier-wrapped compositions

### Substrate state — all 9 user-surface typed entities have classifier names

```
"Symbol", "Keyword", "Tag"  (former variants; now classifier-wrapped)
"Map", "Set", "Vector", "List", "Tuple"  (collections; classifier-wrapped post-arc-228)
"Nil" — special: nil() = symbol("nil"), so classifier is "Symbol" + inner is "nil"
```

## Your scope (sonnet)

### Phase 1 — Mint polymorphic `:wat::holon::is?` predicate verb

In `src/runtime.rs`:
- New Rust fn `eval_holon_is_predicate` — accepts 2 args: a Value (the entity to check) and a Value (the class name as string OR keyword)
- Body: convert the Value to HolonAST (via existing path); call `extract_classifier`; compare result to expected class name; return `Value::bool(matches)`
- Dispatch table entry: `":wat::holon::is?" => eval_holon_is_predicate(args, list_span, env, sym)`

In `src/check.rs`:
- TypeScheme registration: `(is? Value String) -> :bool` — or accept keyword too (`(is? Value (String|keyword))`)
- `infer_list` special-case if needed

### Phase 2 — Mint 9 convenience predicate verbs

For each classifier name in `["Map", "Set", "Vector", "List", "Tuple", "Symbol", "Keyword", "Tag"]`:
- New Rust fn `eval_holon_is_<name>` — accepts 1 arg (the Value to check); calls `extract_classifier`; checks against the hard-coded class name; returns bool
- Verb registration: `:wat::holon::is-Map?`, `is-Set?`, `is-Vector?`, `is-List?`, `is-Tuple?`, `is-Symbol?`, `is-Keyword?`, `is-Tag?`
- TypeScheme: `(is-X? Value) -> :bool`

For `is-Nil?`:
- Special case — Nil is `(Symbol "nil")` per arc 230 nil doctrine
- Body: `extract_classifier == Some("Symbol") && inner Atom string == "nil"`
- Use existing `HolonAST::is_nil()` accessor if available (from arc 230 holon-rs work)

### Phase 3 — Tests

New test file `tests/probe_arc226_stone1_type_predicates.rs` (or similar — match existing arc 22N naming convention):

- **For each of the 9 predicates**: positive case (matching type → true) + negative case (different type → false)
- **For polymorphic `is?`**: takes a class name string + value; verify both true and false cases
- **Edge cases**: 
  - Bare primitive (e.g., `I64(42)`) → all `is-Map?` etc. should return false (no classifier)
  - Nested classifier (e.g., `(Bind (Atom "Map") (Bind ...))` ) — `is-Map?` returns true even when inner Bundle has further classifier-wrapped items
  - Non-Bind top-level (e.g., bare Bundle) → all predicates return false

### Phase 4 — Verification

Run each ONE AT A TIME, foreground, no pipes:

```
cargo build --release -p wat
cargo test --release --lib -p wat -- --skip reset_sighup --skip reset_sigusr1 --skip sigusr1_query --skip sigusr2_and_sighup --skip user_signal_predicates --skip reset_sigusr2
cargo test --release --test probe_arc226_stone1_type_predicates
cargo test --release --test probe_arc216_stone1_hashset_roundtrip
cargo test --release --test probe_arc216_stone2_vector_roundtrip
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip
cargo test --release --test probe_arc216_stone4_predicate_composition
cargo test --release --test probe_arc216_stone7_tuple_roundtrip
cargo test --release --test wat_arc221_keyword_nil_tag_atomization
cargo test --release --test wat_arc143_manipulation
cargo test --release --test mvp_end_to_end
cargo test --release -p wat-edn
cargo clippy --release --all-targets -p wat-edn -- -D warnings
```

All must complete cleanly.

**Holon-rs untouched** — `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` must be empty.

**Write `wat-rs/docs/arc/2026/05/226-type-predicates-vsa-similarity/SCORE-STONE-226.1.md`** mirroring SCORE-STONE-230.1.md shape.

## STOP triggers

- **STOP-1 (compile error UNEXPECTED):** STOP and report
- **STOP-2 (test failure beyond new probe):** STOP + diagnose
- **STOP-3 (240 min elapsed):** wall-clock STOP
- **STOP-4 (holon-rs touched):** STOP and report
- **STOP-5 (scope creep beyond 10 predicates):** Out-of-scope; surface as finding
- **STOP-6 (VSA similarity rabbit hole):** v1 uses STRUCTURAL classifier-name match; do NOT implement VSA similarity scoring in this stone (deferred to 226.2+)
- **STOP-7 (bash discipline):** cargo hang from accidental pipes

## Out-of-scope

- Variant-based predicates for substrate primitives (is-I64? / is-F64? / is-Bool? / is-Char? / is-String? / is-Atom? / is-Bind? / is-Bundle? / is-Permute? / is-Thermometer? / is-Blend? / is-SlotMarker?) — separate sub-stone 226.2; same mechanical pattern but different mechanism (variant match vs classifier extraction)
- VSA similarity scoring with threshold-tunable answers — separate sub-stone 226.3+
- Polymorphic dispatch integration with arc 146/147 multimethod machinery — separate arc 226 closure work
- User-defined type predicates `(is-MyType? x)` — arc 227's scope
- INSCRIPTION (Stone 226.4; blocked on arc 227 closing per spawn-block)
- holon-rs changes
- wat-edn changes
- Aliases for any pre-existing predicate name (HARD CUT)

## Naming convention notes

Predicate naming follows Lisp tradition: trailing `?` for boolean-returning verbs. Already used in substrate (e.g., `:wat::core::empty?`, `:wat::core::contains?`, `:wat::core::atomizable?` from arc 216 Stone 216.4).

Pascal-Case classifier names in verbs (`is-Map?` not `is-map?`) match the classifier atoms which carry Pascal-Case names ("Map" / "Set" / etc.).

## Doctrine context

Arc 226 is the type-system arc — type-checking emerges from substrate algebra. Per [[typed-entities-doctrine]]:

> Type system emerges from VSA similarity:
> ```
> (is-X? value) ≡ similarity(value's class atom vector, prototype-of-X vector)
> ```
> Continuous answer. Duck typing with measurable shape.

Stone 226.1 ships the v1 — EXACT STRUCTURAL MATCH on classifier name (which IS perfect VSA similarity for the prototype atom). Future stones 226.2+ add the continuous-measurement enhancement with threshold-tunable answers.

The substrate IS the type system. The duck has a measurable shape. Stone 226.1 is the first measurement.
