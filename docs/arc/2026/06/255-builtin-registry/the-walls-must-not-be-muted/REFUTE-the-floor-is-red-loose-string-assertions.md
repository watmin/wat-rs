# REFUTE — the floor is RED. `no_loose_string_assert`, 20 sites, all of them this stone's.

> Orchestrator, central floor at the stone's tree. **Do not re-run** — the log is kept at
> `.floor/2026-09-05T06-22-38Z/` and the arm below is verbatim from it.

```
     Summary [ 120.474s] 5166 tests run: 5165 passed, 1 failed, 17 skipped
```

## THE ARM — verbatim, whole. (My first draft of this file cut it at the blank line after the
## panic header, which is the very truncation the doctrine forbids. This is the untruncated block.)

```
thread 'no_loose_string_assert::tests_carry_no_loose_string_assert' (1756031) panicked at /home/john/work/holon/wat-rs/tests/lint/no_loose_string_assert.rs:112:5:


    🔥🔥🔥 LOOSE STRING ASSERTIONS — 20 site(s) assert a value with contains/starts_with/
    ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
    malformed maps, and appended garbage.

    THE FIX (RUBRIC: docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden'): a deterministic
    STRUCTURED value goes in a co-located `<probe>__<label>.edn` golden, compared via
    `wat::assert_edn_eq!(actual, include_str!("...edn"))` (parses both sides, structure-exact) —
    capture the whole value, never guess. A scalar -> byte-identical `assert_eq!`. EXEMPT a
    legitimately-loose one (a value that varies per run: path/pid/hash/timestamp, or a targeted
    absence over a large output) with a per-site `// rune:lint(loose-assert) — <reason>`.

    Drive it to ZERO. Offenders:

    crates/wat-doc/src/print.rs:398
    crates/wat-doc/src/print.rs:413
    crates/wat-doc/src/print.rs:414
    crates/wat-doc/src/print.rs:415
    crates/wat-doc/src/print.rs:416
    crates/wat-doc/src/print.rs:417
    crates/wat-doc/src/print.rs:418
    crates/wat-doc/src/print.rs:419
    crates/wat-doc/src/print.rs:420
    crates/wat-doc/src/print.rs:421
    crates/wat-doc/src/print.rs:425
    crates/wat-doc/src/print.rs:436
    crates/wat-doc/src/print.rs:460
    crates/wat-doc/src/print.rs:468
    crates/wat-doc/src/print.rs:469
    crates/wat-doc/src/print.rs:478
    crates/wat-macros/src/edn_doc.rs:536
    crates/wat-macros/src/edn_doc.rs:537
    crates/wat-macros/src/edn_doc.rs:539
    crates/wat-macros/src/edn_doc.rs:663
```

## THE 20 OFFENDER SITES — all in this stone's new code

```
crates/wat-doc/src/print.rs:398
crates/wat-doc/src/print.rs:413
crates/wat-doc/src/print.rs:414
crates/wat-doc/src/print.rs:415
crates/wat-doc/src/print.rs:416
crates/wat-doc/src/print.rs:417
crates/wat-doc/src/print.rs:418
crates/wat-doc/src/print.rs:419
crates/wat-doc/src/print.rs:420
crates/wat-doc/src/print.rs:421
crates/wat-doc/src/print.rs:425
crates/wat-doc/src/print.rs:436
crates/wat-doc/src/print.rs:460
crates/wat-doc/src/print.rs:468
crates/wat-doc/src/print.rs:469
crates/wat-doc/src/print.rs:478
crates/wat-macros/src/edn_doc.rs:536
crates/wat-macros/src/edn_doc.rs:537
crates/wat-macros/src/edn_doc.rs:539
crates/wat-macros/src/edn_doc.rs:663
```

## WHY YOUR SCOPED RUNS WERE GREEN AND THIS IS NOT

`cargo test -p wat-doc` and `-p wat-macros` do not build `tests/lint/` — that binary lives in the
root crate. Your 58/58 and 90/90 were true and they could not see this. **This is the standing
division, not a criticism:** the rider runs targeted checks, the orchestrator runs the floor
centrally, once, on a quiescent tree. A rider's targeted green is not a verdict.

## WHAT THE LINT DEMANDS

Its own message carries the rubric (`docs/CONVENTIONS.md` § *Test idioms* → *The .edn golden*):

- a deterministic STRUCTURED value → a co-located `<probe>__<label>.edn` golden compared with
  `wat::assert_edn_eq!(actual, include_str!("...edn"))` — **capture the whole value, never guess**
- a scalar → a byte-identical `assert_eq!`
- a legitimately-loose one → **EXEMPT it with a rune carrying a reason**

⚠ **I am not ruling that all 20 are defects.** Some may be genuinely loose — an assertion that an
error message NAMES a field, for instance, is about the naming and not the whole string. That is
yours to argue site by site, and the rune is the honest vehicle where it holds. What is not
available is leaving them as loose assertions with no reason.

⛔ **Do not weaken the round-trip gate to satisfy the lint.** The gate's assertions are the stone's
whole value. If an exact `assert_eq!` is awkward there, an `.edn` golden of the printed row is the
rubric's own answer — and it would strengthen the gate, not weaken it.

## WHAT IS ALREADY VERIFIED, so you do not re-do it

Rows 1–8 of EXPECTATIONS all hold. I re-ran them independently:

- census **571 · 85 · 52** unchanged; `no @syntax` = 535, so your 36 is right
- `git diff --stat crates/wat-edn/` empty
- the three `from_metadata` widenings are **strictly additive** — previously-erroring input now
  accepted. I checked the one non-additive change (`xs...` → rest) against the corpus: **no
  existing wat metadata-map arg name ends in `...` or `…`**, so nothing changes meaning today.
- `@syntax` uncovered, declared with a structural reason — accepted, and it is the same boundary
  the DESIGN drew around `@alias`.

**Only the floor is red. Fix the 20, nothing else.**
