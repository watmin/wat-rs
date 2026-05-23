# SCORE — Arc 228 Stone 228.1 — Substrate collection classifier-wrap + Pascal-Case constructor verbs

**Mode:** A
**Agent:** claude-sonnet-4-6
**Scoring:** claude-sonnet-4-6 (same session; independent re-verification via cargo output)
**Date:** 2026-05-22

## Result: 14/14 deliverables COMPLETE — all green within pre-existing baseline

| # | Deliverable | Status | Citation |
|---|---|---|---|
| 1 | NEW `:wat::holon::Map` constructor verb minted | PASS | `src/runtime.rs` — `eval_algebra_map`: takes `Vec<HolonAST>` via `Value::Vec`, produces `Bind(Atom("Map"), Bundle(items))`; registered in dispatch table; `src/check.rs` TypeScheme registered after `:wat::holon::Blend` |
| 2 | NEW `:wat::holon::Set` constructor verb minted | PASS | `src/runtime.rs` — `eval_algebra_set`: same shape as Map; `src/check.rs` TypeScheme registered |
| 3 | NEW `:wat::holon::Vector` constructor verb minted | PASS | `src/runtime.rs` — `eval_algebra_vector`: substrate auto-applies positional `Bind(I64(i), item)` keys; produces `Bind(Atom("Vector"), Bundle(positional Bind pairs))` |
| 4 | NEW `:wat::holon::List` constructor verb minted | PASS | `src/runtime.rs` — `eval_algebra_list`: takes bare items; produces `Bind(Atom("List"), Bundle(sequential items))`; same sequencing as Vector but sequential (no positional Bind keys) |
| 5 | NEW `:wat::holon::Tuple` constructor verb minted | PASS | `src/runtime.rs` — `eval_algebra_tuple`: identical internals to Vector (positional Bind keys); distinct outer classifier `Atom("Tuple")` |
| 6 | `to_holon_inner` HashSet arm — classifier-wrap | PASS | `src/runtime.rs` — arm now produces `Bind(Atom("Set"), Bundle(items))`; bare Bundle retired |
| 7 | `to_holon_inner` Vec arm — classifier-wrap | PASS | `src/runtime.rs` — arm now produces `Bind(Atom("Vector"), Bundle(positional Binds))`; bare Bundle retired |
| 8 | `to_holon_inner` Tuple arm — classifier-wrap + distinguished from Vec | PASS | `src/runtime.rs` — arm now produces `Bind(Atom("Tuple"), Bundle(positional Binds))`; NOW DISTINCT from Vec at substrate (was identical bare Bundle) |
| 9 | `to_holon_inner` HashMap arm — classifier-wrap | PASS | `src/runtime.rs` — arm now produces `Bind(Atom("Map"), Bundle(K-V Binds))`; bare Bundle retired |
| 10 | `to_holon_inner` List arm — classifier-wrap | PASS | `src/runtime.rs` — `Value::wat__core__List` arm minted (was missing); produces `Bind(Atom("List"), Bundle(sequential items))` |
| 11 | NEW helpers `extract_classifier` + `extract_classifier_inner_bundle` | PASS | `src/runtime.rs` — `extract_classifier(&HolonAST) -> Option<String>`: returns classifier name if outermost form is `Bind(Atom(String(name)), _)`; `extract_classifier_inner_bundle(&HolonAST) -> Option<&Vec<HolonAST>>`: extracts inner Bundle items from classifier-wrapped form |
| 12 | `eval_holon_from_holon` updated to dispatch by classifier | PASS | `src/runtime.rs` — `HolonAST::Bundle` arm and old `_ =>` arm replaced with `other =>` arm: calls `extract_classifier`; dispatches by name ("Map"→HashMap, "Set"→HashSet, "Vector"→Vec, "List"→List, "Tuple"→Tuple); unknown classifier or bare Bundle → `TypeMismatch` error with diagnostic. `_hint_is_hashmap` parsed but ignored (HARD CUT). `from_holon_item` also updated for nested collection dispatch |
| 13 | Arc 216 probe tests updated for new encoding | PASS | All four probe files updated: `probe_arc216_stone1_hashset_roundtrip` (10/10), `probe_arc216_stone2_vector_roundtrip` (12/12), `probe_arc216_stone3_hashmap_roundtrip` (14/14), `probe_arc216_stone7_tuple_roundtrip` (12/12) |
| 14 | All test suites green + holon-rs untouched | PASS | See test summary below |

## Test summary

