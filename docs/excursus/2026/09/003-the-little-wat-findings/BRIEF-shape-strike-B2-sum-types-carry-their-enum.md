# BRIEF — shape strike B2: a sum type's variants carry their enum on the wire

Excursus 003. This strike works through the Strike B worklist in `AUDIT-the-shape-of-an-error.md`:
the S1, S2 and S3 rows and B1's `HashError` row. It builds on `06f70511e` (T3).

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

## Why

A wat sum type is a `defenum`, and its variant values decode from **dotted** tags
(`#ns/Enum.Variant`). Several Rust sum types that ride inside errors are written instead with
**flat** per-variant tags (`#wat.kernel/NotFnForm`). Wat therefore cannot declare them as one enum.
The sweep declared each variant as an unrelated record, and typed the fields that hold them as
`:wat::core::Value`, which means "anything". Since strike T, typed decode checks every field against
its declaration, so a `Value`-typed field is a field that has been opted out of that check.

| sum type | where | writer today | holder field (typed `Value` today) |
|---|---|---|---|
| `EnsureFnInvalidReason` (5 variants) | `src/check/error.rs:385` | `#[derive(ToEdn)]`, default namespace `wat.kernel` (it is a check diagnostic) | `CheckErrorKind::EnsureFnInvalid.reason` |
| `LoadFetchError` (3) | `src/load/loader.rs:192` | hand-written `ToEdn` (`:224`) | `LoadErrorKind::Fetch.cause` |
| `HashError` (8) | `src/hash.rs:471` | hand-written `ToEdn` (`:563`) | `LoadErrorKind::VerificationFailed.cause`; and `RuntimeErrorKind::EvalVerificationFailed.cause`, which holds an interim `Fault` (B1) |
| `ClauseFailureReason` (3) | `src/value/value.rs:513` | `error_edn()` writes flat; 3a's `to_record` retags dotted | `:wat::kernel::ClauseAttempt.failure-reason` |

There is also a lex finding (S3). A lex failure rides inside `ParseErrorKind::Lex.cause` as `Display`
prose, with no tag at all (`LexError`/`LexErrorKind`, `crates/wat-reader/src/lexer.rs:190,198`). The
offending character, the byte position and the kind are all discarded.

## Target

1. **The derive learns dotted tags.** Measured at `crates/wat-to-edn-derive/src/lib.rs`: the derive
   grammar is `namespace` / `key` / `via` / `skip` / `literal` and nothing else. Add one enum-level
   directive that emits `#<ns>/<Enum>.<Variant>` for every variant (for example
   `#[to_edn(namespace = …, qualified)]`). Choose the smallest honest spelling, and name it in your
   report.
   - Do NOT hand-write four writers. The derive is the one place the shape lives.
   - If a type cannot use the derive (a hand-written writer exists for a reason), say why and write
     its dotted tag through the same helper the derive emits.
2. **Each sum type becomes one `defenum` in wat.**
   - Replace the sweep's flat records with one `defenum` per type. Register it with
     `wat_enum_register_from!`. Field types follow each variant's payload.
   - Each holder field is then typed as the enum, not `:wat::core::Value`.
   - Removing the old flat records from the stdlib `.wat` files is a multi-site structural edit:
     use the wat-fix codemod, with a replay fixture.
   - `EnsureFnInvalidReason` moves to a check namespace (`wat.check`) rather than inheriting
     `wat.kernel` by default.
3. **`HashError` is an error.** It gains the floor: `message`, plus `location` as the site that
   verified the hash. Find where that span is in hand, and say how it gets there.
   - It satisfies `:wat::core::Error`, so `RuntimeErrorKind::EvalVerificationFailed.cause` holds the
     real `HashError`, not B1's interim `Fault`. Update `to_record` and `single_cause_fault`'s
     comment.
   - Measure whether `LoadFetchError` is likewise an error. If it is, give it the floor too. If it
     is data (a reason code), it is just an enum. Say which.
