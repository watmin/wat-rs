# SCORE 2a4 — the door reads a stdlib file as stdlib; a stdlib-changing step converts in two phases

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `81c9dd22d` BRIEF 2a4 order-of-work. Stash `grok #22 overlay` held the staged #22 tree
while this stone landed on the clean tree.

```
floor  scripts/floor.sh   .floor/2026-09-14T00-14-15Z
       Summary [ 213.487s] 5438 tests run: 5438 passed, 25 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

First floor `.floor/2026-09-14T00-09-20Z` was RED (captured, not re-run). Arm:
`wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert` — the new
`declared_stdlib_types_registers_reserved_prefix` test used `err.cause.contains("ReservedPrefix")`
without a `rune:lint(loose-assert)`. Cause EDN embeds a span that moves with the call site.
Exemption added. Second floor green. No STOP.

## The door

Sibling verb `:wat::runtime::declared-stdlib-types` (`@arg` forms, `@ret` DeclaredTypes, one
`@example`). `build_env`'s STDLIB half on these forms, against a FRESH copy of the snapshot:

`register_stdlib_defmacros` → `expand_all_with(…, Privilege::Stdlib)` →
`register_stdlib_types_replacing` → `register_variant_types`.

A type the file declares that the snapshot holds DIVERGENTLY is REPLACED in that copy only
(`TypeEnv::retract_for_door_replace` then `register_stdlib_with_span`). Returns every type the
file declared (including replacements), not only new names. Real clashes between two stdlib
files still refuse at `cargo build` + startup.

`wat/fix.wat` `enum-fields` chooses the verb by path: `stdlib-source-path?` is true for `wat/…`
or `*/wat/…` and false if the path contains `wat-scripts`. convert.sh out-dir copies
(`/tmp/…/wat/gen.wat`) take the stdlib door.

`scripts/replay/convert.sh` documents the two-phase recipe (the script itself does not
cargo-build): (a) convert the step's `wat/*.wat`; (b) put them in the tree, `cargo build
--release`; (c) convert the step's other `.wat` with the rebuilt binary. C^ uses HEAD's binary.

Deadlock walk also flags `declared-stdlib-types`. `macros/mod.rs` re-exports `expand_all_with`.

## Fixtures (each cited to a header spec line)

| case | result |
|---|---|
| new stdlib file (`:wat::*` prefix) | `declared_stdlib_types_registers_reserved_prefix`: `:wat::probe2a4::Colour` registers `Red []` / `Blue [n]`. Same text through the user door: `ReservedPrefix`. Header: *a new stdlib file (the #22 shape)*. |
| stdlib file CHANGES an enum's fields | `declared_stdlib_types_replaces_divergent_fields`: V `[a]` then V `[b c]`; env2 answers the NEW fields; env1 (a different copy) still `[a]`. Header: *the converted constructor uses the NEW fields*. |
| consumer in the same step | the two-phase recipe; proven live at #22 (SCORE-4), not a unit test. |

## The bar

| item | result |
|---|---|
| Floor + clippy | **PASS.** 5438/5438, clippy 0. First floor red captured. |
| `run5.sh` unchanged | **PASS.** See table. 0 files losing on match-arm and variant-separator; PC LOSING=1 is the same 2a2 artifact; chain vs main identical **1370**, CHAIN-FAILS **28**. |
| #22 `wat/gen.wat` converts, binary starts, named tests pass | next: stash pop, two-phase, SCORE-4. |

## run5 (`bootstrap/era/probe-S/run5.sh` on `02e1ee81e`, SCORE-only amend after)

| tool | wall (one process) | files LOSING | files GAINING | UNRESOLVED |
|---|---|---|---|---|
| match-arm vs `probe-L/wL` | 147 s | **0** | 1 (`probe-m1-ann-erase2.wat` nested PoolMsg) | 10 (ladder 15) |
| positional-ctor vs `probe-L/pT` | 29 s | 1 (same artifact: `probe_arc278_macro_generates_service.wat`) | 7 | 173 (today 7663) |
| variant-separator vs the census tool | 55 s | **0** | 67 | report lines 35 |

Chain vs main: identical **1370**, differs 119; classifier CHAIN-FAILS **28** (same as 2a2-REFUTE). START 00:23:57Z DONE 00:33:29Z.

## First floor (captured, not re-run)

`.floor/2026-09-14T00-09-20Z/` Summary [212.379s] 5438 tests run: 5437 passed, 1 failed, 25 skipped.

```
        FAIL [   0.051s] ( 117/5438) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
  stdout ───

    running 1 test
    test no_loose_string_assert::tests_carry_no_loose_string_assert ... FAILED

    failures:

    failures:
        no_loose_string_assert::tests_carry_no_loose_string_assert

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 137 filtered out; finished in 0.05s

  stderr ───

    thread 'no_loose_string_assert::tests_carry_no_loose_string_assert' (2386439) panicked at /home/john/work/holon/wat-rs/tests/lint/no_loose_string_assert.rs:112:5:


    🔥🔥🔥 LOOSE STRING ASSERTIONS — 1 site(s) assert a value with contains/starts_with/
    ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
    malformed maps, and appended garbage.

    THE FIX (RUBRIC: docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden'): a deterministic
    STRUCTURED value goes in a co-located `<probe>__<label>.edn` golden, compared via
    `wat::assert_edn_eq!(actual, include_str!("...edn"))` (parses both sides, structure-exact) —
    capture the whole value, never guess. A scalar -> byte-identical `assert_eq!`. EXEMPT a
    legitimately-loose one (a value that varies per run: path/pid/hash/timestamp, or a targeted
    absence over a large output) with a per-site `// rune:lint(loose-assert) — <reason>`.

    Drive it to ZERO. Offenders:

    src/check.rs:23699

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

## Blast radius

`src/freeze/env.rs` (`register_declared_stdlib_types`) · `src/types.rs`
(`retract_for_door_replace`, `register_stdlib_types_replacing`) · `src/reflect/verbs.rs`
(`:wat::runtime::declared-stdlib-types`) · `src/check.rs` (scheme, infer_list sibling, fixtures,
deadlock walk) · `src/macros/mod.rs` (`expand_all_with` re-export) · `wat/fix.wat`
(`stdlib-source-path?`) · `scripts/replay/convert.sh` (two-phase header). Not pushed.
