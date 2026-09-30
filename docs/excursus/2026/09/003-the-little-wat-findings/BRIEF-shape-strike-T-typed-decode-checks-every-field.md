# BRIEF — shape strike T: typed decode checks every field against its declaration

Excursus 003. This fixes the hole found in strike B1: `AUDIT-the-shape-of-an-error.md`, § "Strike B1
landed", first row. It follows § RULING 2026-09-30, where the builder said *"let's continue in the
order you've expressed"*, putting T before B2.

**Read `wat-rs/CLAUDE.md` in full first.** Every count and citation below is a claim. Verify it before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

## The hole, read at the source at `3ca91673a` (line numbers are claims)

`reconstruct_struct` (`src/edn/render.rs:~3869`) and `reconstruct_record` (`:~3936`) decode a tagged
map into a declared aggregate. For each declared field `(fname, fty)` they:

1. look the key up (a missing key → `UnknownStructField`, which is correct);
2. decode the value with **no expected type**: `edn_to_value_caps(fv, …)`;
3. use `fty` **only** to re-wrap an `Option` (`rewrap_option_field`).

**Consequence 1.** A value of the wrong shape is accepted. An untagged map in a record-typed field
decodes as a generic `HashMap`, and a string decodes happily where an `i64` is declared. B1's GB4
measured this: untagging `AttemptedClause` did not redden the sweep's G-strict gate. **Every
G-strict gate from S1–S3 therefore proves that each tag is registered, not that each field's shape
matches.**

**Consequence 2.** Keys that the declaration does not name are silently dropped. `by_key` is built
from every entry, but only declared fields are read. For example, a lingering `:causes` would have
decoded cleanly after B1.

There is **no general checker** of a value against a declared `TypeExpr` today:
- `Record/assoc` compares only `type_name()` of the old and new value (`src/record/update.rs:~296`).
- The closest thing is `value_matches_type_by_name` (`src/function/subsume.rs:78`), which is private
  to function dispatch and used at 11 sites.

## Target

1. **One checker, one door.** Write `value_conforms(value, &TypeExpr, &TypeEnv) -> Result<(), Mismatch>`,
   or extract `value_matches_type_by_name` into it if that is the natural home. **Do not leave two
   answers to one question.** If dispatch has a reason to differ, name the reason and share the
   common core. It covers:
   - primitives;
   - `Option`, `Vector`, `HashMap`, `HashSet`, tuples and the persistent collections, recursively on
     their element types;
   - a named aggregate path: the class matches, respecting the nature/subtype edges `types.rs`
     already registers;
   - a named enum: the value is a variant of it;
   - a **surface** such as `:wat::core::Error`: structural, meaning the value's record carries the
     surface's fields with conforming types;
   - `:wat::core::Value`, which accepts anything;
   - a type variable, which accepts anything, unless the aggregate's type arguments are known at the
     decode site, in which case say what you did.

   The `Mismatch` names the path to the failing position (field and element), the expected type,
   and what was found.
2. **Decode uses it.** Both reconstructors check every field value with `value_conforms` against
   `fty`, **after** the `Option` re-wrap. A failure is a new `EdnReadErrorKind::FieldTypeMismatch
   { type_path, field, expected, got }`. It is a refusal as a value, never a panic.
3. **Undeclared keys are refused.** A key the declaration does not name becomes
   `EdnReadErrorKind::UnknownField { type_path, key }`. Before building this, **search the record**
   (the arcs, DESIGN docs and comments around `reconstruct_*`) for a deliberate forward-compatibility
   tolerance. If one exists, STOP and report it: the builder decides. If none exists, build it.
4. **Foreign (data) mode is unchanged.** `build_foreign_record`, for unregistered tags, has no
   declaration to check against. Leave it alone.

## Measure the fallout: this is the point of the strike

Turning the check on will refuse things that decode today. **Every refusal is a finding. Classify
each one before fixing anything:**
- **(a) Wrong declaration.** A sweep or 3a record declared a field type that does not match what the
  writer emits. Fix the declaration.
- **(b) Wire defect.** The writer emits the wrong shape, for example a bare `nil` where `Option` is
  declared, or an untagged map. Add it to the B2 worklist. **Do not paper over it** with
  `:wat::core::Value`. If it blocks the floor, STOP and report it.
- **(c) Consumer.** A test or program that relied on the looseness. Report it with what it relied
  on.

Put the census in the report, one row per distinct cause.

## Gates (each mutation-proven in RELEASE)

- **GT1, a wrong-shaped field is refused.** One case per checker arm: a primitive, a collection
  element, a named record, an enum variant, a surface and `Option`. Each must be refused with
  `FieldTypeMismatch`, naming the path.
  - Mutation: make `value_conforms` return `Ok` for that arm. RED. **One mutation per arm**: a
    multi-arm gate needs a mutation for each arm.
- **GT2, an undeclared key is refused.** Mutation: drop the check. RED.
- **GT3, the sweep's gates become shape proofs.** Re-run the prescribed mutation from B1's GB4
  (untag `AttemptedClause`) against the **existing** S1 G-strict gate. It must now go RED **by
  itself**. Do the same for one flat-record field in each of S1, S2 and S3.
- The existing gates stay green, with declarations corrected per (a).

## Goldens

Expected: **none change**, because this strike tightens reading, not writing. If a golden changes,
report why.

## Scope fence

- **IN:** the checker, its use in decode, the undeclared-key refusal, declaration fixes of class
  (a), and the gates.
- **OUT:**
  - fixing wire defects of class (b): list them for B2;
  - dotted enum tags, the lex tag, `HashError`'s floor and `fault_value` (B2);
  - C, D, E, F;
  - the stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check
  with `pgrep -x cargo`, never `pgrep -f`. Note that a running floor shows up as `cargo-nextest`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - A timeout: surface it with its history. Do not widen `.config/nextest.toml`.
- Stage by name BEFORE the floor. Never `git add -A`. No `cargo fmt`.
- `.wat` multi-site edits go through the wat-fix codemod.
- New test files need their runes.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- **If your budget runs short, stop at a clean boundary and report where.** "The checker plus GT1
  and GT2 committed on a green floor, fallout census in progress" is a valid boundary.

## Report

- where the checker lives, and whether dispatch now shares it;
- the forward-compatibility search result;
- the fallout census (a/b/c, one row per cause);
- each gate's mutation RED, per arm;
- GT3's results;
- the floor `Summary` line, verbatim;
- the SHA(s).
