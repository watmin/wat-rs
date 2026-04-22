# Arc 014 — Core scalar conversions

**Status:** opened 2026-04-21. Planning phase — opened as a
cave-quest during arc 013 slice 4b, which surfaced the gap.
**Motivation:** wat has per-type arithmetic (`:wat::core::i64::+`),
per-type comparison (`:wat::core::i64::<`), per-type string
predicates (`:wat::core::string::contains?`) — but **no
conversions between scalar types**. An i64 cache value can't
be rendered for printing. A String read from stdin can't be
parsed into an i64. A Boolean result can't be displayed. The
absence has been carried as friction across several slices;
arc 013 slice 4b turned it into a hard block when wat-lru's
integration tests needed to assert on cache-value contents.

Every other scalar-capable language has this. Rust has
`.to_string()` / `.parse::<i64>()`. Clojure has `str` / `parse-long`.
Python has `str()` / `int()`. The absence in wat is not a
design choice — it's an unshipped piece of core.

---

## Non-goals (named explicitly)

- **Implicit coercion.** No hidden casts at arithmetic sites,
  no "i64 + f64 = f64" through implicit widening. Every
  conversion is an explicit named call. This is the disciplined
  stance wat already takes with `:wat::core::i64::+` refusing
  f64 arguments.
- **Generic `as-<T>` or `cast::*` form.** Every conversion gets
  its own keyword path. Matches how `:wat::core::i64::+` and
  friends are structured — per-type namespacing, no overload
  resolution.
- **Parsing beyond the simplest numeric / boolean shapes.** No
  locale handling, no radix prefixes (`0x`, `0b`), no scientific-
  notation edge cases beyond what Rust's `str::parse` already
  supports. `String::to-i64` wraps `str::parse::<i64>()` — same
  grammar, same errors folded into `None`.
- **Holon / Keyword / Vec / Option / Tuple conversions.** Arc
  014 ships the **scalar-scalar** surface only: i64, f64, String,
  bool. Keyword-to-String (or reverse) is deferred until demand
  surfaces. Holon / Vec already have their own serialization
  stories that don't want to be absorbed here.
- **Float-precision configuration.** `:wat::core::f64::to-string`
  uses Rust's `Display` shape (shortest-roundtrip) — no `{:.2}`
  precision control in this arc. Demand-driven; if formatting
  control becomes load-bearing, that's a separate arc.
- **Error messages that carry the unparseable input.** Parse
  failures return `None`, not `Result<T, ParseError>`. Matches
  wat's existing Option-returning idiom; the `Result` story for
  structured parse errors is a later call.

---

## What this arc ships

Eight primitives. Each lives under `:wat::core::<source>::to-<target>`
— exactly the shape the existing per-type namespacing anticipates.

### The surface

| Primitive | Input | Output | Fallible |
|---|---|---|---|
| `:wat::core::i64::to-string` | `:i64` | `:String` | No |
| `:wat::core::i64::to-f64` | `:i64` | `:f64` | No |
| `:wat::core::f64::to-string` | `:f64` | `:String` | No |
| `:wat::core::f64::to-i64` | `:f64` | `:Option<i64>` | **Yes** — NaN / ±∞ / out-of-range → `:None` |
| `:wat::core::string::to-i64` | `:String` | `:Option<i64>` | **Yes** — unparseable → `:None` |
| `:wat::core::string::to-f64` | `:String` | `:Option<f64>` | **Yes** — unparseable → `:None` |
| `:wat::core::bool::to-string` | `:bool` | `:String` | No (`"true"` / `"false"`) |
| `:wat::core::string::to-bool` | `:String` | `:Option<bool>` | **Yes** — only `"true"` / `"false"` parse; rest → `:None` |

Infallible conversions return the target type directly.
Fallible conversions return `:Option<T>` — never `Result` in
this arc.

### Why these eight

- **i64 ↔ String**: covers the overwhelmingly common numeric
  rendering and stdin parsing cases.
- **f64 ↔ String**: same story for floats; `:wat::core::f64::to-string`
  unblocks any scientific / algebraic result display.
- **i64 ↔ f64**: bridges the two numeric tiers. `to-f64` is
  infallible (i64 ⊂ f64 modulo precision past 2⁵³; that loss is
  inherent, not an error). `to-i64` is fallible because f64's
  domain is strictly larger.
- **bool ↔ String**: the "render a control-flow outcome" case.
  The Cache service test that forced this arc's creation needed
  it for a different reason (i64 rendering), but bool deserves
  symmetry — pretending otherwise would be dishonest.

### What's deliberately left out

- No `:wat::core::i64::to-bool` / `bool::to-i64`. Wat doesn't
  conflate integers with booleans (no "0 is false"). Keeping the
  types disjoint is the right discipline. If user code wants
  boolean tests on integers, it writes `(:wat::core::i64::= x 0)`.
- No `:wat::core::string::length-as-i64` — `string::length`
  already returns `:i64` directly; nothing to convert.
- No `:wat::core::char::*`. wat doesn't have a `:char` scalar
  today; the String surface operates on strings, not code
  points. If `:char` ever ships, its conversions land in a
  follow-up arc.

---

## Placement — why `:wat::core::<source>::to-<target>`

- **Mirror symmetry with existing operations.** `:wat::core::i64::+`,
  `:wat::core::string::length`, `:wat::core::bool::and` — the
  per-type namespace already anchors type-specific operations.
  Conversions belong there, not in a separate `:wat::core::convert::*`
  tree.
