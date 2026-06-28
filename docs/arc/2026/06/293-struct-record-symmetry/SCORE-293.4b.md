# SCORE — 293.4b: the generated surface dispatcher

**Verdict: GREEN, weighed by the orchestrator's own re-run.** `cargo nextest run --release` = **4089 passed / 0 failed
/ 93 skipped**. `:Surface/method` now dispatches by runtime type to the satisfier's `defn :T/method`; the 293.4b probe
flipped RED→GREEN; the negative arm proves the dispatcher requires satisfaction; the acceptance demo stays `#[ignore]`'d.

## Scorecard (each row re-run by the orchestrator)
| # | what | result |
|---|---|---|
| 1 | 293.4b probe GREEN (un-ignored) | **PASS** — `surface_method_dispatches_by_runtime_type` |
| 2 | `:Surface/method` RESOLVES (no UnresolvedReference) | **PASS** — startup Ok |
| 3 | dispatch ROUTES by runtime type | **PASS** — Circle ≈ 12.566 (π·2²) / Square = 9.0 (3²), each its own impl |
| 4 | non-satisfier rejected at check time | **PASS** — `non_satisfier_receiver_rejected_at_check_time` (+ `_bad.wat`) |
| 5 | 293.4a un-regressed | **PASS** |
| 6 | acceptance demo stays RED | **PASS** — still `#[ignore]`'d |
| 7 | whole workspace green | **PASS** — 4089 / 0 / 93 (own forced run) |

## What shipped (3-layer mirror of the arc-232 protocol path)
- **`src/resolve/walk.rs`** (+18) — `is_resolvable_call_head` accepts a `:S/m` head when `sym.types` holds a
  `TypeDef::Surface` for the stem with method member `m` (sibling to the protocol head-accept).
- **`src/freeze/env.rs`** (+9) — step 6.97: pre-attach the populated TypeEnv to `sym` BEFORE the resolve pass (so the
  resolver can see surfaces). Documented; `freeze` re-attaches the same data later.
- **`src/check.rs`** (+92) — surface-method call-site check after the protocol arm: receiver must satisfy `S`; result =
  the method member's `ret`.
- **`src/runtime.rs`** (+86) — surface-method dispatch after the `is_protocol` block: eval receiver → concrete type
  FQDN (reuses the protocol path's extraction) → `sym.get(canonical_callable_name(":<T>/<m>")) → apply_function`. **The
  ONE semantic change: a plain `defn` lookup, NOT `extend:<S>:<T>`** (verified on disk).
- **Tests** — `probe_arc293_4b_surface_dispatch.rs` un-ignored + negative arm; `_bad.wat` new.

## Honest deltas (carried, not hidden)
1. **The defn-not-extend-def distinction held.** The executor did NOT copy the protocol `extend:<P>:<T>` lookup
   verbatim — confirmed `sym.get(canonical_callable_name(method_key))` against the disk. The load-bearing risk avoided.
2. **No STOP fired** — the surface registry was reachable in all three layers (the env.rs early-attach was the seam).
3. **Banked temperare:** the env.rs early-attach `types.clone()` + `freeze`'s later `set_types(Arc::new(types.clone()))`
   = two TypeEnv clones at startup. Could share one Arc. Minor, startup-only; banked, not blocking.
4. **Pre-existing warnings** (`head_span`/`list_span` unused, runtime.rs unused imports, `all_match` overwrite) are NOT
   from this slice — they predate it. Folds into the banked dead-helper/warning sweep.

## Next
**293.4c — `extend-type` as the foreign-accessor adapter** (the monkeypatch: teach a foreign built-in like the holon
`Vector` to satisfy a surface by adding `:T/accessor` impls; collisions = `DuplicateDefine`). Then **293.4d** — annihilate
`defprotocol` (ONE live use `:wat::spawn::Locus`) + un-ignore `probe_arc293_acceptance_demo` = the arc's GREEN gate.
