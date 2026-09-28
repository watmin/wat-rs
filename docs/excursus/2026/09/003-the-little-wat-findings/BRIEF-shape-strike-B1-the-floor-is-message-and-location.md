# BRIEF — shape strike B1: the error floor is `{message location}`

Excursus 003. This implements `AUDIT-the-shape-of-an-error.md`: finding F3, its § RULING
2026-09-27 (item 1), and the first rows of its Strike B worklist. The sweep (S1–S3) is complete:
every startup-error taxonomy is a declared wat record. That was the precondition the builder set
(*"should we just do the hard work now?.. deferral usually backfires"*), so no wrapper needs to
survive.

**Read `wat-rs/CLAUDE.md` in full first.** Every count and citation below is a claim. Verify before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

## Why

`causes` is written everywhere and read almost nowhere:
- **0** wat readers;
- **2** Rust readers, both walking aggregates;
- 38 of the 40 runtime kinds always write `[]`.

It also carries two meanings under one name. The two wrapping kinds use it for *causation*, while
the aggregates (moved there by step 3c's ruling, which the audit found wrong) use it for
*membership*. Its one real job was holding a **foreign** checker diagnostic under a `Fault` with a
**fabricated** `<runtime>:0:0` location. The sweep removed the foreignness.

## Target

1. **`:wat::core::Error` is `{message location}`** (`wat/core.wat`, the `defsurface`), and so is
   `:wat::core::Fault`.
   - `Fault/of` (`wat/core.wat:~2194`) stops passing `:causes`.
   - Every declared error record loses its `causes` field. By measurement that is **164 records in
     11 stdlib files**: `wat/{runtime,check,types,load,config,resolve,stdlib,rete,macro,parse}-errors.wat`
     and `wat/core.wat` (`git ls-files 'wat/**/*.wat' | xargs grep -cE "causes +<-"`).
   - That is a structural rewrite across many `.wat` files, so it goes through a **wat-fix codemod**
     (`wat-scripts/fixes/<name>.wat`), with a dry-run diff and a replay fixture, and never by hand.
   - Where a checker change makes the old form illegal, read `wat/fix.wat`'s BOOTSTRAP / STASH-DANCE
     header first.
2. **Rust.** `WatError::causes()` (`src/edn/contract.rs`) is removed from the trait, and so is its
   `:causes` insertion in `error_edn()`.
   - There are 16 `fn causes(&self)` implementations. Each one goes, or moves into its variant, as
     below.
   - `RuntimeError::to_record` (`src/value/runtime_records.rs`) drops its `empty_causes()` element
     from every arm.
3. **Aggregates hold `errors`, not `causes`.** `CheckErrors`, `ReteCheckErrors` and
   `UnresolvedReferences` put their items under an `errors` field, declared in their records.
   - This reverts step 3c's move. Its `location` rule (the first item's location) stays.
   - Update the two Rust readers: `src/rete/validate/mod.rs:~1527` and
     `tests/rete/probe_freeze_validator_lift_rete_namespace.rs:~69`.
4. **The two runtime wrapping kinds carry a named `cause <- :wat::core::Error`.**
   - `EvalVerificationFailed` wraps a `HashError`; `MacroExpansionFailed` wraps a `MacroError`.
     Since the sweep, both are **typed records**, so `cause` holds the real error (its declared
     record), not step 3a's lossy `Fault`.
   - The other taxonomies' wrapping kinds already use a `cause` field (`MacroErrorKind`'s two, load
     `Parse`, stdlib `ParseFailed`); confirm this.
5. **The four wrapper sites carry the diagnostic itself.** The sites:
   - `check_failed_cause` (`src/runtime.rs:~12583`);
   - the second `CheckFailed` producer (`:~12823`);
   - `read_outcome_malformed` and `read_json_outcome_malformed` (`src/edn/render.rs:~276,~613`).

   What changes at each:
   - `:CheckFailed [cause <- Error]` and `:Malformed [cause <- Error]` receive the strictly decoded,
     typed diagnostic directly: no `Fault` wrapper, and no fabricated span.
   - `fault_with_cause` is deleted.
   - If strict decode fails, that is a defect. Do not fall back to foreign, and do not fabricate a
     location. The site carries a `Fault` whose message states that the diagnostic did not decode,
     and why, with the diagnostic's **real** location (`e.location()` is in hand).
   - Measure whether that arm is reachable after item 6. If it is not, say so.
