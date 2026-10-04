# BRIEF — shape strike F2: one location rule, for raised and returned errors alike

Excursus 003. This strike cures the finding in `AUDIT-the-shape-of-an-error.md`, § "Strike F landed".
It builds on `1fd6a13d0`.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Decide anything the brief leaves open by the four
questions (Obvious? Simple? Honest? Good UX?, each a flat YES or NO) and show the answers. **Never
present options.**

## Why

Strike F gave `cache::Fault`, `sqlite::Fault` and `query::Fault` a `location`, minted by
`(:wat::kernel::here)` **inside the stdlib**: `wat/cache.wat`, `wat/sqlite.wat`, `wat/query.wat`,
`wat/query/sqlite-store.wat` and `wat/telemetry/*`. A user's `(:wat::cache::Lru/new … 0)` therefore
returns a fault located at a line of `wat/cache.wat`. That is C-114's defect again, this time in a
**returned** value.

Step 4 (D4) cured it for **raised** errors only. `derive_primary_location_and_frames`
(`src/value/signal.rs:~239`) takes the innermost frame in user source, using the per-file
user-source record and the stdlib-label wall.

## Target

1. **Expose D4's derivation to wat, as one primitive.**
   - It answers "the location an error minted here should carry": the current span if it is in user
     source, otherwise the innermost `CALL_STACK` frame whose location is in user source.
   - Use the **same** code path `RuntimeError::new` uses. Factor it if needed. Do not write a second
     derivation.
   - Name it so it reads as what it is, beside `:wat::kernel::here` and `:wat::kernel::call-site`
     (`src/intrinsic/kernel/source.rs:~76,~114`), and give it their purity and totality annotations.
   - **When no frame is in user source** (for example a fault minted while the stdlib runs on its own
     behalf), it returns what D4 returns in that case: the raise span. Say so in its doc.
2. **Every stdlib fault mint site uses it** in place of `(:wat::kernel::here)`.
   - Strike F's mint sites span many files, so this is a multi-site structural edit: use a wat-fix
     codemod with a dry-run diff and a replay fixture.
   - `lift-fault` in `wat/query/sqlite-store.wat` propagates the originating `sqlite::Fault`'s own
     location. It already carries a user-derived location once the origin is cured, so keep the
     propagation.
3. **Measure whether `here` has any remaining error-location use.** Any other stdlib site that puts
   `here` into an error's `location` gets the same treatment. User code calling `here` is unaffected:
   in user source, `here` is already the right answer.
4. **`Fault/of`.** Measure what it captures today (`wat/core.wat:~2194`, which splices `here` into the
   caller's expansion). When it is called from user code it is correct as is. When it is called from
   stdlib it has the same defect. Decide by the four questions whether it uses the primitive.

## Gates (each mutation-proven in RELEASE)

- **GF2a, a returned fault locates at the user's line.** For each of `cache`, `sqlite` and `query`:
  a user fixture's failing call yields a fault whose `location` is the fixture's own file and line.
  This replaces the claim in GF1's fixture comment, so update GF1.
  - Mutation: restore `here` at one mint site. RED for that record.
- **GF2b, one derivation.** Raised and returned errors agree. A user call that raises (D4's path)
  and a user call that returns a fault, from the same line, report the same location.
  - Mutation: give the primitive its own copy of the rule, made subtly different. RED.

## Goldens

Recapture with `UPDATE_EDN=1` and **read every diff**. The only allowed change is a stdlib-minted
fault's `location` moving to the user's line. Report anything else.

## Scope fence

- **IN:** items 1–4 and the gates.
- **OUT:**
  - the post-F strikes:
    - retiring the 7 dead kinds;
    - retiring the provenance machinery;
    - the startup-message type;
    - declarable `char`;
    - `LoadOther`;
  - the open questions;
  - the stdlib-freeze excursus.

## Discipline

- **No forks or sub-agents.** Work serially.
- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- Iterate with targeted runs and `cargo wat` dry-runs. Run the full floor at the commit.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - **Never commit a red floor.**
- Stage by name BEFORE the floor. Never use `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- If your budget runs short, stop at a clean boundary and report where.

## Report

- the primitive's name, and its shared path with `RuntimeError::new`;
- the mint-site census (codemodded or kept, with the reason for each);
- the `Fault/of` decision and its four answers;
- each gate's mutation RED;
- any golden that changed beyond the allowed change;
- the floor `Summary` line, verbatim;
- the SHA.