4. **The lex error is tagged.**
   - `LexError` gets a real tagged wire form inside `crates/wat-reader`, the same way `ParseError`
     has one. Mirror whatever two-layer arrangement `ParseError` uses (the reader crate's `ToEdn`,
     plus `wat`'s `WatError`).
   - `ParseErrorKind::Lex.cause` carries it, not prose.
   - Declare `:wat::lex::*` (10 kinds), and retire S3's `g_lex_never_produces_a_tag` gate. It
     asserted the absence this strike cures; replace it with a G-list/G-strict pair like the other
     taxonomies.
5. **`ClauseFailureReason`'s two writers agree.**
   - `error_edn()` writes dotted, the same as `to_record`, so the 3a gap noted in
     `wat/kernel/diagnostics.wat` (the comment beside `ClauseFailureReason`) closes.
   - Delete that comment's caveat.

## Gates (each mutation-proven in RELEASE)

- **GB2a, every sum value is dotted.** For each of the four types, plus the lex kinds, drive every
  variant's wire form and assert the tag is `#<ns>/<Enum>.<Variant>` and decodes typed **as the
  enum**. Since T, a wrongly-shaped value is refused, so this is a real shape proof.
  - Mutation: drop the directive from one type. RED for that type.
  - **One mutation per type.** A multi-arm gate needs one mutation per arm.
- **GB2b, holder fields are typed.** No field in any stdlib error declaration is typed
  `:wat::core::Value` unless its consumer genuinely accepts any value. List every survivor and why.
  - Mutation: retype one holder back to `Value`, then hand it the wrong enum's value. It must be
    refused once the field is typed (show both sides).
- **GB2c, `HashError` is an Error.** `EvalVerificationFailed.cause` decodes typed as a `HashError`
  record that satisfies `:wat::core::Error`.
  - Mutation: drop its floor. RED.
- **GB2d, lex is structure.** `ParseErrorKind::Lex.cause` decodes typed, and carries the character,
  position and kind.
  - Mutation: restore the `to_string()`. RED.
- Every earlier gate (3a, S1–S3, T/T2/T3, B1) stays green. Update the sweep's G-list exceptions as
  the flat records become enums; do not weaken what they check.

## Goldens

Recapture with `UPDATE_EDN=1`, in the foreground, and **read every diff**. Allowed changes:
- flat sum tags become dotted;
- `EnsureFnInvalidReason`'s namespace moves to `wat.check`;
- `HashError` and `LoadFetchError` (if it is an error) gain the floor;
- `EvalVerificationFailed.cause` becomes a `HashError`;
- lex prose becomes a tagged lex record;
- `ClauseFailureReason`'s `error_edn` becomes dotted.

Report anything else.

## Scope fence

- **IN:** items 1–5 and the gates.
- **OUT (strike B3):**
  - `fault_value`'s synthesized `<runtime>` location;
  - `MacroExpansionFailed.cause`'s interim `Fault` (it needs a type registry in `to_record`);
  - `read-json`/`read-foreign` failures being stringified upstream;
  - `UPDATE_EDN` writing raw Rust line numbers while comparison normalizes them (T3's golden
    churn);
  - the serve-loop codegen's unreachable `assertion-failed!` arm (T3).
- **OUT:** C, D, E, F; the stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build, recapture or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - On a timeout: surface it with its history; do not widen `.config/nextest.toml`.
  - **Never commit a red floor.**
  - **"Unrelated to my change" is not a disposition**: find the mechanism, or surface it.
- Stage by name BEFORE the floor. Never `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- **If your budget runs short, stop at a clean boundary and report where.** "The derive directive
  and two of the four types, on a green floor" is a valid boundary.

## Report

- the directive's spelling;
- per type: the decision (error or data), its namespace, and the codemod;
- the lex wire form;
- the `Value`-typed survivors and why;
- each gate's mutation RED, per type;
- any golden that changed beyond the allowed list;
- the floor `Summary` line, verbatim;
- the SHA(s).