```
cargo build --release -p wat                                        — 0 errors (5 pre-existing unused-fn warnings)
cargo test --release --lib -p wat [skip 5 signal tests]            — 822/822 PASS
cargo test --release --test probe_arc216_stone1_hashset_roundtrip  — 10/10 PASS
cargo test --release --test probe_arc216_stone2_vector_roundtrip   — 12/12 PASS
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip  — 14/14 PASS
cargo test --release --test probe_arc216_stone7_tuple_roundtrip    — 12/12 PASS
cargo test --release --test wat_arc221_keyword_nil_tag_atomization  — 6/6 PASS
cargo test --release --test wat_arc143_manipulation                 — 8/8 PASS
cargo test --release --test mvp_end_to_end                          — 10/10 PASS
cargo test --release -p wat-edn                                     — 23+1 PASS
cargo clippy --release --all-targets -p wat-edn -- -D warnings      — 0 warnings

holon-rs contamination check:
  git -C /home/watmin/work/holon/holon-rs/ diff --name-only         — empty (untouched)
```

## Deltas from EXPECTATIONS

### Delta 1 — `extract_classifier_inner_bundle` helper added (not in EXPECTATIONS; honest extension)

The BRIEF specifies `extract_classifier(&HolonAST) -> Option<String>` only. During Phase 3 implementation, a second helper `extract_classifier_inner_bundle(&HolonAST) -> Option<&Vec<HolonAST>>` was added to extract the inner Bundle items. This is an implementation-level detail (same logical operation decomposed into two clean helpers) and does not change the observable contract.

### Delta 2 — `from_holon_item` updated (not explicitly in EXPECTATIONS; required by cascade)

The BRIEF specifies updating `eval_holon_from_holon`. During cascade testing, nested collection round-trips (e.g., `Vec<HashSet<i64>>`, `Tuple(Vec<i64>, String)`) failed because the inner item decoder `from_holon_item` still used the old bare-Bundle heuristic. `from_holon_item` was updated to recognize classifier-wrapped forms and dispatch identically to `eval_holon_from_holon`. Required by the typed-entities doctrine: classifier-dispatch must be recursive at ALL levels.

### Delta 3 — Probe 7 `run_value` + explicit Tuple type annotation pattern (stone7 cascade)

`probe_arc216_stone7_tuple_roundtrip.rs` required a new `run_value` helper (returns `Value` directly, not extracted i64) for probes where the `from-holon` result is a Tuple. The type checker cannot statically infer that `from-holon` returns a Tuple (`from-holon` declares `-> ?T`), so `first`/`second` accessors fail the check phase with `expected: "tuple or Vec<T>", got: "?n"`.

The fix: probe functions declare the Tuple return type explicitly (e.g., `-> :(wat::core::i64,wat::core::String)`) so the checker unifies `?T` with the declared type, and the Tuple is returned to Rust for element extraction via `Value::Tuple`. This is the correct pattern for any probe that returns a Tuple from `from-holon` — documented here for future stones.

For nested Tuple type annotations (probe 4: inner Tuple type), the `:(...)` form cannot nest inner `:(...)` types (inner types starting with `:` are rejected by `parse_type_inner`). The workaround is `(wat::core::i64,wat::core::i64)` without leading `:` for the inner Tuple element — this parses correctly as a nested Tuple type.

### Delta 4 — `_hint_is_hashmap` variable renamed (HARD CUT honesty)

The `-> :HashMap` consumer hint syntax was previously the only way to disambiguate empty Bundles in `from-holon`. Arc 228 retires this via the classifier. The variable that parsed the hint was renamed from `hint_is_hashmap` to `_hint_is_hashmap` (leading underscore suppresses unused warning) and plays no role in the dispatch. The syntax is still accepted by the parser (no breaking change to call sites) but the hint is ignored — the classifier is the sole discriminator.

## STOP trigger audit

- **STOP-1 (unexpected substrate compile error):** DID NOT TRIGGER. All compilation errors were cascade consequences of classifier-wrap (tests calling `Bundle/children` on the new Bind top-level; tests checking for old Vec/HashMap asymmetry).
- **STOP-2 (test failure beyond cascade consequences):** DID NOT TRIGGER. All probe failures were direct consequences of the classifier-wrap encoding change (broken-by-this-stone, not pre-existing).
- **STOP-3 (300 min elapsed):** DID NOT TRIGGER.
- **STOP-4 (holon-rs touched):** DID NOT TRIGGER. `git -C holon-rs/ diff --name-only` empty.
- **STOP-5 (scope-extension beyond 5 collection types):** DID NOT TRIGGER. The 5 types (Map/Set/Vector/List/Tuple) are the full scope; no other collection types discovered.
- **STOP-6 (round-trip semantics break):** DID NOT TRIGGER. All round-trips (HashSet, Vec, HashMap, Tuple, nested compositions) verified via probe suite.
- **STOP-7 (bash discipline):** DID NOT TRIGGER. One cargo command at a time, foreground, no pipes.

