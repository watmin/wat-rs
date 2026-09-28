# BRIEF — the sweep: every startup error kind is a declared wat record

Excursus 003. It follows `AUDIT-the-shape-of-an-error.md` § RULING 2026-09-27, and a second ruling
made on the same day. Strike B (`causes` leaves the floor) found that `causes`' only real job is to
carry a **foreign** checker diagnostic at four decode sites:
- `check_failed_cause`, `src/runtime.rs:~12583`;
- a second `CheckFailed` producer, `src/runtime.rs:~12823`;
- `read_outcome_malformed` and `read_json_outcome_malformed`, `src/edn/render.rs:~276,~613`.

Those diagnostics are foreign because their tags (`#wat.check/…`, `#wat.resolve/…`, …) are not
declared wat types, and a foreign tree cannot satisfy `:wat::core::Error`. So it rides as a cause
under a `Fault` whose location is fabricated (`Span::new("<runtime>", 0, 0)`). The builder chose the
hard work now over an interim wrapper: *"should we just do the hard work now?.. deferral usually
backfires"*. This sweep runs first; strike B follows it with no wrapper left anywhere.

**Read `wat-rs/CLAUDE.md` in full first.** Every count and citation is a claim; verify before relying
on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it and name
it.

## The one invariant: the sweep is PURE DECLARATION

Each record mirrors **exactly** what that error's `WatError::error_edn()` emits on the wire TODAY:
- the same tag;
- the same keys (the current floor `message location causes`, plus the kind's own fields);
- the same value shapes.

**No golden changes.** No writer changes. The only behaviour change is at the four decode sites: the
strict decode (`decode_trusted_wire`) now succeeds, and a **typed** record replaces the foreign one.
Strike B reshapes everything afterwards in one move.

A golden that changes during the sweep means a record did not mirror the wire. That is a bug in the
declaration, not something to recapture.

## Scope: three strikes, one commit each, each on a green floor

Kind counts were measured at `e4dc02520` by counting variants to each enum's closing `}` and checking
the last variant. Re-measure them.

| strike | taxonomies | kinds |
|---|---|---|
| **S1** | check (`CheckErrorKind`, `src/check/error.rs:87`) and the `CheckErrors` aggregate | 34 |
| **S2** | types (`TypeErrorKind`, `src/types/error.rs:76`), load (`LoadErrorKind`, `src/load/loader.rs:295`), config (`ConfigErrorKind`, `src/config.rs:222`), resolve (`ResolveError`, `src/resolve/error.rs:25`, plus `UnresolvedReference`), stdlib (`StdlibErrorKind`, `src/load/stdlib.rs:716`) | 23 + 8 + 8 + 1(+1) + 1 |
| **S3** | rete (`ReteCheckErrorKind`, `src/rete/validate/error.rs:23`, and `ReteCheckErrors`), macro (`MacroErrorKind`, `src/macros/error.rs:42`), parse (`ParseErrorKind`, `crates/wat-reader/src/parser.rs:37`), lex (`LexErrorKind`, `crates/wat-reader/src/lexer.rs:198`) | 18 + 16 + 11 + 10 |

**This invocation runs the strike the orchestrator names.** Do not run ahead into the next one.

## What each strike builds

1. **Declarations.**
   - One `defrecord :wat::<ns>::<Kind>` per kind, in one stdlib file per taxonomy (e.g.
     `wat/check-errors.wat`), wired into `src/load/stdlib.rs` the way step 3a wired
     `wat/runtime-errors.wat`.
   - `<ns>` is the tag's own namespace (`#wat.check/TypeMismatch` → `:wat::check::TypeMismatch`).
   - Each has a `;;` doc line that says what it means.
   - Nested sub-values that already carry a tag need their own declarations, like 3a's
     `ValueSnapshot`/`ClauseAttempt`: a `TypeExpr` rendering, a sub-record, a nested error.
     Measure each one.
   - **Untagged record-shaped maps.** If one is on today's wire, the builder has ruled that
     record-shaped values are always tagged. Do not declare around it: STOP and list it, with its
     kind and field. Tagging it changes the wire, which makes it strike B's business.
2. **Rust derived from the `.wat`.**
   - Registration via `wat_record_from!`/`wat_enum_register_from!`, beside 3a's.
   - No hand-typed Rust copy of any record. `TypeEnv::with_builtins()` must record zero duplicates:
     the 3a duplicate guard is live, so this shows up at once.
3. **Nothing else.** No `to_record` conversions: these errors reach wat only as EDN through the
   decode ladder, and the declaration is what the ladder needs. If a kind turns out to be
   constructed in-process as a `Value`, name it.

## Gates (per strike, each mutation-proven in RELEASE)

- **G-list, the declaration is the list.** For every kind in the strike's enums, drive a real
  instance, as 3a's `all_variants` did. The set of tags its `error_edn()` produces must **equal**
  the set of records declared in that taxonomy's file.
  - Mutation: add a stray record. RED.
  - (Removing a kind's variant fails the build; say so.)
- **G-strict, every kind decodes typed.** For every kind, `decode_trusted_wire(error_edn())` must
  succeed as a typed aggregate of the declared class, **not** `ForeignRecord`. That makes the four
  decode sites typed for this taxonomy.
  - Mutation: remove one `wat_record_from!`. RED for that kind, and it must report which kind
    fell to foreign.
- **G-mirror, no golden moved.** `git diff --stat -- '*.edn'` is empty after the floor. State it.
- **The four sites, measured.** After S3 (and partially after each strike), count how many of the
  decode ladder's results are still foreign for startup diagnostics. Report the number per strike.
  After S3 it should be 0. If it is not, list what remains.

## Scope fence

- **IN:** the declarations, registration, and gates for the named strike.
- **OUT:**
  - any wire or golden change;
  - removing `causes`, the fabricated `<runtime>` span, and the four wrapper sites themselves
    (strike B, after S3);
  - `EvalError` (E);
  - `Frame` (D);
  - `provenance` (C).

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`**, including targeted runs.
- One cargo process at a time. Check with `pgrep -x cargo`, never `pgrep -f`.
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while a run is in flight.**
- **Never hand back while a build or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating it past the tool's 10-minute cap.
- The floor:
  - Run `scripts/floor.sh`.
  - The floor is 0 failed, 0 timed out.
  - On any red, surface it before any re-run: capture it verbatim and name the arm.
- Stage every path by name before the floor. Never use `git add -A`. No `cargo fmt`.
- Multi-site `.wat` edits go through the wat-fix codemod. Writing a NEW stdlib file is not a
  migration.
- New test files carrying wat-looking strings need their rune.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.
- Push to `origin/reason/little-wat-findings`.
- If your budget runs short, stop at a clean boundary and report where.

## Report (per strike)

- the re-measured kind counts;
- the nested sub-values declared;
- any untagged map (STOP list);
- each gate's mutation RED;
- `git diff --stat -- '*.edn'` (it must be empty);
- the foreign-count at the decode sites;
- the floor `Summary` line, verbatim;
- the SHA.
