# SCORE — STONE 255.77: cutover stone 3 — the UUID type goes home: `:wat::core::Uuid` → `wat.uuid/UUID`

**Executor: a Sonnet subagent, working solo.** Drawn against `c16915636`. Commit: see the
accompanying `git log` entry on `main` beside this file (committed locally, not pushed).

## What changed

**The key** (`src/`): the type's registered key becomes `:wat::uuid::UUID`. The `Value` variant
`wat__core__Uuid` → `wat__uuid__Uuid` (FM 14 — a retired path leaves no live internal identifier
encoding it) in `src/value/value.rs:302` (declaration), plus every match arm/construction site:
`src/holon/ast.rs:397`, `src/runtime.rs:5758`, `src/value/observe.rs:441`,
`src/closure_extract.rs:2028`, `src/function/subsume.rs:285`, `src/edn/render.rs:2363,2612,4805`.
Registration/membership strings `":wat::core::Uuid"` → `":wat::uuid::UUID"` at `src/types.rs:3560`
and `:9267` (the two closed-membership lists/tests), `src/check.rs:1634` (`is_atomizable`),
`:15029` (`BUILTIN_PRIMITIVES`-adjacent pure-scalar match in `rete/purity`-style code),
`:19995` (`uuid_ty` TypeScheme builder), and `src/runtime.rs:10267`
(`BUILTIN_PRIMITIVES`). `src/intrinsic/uuid.rs`'s doc comments and `expected:` TypeMismatch
strings (lines 95,188,241,279 per the brief, plus every doc `@arg`/`@ret`) now read
`:wat::uuid::UUID`.

**The retirement door** (`src/check.rs:1010-1025`): a new HARD-CUT arm in
`walk_for_bare_primitives`, same shape as the `:wat::core::Char` arm immediately above it —
`if s == ":wat::core::Uuid"` fires in ANY keyword position (no privileged paths), refusing with a
`MalformedForm` naming `wat.uuid/UUID`. `src/remedy/retirement.rs` gained one row:
`RetirementEntry { retired: ":wat::core::Uuid", replacement: "wat.uuid/UUID", note: None }` — the
**first** row in the table whose `replacement` is the `.wat` surface symbol spelling rather than a
colon key, because U1 (builder, 2026-10-01) named it that way: `wat.type/` holds only the 24 hard
primitives, Uuid is not a member, so its home is a namespace-preserving symbol (the
`wat.time/Instant` shape), unlike every other row (all verbs, still keyword-FQDN-spelled).