## Arc 216 forward-correction (broken-by-this-stone — honest framing)

All arc 216 probe failures were broken-by-this-stone, not pre-existing:
- `Bundle/children h` on the new `Bind(Atom(...), Bundle(...))` top-level returns `TypeMismatch` (expects Bundle, gets Bind)
- `atom-value` (arc 216 era name) replaced by `from-holon` (arc 225); probes already used `from-holon`
- The arc 216 "honest asymmetry" for Tuple (Tuple and Vec had identical encoding; consumer-declared type was the discriminator) is **retired by arc 228**: the classifier atom is now the sole discriminator; `from-holon` on Tuple-classified form returns `Tuple`, not `Vec`

These are doctrine supersessions, not bugs. Arc 216 INSCRIPTION (when it ships) will record the original bare-Bundle encoding as historical. Arc 228 SCORE is the forward-correction record.

## Files changed

**wat-rs source (Rust):**
- `src/runtime.rs` — (a) 5 new constructor fns (`eval_algebra_map`, `eval_algebra_set`, `eval_algebra_vector`, `eval_algebra_list`, `eval_algebra_tuple`) + dispatch table entries; (b) 5 `to_holon_inner` collection arms updated with classifier-wrap; `Value::wat__core__List` arm minted; (c) `extract_classifier` + `extract_classifier_inner_bundle` helpers added; (d) `eval_holon_from_holon` Bundle arm + `_ =>` arm replaced with classifier-dispatch `other =>` arm; (e) `from_holon_item` updated with classifier-dispatch; (f) `_hint_is_hashmap` renamed
- `src/check.rs` — 5 TypeScheme registrations for new constructor verbs

**Test files (Rust, cascade sweep — 4 files):**
- `tests/probe_arc216_stone1_hashset_roundtrip.rs` — probes 1 and 7 second sub-check updated: `Bundle/children h` → round-trip `from-holon` + `HashSet/length`
- `tests/probe_arc216_stone2_vector_roundtrip.rs` — probes 1, 3, 7, 8 second sub-checks updated: `Bundle/children h` → round-trip; probe 3 updated from "empty Bundle ambiguous" to "classifier Vector unambiguous"
- `tests/probe_arc216_stone3_hashmap_roundtrip.rs` — probes 1, 3, 7, 8, 9 second sub-checks + probe 14 updated: `Bundle/children h` → round-trip; probe 14 "unannotated empty Bundle → HashSet" → "arc 228: empty Map → HashMap"
- `tests/probe_arc216_stone7_tuple_roundtrip.rs` — all 6 WAT-surface probes (1–6) updated: `Bundle/children h` → `run_value` + explicit Tuple return type + Rust-level element extraction; `run_value` + tuple extraction helpers added; probe 2 "honest asymmetry retired" note added

**Total: 2 modified Rust source files + 4 modified test files + 1 new SCORE doc.**

## Substrate state post-Stone-228.1

**Typed-entities doctrine now enforced for all 5 collection types:**
- `(:wat::holon::to-holon v)` on any collection → `Bind(Atom("ClassName"), Bundle(items))` with classifier
- `(:wat::holon::from-holon h)` dispatches by classifier name → correct collection type
- `(:wat::holon::Map ...)` / `Set` / `Vector` / `List` / `Tuple` — explicit Pascal-Case constructors at algebra tier
- Bare Bundle (no classifier) → `TypeMismatch` error with diagnostic on `from-holon`

**Vec/Tuple distinction now substrate-honest:**
- Pre-arc-228: identical bare-Bundle encoding; consumer-declared type was the only discriminator
- Post-arc-228: distinct classifier atoms ("Vector" vs "Tuple"); substrate is the sole discriminator

**`HolonRepresentable` (Rust-level API) NOT changed:** `src/comms/mod.rs` still produces bare Bundles for Rust-level `to_holon_ast`/`from_holon_ast`. This is arc 230 scope.

## Unblocks

- Arc 226 (type predicates via VSA similarity) — can now use `extract_classifier` to recover type from substrate data
- Arc 227 (user-defined types classifier-wrap) — pattern now established; user types use `Bind(Atom("UserTypeName"), Bundle(data))` mirroring collection types
- Arc 228 Stone 228.4 (INSCRIPTION) — blocked on arc 230 closing; this stone is now closed
- Arc 230 (HolonRepresentable / `src/comms/mod.rs` classifier-wrap) — now has the substrate encoding to match against
