# Arc 159 — INSCRIPTION

**Inscribed 2026-05-07 by orchestrator.** All slices shipped.

## What shipped

`:wat::core::let` bindings drop the per-binding type annotation `:T`.
Each binding's type is inferred from its expression — same lesson as
arc 145 / arc 157 (`def`), applied to the inner-binding slot.

| Before (legacy) | After (canonical) |
|---|---|
| `(:wat::core::let (((name :T) expr) ...) body)` | `(:wat::core::let ((name expr) ...) body)` |

User end-state: `(:wat::core::let ((x 2)) (:wat::core::+ x 2))`
type-checks and evaluates to `4` (`:wat::core::i64`) — no annotation
required.

The legacy `((name :T) expr)` shape ran through the migration window
behind a `LegacyTypedLetBinding` walker error; once the wat-rs
consumer sweep cleared all sites, the walker body retired (variant +
Display preserved as orphaned scaffolding per arc 113 precedent).

## Slices

| Slice | Commit | What landed |
|---|---|---|
| 1 (sonnet ran) | `25c8067` | Substrate accepts new shape; walker fires `LegacyTypedLetBinding` on legacy; runtime `parse_let_binding` extension; ~13 tests in `tests/wat_arc159_let_bindings.rs` + `src/runtime.rs::tests` + `src/check.rs::tests` |
| 1 (orchestrator hardening) | `c6b8d74` | Manual fixups for 11 lib unit tests (cleanup post-substrate-change) |
| 2 (orchestrator ran) | `c6b8d74` (atomic with slice 1) + `ec5b36e` | Python-driven mechanical sweep across ~951 wat-rs sites; one tests file hand-fix |
| 2.1 — substrate gap arc 160 | `7ae2093` | Variant constructor inference dead code; cleared 9 of 10 originally-failing tests |
| 2.2 — substrate gap arc 161 | `f6ab13f` | Symbol-headed application inference; cleared the final 1 test |
| 3 | `23808fb` (walker retirement); (this commit, paperwork) | `validate_legacy_typed_let_binding` walker body retired; INSCRIPTION + 058 row + USER-GUIDE + WAT-CHEATSHEET |

## Substrate impact

| Test outcome | Pre-arc-159 | After slice 1 + sweep | After arcs 160/161 | After walker retirement |
|---|---|---|---|---|
| Workspace failures | 0 | 9-10 (substrate inference gaps surfaced by clean break) | 0 | 0 |

Failure count went UP after arc 159's substrate change because the
clean break exposed two latent inference gaps the legacy
let-annotation had been masking. Both gaps closed in dependency
arcs (arc 160, arc 161) per the user's mass-refactor doctrine: *"this
work is meant to catch exactly these kinds of problems."*

## Settled design

### Why clean break (Path A) over transitional alias (Path B)

Per arc 154/155 precedent — when a clean canonical shape is
known and the consumer surface is bounded (~951 wat-rs sites),
clean break is shorter wall-clock + crisper diagnostic stream
than a months-long deprecation period. The `LegacyTypedLetBinding`
walker fires per legacy binding site as a structural migration
diagnostic; consumer sweep treats the diagnostic stream as the
work list.

### Why arc 158a + arc 159 split

Arc 158 v1 attempted the binding-shape change directly; reverted
post-verification when 24 lib unit tests broke. Root cause: the
scope-deadlock walker family (arcs 117/126/128/131/133/134) read
declared `:T` from AST for pair-anchor identity tracking. v1
stripped declared `:T` at inference (per arc 145 lesson) but
didn't migrate the walker — so walker lost its type-info source.

Arc 158a closed that dependency: walkers migrated to RHS
pattern-match (closed-set channel-related shapes); accept both
binding shapes; legacy paths unchanged. Arc 159 then ships the
binding-shape change atop the migrated walker.

This split is the canonical proactive-stepping-stones example
(memory `feedback_stepping_stones_proactive.md` + recovery doc
§ 5): arc 158 v1's BRIEF should have included a walker audit
as a pre-flight item; v2 (arc 158a + 159) ships the dependency
first via the explicit stepping stone v1 skipped.