**The corpus** (`wat-scripts/fixes/uuid-type-goes-home.wat`, new, copying
`rename-four-families-to-their-homes.wat`'s shape): one rule, exact-string match on the whole
keyword token `:wat::core::Uuid`, converts to the symbol `wat.uuid/UUID`. Finder census (re-derived
myself, not trusted from the brief):

```
tests/rete/probe_arc278_6a_purity.wat:7
tests/types/probe_arc241_stone8_defstruct_c04.wat:6
tests/types/probe_arc241_stone8_defstruct_c04.wat:7
tests/value/wat_arc221_keyword_nil_tag_atomization.wat:119
tests/value/wat_arc221_keyword_nil_tag_atomization.wat:130
wat/class.wat:58
wat/service.wat:128
wat/telemetry.wat:77
wat/telemetry/journal.wat:30
wat/telemetry/span.wat:20
tests/types/probe_arc255_77_framing_floor_pin.wat:16   (my own new pin fixture, item 1)
```

10 matches, 9 corpus files + my own fixture. `tests/types/uuid_v4_returns_typed_uuid.wat:1` and
`wat/telemetry.wat:15` (both comments) correctly produce **zero** matches — the rule only sees
`Written` keyword facts, which exclude comments and string literals by construction. Dry-run on a
`/tmp` copy, diffed (span-faithful, only the targeted token changed, surrounding whitespace
preserved per `wat/fix.wat`'s own contract), then applied to all 9 real paths + my fixture.
Idempotent: re-running the finder over the WHOLE tracked `*.wat` corpus after applying returns 0
matches.

**The trap** (`wat/telemetry.wat`'s `framing-floor-of`, lines ~316-324): the pre-stone branch
compared `(:wat::core::ast-name t)` against literal strings (`"wat.type/i64"`, `"wat.type/Uuid"`,
etc.) — exactly the renderer-fragile pattern the brief warned about. Replaced **all four
branches** (not just Uuid) with `:wat::core::type-equal?` — the type door, parsing both the field's
type node and a freshly-built reference node (`(:wat::core::keyword-node ":wat::core::i64")` etc.)
to `TypeExpr` and comparing structurally — so a future rename of any of these four names can no
longer silently fall to `:else 0`.

**The renderer finding** (brief item 4, "find out why `field-types-of` renders `wat.type/Uuid`
today"): `src/edn/render.rs`'s `type_expr_to_clojure_form`, Case 1 of both the `Path` and
`Parametric` arms, does `body.strip_prefix("wat::core::")` and unconditionally renders the tail as
`wat.type/<tail>` — **it never checks the tail against the 24-name closed set**
(`WAT_TYPE_HARD_PRIMITIVES`, `src/types.rs:241`, a private `const` not reachable from
`edn/render.rs`). Before this stone, `Uuid` lived at `:wat::core::Uuid`, hit this un-gated branch,
and rendered as `wat.type/Uuid` — a spelling that, read back, is **not** a member of `wat.type/`
(confirmed: zero corpus occurrences of `wat.type/Option` or `wat.type/Result` either, which would
hit the identical bug). **Not cured here** — reported per the brief's explicit escape hatch ("report
it if the cure is wider than this stone"): fixing Case 1 to consult the closed set touches the
renderer's dispatch for every `:wat::core::`-prefixed path in the whole runtime (not just Uuid),
including `Option`/`Result`/every present and future `:wat::core::` name, and verifying it changes
nothing for those (no golden currently exercises `wat.type/Option`, so the blast radius is
unmeasured) is its own stone. This stone's rename makes the bug **inert for Uuid specifically**
(the type's key no longer starts with `wat::core::`, so Case 1 no longer applies — it falls through
to Case 3, the namespace-preserving symbol branch, producing the correct `wat.uuid/UUID`), but the
general gap (any future `:wat::core::X` for a non-hard-primitive X renders as an invalid
`wat.type/X`) remains open.

## Tests

- `tests/types/probe_arc255_77_framing_floor_pin.{wat,rs}` — item 1: pins `framing-floor-of`'s
  numbers for isolated i64/f64/Uuid/bool fields, measured against the unmodified draw
  (**38, 42, 55, 24**). The fixture's own Uuid field is one of the codemod's corpus targets, so
  this test exercises the POST-rename spelling while asserting the PRE-rename numbers — identical,
  confirming both the rename and the trap's cure are numerically inert.
- `tests/types/probe_arc255_77_uuid_retirement.{wat.bad,rs}` — the retirement refusal, mirroring
  the `:wat::core::Char` precedent (`probe_arc242_stone1_lexeme_role.rs::contract_03`): pattern-
  matches `CheckErrorKind::MalformedForm { head: ":wat::core::Uuid", reason: "… use 'wat.uuid/UUID'
  instead …" }`.
- `tests/types/probe_arc255_77_uuid_home_positions.{wat,rs}` — `wat.uuid/UUID` checks and runs as
  a fn header's param+return type, and as a `HashSet` element (dedup via Equatable+Hashable).
  Record-field and map-key positions are covered by the corpus's own converted sites
  (`probe_arc241_stone8_defstruct_c04.wat`, `wat_arc221_keyword_nil_tag_atomization.wat`); the
  `extend-type … Equatable` by `wat/class.wat:58`.
- `tests/cli/every_recorded_migration_replays.rs` — the end-to-end gate requires every
  `wat-scripts/fixes/*.wat` to carry a replay fixture; added
  `wat-scripts/fixes/replay/uuid-type-goes-home/{before.pre,after.post,ORACLE}` (a brand-new
  codemod has no prior landed commit, so provenance is `spec`/`spec-before` citing the codemod's
  own header text, same shape as `bare-symbol-shorthand-to-fqdn`'s fixture).
- `tests/cli/retirement_table_reachable.rs` — walks `RETIREMENT_TABLE` itself end-to-end; picked
  up the new row automatically (no hand-list to update) and passed on the first floor it ran in.

## The floor went RED once, from this stone's own change — cured, re-floored green

First floor (`.floor/2026-10-01T09-19-28Z`, 6337/6339, 2 failed) — both captured whole from
`ARM.txt`, both caused by this stone's own change, neither a flake:

1. **`wat::lint keyword_heresy_ledger::the_heresy_ledger_matches_its_frozen_census`** —
   `walk_for_bare_primitives`'s ledger row grew from 4 (`Ax4`) to 5 (`Ax5`): the new HARD-CUT arm
   `s == ":wat::core::Uuid"` is counted as shape-A heresy (an exact literal compare on a `WatAST`
   keyword payload) by the same instrument that already counts its let*/lambda/unit/Char siblings
   in the same function. This is the established, tolerated idiom for this exact walker (every
   sibling arm is the identical shape, already in the frozen baseline) — not a dual-spelling
   decision needing a canonical-identity door (the name is a fixed invariant literal at this site,
   never arriving as a Symbol here). Cure: bumped `LEDGER_TOTAL` 148 → 149 and the
   `walk_for_bare_primitives` row 4 → 5 in `tests/lint/keyword_heresy_ledger.rs`, with a dated
   comment, same convention every prior stone's bump used.
2. **`wat::cli every_recorded_migration_replays::every_recorded_migration_is_fixtured_or_runed`** —
   `uuid-type-goes-home: no fixture, no rune:replay(unreadable-preimage)`. Cure: added the replay
   fixture described above (not a `rune:replay` exemption — the codemod IS fixturable; a minimal
   before/after pair with `spec`/`spec-before` provenance citing the codemod's own header text).

Re-floored clean (foreground, nothing else running): `.floor/2026-10-01T09-29-58Z`,
**6339/6339 passed, 0 failed, 24 skipped** (384.480s). 6339 = 6336 (the prior green floor,
`.floor/2026-10-01T08-16-09Z`) + 3 (this stone's three new test files).

## Gates — verbatim results

| gate | result |
|---|---|
| pinned frame floors | `38 42 55 24` before AND after — identical (STOP-2 clear) |
| no old spelling left | only the retirement row + its doc-table entry, its own test, `uuid-type-goes-home.wat`'s own text + sibling codemods' unrelated VERB-retirement precedent text, and historical "arc 255.77 moved the key home from X" comments in `src/` and the new test files — see full breakdown in the executor's report |
| idempotent | `0` matches re-running the finder over the whole tracked `*.wat` corpus after applying |
| census | `scripts/replay/census.sh --diff` — `census-diff: no STOP-8` (rc 0), both before the red→green cure cycle and again at the final state |
| release floor | `Summary [ 384.480s] 6339 tests run: 6339 passed (27 slow), 24 skipped` — clean after curing the two self-caused reds above |
| clippy | `cargo clippy --release --all-targets -- -D warnings` → rc 0, no warnings |

## STOPs

None. No Uuid site outside the measured list needed a design decision (STOP-3 did not fire). No
printed form changed in a way that is not this rename (STOP-1 did not fire — the EDN tag stays
`#uuid "…"`, unchanged, per the measurement's own prediction). No `framing-floor-of` pinned number
changed (STOP-2 did not fire).

The renderer finding (`type_expr_to_clojure_form`'s un-gated `wat::core::` → `wat.type/` Case 1) is
reported, not cured — see "The renderer finding" above — because the cure is wider than this
stone.