6. **Tag the one untagged map (Strike B worklist, S1 row).** `NoMatchingClauseAtCallSite.attempted-clauses`
   (`src/check.rs:~442`, `clause_attempts_to_edn`) emits untagged `{:arity :param-types}` maps.
   - Give each one a tagged, declared record.
   - Check whether `:wat::kernel::ClauseAttempt` (step 3a, the runtime `NoMatchingClause` attempt)
     carries the same meaning. If it does, reuse it. If it does not, declare a distinct name that
     says what it is.
   - Then declare the kind in `wat/check-errors.wat`, and remove it from S1's G-list exception.
   - After this, the foreign count at the four sites must be **0**.

## Gates (each mutation-proven in RELEASE)

- **GB1, no floor record carries `causes`.** A lint parses every tracked `.edn` golden and fails on
  any `:causes` key, anywhere. It must first be RED on today's goldens; report the count (about 243
  files).
  - Mutation: re-add `:causes []` to `error_edn()`. RED.
- **GB2, the surface is two fields.** `:wat::core::Error`'s registered surface has exactly `message`
  and `location`. Drive it.
  - Mutation: re-add the field. RED.
- **GB3, the wrapper sites are typed.** For each of the four sites, drive one real producer:
  - a REPL/eval check failure;
  - an EDN `read` of a malformed error;
  - a JSON one;
  - the second `CheckFailed` producer.

  Assert that the payload is the diagnostic's own declared record class, not `Fault` and not
  foreign.
  - Mutation: restore the `Fault` wrap. RED.
- **GB4, the foreign count is 0.** Extend S1–S3's G-strict gates so `NoMatchingClauseAtCallSite`
  now decodes typed, and remove its exception.
  - Mutation: untag the attempt again. RED.
- The existing gates from 3a, S1–S3 and D1–D4 stay green. They will need their field lists updated.
  Update them; do not weaken what they check.

## Goldens

- About 243 files carry `:causes`. Recapture with `UPDATE_EDN=1`, in the foreground, and **read
  every diff**. The allowed changes:
  - `:causes …` removed;
  - aggregates' items move to `:errors`;
  - the two wrapping runtime kinds gain `:cause`;
  - wrapper-site payloads lose their `Fault` wrap;
  - the attempt maps gain a tag.
- Report anything else.
- Any `assert_edn_eq!` + `include_str!` goldens have no `UPDATE_EDN` path. Regenerate them by
  driving the binary, as earlier strikes did.

## Scope fence

- **IN:** items 1–6.
- **OUT (strike B2):**
  - dotted enum tags for sum-typed sub-values (`EnsureFnInvalidReason`, `LoadFetchError`,
    `HashError`, `ClauseFailureReason`);
  - a tagged lex error;
  - `EnsureFnInvalidReason`'s namespace.
- **OUT:** C (`provenance`), D (`Frame`), E (`EvalError`), F (the domain `Fault`s), and the
  stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check
  with `pgrep -x cargo`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build, recapture or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On ANY red: surface it before any re-run, captured verbatim with the arm named.
  - A timeout: surface it with its duration history. Do not widen `.config/nextest.toml`; the
    orchestrator manages it.
- Stage by name BEFORE the floor. Never `git add -A`. No `cargo fmt`.
- New test files carrying wat-looking or EDN-looking strings need their runes.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- **If your budget runs short, stop at a clean boundary and report where.** "The codemod committed
  on a green floor" is a valid boundary.

## Report

- the re-measured counts;
- the codemod's name, its dry-run diff summary, and idempotence;
- the attempt-record decision;
- whether the decode-failure arm is reachable;
- each gate's anchor and its mutation RED;
- any golden changed beyond the allowed shapes;
- the floor `Summary` line, verbatim;
- the SHA(s).
