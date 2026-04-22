# Arc 014 — Core scalar conversions — Backlog

**Opened:** 2026-04-21.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** the living ledger — tracking, decisions, open
questions as slices land.

Arc 014 is a **cave-quest arc** cut mid-013. Arc 013 slice 4b
surfaced the absence of scalar conversions, paused with two
`#[ignore]`'d tests, cut this arc, then resumes when this closes.
Kept tight — the scope fences are in DESIGN.

---

## The gap

wat has every per-type operation family **except** conversion:

- Arithmetic: `:wat::core::i64::+`, `:wat::core::f64::*`, …
- Comparison: `:wat::core::i64::<`, `:wat::core::string::=`, …
- Predicates: `:wat::core::string::contains?`, …
- Indexed access: `:wat::core::first`, `:wat::core::nth`, …
- **Conversion: missing.**

Users can't render an i64 for printing, can't parse stdin, can't
bridge i64 and f64. The absence has shown up as friction across
several slices and turned into a hard block at arc 013 slice 4b's
integration-test shape.

---

## 1. Dispatch + schemes for the eight primitives

**Status:** ready. Concrete approach in hand.

**Problem.** Eight new `:wat::core::<source>::to-<target>` paths
need runtime dispatch entries, type schemes registered with the
checker, and their own unit tests.

**Approach.**

- **Runtime dispatch.** `src/runtime.rs`'s dispatch match
  (around line 1668, the `:wat::core::i64::+` area) grows eight
  arms. Each arm calls an `eval_*` helper:
  - `eval_i64_to_string(args, env, sym)` — straight `format!("{}", n)`.
  - `eval_i64_to_f64(args, env, sym)` — `n as f64`.
  - `eval_f64_to_string(args, env, sym)` — `format!("{}", f)`.
  - `eval_f64_to_i64(args, env, sym)` — `if f.is_finite() &&
    f >= i64::MIN as f64 && f <= i64::MAX as f64 { Some(f as i64)
    } else { None }`.
  - `eval_string_to_i64(args, env, sym)` — `s.parse::<i64>().ok()`.
  - `eval_string_to_f64(args, env, sym)` — `s.parse::<f64>().ok()`.
  - `eval_bool_to_string(args, env, sym)` — `if b { "true" } else
    { "false" }`.
  - `eval_string_to_bool(args, env, sym)` — match `"true"` →
    `Some(true)`, `"false"` → `Some(false)`, else `None`.
- **Schemes.** Each primitive gets its scheme registered
  alongside where `:wat::core::i64::+`'s scheme lives (verify
  exact location at slice time — likely `src/check.rs`'s
  builtin-scheme table). Fallible ones: parameter type is the
  source, return type is `Option<target>`. Infallible ones:
  straight `source -> target`.
- **Tests.** Each primitive gets at least one happy-path test +
  (for fallible ones) at least one None-path test, in
  `src/runtime.rs`'s `#[cfg(test)] mod tests`. Pattern matches
  existing arithmetic-op tests (`local_i64_add_works` or similar).

**Spec tension.** Adds eight public primitives to wat's core
language surface. Each needs one line in the 058 spec update
(slice 2).

**Unblocks:** arc 013 slice 4b's `#[ignore]`'d tests
un-`#[ignore]`; slice 5 can use casts naturally; any future
arc wanting to render scalars has the surface.

---

## 2. 058 spec update

**Status:** blocks on slice 1 landing.

**Problem.** `058-ast-algebra-surface` enumerates every
`:wat::core::*` primitive. Eight new primitives need the same
treatment — one line each under a new "Scalar conversions"
subsection.

**Approach.**

- Locate the core primitives section in
  `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`.
- Add a **Scalar conversions** subsection cross-referenced from
  the arithmetic / string / bool subsections.
- One line per primitive with source type, target type, and
  fallibility. Worked examples optional — the primitives are
  obvious enough.

**Spec tension.** 058 is the language-level source of truth.
Additions here register the conversions as part of wat's
permanent core surface.

**Unblocks:** arc 014 INSCRIPTION has a resolved spec to cite.

---

## 3. Un-ignore arc 013 slice 4b's paused tests

**Status:** blocks on slice 1 landing.

**Problem.** Two tests in `crates/wat-lru/tests/wat_lru_tests.rs`
carry `#[ignore = "arc 014 — awaits :wat::core::i64::to-string"]`.
Slice 1 gives them the primitive; this slice removes the ignore
gate.

**Approach.**

- `local_cache_put_then_get_returns_some` — drop the `#[ignore]`,
  confirm it passes (should print `"42"`).
- `local_cache_put_overwrites_existing_key` — same pattern, should
  print `"99"`.
- Run `cargo test --package wat-lru` — both pass, no ignored
  remaining (the third ignored marker is pre-existing, unrelated).

**Inscription target:** the un-ignore is part of arc 014's
close + slice 4b's resume. Recorded in both arcs' INSCRIPTIONs
when they close.

**Unblocks:** arc 013 slice 4b can resume toward its remaining
cleanup + commit.

---

## 4. Arc 014 INSCRIPTION

**Status:** blocks on slices 1–3 landing.

**Problem.** Every shipped arc writes an INSCRIPTION — the
permanent record of what landed, what didn't, what the arc's
thread resolved.

**Approach.**

- Single `INSCRIPTION.md` in this arc's directory.
- Sections: what shipped (the eight primitives), what was
  deliberately left out (char, keyword, holon, result-returning
  variants), what open questions resolved at slice time
  (whitespace handling, negative-number parsing, etc.), what the
  arc-013-pause resolution looked like.
- Length: matches the prior arcs' inscriptions — dense, complete,
  no filler.

---

## Open questions carried forward

- **Negative-number parse shape.** `str::parse::<i64>()` accepts
  `-42`; arc 014 inherits that. Confirm at implementation.
- **Whitespace in `string::to-*` primitives.** Rust's strict-
  parse shape. Users who want trimming call `:wat::core::string::trim`
  first. Documented in DESIGN open questions.
- **Special-value rendering in `f64::to-string`.** `NaN`, `inf`,
  `-inf` — Rust's Display shape. Pin at slice time if the exact
  strings need renaming.
- **Round-trip guarantees?** `(to-i64 (to-string n))` should =
  `Some(n)` for every finite in-range i64. Worth a test;
  documented in slice 1.

---

## What this arc does NOT ship

- Implicit coercion at arithmetic / comparison sites.
- Generic `as-<T>` / `cast::*` dispatcher forms.
- `:char` surface — wat doesn't have `:char` yet.
- Keyword / Holon / Vec / Tuple conversions.
- `Result`-returning variants with structured parse errors.
- Format specifier / precision control.
- Locale-sensitive parsing.

---

## Why this matters

wat is a systems-shaped language layered over a VSA substrate.
It already has the arithmetic, comparison, and logical surfaces
any scalar-capable language ships. The missing conversions have
been eight small holes that users hand-rolled around or avoided
entirely. Arc 014 closes the holes in one pass — small in scope,
large in sanding.

The arc-013 slice-4b cave-quest shape is itself load-bearing
discipline: when the door needs a key we don't own, we name the
key, park the door, cut the quest, return. We do not paper the
door to pretend we entered. This is the first arc cut from a
paused slice; the shape is now precedent.
