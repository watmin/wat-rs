# BRIEF — STONE P-1 RELAND-1: the question widens, and the blanket dies

## The work, in one paragraph

The wall is right. It found three real phantoms in one floor run — `:wat::core::Int`,
`:wat::core::Keyword`, and `:wat::kernel::ExitCode` (a type arc 170 RETIRED on 2026-05-10, still
being annotated). It is wrong five times, and all five are one cause: it asks two membership stores
when there are at least four. Widen the question to the real union, delete both
`is_reserved_prefix` skips, and fix the three phantom annotations the wall correctly caught.

★ **Nothing here is a revert.** The tree as it stands is the starting point.

## ⛔ Delete these two lines. They are arc 255's founding defect.

`src/check.rs`, in `validate_named_type_annotations`, twice:

```rust
if crate::resolve::is_reserved_prefix(name) { continue; }
```

`is_reserved_prefix` is the `:wat::*` blanket-accept this whole campaign exists to remove. Measured
with both removed: **838 of 845 corpus files refuse on exactly TWO names**, both `:rust::sqlite::*`
— a false-positive pair, not a class. The skip is not load-bearing once the question is right.

## The real union — read these four stores

```
1  TypeEnv::contains          src/types.rs:601   = types ∪ builtin_names        ALREADY ASKED
2  is_builtin_primitive       src/runtime.rs                                    ALREADY ASKED
3  UseDeclarations::contains  src/rust_deps/mod.rs:290                          ⬜ ADD
4  derive markers             src/types.rs:542 `subtype_edges`                  ⬜ ADD
```

**Store 3 — `:rust::*` FFI imports.** `(:wat::core::use! :rust::sqlite::Connection)` validates
against `RustDepsRegistry::has_type` and records into `UseDeclarations`
(`src/resolve/rust_use.rs:41-48`). ⚠ **Match `resolve/walk.rs:112`'s coverage rule EXACTLY** — it is
prefix coverage, not equality:

```rust
use_decls.list().any(|decl| head == decl
    || (head.starts_with(decl) && head[decl.len()..].starts_with("::")))
```

That wall already guards `:rust::*` CALL HEADS per-program. The annotation position must ask the
same question of the same store, by the same rule. Do not write a second rule; if the predicate can
be shared, share it.

**Store 4 — derive markers.** `(:wat::core::derive :t::A :t::Marker)` makes `:t::Marker` a usable
BOUND (`[m <- :t::Marker]`) without making it a `types` key. Markers appear in `subtype_edges`
(`src/types.rs:542`) as VALUES. A name that is a declared marker is a legitimate annotation.

## Read in order

1. `docs/.../SCORE-ORCHESTRATOR-STONE-P1-the-wall-is-right-22-of-27.md` — the full floor roster,
   the six names, and which population each belongs to. **Read this first; it is the measurement.**
2. `src/check.rs` `validate_named_type_annotations` — your own code; the two `continue`s.
3. `src/declare/typevar.rs` `first_unknown_named_type` — the predicate that needs stores 3 and 4.
4. `src/resolve/walk.rs:104-124` — the existing `:rust::*` coverage rule, verbatim, to mirror.
5. `src/rust_deps/mod.rs:277-292` — `UseDeclarations`.
6. `src/types.rs:542` — `subtype_edges`.

## The three phantoms — fix them, they are real

```
:wat::core::Int         16 failures   tests/collection/list.rs's beside-fixture.   -> :wat::core::i64
:wat::core::Keyword      5 failures                                                -> the real spelling
:wat::kernel::ExitCode   1 failure    tests/program/wat_arc170_slice_1e_user_main_nil.rs
                                      Arc 170 DESIGN §482: "Nil IS the exit code (no ExitCode
                                      type — superseded 2026-05-10)". The fixture is annotating a
                                      RETIRED type. `:user::main -> :wat::core::nil` is the canonical
                                      signature (recovery doc § 13).
```

⚠ Confirm each replacement against the declaration, not against what looks plausible. A wrong name
does not fail — it names something else. `[[feedback_a_wrong_name_does_not_fail_it_names_something_else]]`

## Acceptance

```
cargo nextest run --release -E 'test(p1_annotation)'          7 passed, 0 skipped
./target/release/wat --check <each of the 7 fixtures>         EXITs unchanged from the SCORE table
```

Plus, for the widened question, **add fixtures and tests to the existing probe file** — a `use!`d
`:rust::*` annotation must be ACCEPTED, and a derive-marker bound must be ACCEPTED. These are the
new over-reach detectors; without them the union's two new members are asserted, not proven. A
`:rust::*` path with NO `use!` must still refuse.

⛔ The floor is the orchestrator's. Do not run it. Your targeted checks are the probe binary and
per-fixture `--check`.

## STOP triggers — each is a REJECTION

**STOP-1.** If deleting either `continue` requires a THIRD skip, an allow-list, or any predicate of
the form "names starting with X are exempt" — STOP and report. Widening the question is the stone;
another blanket is the defect wearing a new prefix.

**STOP-2.** If a store you need is not reachable from `validate_named_type_annotations`'s call site
in `freeze/env.rs` — STOP and report which, with the call chain. Do not thread a global.

**STOP-3.** If the corrected spelling for any of the three phantoms is not obvious from a
declaration you can cite — STOP on that one and report it. Two of the three look like case errors
and the third is a retirement; a guess here creates the exact defect the wall exists to catch.

**STOP-4.** If removing the skips leaves ANY corpus file refusing after stores 3 and 4 are added —
STOP and report the names. Measured prediction: zero. A non-zero answer means a fifth store.
