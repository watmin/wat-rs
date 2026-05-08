# Arc 160 — INSCRIPTION

**Inscribed 2026-05-07 by orchestrator.** All slices shipped.

## What shipped

`infer_list` in `src/check.rs` had two structurally-separate regions:

- **Region A** (keyword-headed): `if let WatAST::Keyword(k, _) = head { match k.as_str() { ... } }`
- **Region B** (bare-Symbol head): `head_is_*_bare` and (formerly) `head_is_*_fqdn` checks

Pre-fix, the FQDN keyword paths for variant constructors
(`(:wat::core::Ok value)`, `(:wat::core::Err value)`,
`(:wat::core::Some value)`) hit Region A's `match k.as_str()` block,
which had no arm for them → fell through to `_ => {}`. The matching
`head_is_*_fqdn` checks lived in Region B but were dead code: Region
A's `if let WatAST::Keyword` already intercepts every Keyword head
before Region B is reached.

Result: FQDN keyword constructor calls returned `None` from
`infer_list`. The `let` binding's RHS inference produced no type;
match arm pattern checks resolved against `?fresh` instead of the
parametric type the constructor should have produced.

Arc 160 fixes this by hoisting the constructor inference logic
into three helper functions (`infer_ok_constructor`,
`infer_err_constructor`, `infer_some_constructor`), called from
BOTH:

- Region A's match arms for `:wat::core::Ok` / `:wat::core::Err` /
  `:wat::core::Some` (FQDN keyword path — the fix)
- Region B's `head_is_*_bare` checks (bare-Symbol path — backward
  compat / Pattern 2 poison emission, unchanged surface behavior)

Region B's now-dead `head_is_*_fqdn` blocks were retired.

## Slices

| Slice | Commit | What landed |
|---|---|---|
| 1 | `d4b5131` | BRIEF: diagnostic — find the lossy step |
| 1 (sonnet ran) | (no edits — diagnostic only) | Identified the dead-code path; proposed the hoist-into-Region-A fix |
| 2 | `8b6aebe` | BRIEF: the fix |
| 2 (sonnet ran) | `7ae2093` | Three helper functions added; Region A match arms call helpers; Region B `head_is_*_fqdn` blocks retired |
| 3 | (this commit) | Closure paperwork |

## Substrate impact

| Test outcome | Pre-arc-160 (post-arc-159 sweep) | Post-arc-160 |
|---|---|---|
| Workspace failures | 9 | 1 |

The 9 originally-failing tests cleared by arc 160 all shared the
same root cause: variant-constructor-typed `let` bindings produced
`?fresh` payload positions where they should have produced
concrete types from the constructor's argument.

The 1 remaining failure is `deftest_wat_telemetry_test_svc_tel_null_translator` —
a SEPARATE substrate gap (Symbol-headed function-value application
inference). Tracked in arc 161 (DESIGN at
`docs/arc/2026/05/161-symbol-headed-application-inference/DESIGN.md`).

## Settled design

### Helper extraction shape

Each helper takes the same parameter set as `infer_list`'s callers
plus an `is_bare: bool` flag. When `is_bare` is true, the helper
pushes a Pattern-2 poison TypeMismatch error before producing the
inferred type; this preserves arc 109 slice 1h's bare-Symbol
poison discipline. When false, no poison is emitted (the FQDN form
is the canonical surface).

### Why hoist over inline-in-Region-A?

Hoisting into shared helpers preserves the bare-Symbol path's
exact surface behavior (Region B's poison emission discipline)
while routing FQDN keyword paths to the same logic. The
alternative — inlining the constructor inference into Region A —
would have required duplicating ~30 LOC per constructor for the
poison-vs-clean variation. Helpers won on simplicity:
single source of truth for each constructor's inference, two
call sites for the two grammar surfaces.

### Why this gap existed

Pre-arc-109 (FQDN-everything campaign), variant constructors were
bare Symbols only. Region B's check-by-name was the only path.
Post-arc-109 slice 1h/1i, the FQDN keyword forms became canonical;
Region B grew `head_is_*_fqdn` checks to handle them — but those
checks ran AFTER Region A's keyword-pattern match. Arc 109 didn't
notice because most arc 109 consumer code carried explicit
let-binding annotations (`((resp :wat::core::Result<i64,String>) (rhs))`)
which masked the inference gap. Arc 159 (drop the annotation)
surfaced it.

## Honest deltas surfaced by sonnet

1. **`Some` constructor is parametric over `Option<T>`, not
   `Result<T, E>`.** Sonnet's slice 2 implementation correctly
   produced `TypeExpr::Parametric { head: "Option", args: vec![inner_ty] }`
   for `Some` — not `Result<inner_ty, fresh>`. The DESIGN's slice
   2 BRIEF stub mentioned "Some has no payload-arity-1 invariant;
   check sonnet's slice 1 diagnostic + the existing Region B code
   for Some's exact shape" — sonnet read Region B and matched.

2. **Region B's bare-Symbol path retains poison emission.** The
   `head_is_*_bare` checks still fire `TypeMismatch` with the
   "(retired bare-symbol exception)" param label per arc 109's
   poison discipline. Helper extraction preserved this exactly.

3. **No additional shared patterns surfaced for unification.**
   The three constructors (Ok / Err / Some) share enough shape
   that a single generic helper was tempting; sonnet kept them
   separate. Justified: each constructor's payload arity differs
   (Ok/Err: 1; Some: 1; Err's parametric structure differs from
   Ok's). The duplication is shallow and honest.

## Tests

No new tests added by arc 160. The 9 currently-failing workspace
tests verified the fix end-to-end:

- `wat_recursive_patterns::literal_fallback_to_general_arm`
- `wat_recursive_patterns::nested_match_in_some_arm`
- `wat_recursive_patterns::int_range_pattern`
- (six others in the same family)

Workspace post-fix: 2027 passed / 1 failed (the unrelated arc 161
gap).

## Out of scope (other arcs close)

- **Arc 161 — Symbol-headed function-value application inference.**
  The 1 remaining workspace failure is a SEPARATE substrate gap.
  Arc 159 surfaced both gaps simultaneously; arc 160 closes the
  variant-constructor side; arc 161 (DESIGN at
  `docs/arc/2026/05/161-symbol-headed-application-inference/DESIGN.md`)
  closes the application side.
- **Arc 159 closure** — waits on arc 161 closing first
  (workspace = 0-failed prerequisite). The closure chain is:
  arc 160 → arc 161 → arc 159.

## Cross-references

- **Arc 159** — discovery vehicle; this arc's gap was masked by
  arc 159's pre-state (legacy let-annotations forced the type
  directly into scope, bypassing constructor inference)
- **Arc 109 slice 1h/1i** — minted FQDN variant constructor
  forms; the dead-code path traces to this arc's structure
- **Arc 158a** — sibling pattern (substrate adapts to arc 159's
  shape change)
- **Arc 161** — sibling substrate fix; same arc 159 discovery
  pattern, different gap
- **Memory `feedback_v1_backout_dependency_arc.md`** — the
  cascading-arcs naming pattern (arc 159 v3 ships when both
  dependency arcs land)

## Commit chain

- `80ebca6` arc 160 opens (DESIGN)
- `d4b5131` arc 160 slice 1 BRIEF (diagnostic)
- `8b6aebe` arc 160 slice 2 BRIEF (the fix)
- `ec5b36e` arc 159 sweep: wat_run_sandboxed.rs embedded wat
- `7ae2093` arc 160 slice 2: hoist variant-constructor inference into Region A
- (this commit) arc 160 slice 3: closure paperwork
