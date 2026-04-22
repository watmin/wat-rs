# Arc 014 — Core scalar conversions — INSCRIPTION

**Status:** shipped 2026-04-21. Cave-quest arc cut mid arc-013
slice 4b; ships + closes same day.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the living ledger.
**This file:** completion marker.

---

## Motivation

wat had every per-type scalar operation family **except**
conversion. Arithmetic: `:wat::core::i64::+`, `f64::*`. Comparison:
`:wat::core::<`, `:wat::core::=`. Predicates: `string::contains?`.
Indexed access: `first`, `nth`. But no way to render an `i64` for
printing, parse a `String` into an `i64`, or bridge `i64` and
`f64` numerically.

The gap had shown up as friction across several slices. Users
hand-rolled around it or avoided the situation. At arc 013
slice 4b it turned into a hard block: wat-lru's integration tests
needed to format an `i64` cache value into stdout for assertion.
The cleanest response was to pause 013 mid-motion and cut an
honest arc for the missing primitives.

Builder direction: *"scaler conversions are done or ready to
begin?"* Then: *"yes — let's do it — find our loot."*

---

## What shipped

One slice of runtime + schemes, one spec update, one un-ignore
of the arc-013 tests. Landed in that order.

### Slice 1 — eight primitives

Commit `878025c`.

**Dispatch** — eight arms in `src/runtime.rs`'s match, each
routing to a small `eval_*` helper defined immediately after
`eval_f64_arith`:

