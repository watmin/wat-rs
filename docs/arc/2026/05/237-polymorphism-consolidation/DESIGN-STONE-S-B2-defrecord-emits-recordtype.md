# DESIGN — Stone S-B.2 — defrecord emits `recordtype` + drops its hand-rolled predicate

**Arc:** 237, records-first-class thread.
**Status:** READY (sub-DESIGN). Makes the everyday `defrecord` surface ride B.1's
machinery. Wat-side (Record.wat macro); no Rust substrate change.
**Builds on:** S-B.1 SHIPPED (`89c01888`) — `recordtype` + `TypeDef::Record` +
∀T `is-X?` synthesis.

## Why this stone

S-B.1 made `recordtype` a real type-decl form, but only a hand-written `recordtype`
call produces a first-class record type. The everyday surface — `(:wat::Record::def
:my::Circle [...])` — still emits its OWN hand-rolled `is-Circle?` predicate (the
narrowing `[v <- :wat::Record]` form that type-errors on a non-record) and registers
NO type. S-B.2 rewires the macro to emit `recordtype` (so the class becomes a
`TypeDef::Record`) and DROP its hand-rolled predicate (so the type system's
autonomous `register_type_predicates` synthesizes the ∀T `is-Circle?`). This is the
division of labor landing on the everyday surface: the macro emits the
field-specific constructor + accessors; type-registration mints the uniform predicate.

## What this stone delivers (Record.wat ONLY)

Two edits to the `:wat::Record::def` macro expansion (`wat/Record.wat`):

1. **ADD** `(:wat::core::recordtype ~fqdn :wat::Record)` to the emitted `do` block
   (emit it FIRST, before the constructor). Parent is `:wat::Record` (base umbrella).
   The flavor split (base→`:wat::Record` vs holonic→`:wat::holon::Record`) is S-C;
   B.2 emits `:wat::Record` for the current single macro.
2. **REMOVE** the hand-emitted predicate defn (the last `do` form, ~lines 220-232:
   the `(:wat::core::defn ~(...is-<base>?-name...) [v <- :wat::Record] -> :bool
   (:wat::core::conforms? v ~fqdn))`). `register_type_predicates` now synthesizes
   `:my::is-Circle?` ∀T autonomously (B.1). Dropping it avoids a `DuplicateDefine`
   collision (the type system + the macro would both try to define the name).

**Constructor return type stays `-> :wat::Record`** (UNCHANGED). Do NOT flip to
`-> :my::Circle`: the macro's own accessors (`[v <- :wat::Record]`) and their
internal `:wat::Record/field-at` call would reject a `:my::Circle` value, because
arg-boundary subtyping isn't wired until S-A1. The per-class return type is the
S-A1 pairing, NOT B.2.

## What B.2 proves that B.1 couldn't

B.1 had no constructor (recordtype declares a type, mints no constructor), so it
couldn't exercise the is-X? **TRUE-path**. B.2's defrecord HAS the constructor, so:
`(:my::is-Circle? (:my::Circle 1.0))` → `true` (the B.1-deferred contract, runtime
`conforms?` on `class_fqdn`). The asymmetry-kill is now on the EVERYDAY surface.

## Out of scope (REJECTED — not deferral)

- **Per-class constructor return type** (`-> :my::Circle`) — the S-A1 pairing
  (needs arg-boundary subtyping to not break the accessors).
- **The flavor split** (`:wat::Record::def` base vs `:wat::holon::Record::def`
  holonic) — S-C. B.2 keeps the single current macro; parent `:wat::Record`.
- **No Rust substrate change** — B.1 shipped the machinery; B.2 is pure wat-macro.

## FM 2-bis probe (NEW — committed before the BRIEF)

`tests/probe_arc237_sB2_defrecord_recordtype.rs`. Drives `(:wat::Record::def ...)`.
Pre-stone: fails (the everyday macro doesn't emit recordtype → the class isn't a
type → `subtype?`/synthesized-`is-X?`-∀T-on-nonrecord behave per pre-B.2). Post-stone:
all PASS. Contracts:

1. **everyday is-X? ∀T (asymmetry dead on the real surface)** — `(:wat::Record::def
   :my::Circle [radius <- :f64])`; `(:my::is-Circle? 42)` → `false`, NOT a type error.
2. **is-X? TRUE-path (B.1-deferred, now provable)** — `(:my::is-Circle? (:my::Circle 1.0))`
   → `true`.
3. **is-X? cross-class false** — `(:my::is-Circle? (:my::Square 2.0))` → `false`.
4. **edge wired by the emitted recordtype** — `(subtype? :my::Circle :wat::Record)` → `true`.
5. **accessors + constructor still work** (regression: dropping predicate + adding
   recordtype didn't break the rest) — `(:my::Circle/radius (:my::Circle 1.0))` → `1.0`.
6. **no DuplicateDefine** — defrecord + the synthesized predicate coexist (startup
   succeeds; the macro dropped its own predicate). (Implied by 1-5 all running.)

Plus baseline + the defrecord regression suite (see BRIEF).

## Proven-moves template + the consumer-ripple note

- Pure wat-macro edit; mirror the existing `do`-block emission shape. No Rust.
- **Consumer ripple (the FM-9 surface):** ~17 `tests/*.rs` files defrecord classes
  + call `is-<Name>?`. After B.2 their `is-X?` is the SYNTHESIZED ∀T form (false on
  non-record) instead of the macro's narrowing form (type-error on non-record). Most
  will pass unchanged (they call is-X? on records). If a test asserted the OLD
  type-error-on-non-record behavior, its expectation shifts to `false` — a mechanical
  update reflecting the asymmetry-kill (legitimate B.2 ripple). If a NON-test
  (substrate) breakage appears, STOP — that's not expected.
- No `wat/` or `wat-tests/` defrecord consumers (only Record.wat itself) → the
  substrate-bundled sources are safe; ripple is confined to `tests/*.rs`.

## Files

- `wat/Record.wat` — the two macro edits.
- `tests/probe_arc237_sB2_defrecord_recordtype.rs` — the probe (committed pre-BRIEF).
- Possibly a handful of `tests/*.rs` defrecord probes IF their is-X? expectation
  shifted (test-expectation updates only; substrate-as-teacher).
- NO src/*.rs. NO holon-rs (STOP-5).

## Calibration

Two-line-class macro edit + a behavior-shift consumer re-run. Lighter than a
substrate stone, but the consumer ripple is the variable. **Target band: 30–60 min
Mode A; 80 STOP-3; 110 STOP-4.** If >3 test files need expectation updates, that's a
signal to pause + report (not auto-sweep). Mirror SCORE-STONE-S-B1 shape; cite
S-B.1 + 237.6 in the BRIEF.
