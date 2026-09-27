# WEIGH — STONE 255.51: bounded type parameters on `fn` — ACCEPTED

**Executor: grok via pulsare, commits `58a0d9a4b` (the build, STOP-1) and `161f6993f` (the amend, F1).** Weighed by
the orchestrator on 2026-09-26.

## STOP-1, and its ruling

The one binder door refused `wat-scripts/fmt/fixtures/generic-fn.wat`: `(:wat::core::fn :- [:wat::core::i64] …)`
put a **type** where a type-parameter name goes. The old `peel_type_binder` dropped it silently, so the `fn` was
never generic. STOP-1 fired as drawn, and grok stopped. **The builder ruled F1:** the lambda is now a real generic
`fn` in the target spelling (`AMEND-STONE-255.51-the-fixture.md`). The builder also confirmed the design: the bound
lives on `fn`, and `defn` (which is `def` + `fn`, `wat/core.wat:1396-1400`) inherits it.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor | `.floor` at HEAD `161f6993f`: **6155 passed / 22 skipped** (6145 + the 10 new rows). It includes grok's post-floor clippy alias |
| the 10 new rows | `cargo test --release --test types probe_arc255_51`: 10 passed |
| extra probe, a bound on a vector's element: `(count-marks [(In …)])` / `[(Out …)]` | rc 0 / *":u::count-marks: type parameter T is bounded by :u::Mark; got :u::Out"* |
| extra probe, a parametric bound `[E :< (Spawned :- [S R])]`: a `Thread` / a `String` | rc 0 / *":u::owner-only: type parameter E is bounded by (:wat::spawn::Spawned :- [_ _]); got :wat::core::String"* |
| census / delta | grok's: `no STOP-8`, 215 non-zero of 2287; NEW 2 / RECOVERY 0, same two files. Not re-run |
| clippy | grok's second run rc 0, after `PeeledTypeBinder` alias; covered by my floor's build, not re-run as clippy |

## The diff, read

- **One binder door:** `parse_binder_entries` (`src/types.rs:5864`), used by `fn`/`defn`, surface methods and
  `extend-type`. A bare name or `[Name :< Type]`; anything else names the entry. `extend-type` refuses a bound, and
  the refusal says why.
- **The bound is stored** as `type_param_bounds`, parallel to `type_params`, on `Function` and `TypeScheme`. It is
  carried through both `instantiate`s (`Instance.bounds`). `TypeExpr` is untouched; STOP-2 did not fire.
- **Enforced:** `enforce_type_bounds` (`src/check.rs:17974`) runs after unification. A variable still open becomes a
  pending bound, and `flush_pending_bounds` (`:18008`) refuses it at the end of the definition (Refuse).
- **In the body:** `assignable` treats a bounded letter as its bound (`src/check.rs:~17308`).

## Carried

- **A misplaced doc comment:** "Instantiate a scheme's universally-quantified type parameters…" now sits above
  `enforce_type_bounds`, not `instantiate`. A comment that lies; fix at the next touch of that region.
- `SurfaceMember::Method` stores its bounds, but method dispatch does not consult them yet.
- Measured for later stones (SCORE § "Later stones"): 6 multi-clause forms besides `sort`/`sort-by` use differing
  letter sets (ruling L covers them); 0 `derive`s into a surface; `sort` has 113 calls in 100 files, 45 feeding
  `reverse` and 23 feeding a `Vector` parameter; **`reverse` refuses a `Stream`**, so the Seqable ruling needs
  `reverse` (or its callers) to follow.
- The return position does not resolve `wat.core/i64` as a type (it needs `wat.type/i64`): a clojurification item.