- **Source-type as prefix, target-type in name.** Reads left-to-
  right as the conversion: *"from i64, to string."* Matches how
  Rust reads `i64::to_string()` and Clojure reads `(str 42)`.
- **No generic dispatcher.** Every pair is its own keyword path.
  Keeps wat's no-overload-resolution discipline honest — the
  reader sees exactly which conversion fires.
- **Alternative rejected: `:wat::core::cast::<from>-><to>`.**
  Symmetric but adds a level of nesting that buys nothing. Also
  conflicts with wat's reading convention — users already look
  under `:wat::core::i64::*` for things i64 knows how to do;
  pulling conversion out to a peer tree fragments that mental
  model.
- **Alternative rejected: `:wat::std::*`.** Conversions are
  core-language surface. `:wat::std::*` is where stdlib forms
  live; shipping these as stdlib forms would make them
  macro-expandable — needless. These are atomic primitives, like
  `+`.

---

## Implementation shape (pins at slice time)

- **Dispatch.** Each primitive gets a `eval_*` helper in
  `src/runtime.rs` — sibling to `eval_i64_arith` / existing
  `string_ops::*`. The runtime dispatch table at
  `src/runtime.rs:~1668` grows eight entries.
- **Schemes.** Each primitive gets a type scheme registered via
  the same path existing `:wat::core::*` primitives use.
  Infallible ones have straight `(:i64) -> :String` shape;
  fallible ones have `(:f64) -> :Option<i64>` shape.
- **Check pass.** `src/check.rs` already routes
  `:wat::core::i64::*` through the existing scheme registry; the
  new paths plug in the same way. No special-casing.
- **Resolve pass.** `:wat::core::*` is a reserved prefix that
  `is_resolvable_call_head` accepts as-is
  (`src/resolve.rs:210`). Nothing to change — the new paths
  land under an already-accepted prefix.
- **Tests.** One unit test per primitive in `src/runtime.rs`'s
  `#[cfg(test)]` block — the existing arithmetic / string-op
  tests establish the pattern. Round-trip sanity: `(to-string
  (to-i64 "42"))` = `"42"`. Edge cases: `(string::to-i64 "abc")`
  = `None`; `(f64::to-i64 f64::NAN)` = `None`.

### `:Option<T>` representation

wat-rs already has `Value::Option(Some(Box<Value>) | None)` as
of the arc that introduced Option (verify at slice time). The
fallible primitives construct `Some(...)` on success, `None` on
failure. No new runtime value variant.

---

## Resolved design decisions

- **2026-04-21** — **Arc cut from slice-4b cave quest.** wat-lru's
  integration tests needed `:wat::core::i64::to-string` to assert
  on cache values. Rather than hack the tests around the gap,
  pause 4b, ship 014, resume.
- **2026-04-21** — **Per-type namespacing.** `:wat::core::<source>::to-<target>`
  over any alternative shape.
- **2026-04-21** — **`Option<T>` for fallible, not `Result`.**
  Matches wat's existing idiom. `Result`-returning conversions
  are a later concern if structured parse errors become
  load-bearing.
- **2026-04-21** — **No implicit coercion at arithmetic sites.**
  Arc 014 adds explicit named conversions only; `:wat::core::i64::+`
  still refuses a mixed `(i64, f64)` call.
- **2026-04-21** — **Eight primitives, no char / keyword /
  holon surface.** Scope fences at the minimum that discharges
  the debt in-hand.

---

## Open questions to resolve as slices land

- **Negative-number parsing.** Rust's `str::parse::<i64>()`
  accepts `"-42"`. Matching that is the obvious choice; pin at
  implementation slice.
- **Whitespace handling in `string::to-i64` / `to-f64`.** Rust's
  `str::parse` is strict — `" 42 "` fails. wat inherits the
  strict shape; users who want trim semantics call
  `:wat::core::string::trim` first. Document explicitly.
- **`f64::to-string` for special values.** Rust renders `NaN` as
  `"NaN"`, `inf` as `"inf"`. wat adopts Rust's shape; pin when
  implementation lands if the exact strings need adjustment.
- **Can tests exercise these from wat source?** Yes — existing
  `#[cfg(test)] mod tests` in `src/runtime.rs` runs wat through
  `eval_expr`. That's the ergonomic pattern; add tests there,
  plus one integration in `wat-tests/` if a round-trip story is
  worth naming explicitly.

---

## What this arc does NOT ship

- Anything in `:wat::std::*`, `:wat::algebra::*`, `:wat::kernel::*`.
- Implicit coercion at arithmetic / comparison sites.
- `Result`-returning variants — future concern.
- Keyword / Holon / Vec / Tuple conversions.
- Formatting / precision control (`{:.2}`, width, padding).
- Locale-sensitive parsing.

---

## The thread this continues

Arc 013 slice 4b pauses mid-execution — two wat-lru integration
tests sit `#[ignore]`'d waiting for `:wat::core::i64::to-string`
to land. Arc 014 is the cave; its close un-ignores those tests,
and slice 4b resumes toward its own INSCRIPTION.

Beyond the immediate unblock, arc 014 tightens wat's core
language surface. Every "what about converting X to Y?" question
from here forward has an answer under `:wat::core::*` or lands
cleanly as a future arc extension — no ad-hoc stdlib forms, no
macro hacks, no hand-rolled multiplication tricks. The shape is
set.
