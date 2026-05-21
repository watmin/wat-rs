# BRIEF — Arc 214 Slice 4 Stone 4.2 — `:wat::program::Env/get` single-step trio

**Stone:** mint three accessor verbs for `:wat::program::Env` — `/get`, `/expect-get`, `/get-default`. Single-step (one key lookup; one HolonAST→T extraction).
**Type:** Sonnet Mode A.
**Time budget:** 45-75 min target; 90 min STOP.
**Depends on:** Stone 4.1 (commit `f55a757`) — `:wat::program::Env` typealias minted.
**Unblocks:** Stone 4.3 (multi-step `/dig` trio composes on `/get`).

## Goal

Mint the single-step accessor trio. Each verb takes an `:wat::program::Env`, a key, and (where applicable) a default value; returns either `Option<T>` or `T` (panic / fallback variants).

Per DESIGN forward-correction Q6 — Option<T> uniformly for `/get`; expect-variant panics with KeyError flavor; default-variant returns supplied fallback.

## Pre-flight verified

- Stone 4.1 shipped: `:wat::program::Env` typealias registered
- `eval_hashmap_get` at `src/runtime.rs:7813` — pattern template for the lookup path
- `eval_atom_value` at `src/runtime.rs:12184` — HolonAST→T extraction mechanism; takes a typed return annotation
- arc 107 typed-expect at `src/runtime.rs` (`:wat::core::option::expect`) — pattern for the panic-variant
- Baseline tests green (probe_arc214_slice4_stone1 6/6 + probe_arc215_stone2 13/13 + probe_arc215_collection_literal_inference 12/12 + probe_brace_map_literal 9/9 + probe_hashmap_ctor_vector_symmetric 9/9)

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- `cargo test` is the verification path

## Your scope

1. **Mint three verbs** at the wat surface:
   - `(:wat::program::Env/get env key -> :wat::core::Option<T>)` — Option<T> on miss/wrong-type
   - `(:wat::program::Env/expect-get env key -> :T)` — T, panic with KeyError flavor on miss/wrong-type
   - `(:wat::program::Env/get-default env key default -> :T)` — T, return supplied default on miss/wrong-type

2. **Implementation pattern (each verb):**
   - Type-check: extract the expected T from the `-> :T` return annotation; verify env is `:wat::program::Env` (or its underlying `HashMap<keyword, HolonAST>`)
   - Runtime: look up key in HashMap; if Some(HolonAST), extract to T via the value_to_value-from-HolonAST mechanism (essentially what `eval_atom_value` does); if extraction succeeds return Some(value); else return None
   - `/expect-get`: same but panic-with-diagnostic on None (mirror `:wat::core::option::expect`'s pattern — arc 107)
   - `/get-default`: same but return the default arg on None (default arg's type must unify with T)

3. **Substrate placement:**
   - Register the three verbs in `src/runtime.rs` next to `eval_hashmap_get` (line 7813-ish) — adjacent to the related HashMap/get implementation
   - Type-check entries in `src/check.rs` — add a dispatch arm for each verb head; the inference returns `Option<T>` or `T` based on the return annotation
   - No polymorphic `:wat::core::get` dispatch entry in this stone — that's a future cleanup; explicit `:wat::program::Env/*` verbs only

4. **Probe matrix** — `tests/probe_arc214_slice4_stone2_env_get_trio.rs` with ~15 probes:
   - **`/get`:**
     1. Found + correct type: `(get env :foo -> :wat::core::String)` returns Some("bar") when env has :foo → "bar"
     2. Not found: returns None for missing key
     3. Wrong type: returns None when stored type doesn't match requested T
     4. Different T types: i64, String, bool, keyword — all extract correctly
   - **`/expect-get`:**
     5. Found + correct type: returns T directly
     6. Not found: panics with diagnostic naming the key + env context
     7. Wrong type: panics with diagnostic naming the type mismatch
   - **`/get-default`:**
     8. Found: returns the found value (default ignored)
     9. Not found: returns the default
     10. Wrong type: returns the default
     11. Default type unifies with T at check time
   - **Cross-verb:**
     12. All three on the same env produce consistent results
     13. Empty env works (get → None; expect-get → panic; get-default → default)
     14. HolonAST::Atom unwrap: stored HolonAST::Atom(primitive) extracts cleanly to T
     15. Nested holon (HolonAST::Bundle): stored as bundle; get/expect-get/get-default treat as wrong-type for primitive T (returns None / panics / default)

5. **WAT-CHEATSHEET update** — brief mention of the trio under the `:wat::program::Env` subsection

6. **SCORE doc** at `docs/arc/2026/05/214-concurrency-toolkit/slice-4-program-env/SCORE-STONE-4.2.md` — 18-row scorecard matching EXPECTATIONS

## NOT your scope

- `/dig`, `/expect-dig`, `/dig-default` — Stone 4.3 (multi-step trio composes on this stone)
- Polymorphic `:wat::core::get` dispatch entry for Env — future stone; explicit verbs only here
- spawn-program' — Stone 4.4
- Kernel verbs (send'/recv'/etc.) — Stone 4.5
- Integration tests — Stone 4.6
- INTERSTITIAL — orchestrator post-ship
- WARD-PASS — out-of-zone

## STOP triggers

- STOP-1: HolonAST→T extraction mechanism doesn't already exist in a reusable form — if `eval_atom_value` can't be cleanly reused or composed, the extraction needs new substrate. Flag the scope before building.
- STOP-2: Return-type annotation extraction from `-> :T` is more complex than arc 107's pattern suggests. Flag if the typed-expect machinery doesn't compose cleanly.
- STOP-3: any existing test fails — should be additive but verify
- STOP-4: 90 min elapsed

## Verification

Single commands per line (firewall-friendly):

```
cargo build --release
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
- Final PASS count out of 18
- Honest deltas
- Verification summary
- Elapsed time
- Anything discovered that wasn't in the BRIEF

Don't commit. Orchestrator commits after reviewing SCORE.
