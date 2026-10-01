# WEIGH — STONE 255.74: a set element or map key must be data — ACCEPTED (with the D3 amendment)

**Executor: a Sonnet subagent, commits `3e214a67e` (the stone), `f39a41b47` (AMEND D3), `2ffcdfa3e` (two corrections).**
Weighed by the orchestrator on 2026-10-01.

## Re-run by the orchestrator at `2ffcdfa3e`

| row | result |
|---|---|
| release floor | `.floor/2026-10-01T05-33-41Z`: **6257 passed / 24 skipped**, exit 0 (6236 at `9a54f673c` + 21 tests) |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 |
| map keys that are data (`u8`, `bigint`, `rational`, `(Option :- [i64])`, `(PersistentVector :- [i64])`, `Instant`, `f64`) | `--check` rc 0 each |
| map keys that are not (`[i64 :-> i64]`, `(Vector :- [[i64 :-> i64]])`) | `--check` rc 1 each |
| the three `wat-scripts/probes/arc-255/*.wat.bad` (fn element, vector of fns, set of `Capability`) | `--check` rc 1 each |
| `probe-compound-upcast.wat` (the census's panic) | prints `"compound-upcast: ok"`, rc 0; now run by a driven test |
| new `ann-form` code | none (`git diff ec77a35e1..HEAD`: every added line naming it is a comment or an assertion message) |

## What landed

- **The checker refuses a non-data set element or map key** at nine sites: the `HashSet`/`HashMap`/`PersistentMap`
  constructors, the `#{}`/`{}` literals (inferred and checked-against), `conj`, `assoc`. One door,
  `key_eligible_or_error` (`src/check.rs`), asks **`require_class(T, Equatable)`** (D3). An inference variable defers
  through the pending-bound path and is refused only if it stays unresolved (the Refuse ruling). A declared parameter
  needs `[T :< Equatable]`; two stdlib functions gained it (`wat/seq.wat` `distinct`, `distinct-walk`).
- **The runtime guard** `value_is_hashable` descends into containers, tuples and aggregates and classifies leaves from
  `Value::key_eligibility()`, so it no longer keeps a second list (it had omitted `Stream`).
- **The two layers are tied:** `every_key_eligibility_row_agrees_with_equatable` checks every row of
  `all_key_eligibility()` against `Equatable`; mutation-proved by the agent (flip `u8`, red, restore). It found ten rows
  the Rust table marked never-a-key that `wat/class.wat` already declared Equatable; they are `Hashable` now.

## The brief's own defect (on the record)

The brief named `is_atomizable` as the door. That predicate means "encodable as a holon atom", and routing keys through it
refused data the runtime hashes (`u8`, `bigint`, `rational`, `Instant`, `Option`, `PersistentVector`). Measured on
`3e214a67e`, ruled D3 by the builder, amended. The agent's first D3 pass then pinned four empty test literals with
`ann-form` (ascription, ruled out); corrected to typed consumers in `2ffcdfa3e`.

## Consequence, by ruling

`(length {})` with nothing to fix the key type is now refused: the key's Equatable bound never resolves (the Refuse
ruling). A header or a typed constructor fixes it. Three test fixtures met it; no corpus file did (census: zero rc
flips).

## Open finding (not this stone's)

The rete engine's internal indexes (`alpha_tree.rs`, `where_tree.rs`, `compiled_cond.rs`, `fire/delta.rs`,
`kernel/session.rs`) hash `Value`s with neither guard. They rely on facts never carrying a resource; nothing checks it.

## Verdict

Accepted and pushed.