### Why arc 159 surfaced two more substrate gaps

The legacy let-annotation `((name :T) expr)` typed `name` directly
in scope without going through expression inference. Two paths
the substrate "supported" but never exercised end-to-end:

1. **Variant constructor inference (arc 160).** FQDN keyword paths
   for `:wat::core::Ok` / `:wat::core::Err` / `:wat::core::Some`
   fell through `infer_list`'s Region A (no match arm) into
   Region B's `head_is_*_fqdn` checks — but Region A's keyword
   pattern always intercepted first. Region B's checks were dead
   code. The legacy let-annotation made constructor inference
   irrelevant: the binder's declared `:T` typed `resp` directly;
   match arm pattern inference unified against the declared type.

2. **Symbol-headed application inference (arc 161).** `(t arg)`
   where `t` is a Symbol bound to a Fn value fell into
   `infer_list`'s no-op fall-through (lines 4606-4613) and
   returned `None`. The legacy let-annotation typed `t` directly;
   downstream uses got the type from the binder, not from the
   application.

The mass-refactor discipline turned both up. Per user direction:
*"this mass refactor work is meant to catch exactly these kinds
of problems."* Each gap shipped as its own arc gating arc 159
closure.

### Honest deltas

1. **Lab consumer sweep (originally slice 3) removed from scope.**
   Per memory `project_lab_reconstruction.md`: lab is being
   archived as reference; reconstruction tests fresh-user-follow-along;
   wat-rs is the durable substrate; substrate work doesn't wait
   for lab. Arc 159 closes on wat-rs scope alone. (The sweep
   script `/tmp/wat_let_sweep.py` is preserved and works against
   any wat-rs-style codebase if/when reconstruction reaches that
   point.)

2. **Arc 159 v1 destructure-mangling regression caught early.**
   v1's sweep treated `(((a b) p))` (destructure) as legacy-typed
   shape. Arc 159 v3's substrate explicitly distinguishes
   destructure (binder children all Symbols) from typed (binder[1]
   is Keyword); the Python sweep script's heuristic was hardened
   accordingly; tests 9-10 in `src/runtime.rs::tests` verify the
   distinction.

3. **Arcs 160 + 161 shipped as their own arcs, not slices of 159.**
   Memory `feedback_v1_backout_dependency_arc.md` codified the
   pattern: when an arc reverts (or surfaces) due to a discovered
   substrate dependency, that dependency fix ships under its own
   arc number; the v2 of the original ships under a further arc.
   Arcs 160 + 161 closed the latent substrate gaps; arc 159 closes
   on top of both.

4. **Walker firing tests (3 in `tests/wat_arc159_let_bindings.rs`,
   3 in `src/check.rs::tests`) retired alongside walker body.**
   Vacuous post-retirement (no walker = nothing to fire = assertions
   pass trivially). Mirrors arc 154/155 walker-test retirement
   pattern. The runtime end-to-end tests (5 in `src/runtime.rs::tests`)
   + the regression test (`arc159_scope_deadlock_fires_on_new_shape_channel_binding`
   in `src/check.rs::tests`) stay — those exercise permanent features.

## Tests

Total arc 159 tests: ~13.

Retained (exercise permanent features):
- `arc159_new_shape_basic_addition` (test 1) — user end goal
- `arc159_new_shape_multi_binding_sequential` (test 2)
- `arc159_new_shape_closure_capture` (test 3)
- `arc159_new_shape_sequential_cross_reference` (test 4)
- `arc159_new_shape_nested_let` (test 5)
- `arc159_destructure_two_element_with_new_shape_source` (test 9)
- `arc159_destructure_three_element` (test 10)
- `arc159_mixed_new_shape_and_destructure` (test 12)
- `arc159_scope_deadlock_fires_on_new_shape_channel_binding` (test 11)

