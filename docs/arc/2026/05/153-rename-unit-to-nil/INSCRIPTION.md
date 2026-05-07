# Arc 153 — Rename `:wat::core::unit` → `:wat::core::nil` — INSCRIPTION

## The closing

Arc 153 closes 2026-05-06 the same day it opened. Three slices
shipped in one session: substrate (1a), consumer sweep (1b),
substrate retirement + paperwork (slice 2). The triplet
`nil` / `Some(t)` / `None` now reads cleanly across the
substrate, wat-rs's stdlib, every test, and every example.

The user direction that opened the arc:

> *"its a strong marker for something like a Python None, a Ruby
> or Clojure nil, Java null and so on... its a visual marker that
> doesn't have the 'null pointer exception' while still operating
> like a nil"*

> *"so... wat's nil is Rust's Unit. that's what we're agreeing to?"* — YES

The user direction that closed it:

> *"alright - let's get the paper work done."*

## What shipped

### Slice 1a — substrate (`fd1b3fe`, atomic with 1b)

Two coordinated substrate changes per substrate-as-teacher
Pattern 3 (symbol migration, mirroring arc 109 slice 1d's
`BareLegacyUnitType` precedent):

- **Type-position rename.** `:wat::core::unit` retired;
  `:wat::core::nil` minted as the canonical FQDN. New
  `CheckError::BareLegacyUnitName` variant + body walker
  (`walk_type_for_bare`'s Path-arm extension) + signature-pass
  walker (`walk_type_for_legacy_unit_name`) emitted one
  migration error per offending site.
- **Value-position recognition.** `:wat::core::nil` at value
  position parses as a Keyword; `infer` types it as the nil
  singleton (`Tuple(vec![])`); `eval` returns `Value::Unit`.
  The empty-list literal `()` continues to evaluate to the
  same singleton (transitional spelling, retained for
  cross-form ergonomics).

Four files: `src/types.rs`, `src/check.rs`, `src/runtime.rs`,
NEW `tests/wat_arc153_nil_rename.rs` (10 tests covering
type-position retired/canonical, value-position works,
type-mismatch, mixed empty-list/nil, parametric containment,
narrow special-case HashMap-key regression).

### Slice 1b — consumer sweep (`fd1b3fe`, atomic with 1a)

Workspace-wide migration driven by the substrate's diagnostic
stream. 90 wat + Rust files swept across two transforms:

- **Type-position** (substrate-as-teacher walker-driven):
  `:wat::core::unit` → `:wat::core::nil` at every annotation
  site across stdlib (`wat/`), per-crate substrates, wat-tests,
  examples, embedded wat in `tests/` + `src/` lib tests.
- **Value-position** (mechanical grep-driven): `()` →
  `:wat::core::nil` at function bodies, match arms, if branches,
  do-form final forms.

Atomic commit when workspace = 0-failed.

### Slice 2 — substrate retirement + paperwork (this commit)

Per substrate-as-teacher § "Retire the hint when its window
closes":

- **`:wat::core::unit` typealias removed** from
  `src/types.rs`. The transitional alias resolved the legacy
  spelling to `Tuple(vec![])` during the deprecation window;
  with all in-tree consumers swept, the alias retires.
- **`walk_type_for_legacy_unit_name` walker body retired** in
  `src/check.rs`. Comment names arc 153 as the retirement arc;
  the call site at `check_program` retires alongside.
- **`walk_type_for_bare`'s `:wat::core::unit` Path-arm
  detection retired**.
- **`CheckError::BareLegacyUnitName` variant + Display +
  diagnostic field-emit retained as orphaned scaffolding**
  per arc 113 precedent — the variant stays for testing /
  teaching / reintroduction; only the firing body retires.
- **Runtime `("wat::core::unit", "()")` value-tag match arm
  retired** in `src/runtime.rs`. Same retirement window; the
  arm was scaffolding for hand-crafted user-source spellings
  that survive parse-time as `Path(":wat::core::unit")` —
  with the typealias removed, no such spelling reaches the
  arm.
- **Internal anchor strings updated** in `src/check.rs`'s
  channel-pair-deadlock walker (synthetic
  `:wat::kernel::Channel<wat::core::nil>` /
  `Sender<wat::core::nil>` / `Receiver<wat::core::nil>` type
  annotations) plus rustdoc references for consistency.

Tests #1, #6, #10 in `tests/wat_arc153_nil_rename.rs` updated
for post-retirement behavior:

- Test #1 (`type_position_unit_post_retirement_is_unknown_fqdn`):
  formerly asserted `BareLegacyUnitName` walker fired on the
  user-source `:wat::core::unit` site. Post-retirement, the
  spelling parses as `Path(":wat::core::unit")`, `expand_alias`
  returns it unchanged (no longer registered), and unification
  surfaces `ReturnTypeMismatch` with `expected: ":wat::core::unit"`
  against `got: ":()"`.
- Test #6 (`reverse_mixed_nil_body_with_retired_unit_sig_post_retirement`):
  parallel update. The body is the canonical `:wat::core::nil`;
  the signature spells the retired FQDN; the same
  `ReturnTypeMismatch` shape surfaces.
- Test #10 (`bare_legacy_unit_name_walker_retired`): formerly
  verified the walker fired inside parametric args
  (`Option<wat::core::unit>`). Post-retirement, the walker is
  gone; the test now asserts `BareLegacyUnitName` does NOT
  appear in the error stream — the variant body retired
  cleanly, the variant + Display remain as scaffolding.

### Closure paperwork

- This `INSCRIPTION.md`.
- 058 changelog row at
  `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  (chronological order before PERSEVERARE).
- USER-GUIDE entry at `docs/USER-GUIDE.md` § 4 — `:wat::core::nil`
  section under "Writing functions" naming the singleton +
  the cross-language analogy + the triplet contrast with
  `:wat::core::None` / `:wat::core::Some(t)`.
- WAT-CHEATSHEET update at `docs/WAT-CHEATSHEET.md` § 3 —
  added `nil` row to the FQDN table; new "`:wat::core::nil` —
  the singleton (arc 153)" subsection; updated 7 lingering
  `:wat::core::unit` example usages in the channel-pair-deadlock
  illustration (now reading `wat::core::nil`); updated the
  `Thread/join-result` and `Process/join-result` table rows
  (`:Result<:(),...>` → `:Result<wat::core::nil,...>`).
- CONVENTIONS update at `docs/CONVENTIONS.md` § "Batch
  convention" — Put-verb signature description updated to
  `(put entries) -> :wat::core::nil` with explicit arc 153
  cross-reference.
- Task list update: arc 109 J-PIPELINE "1d post-rename:
  `:wat::core::unit` → `:wat::core::Unit`" entry struck
  through and marked SUPERSEDED by arc 153 with the
  user-direction rationale (chose `nil` over `Unit` for the
  marker effect).

## Why

User direction 2026-05-06: *"wat's nil is Rust's Unit."*

The name `nil` ships the marker effect of a Lisp's `nil` while
preserving wat's existing `Option<T>::None` / `Some(t)`
discipline. The substrate enforces the four-way split that
classic Lisp conflates: `nil` ≠ `None` ≠ `false` ≠ empty-list.

The triplet `nil` / `Some(t)` / `None` reads cleanly:
- `:wat::core::nil` — unit type (singleton; "no meaningful
  return value")
- `:wat::core::Some(t)` — `Option<T>`'s presence variant
- `:wat::core::None` — `Option<T>`'s absence variant

Three names, three roles, no overlap. The type system enforces
the split; user code learns the distinction once.

This supersedes task #182 (rename `unit` → `Unit`), surfaced
during arc 109 slice 1d's /gaze pass. The `Unit` rename argued
for PascalCase consistency with substrate-named types; the
`nil` rename argued for the marker effect Lisp programmers
already know. User direction chose `nil`.

## The four questions

Run on the rename 2026-05-06:

1. **Obvious?** YES. `:wat::core::nil` reads as "the nothing
   singleton" across Lisp / Ruby / Python / JS-null traditions.
   Stronger marker than `unit` (type-theoretic; less universal).
2. **Simple?** YES. Atomic name swap. Same type-theoretic role.
3. **Honest?** YES. Wat's `nil` is honestly "no meaningful
   return value singleton." The substrate enforces the four-way
   split (nil ≠ None ≠ false ≠ empty-list).
4. **Good UX?** YES. Three chars (`nil`) vs four (`unit`);
   marker effect stronger; cross-language familiarity reduces
   learning cost.

REQUIRED `-> :T` failure mode (typed-let arc 145) does NOT
apply; this is a name change, not a redundant declaration.

## Cross-references

- **DESIGN**: `docs/arc/2026/05/153-rename-unit-to-nil/DESIGN.md`
- **Per-slice BRIEFs**: BRIEF-SUBSTRATE.md (slice 1a),
  BRIEF-CONSUMERS.md (sweep 1b), BRIEF-CLOSURE.md (slice 2)
- **Atomic commit**: `fd1b3fe` (1a + 1b together per recovery
  doc § 7 atomic-commit-across-coordinated-sweeps)
- **Closest precedent**: arc 109 slice 1d (mint
  `:wat::core::unit`; retire `:()` as a type annotation) —
  arc 153 inverts the slice 1d pivot, using the same Pattern 3
  walker mechanics and the same retirement window discipline
- **Substrate-as-teacher discipline**: `docs/SUBSTRATE-AS-TEACHER.md`
  § "Retire the hint when its window closes" — slice 2 is the
  worked application
- **Orphaned scaffolding precedent**: arc 113 (variant +
  Display preserved after firing body retires)
- **Adjacent arc**: arc 136 (do-form arc) — slice 2 closure
  runs after this; the do form's return positions are
  canonically `:wat::core::nil`
- **Superseded task**: arc 109 J-PIPELINE "1d post-rename:
  `:wat::core::unit` → `:wat::core::Unit`" — marked SUPERSEDED
  by arc 153 with user-direction rationale

## Calibration record

- **Arc opened**: 2026-05-06 (DESIGN at commit `4029173`)
- **Slice 1a substrate shipped**: 2026-05-06 (atomic with 1b
  at commit `fd1b3fe`)
- **Sweep 1b consumer migration shipped**: 2026-05-06 (same
  atomic commit)
- **Slice 2 retirement + paperwork**: 2026-05-06 (this commit)
- **Total arc duration**: one session
- **Honest deltas**: substrate retirement closed cleanly with
  one substrate cross-reference update (the runtime
  `value_tag` match arm — also scaffolding from the migration
  window) plus three test reshape updates plus internal
  anchor-string consistency updates in the channel-pair
  walker. The walker's anchor strings were technically
  internal (synthetic; never user-visible) but updating to
  `wat::core::nil` keeps the substrate self-consistent.

## Status

**Arc 153 closes here.** The substrate's canonical singleton
type is `:wat::core::nil`; the legacy `:wat::core::unit`
spelling is retired in full (typealias gone, walkers retired,
runtime value-tag arm gone). The triplet `nil` / `Some(t)` /
`None` reads cleanly across substrate, stdlib, tests, and
examples. The orphaned `BareLegacyUnitName` variant + Display
remain as scaffolding for testing / teaching / future
symbol-migration arcs to reuse.

**Arc 109 v1 closure trajectory clearer.** One naming-cleanup
chain link closes; arc 136 slice 2 (do-form closure) runs
next with return positions canonically `:wat::core::nil`.

The arc started narrow + grew through user direction (the
choice of `nil` over `unit` over `Unit` was a single
exchange). The cascade closed in one session.

---

*the singleton has its name. the triplet reads cleanly. the
type system enforces the split. forward progress only.*

**PERSEVERARE.**
