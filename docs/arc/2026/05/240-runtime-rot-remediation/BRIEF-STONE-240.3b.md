# BRIEF — Stone 240.3b — consumer `.wat` drift sweep (telemetry + telemetry-sqlite)

Mechanical consumer-`.wat` drift repair across the wat-telemetry + wat-telemetry-sqlite
crates. The recipe is PROVEN on the exemplar `crates/wat-telemetry/.../WorkUnitLog.wat`
(both prod `wat/telemetry/WorkUnitLog.wat` and test `wat-tests/telemetry/WorkUnitLog.wat`,
committed `7d5fbcbd`) — read that commit's diff first; you are mirroring it across the
rest of these two crates. **NO `src/*.rs`. NO holon-rs. NO lru / holon-lru (deferred).**

## The proven recipe (apply per-site by what the substrate error names in `got:` / `unknown function:`)

1. `(:wat::holon::Atom <value>)` where `<value>` is a keyword / String / i64 / runtime value
   → `(:wat::holon::to-holon <value>)`   (arc 225; Atom narrowed to HolonAST→HolonAST)
2. `(:wat::holon::Atom <watast>)` where the arg is `:wat::WatAST` (a `(:wat::core::quote …)`
   form, or a param declared `:wat::WatAST`)
   → `(:wat::holon::from-wat <watast>)`   (arc 225 rename of from-watast; WatAST→HolonAST)
3. `(:wat::core::atom-value <h>)`  → `(:wat::holon::from-holon <h>)`  (arc 225; HolonAST→value)
4. `(:wat::core::HashMap :Tag)` (1 type-arg; a `(K,V)`-tuple alias like `:wat::telemetry::Tag`
   = `(HolonAST,HolonAST)`) → `(:wat::core::HashMap :K :V)` i.e. expand to the two element
   types (for `Tag`: `:wat::holon::HolonAST :wat::holon::HolonAST`).  (arc 215 2-arg constructor)

Also update any **stale comment** that names a retired verb (e.g. a comment saying "Atom is
polymorphic" or "Atom's WatAST arm") to the new verb — no retired-verb leftovers (FM-14).

## The files (sweep these; the test loop will name exact sites)

- `crates/wat-telemetry/wat/telemetry/WorkUnit.wat` (prod)
- `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` (test — ~20 Atom sites + HashMap)
- `crates/wat-telemetry-sqlite/wat-tests/telemetry/hashmap-field.wat`
- `crates/wat-telemetry-sqlite/wat-tests/telemetry/edn-newtypes.wat`
- `crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat`
- any other `.wat` under these two crates the loop surfaces.

## The loop (substrate-as-teacher)

1. `cargo test --release -p wat-telemetry 2>&1` — read the check / runtime errors. Each names
   a file:line + the drift. Apply the matching recipe element.
2. Iterate until `cargo test --release -p wat-telemetry` → **0 failed**.
3. Same for `cargo test --release -p wat-telemetry-sqlite` → **0 failed**.
4. `cargo build --release --tests --workspace 2>&1 | grep -c "^error"` → **0** (nothing else broke).

These crates' deftests are thread-based (stub Service threads) — they do NOT leak processes.
Do **NOT** run the full `cargo test --workspace` (it leaks ambient-stdio/fork processes — out
of scope, arc 170). Use the targeted `-p <crate>` runs above.

## STOP triggers (REJECTION — surface, do not work around)

- Any error that is **not** one of the 4 recipe drifts above (a logic/assertion mismatch, a
  deadlock, a panic unrelated to verb-rename/arity, a process leak). STOP and surface it
  verbatim — it is NOT this stone's mechanical drift; it may be an in-flight-arc concern.
- Any urge to edit `src/*.rs`, holon-rs, or the lru/holon-lru wat-tests. STOP — out of scope.

## Definition of done

- `cargo test --release -p wat-telemetry` → 0 failed; `-p wat-telemetry-sqlite` → 0 failed.
- `cargo build --release --tests --workspace` → 0 errors.
- Only `.wat` files under `crates/wat-telemetry/` + `crates/wat-telemetry-sqlite/` touched.
- Write `SCORE-STONE-240.3b.md` (sibling) with each crate's test-result line + the per-file
  site counts + any STOP. Do NOT commit (orchestrator scores + commits).