Retired with walker:
- `walker_fires_on_single_legacy_binding`
- `walker_fires_per_legacy_binding_in_multi_binding_let`
- `walker_fires_only_on_legacy_in_mixed_let`
- (3 sibling unit tests in `src/check.rs::tests`)

Workspace post-walker-retirement: 2041 passed / 0 failed.

## Out of scope

- **Square-bracket binding form** `[name expr name expr]`. Out of
  arc 159's scope. Tracked separately when/if the user directs;
  arc 159 keeps the existing paren-grouped binding shape and
  closes the type-annotation removal cleanly first.
- **Lab consumer sweep.** Out of arc 159's scope. Reason: lab is
  in reconstruction (memory `project_lab_reconstruction.md`);
  substrate work doesn't wait for lab. Not tracked elsewhere
  because the sweep is mechanical and the script preserves
  against future need.
- **Arc 162 (lambda internal-identifier rename).** Out of arc
  159's scope. Tracked in arc 162 (DESIGN at
  `docs/arc/2026/05/162-lambda-internal-rename/DESIGN.md`).
  Surfaced during arc 159 closure as a separate identifier-naming
  audit; opens after the 159/160/161 chain ships.

## Cross-references

- **Arc 158 v1** — reverted; REALIZATIONS at
  `docs/arc/2026/05/158-untyped-let-bindings/REALIZATIONS.md`
  captured the walker-vs-binding-shape coupling
- **Arc 158a** — precursor stepping stone (walker family migrated
  from declared `:T` to RHS pattern-match)
- **Arc 145** — paid-for lesson: declared type is redundant when
  inference suffices
- **Arc 154** — kill `let*`; clean-break + walker recipe arc 159
  mirrored
- **Arc 155** — fn rename; clean-break + walker recipe arc 159
  mirrored
- **Arc 157** — `def` form; sibling untyped-binding shape (top-level
  position rather than inner-binding position)
- **Arc 160** — variant constructor inference fix (gates arc 159)
- **Arc 161** — Symbol-headed application inference fix (gates arc 159)
- **Memory `feedback_substrate_already_typed.md`** — paid-for
  lesson driving the no-annotation decision
- **Memory `feedback_stepping_stones_proactive.md`** — slicing
  framework; arc 158a stepping stone
- **Memory `feedback_v1_backout_dependency_arc.md`** — naming
  pattern: dependency fix arcs (160, 161) ship separately; 159
  closes when chain green
- **Memory `project_lab_reconstruction.md`** — lab out-of-scope
  rationale

## Commit chain

- `5b51b67` arc 158 v1 back-out + REALIZATIONS
- `eb7c29e` arc 158a opens
- `ca43e56` arc 158a slice 1: walker pattern-matches RHS for new-shape let bindings
- `42a7803` arc 158a closes: INSCRIPTION
- `e0b9679` arc 159 opens (V2; user-visible)
- `25c8067` WIP arc 159 — substrate + sweep; 11 lib unit tests pending cleanup
- `c6b8d74` WIP arc 159 — test cleanup down to 9 failures (substrate inference gap surfaced)
- `80ebca6` arc 160 opens (gates arc 159 closure)
- `d4b5131` arc 160 slice 1 BRIEF (diagnostic)
- `8b6aebe` arc 160 slice 2 BRIEF (the fix)
- `ec5b36e` arc 159 sweep: wat_run_sandboxed.rs embedded wat
- `7ae2093` arc 160 slice 2: hoist variant-constructor inference into Region A
- `d221de1` arc 161 opens (gates arc 159 closure)
- `f6ab13f` arc 161 slice 1: Symbol-headed application inference
- `f8f9c0c` arc 162 queued: lambda internal-identifier rename
- `e615572` arcs 160 + 161 close: INSCRIPTIONs + SCORE
- `289a21b` arc 159 DESIGN correction: drop lab sweep slice
- `23808fb` arc 159 slice 3 substrate: retire `LegacyTypedLetBinding` walker
- (this commit) arc 159 slice 3: closure paperwork
