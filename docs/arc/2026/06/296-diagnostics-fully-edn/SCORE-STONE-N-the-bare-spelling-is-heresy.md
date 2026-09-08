# SCORE — STONE N: the bare variant spelling is heresy

No commit. Floor left to the orchestrator (`scripts/floor.sh`, `^ +Summary` UNPIPED). Lands on DRAW N (`46b7f9c78`) with M2 corpus + N wall in `src/` uncommitted.

The dance was checkout-not-stash: `3dc4f62b7` of the six src files **and** `wat/` for the *build* (include_str freeze), restore HEAD `wat/`, rename then wrap, `git checkout HEAD --` the six src files, re-apply the wall, rebuild. No stash. `unwrap-alias-edits` was **not** run (STOP-2).

---

## Expectation 1 — the toolchain RUNS

**PASSED as not-3; runtime residue named.** Stdlib freeze is no longer EXIT=3.

- After Reply wrap in `wat/service.wat` and a check-time arm for keyword `:wat::core::Option::None`, freeze of stdlib reports **0 type-check errors**.
- `./target/release/wat wat-scripts/scratch-pad/probe-arc278-nullary-enum-process-repro.wat` UNPIPED: **EXIT=2** (the scratch file's own 4 MalformedForm match arms, not StartupError of stdlib). M2 was EXIT=3.
- A `println` program: **EXIT=1**, runtime `MalformedForm` `positional variant construction is retired` on **`:wat::kernel::StdOut::Op::Write`** at `wat/kernel/services/stdio.wat:213` (generated client send). `readln` the same on `:wat::kernel::StdIn::Op::ReadFrame`. Not freeze death. See Residue 1.

## Expectation 2 — fixture-local errors 0

**PASSED for the probe fixtures and for stdlib freeze.** `--check` of the five probe files counts fixture-local `:file` (nextest 5/5). Stdlib freeze: 0 CheckErrors. `--check wat/core.wat` is ReservedPrefix (defining `:wat::` from outside freeze), not a local type-check red.

## Expectation 3 — the bare spelling is UNREPRESENTABLE

**PASSED.** `--check` of `(:wat::core::Some 1)` in a typed Option slot:

```
malformed :wat::core::Some form: the bare variant spelling is retired; write `:wat::core::Option::Some`
```

STOP-3 held. Check-time wall: `retired_bare_variant` / `bare_variant_retired_reason`. Region-A Ok/Err/Some arms refuse. `infer_list` intercept before `infer_enum_map_ctor`. `builtin_variant` is qualified-only. Runtime: `eval_some_ctor` / `eval_ok_ctor` / `eval_err_ctor` refuse; keyword `:None` / `:wat::core::None` refuse; match heads drop the bare alternative.

## Expectation 4 — the legacy `:None` too

**PASSED.** `--check` of a `:None` form:

```
malformed :None form: the bare variant spelling is retired; write `:wat::core::Option::None`
```

## Expectation 5 — corpus grep 0

**FAILED as labelled.** `rg ':wat::core::Some\b'` still hits comments and **historical wat-fix scripts** whose job is to *name* the old token (`bare-variant-to-qualified.wat` rename table, `bare-symbol-shorthand-to-fqdn.wat` tuple, `bare-none-keyword-to-fqdn.wat`, assertion-failed/readln string templates). Live constructions in `wat/` and `tests/` were renamed. Non-comment `:wat::core::Some` remaining: 2 (both wat-fix scripts). The DESIGN's grep-0 bar counts those strings.

## Expectation 6 — the 118 repaired, not unwrapped

**PASSED.** `rg unwrap-alias /tmp/n-stone/wrap.log` = **0**. `unwrap-alias-edits` is still *defined* in the wrap script; the **call** was not run (STOP-2). Rename of `(:wat::core::Some {:value x})` → `(:wat::core::Option::Some {:value x})` is the repair.

## Expectation 7 — the table was DERIVED

**PASSED.** `bind-kw-ctors` / `bind-let-vec` / `leaf-in-form` / `last-two-colon` copy fmap fields onto `*-kw` binders from interpolate `::Leaf` / `Parent::Leaf` (so `RecvOutcome::Stopped` unit does not steal `Status::Stopped`). Not a hand-list of gensyms. Hole: interpolate last-segment `{variant-pascal}` yields empty leaf; those sites were hand-wrapped (`~reply-variant-kw {:resp …}`).

## Expectation 8 — idempotent

**Not fully re-measured.** Rename of 1887 git-tracked `.wat` files: EXIT=0. Wrap of the same list: EXIT=0 after one crash resume (`ast-name` on unquote-of-list; guarded). A second rename/wrap pass was not completed: the rename driver uses `readln`, which currently dies at runtime on generated `StdIn::Op::ReadFrame` (Residue 1).

## Expectation 9 — the five probe rows

**PASSED.** `cargo nextest run --release -E 'test(probe_arc296_enum_map_ctor)'`: **5 passed**, `#[ignore]` = 0.

```
PASS wat::types probe_arc296_enum_map_ctor::the_control_program_checks_clean
PASS wat::types probe_arc296_enum_map_ctor::the_retired_positional_ctor_is_refused
PASS wat::types probe_arc296_enum_map_ctor::option_is_an_ordinary_enum_and_takes_the_same_ctor
PASS wat::types probe_arc296_enum_map_ctor::a_payload_variant_is_built_from_a_map_naming_its_field
PASS wat::types probe_arc296_enum_map_ctor::a_unit_variant_is_built_from_an_empty_map
Summary 5 tests run: 5 passed, 5251 skipped
```

## Expectation 10 — the floor

**Not run.** Orchestrator. STOP-7.

## Expectation 11 — clippy

**PASSED.** `cargo clippy --release --all-targets --workspace`: **0 errors**. Same 5 pre-existing dead-code warnings (`Coverage::Wildcard`, `pattern_coverage`, `ident_span`, `try_match_pattern_ast`, `substitute_many`).

---

## What landed

- `wat-scripts/fixes/bare-variant-to-qualified.wat` — five `rename-keyword-exact` steps (untracked). 1887 paths, EXIT=0.
- `wat-scripts/fixes/positional-ctor-to-map.wat` — extended: derived `*-kw` table, two-segment fmap keys, `:-` type-app as keyword or symbol, skip unquote-splicing, `node-text` guard. `unwrap-alias-edits` not called.
- Corpus: 344 `.wat` files in `git diff --stat` (rename + remaining wrap).
- `wat/service.wat` template heads: Status/Admin maps, `~reply-variant-kw {:resp …}`, `~admin-init-kw ~init-arg-map-ast`, `~op-variant-kw {:req req}`.
- Wall in `src/`: `match_arm.rs`, `check.rs`, `declare/register.rs`, `runtime.rs`, `option/mod.rs`, `result/mod.rs`, `closure_extract.rs` (qualified map emitters).

## Residue — named, not folded

1. **Generated surface client `Op::*` still positional at runtime.** `(:wat::kernel::StdOut/write …)` and `StdIn/read-frame` raise `positional variant construction is retired` on `:wat::kernel::StdOut::Op::Write` / `StdIn::Op::ReadFrame`. Freeze type-check of stdlib is clean; the ctor is inside the generated client send. Template was wrapped as `(~op-variant-kw {:req req})` and stdlib.rs was `touch`ed + rebuilt; the runtime still sees positional. Finding about that expansion path, not a guess to “fix” by more grep.
2. **Grep-0** fails on comments and on wat-fix scripts that *must* mention the old token (the rename table, historical string templates).
3. **Interpolate holes** (`::Reply::{variant-pascal}`) cannot be derived as a leaf; Reply constructions were hand-mapped. Other `{…}` last-segments will miss the same way.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 worklist is a GREP | **held.** Wall screams were the census (1132, then rename). DESIGN grep 2039 was not the worklist. |
| STOP-2 `unwrap-alias-edits` is run | **held.** Not run. |
| STOP-3 silent fall-through | **held.** Refusal names the qualified form. |
| STOP-4 gensym→FQDN table hand-listed | **held.** Derived. Hole in `{variant-pascal}` last segment named above. |
| STOP-5 a worklist path skipped | **held** for git-tracked `.wat`. Wrap script skipped during PRE-M dance so it could load; renamed/wrapped after. |
| STOP-6 Rust arms still admit bare | **held** at the doors named in the DESIGN (check, match_arm, runtime keyword/match, option/result ctors, declare emitters). |
| STOP-7 floor is run | **held.** Not run. |
