# BRIEF — Arc 214 Slice 4 Stone 4.3 — `:wat::program::Env/dig` multi-step trio

**Stone:** mint three multi-step accessor verbs for `:wat::program::Env` — `/dig`, `/expect-dig`, `/dig-default`. Walks a path of keyword keys through nested HashMap structures.
**Type:** Sonnet Mode A.
**Time budget:** 60-75 min target; 90 min STOP.
**Depends on:** Stone 4.2 (commit `4979aa0`) — `/get` trio + atom-value extraction pattern.
**Unblocks:** Stone 4.4 (unified spawn).

## Goal

Mint the multi-step accessor trio. Each verb takes an `:wat::program::Env`, a `path: Vector<HolonAST>`, and (for `/dig-default`) a default value; returns `Option<T>` or `T`.

The walk: start with env; for each step in path, look up the step as a keyword key in the current HashMap; unwrap the value via atom-value; if the unwrapped value is itself a HashMap, recurse; if it's a leaf (or any non-HashMap), terminate. Final step extracts to T.

Per DESIGN forward-correction Q5-Q7: path is `Vector<HolonAST>` (homogeneous per arc 215 discipline); each step's lookup uses arc 057 slice 3's `hashmap_key accepts HolonAST` machinery; expect-variant panics with KeyError flavor; default-variant returns supplied fallback.

## Pre-flight verified

- Stone 4.2 shipped (`4979aa0`): `/get` trio + `holon_ast_extract` + `program_env_lookup` helpers at `src/runtime.rs:7831-8073`; corresponding check.rs infer functions
- `eval_atom_value` at `src/runtime.rs:12184` — HolonAST→Value unwrap
- `value_to_atom` at `src/runtime.rs:12249` — Value→HolonAST wrap (for understanding atomizable set)
- Baseline tests green (probe_arc214_slice4_stone2 15/15 + Stone 1 6/6 + arc 215 stones + P1/P2)

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

1. **Mint three verbs:**
   - `(:wat::program::Env/dig env path -> :wat::core::Option<T>)` — Option<T> on miss/wrong-type/non-traversable
   - `(:wat::program::Env/expect-dig env path -> :T)` — T, panic with KeyError flavor on miss/wrong-type
   - `(:wat::program::Env/dig-default env path default -> :T)` — T, return default on miss/wrong-type

2. **The walk semantics:**
   - Path is a `Vector<HolonAST>` — each element is a single navigation step (a keyword key for HashMap lookup)
   - Empty path: undefined for Stone 4.3 — probably reject at check with "path must be non-empty" diagnostic; flag in SCORE if you take a different path
   - Walk loop:
     - current = env (initial HashMap<keyword, HolonAST>)
     - for each step in path:
       - step must be a keyword (validate; non-keyword → return None or fail-at-check)
       - lookup step in current HashMap → Option<HolonAST>
       - if None → return None (or panic/default per verb variant)
       - if Some(holon) → unwrap via the atom-value mechanism Stone 4.2 already uses
       - if unwrapped value is HashMap AND there are more steps → set as current; continue
       - if unwrapped value is HashMap AND this was last step → result type must be HashMap (likely user-error; T-mismatch returns None)
       - if unwrapped value is anything else AND there are more steps → return None (terminated early)
       - if unwrapped value is leaf AND this was last step → extract to T per Stone 4.2's pattern; return Some(T)

3. **Substrate placement:**
   - Register the three verbs in `src/runtime.rs` near `eval_program_env_get` (Stone 4.2's eval functions)
   - Type-check entries in `src/check.rs` near `infer_program_env_get` (Stone 4.2's infer functions)
   - Compose on Stone 4.2's `holon_ast_extract` + `program_env_lookup` helpers — the walk is a loop of single-step lookups + extract

4. **Probe matrix** — `tests/probe_arc214_slice4_stone3_env_dig_trio.rs` with ~18 probes:
   - **`/dig` single-step:**
     1. Single-key path equivalent to /get: `(dig env [:foo] -> :T)` returns same as `(get env :foo -> :T)`
     2. Single-key path with miss: returns None
   - **`/dig` multi-step:**
     3. Two-step path through nested HashMap: env = `{:outer (Atom {:inner (Atom 42)})}`; `(dig env [:outer :inner] -> :i64)` returns Some(42)
     4. Three-step path: deeper nesting works
     5. Path missing intermediate key: returns None
     6. Path missing final key: returns None
     7. Path traversing into non-HashMap intermediate: returns None (terminated early)
   - **`/dig` type extraction:**
     8. Found + correct T: Some(value)
     9. Found + wrong T: None
     10. Multiple T types: i64, String, bool, keyword
   - **`/expect-dig`:**
     11. Found + correct: returns T
     12. Not found: panics with diagnostic naming the path + offending step
     13. Wrong type at terminal: panics
   - **`/dig-default`:**
     14. Found: returns found value (default ignored)
     15. Not found: returns default
     16. Wrong type / non-traversable: returns default
   - **Edge cases:**
     17. Empty path: behavior per design choice; document in probe
     18. Non-keyword path step (e.g., int): rejected at check or handled at runtime — sonnet picks; document

5. **WAT-CHEATSHEET update** — extend the `:wat::program::Env` section with /dig trio

6. **SCORE doc** at `docs/arc/2026/05/214-concurrency-toolkit/slice-4-program-env/SCORE-STONE-4.3.md` — 21-row scorecard

## NOT your scope

- Path with non-keyword steps (e.g., integer index for vector navigation) — future stone
- Path traversing into HolonAST::Bundle (holon-explicit values) — future stone (probably bundled with explicit holon support)
- Polymorphic `:wat::core::dig` dispatch entry — future stone
- spawn-program' — Stone 4.4
- Kernel verbs — Stone 4.5
- Integration tests — Stone 4.6
- INTERSTITIAL, WARD-PASS — orchestrator post-ship

## STOP triggers

- STOP-1: nested HashMap traversal requires the value at intermediate step to be `(:wat::holon::Atom hashmap-value)` — but the atom-value extraction may not return a HashMap directly (since the atomizable set is `{primitives, HolonAST, WatAST}` per arc 215 dig). If the substrate can't traverse Atom-wrapped HashMap cleanly, the walk semantics need substrate-side adjustment. Flag this; consider scope reduction (e.g., support single-step only for now and defer multi-step).
- STOP-2: arc 215 walked away from Atom-wrapping-HashMap because of atomizable-set issues — verify the unwrap path for nested HashMap is honest before relying on it
- STOP-3: any existing test fails
- STOP-4: 90 min elapsed

## Verification

Single commands per line:

```
cargo build --release
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio -p wat
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio -p wat
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat
cargo test --release --test probe_arc215_stone2 -p wat
cargo test --release --test probe_arc215_collection_literal_inference -p wat
cargo test --release --test probe_brace_map_literal -p wat
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat
cargo clippy --release -- -D warnings
```

## When you finish

Report:
- Final PASS count out of 21
- Honest deltas (especially Stone 4.3-specific: how does multi-step navigation actually work given the atomizable-set considerations from arc 215? if you had to make design calls, document them)
- Verification summary
- Elapsed time
- Anything discovered

Don't commit. Orchestrator commits after reviewing SCORE.
