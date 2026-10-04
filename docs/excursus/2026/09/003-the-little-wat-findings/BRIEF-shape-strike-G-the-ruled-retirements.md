# BRIEF — shape strike G: the ruled retirements and the four small cures

Excursus 003. This strike carries out the post-F items ruled in `AUDIT-the-shape-of-an-error.md`:
§ RULING 2026-10-03 (four decisions, each by the four questions) and § "Strike D3 landed" (the
retirement candidates). It builds on the audit commit that follows `ab769b878`.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Decide anything left open by the four questions
(Obvious? Simple? Honest? Good UX?, each a flat YES or NO) and show the answers. **Never present
options.**

**Land each item as its own commit on a green floor, in this order.** No item may be left half-built.

## Items

1. **Retire the 7 dead `RuntimeErrorKind`s:** `ParamShadowsBuiltin`, `NoEncodingCtx`,
   `NoSourceLoader`, `NoMacroRegistry`, `ChannelDisconnected`, `SandboxScopeLeak`, `AssertionFailed`.
   - D3 measured zero construction sites for each.
   - **Re-measure before deleting.** Grep is not enough: build with the variant removed and let the
     compiler list every reference.
   - For each one, delete the variant, its `Display` arm, its `to_record` arm, its wat record in
     `wat/runtime-errors.wat`, its registration, and its census row (D3's GD2a lists it as dead;
     the row goes).
   - `AssertionFailed` as a *runtime* kind: confirm that `:wat::runtime::AssertionFailed` the
     **record** is still used by assertion panics (strike A). If it is, keep the record and retire
     only the Rust variant.
2. **`LoadFetchError`'s variant is renamed.** Rename `LoadFetchError::Other` to `LoadOther`
   (`src/load/loader.rs`), so the derive covers it and the hand-written writer at `:~224` retires.
   - The wire tag is unchanged (`LoadFetchError.LoadOther`), so no golden moves. Assert that.
3. **`:wat::core::char` is declarable.**
   - Register it as a leaf. The Stone-Q census (`src/types.rs:~8571`) pins the hole so that this is
     a deliberate test edit: update that test.
   - The lex kinds' character fields move from one-character `String` back to `char`
     (`wat/lex-errors.wat`, and `crates/wat-reader`'s `char_to_edn_string` `via`).
   - EDN has a character literal: confirm the writer and the reader round-trip it.
   - Recapture any lex golden, and read the diff.
4. **Retire the provenance machinery.**
   - Strike C measured it dead end to end: producers write it, the one reader re-wraps it into
     nothing, and `provenance_to_edn` has no production caller.
   - Retire `Provenance`, the provenance half of `TrackedValue`, the producer stamps (literal eval,
     the `keyword`/`ast`/`edn`/`holon` intrinsics), the `wat_intrinsic` shim's provenance handling,
     and `Environment::lookup`'s re-wrap.
   - **Measure first:** does `TrackedValue` carry anything besides provenance? If it does, it stays,
     minus provenance. If it does not, it retires too.
   - Let the compiler list every reference.
5. **The startup-message type.** Strike B3 measured that a service's FIRST admin message is only ever
   `Init` or `Resume`, sent by the generated `start`/`resume`.
   - Give that first message its own type, admitting only `Init` and `Resume`, so that
     Stop-before-Init and Hibernate-before-Init have **no form**.
   - The `dispatch-admin` arms that panic on them (`wat/service.wat:~1240,~1242`) then cannot be
     written, and they go.
   - Measure which other owner-only protocol panics (B3 counted ~24) become unrepresentable the same
     way, and which remain. Report the census; do not redesign `Locus::launch` beyond the
     first-message type.

## Gates (each mutation-proven in RELEASE, one per item)

1. The census gate (D3) still accounts for every kind, now with 33 rows. Mutation: re-add a dead
   variant with no producer. The census must name it.
2. `LoadOther` round-trips via the derive. Mutation: drop the derive directive. RED.
3. A lex error's character field decodes typed as `:wat::core::char`. Mutation: retype it `String`.
   RED.
4. The build is the gate: nothing named `Provenance` remains. Also add a standing assertion that the
   snapshot and environment carry no provenance (one probe).
5. A first admin message of `Stop` does not type-check: a `.wat.bad` fixture beside its test.
   Mutation: widen the type. The fixture loads, and the test goes RED.

## Scope fence

- **IN:** items 1–5.
- **OUT:**
  - the two open questions for the builder: eval's missing check pass, and the 33 deleted scratch
    probes;
  - `Locus::launch`'s general redesign;
  - the stdlib-freeze excursus.

## Discipline

- **No forks or sub-agents.** Work serially.
- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- Iterate with targeted runs and `cargo wat` dry-runs. Run the full floor at each commit.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - **Never commit a red floor.**
- Multi-site `.wat` edits go through the wat-fix codemod, with a replay fixture.
- Stage by name BEFORE the floor. Never use `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings` after each item.
- If your budget runs short, stop after the last fully landed item and report where.

## Report

Per item:
- the measurement;
- the four-question answers where a decision was made;
- each gate's mutation RED;
- any golden that changed;
- the floor `Summary` line, verbatim;
- the SHA.