- `:wat::core::i64::to-string` — `format!("{}", n)`. Infallible.
- `:wat::core::i64::to-f64` — `n as f64`. Infallible (inherent
  precision loss past 2⁵³ is the type's nature, not an error).
- `:wat::core::f64::to-string` — `format!("{}", f)`. Infallible.
- `:wat::core::f64::to-i64` — range-checked
  `if f.is_finite() && f in i64::MIN..=i64::MAX { Some(f as i64)
  } else { None }` → `:Option<i64>`.
- `:wat::core::string::to-i64` — `s.parse::<i64>().ok()` →
  `:Option<i64>`.
- `:wat::core::string::to-f64` — `s.parse::<f64>().ok()` →
  `:Option<f64>`.
- `:wat::core::bool::to-string` — `"true"` / `"false"`. Infallible.
- `:wat::core::string::to-bool` — matches `"true"` / `"false"`
  exactly → `:Option<bool>`. Case-sensitive; no `"1"`/`"0"`
  shortcuts.

A small `eval_one_arg` helper absorbs the arity-1 + type-extract
boilerplate so each conversion helper stays three-ish lines.

**Schemes** — eight `env.register` calls in `src/check.rs`'s
`register_builtins`, right after the f64-arithmetic block.
Local closures `opt_i64_ty` / `opt_f64_ty` / `opt_bool_ty`
construct the `Option<T>` return types for the fallible ones.

**Tests** — 13 new tests in `src/runtime.rs`'s `#[cfg(test)] mod
tests`:

- One happy-path per primitive (8).
- `string_to_i64_returns_none_for_unparseable` — `"abc"`, `""`,
  `" 42 "` (whitespace strictness documented).
- `string_to_f64_returns_none_for_unparseable`.
- `f64_to_i64_rejects_nan` — covers out-of-range via `1e19` /
  `-1e19`.
- `string_to_bool_returns_none_for_unparseable` — `"True"`,
  `"1"`, `""`.
- `i64_string_roundtrip` — `(to-i64 (to-string 12345))` ⇒
  `Some(12345)`.
- `conversions_reject_wrong_input_type` — runtime defensive
  type-mismatch for `i64::to-string 2.5` and `f64::to-string
  "abc"`.

176 runtime tests pass. Workspace green.

### Slice 2 — 058 spec update

Commit `787b59c` in holon-lab-trading.

- `058-ast-algebra-surface/FOUNDATION.md` — reserved-prefix
  enumeration at line 2266 extended to include the eight new
  `:wat::core::<source>::to-<target>` paths alongside existing
  arithmetic primitives.
- `058-ast-algebra-surface/INDEX.md` — new audit-history entry
  dated `2026-04-21 (later)` recording arc 014's landing.
- **No separate 058-NNN sub-proposal.** Precedent set in the
  prior 2026-04-21 audit entry: arithmetic / comparisons /
  string ops are "Lisp-fundamentals — correctly not at the
  sub-proposal tier." Scalar conversions fit the same shelf.

### Slice 3 — arc-013 un-ignore

Commit `4e5c6dd`.

Dropped `#[ignore = "arc 014 — awaits :wat::core::i64::to-string"]`
from the two paused wat-lru integration tests:
`local_cache_put_then_get_returns_some` and
`local_cache_put_overwrites_existing_key`. Both pass — end-to-end
proof that slice 1's primitives land through the full
`Harness::from_source_with_deps` pipeline (dep composition,
macro expansion, type check, resolve, evaluation, stdio capture).

All four wat-lru integration tests green. Arc 013 slice 4b
(`#282`) unblocked to resume.

---

## Resolved design questions

- **Per-type namespacing** over generic `as-<T>` / `cast::*`.
  Mirrors existing `:wat::core::i64::+`, `:wat::core::string::contains?`.
- **`Option<T>` for fallible paths**, not `Result`. Matches wat's
  existing Option-returning idiom (`HashMap::get`, `List::first`
  on empty, etc.).
- **No implicit coercion** at arithmetic / comparison sites.
  `(:wat::core::i64::+ 3 2.0)` still errors. Every conversion is
  explicit at the call site.
- **Eight primitives**, no char / keyword / holon / Result
  variants. Scope fences at the minimum that discharged the
  in-hand debt.
- **Whitespace strictness.** `string::to-i64 " 42 "` returns
  `None`. Users who want trim semantics call
  `:wat::core::string::trim` first. Documented in the test;
  matches Rust's `str::parse` shape.
- **Special-value rendering.** `f64::to-string` inherits Rust's
  `Display` shape (`NaN`, `inf`, `-inf`). No custom overrides;
  no demand surfaced.
- **Negative-number parse shape.** `string::to-i64 "-7"` →
  `Some(-7)`. Matches `str::parse::<i64>()`.

---

## What's NOT in this arc (named as future)

- **`Result`-returning variants** with structured parse errors.
  `None` is what `HashMap::get` does; matching it keeps the
  surface coherent. If a caller needs "what specifically was
  wrong about the input," that's a later arc or a separate
  `:wat::core::<source>::parse-<target>` family returning
  `:Result<T, ParseError>`.
- **`:char` surface.** wat doesn't have `:char` as a scalar
  today. If it ever ships, its conversions land in a follow-up.
- **Keyword / Holon / Vec / Tuple conversions.** Keyword-to-String
  or reverse deferred pending demand. Holon serialization already
  has its own story (EDN canonicalization); Vec / Tuple don't
  have obvious single-target conversions to name.
- **Format specifier / precision control.** No `{:.2}`-style
  widths or precision. If a caller needs formatted output, stdlib
  or a dedicated formatting primitive is the home.
- **Locale-sensitive parsing.** Not shipping; `str::parse` is the
  shape. Locale handling would be a substantial arc of its own.

---

## Why this matters

Arc 013 slice 4b paused mid-execution rather than papering over
a real substrate gap with literal-only test branches. The
cave-quest discipline — *"you cannot open this door yet; go to
that cave to find it"* — turned out to be right: the absence of
scalar conversions was real, the primitives were obvious, and
shipping them as their own arc kept both 013 and 014 honest.

Arc 014 tightens wat's core language surface. Every "what about
converting X to Y?" question from here forward has an answer
under `:wat::core::*` — no ad-hoc stdlib forms, no macro hacks,
no hand-rolled multiplication tricks. The shape is set.

Beyond the immediate unblock, arc 014 is the first arc cut from
a paused slice. The shape is now precedent for future cave-quest
splits when a slice surfaces a real substrate debt.

---

**Arc 014 — complete.** Eight runtime primitives, eight schemes,
13 tests, one spec update, one arc-013 unblock. Three commits:

- `878025c` — slice 1 (eight primitives + tests)
- `787b59c` — slice 2 (058 spec update, in holon-lab-trading)
- `4e5c6dd` — slice 3 (un-ignore arc 013 slice 4b paused tests)

Arc 013 slice 4b (`#282`) is now unblocked. Resume resumes.
