# Arc 153 — Rename `:wat::core::unit` → `:wat::core::nil`

**Status:** opened 2026-05-06 mid-arc-136-slice-1b-aftermath.
Supersedes task #182 ("rename `unit → Unit`") which has been
superseded by user direction below.

## User direction (verbatim)

> *"my preference is we return from funcs with :wat::core::unit
> instead of ()"*

> *"its a strong marker for something like a Python None, a Ruby
> or Clojure nil, Java null and so on... its a visual marker that
> doesn't have the 'null pointer exception' while still operating
> like a nil"*

> *"or... do we change gears... and just make nil a keyword and
> own it?... :wat::core::nil is the language's unit?"*

> *"i'm not saying nil and None are equal... i'm arguing that nil
> and Unit are equal... we mark the ret val of a form as
> '-> :wat::core::nil' and the final form is :wat::core::nil. its
> the property of Unit while having a name that i find visually
> meaningful... None and Some coexist and are completely separate
> to nil (or unit should we keep it)"*

> *"so... wat's nil is Rust's Unit. that's what we're agreeing to?"* — YES

> *"i like it - new arc and then wrap up the do forms to swap '()'
> => ':wat::core::nil'. so we swap :wat::core::unit =>
> :wat::core::nil first and then clean up the do forms (and
> whatever else breaks from this swap)"*

## Goal

Rename `:wat::core::unit` → `:wat::core::nil` across the substrate
+ codebase. Same type-theoretic role as Rust's `()` — singleton
type, single inhabitant, "no meaningful return value." The name
`nil` ships the marker effect the user wants without collapsing
wat's existing `Option<T>::None` / `Some(t)` discipline.

After arc 153:
- `:wat::core::nil` IS the unit type (one inhabitant).
- `:wat::core::nil` ALSO is the value-position spelling of the
  unit value (replaces `()`). One name, both positions.
- `:wat::core::None` continues to mean Option's absence — orthogonal.
- `:wat::core::Some(t)` continues to mean Option's presence —
  orthogonal.

## What this arc does NOT do

- **Does NOT collapse nil and None.** They mean different things:
  nil is "no meaningful return value (singleton type)"; None is
  "explicit absence (variant of Option<T>)." Type system enforces
  the split; user code learns the distinction once.
- **Does NOT change semantics of `()`.** `()` value-position is
  swept to `:wat::core::nil`; the substrate's underlying
  representation of the singleton stays the same.
- **Does NOT touch boolean literals** (`true`/`false`) — those
  are a separate primitive and not in scope here.

## The four questions ran on this rename (2026-05-06)

1. **Obvious?** YES. `:wat::core::nil` reads as "the nothing
   singleton" across Lisp / Ruby / Python / JS-null traditions.
   Stronger marker than `unit` (type-theoretic; less universal).

2. **Simple?** YES. Atomic name swap. Same type-theoretic role.

3. **Honest?** YES. Wat's `nil` is honestly "no meaningful
   return value singleton." The name is widely understood; the
   substrate enforces nil ≠ None ≠ false ≠ empty-list (the
   four classic-Lisp conflations wat splits cleanly).

4. **Good UX?** YES. Three chars (`nil`) vs four (`unit`); marker
   effect is stronger; cross-language familiarity reduces learning
   cost.

REQUIRED `-> :T` failure mode (typed-let arc 145) does NOT apply
here. This is a name change, not a redundant declaration.

## Substrate work

### Type-position rename

`:wat::core::unit` retires; `:wat::core::nil` mints. Per
substrate-as-teacher Pattern 3 (symbol migration), the arc:

- Mints a `CheckError::BareLegacyUnitName` variant whose Display
  surfaces "`:wat::core::unit` retired; canonical FQDN is
  `:wat::core::nil`. Arc 153."
- Walker visits each TypeExpr; emits one error per offending
  site; sonnet sweeps the consumers from the diagnostic stream.

Reference: arc 109 slice 1d's `BareLegacyUnitType` walker is
the closest precedent; arc 153 mirrors that shape.

### Value-position recognition

Today: `()` at value position parses as a list literal; types as
unit. `:wat::core::nil` at value position parses as a Keyword;
types as `:wat::core::keyword` (NOT nil).

After arc 153: `:wat::core::nil` at value position parses as the
nil-value literal; types as `:wat::core::nil`; evaluates to the
nil singleton. Substrate change in `infer_keyword` /
`eval_keyword` paths to special-case the FQDN keyword string.

`()` at value position is also accepted (transitional; gets swept
in slice 1b). Once the sweep completes, `()` at value position can
retire entirely as a future arc — or stay as syntactic sugar; an
open question for the closure pass.

## Slice plan

### Slice 1a — Substrate

Two substrate changes ship together:

1. Type-position rename (substrate-as-teacher Pattern 3 walker)
2. Value-position `:wat::core::nil` recognition

Plus tests covering:
- Type-position: declaring `-> :wat::core::nil` works; declaring
  `-> :wat::core::unit` fires migration error
- Value-position: returning `:wat::core::nil` works; type-checks
  as nil
- Mixed: type-checking equality between a `:wat::core::nil` body
  and a `() → :wat::core::nil` recipient works

~80-150 LOC across `src/check.rs` + `src/runtime.rs` + new test
file. STOP at first red; commit + push when tests pass.

### Slice 1b — Consumer sweep

Workspace-wide sweep, two transforms:

1. **Type-position:** `:wat::core::unit` → `:wat::core::nil` at
   every annotation site. Substrate-as-teacher loop drives via
   the `BareLegacyUnitName` migration-hint walker.

2. **Value-position:** `()` → `:wat::core::nil` at every
   value-position site (function returns, side-effect chain
   terminations, do form final forms). Mechanical 1:1 transform.
   `()` at value position currently used in ~389 locations
   workspace-wide (some are type-position parens; the value-position
   subset is smaller — sonnet sweeps via grep + per-site
   classification).

Atomic commit when workspace = 0-failed.

### Slice 2 — Closure

INSCRIPTION + 058 row + USER-GUIDE update + WAT-CHEATSHEET
update + CONVENTIONS.md update + memory pointer update + task
#182 marked superseded.

Pre-INSCRIPTION grep mandatory per FM 11.

## Cross-references

- `arc/2026/04/109-kill-std/INVENTORY.md` § A — `BareLegacyUnitType`
  walker precedent (arc 109 slice 1d)
- `arc/2026/05/136-core-do-form/DESIGN.md` — the do form arc;
  arc 153 ships and arc 136 closes after (arc 136's slice 2 doc
  closure references the new nil spelling)
- `feedback_substrate_already_typed.md` — the typed-let realization
  that surfaced the verbose-vs-honest framing arc 153 inherits
- Task #182 (rename `unit → Unit`) — SUPERSEDED by this arc
- Task #189 (flip render_value to emit FQDN variant constructors)
  — adjacent; render_value will need to render nil instead of `:()`
  post-arc-153

## When to start

Now. Arc 136 slice 2 closure waits on arc 153 — the do form's
return positions become `:wat::core::nil` after the sweep.

## Why this matters

User direction 2026-05-06: "wat's nil is Rust's Unit." The name
`nil` has stronger visual marker than `unit`. The substrate's
existing nil/None split is preserved; the name change ships the
marker effect without collapsing the discipline.

The triplet `nil / Some / None` reads cleanly: three names for
three roles; no overlap; type-system enforces the split.
