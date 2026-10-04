# BRIEF — shape strike F: a record named `Fault`/`Failure` is what its name claims

Excursus 003. Design: `AUDIT-the-shape-of-an-error.md` F8, and its § RULING 2026-09-27 item 5 ("the
domain `Fault`s are resolved: conform or rename"). It builds on `342474631`.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Decide every per-type question by the four questions
(Obvious? Simple? Honest? Good UX?, each a flat YES or NO) and show the answers. **Never present
options.**

## Why

Four records reuse the floor's names, but they do not satisfy `:wat::core::Error` (`{message location}`):

| record | fields | wat mentions (outside scratch-pad) | Rust |
|---|---|---|---|
| `:wat::cache::Fault` (`wat/cache.wat:~106`) | `code diagnostic message` | 35 | 2 |
| `:wat::query::Fault` (`wat/query.wat:~83`) | `message` | 37 | 0 |
| `:wat::sqlite::Fault` (`wat/sqlite.wat:~46`) | `op code diagnostic message` | 7 | 2 |
| `:wat::doctest::Failure` (`wat/doctest.wat:~74`) | `fqdn reason` | 17 | 0 |

A record named `Fault` claims to be an error. One that cannot be a cause, a `Failure.error`, or
anything an error consumer accepts is telling a lie with its name. Re-measure these counts before
relying on them.

## Target: one decision per record, by the four questions

For each record, measure **what it is** and act on that:

- **It is an error**: the failure of an operation, carried in a `Result`'s `Err`, which a caller
  handles or propagates.
  - It **conforms**: it gains `location`, satisfying the `:wat::core::Error` surface.
  - Find where the span is in hand at each construction. For a Rust-built fault that is the Rust raise
    site; for a wat-built one, the wat call.
  - Its domain fields (`code`, `diagnostic`, `op`) stay. They are the kind-specific data, exactly as
    the runtime records carry `op`, `a` and `b`.
- **It is not an error**: a domain outcome or report, such as a doctest's per-test verdict.
  - It is **renamed**, so its name stops claiming to be an error. For example, `doctest::Failure`
    becomes something that says it is a doctest result.
  - Choose the name by the four questions. Renaming its call sites across the `.wat` corpus is a
    multi-site rename, so use a wat-fix codemod with a dry-run diff and a replay fixture.

Measure before deciding. Read each record's construction sites and its consumers, to see whether it
travels as an error or as data.

## Gates (each mutation-proven in RELEASE)

- **GF1, every conforming domain fault IS an Error.** For each record you conform, drive one real
  producer. Its value satisfies `:wat::core::Error`: it decodes typed and passes as a `Failure.error`.
  - Mutation: drop `location` from one record. RED.
- **GF2, no record named `Fault`/`Failure` fails the surface.** A test that walks the registered
  types: every record whose name ends in `::Fault` or `::Failure` either satisfies `:wat::core::Error`
  or is `:wat::kernel::Failure` (the envelope), and nothing else. It must be RED on today's tree;
  report the anchor.
  - Mutation: re-add a non-conforming `Fault`. RED.

## Goldens

Recapture with `UPDATE_EDN=1` and **read every diff**. The allowed changes are: conformed faults gain
`location`; renamed records carry their new tag. Report anything else.

## Scope fence

- **IN:** the four records, their consumers, and GF1/GF2.
- **OUT:**
  - the post-F strikes:
    - retiring the 7 dead runtime-error kinds;
    - retiring the provenance machinery;
    - the startup-message type;
    - declarable `char`;
    - the `LoadOther` rename;
  - the open questions (eval's missing check pass, the 33 deleted scratch probes);
  - the stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **No forks or sub-agents.** A fork inherits "you are the executor" and races you on the checkout.
  Work serially.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- Iterate with targeted runs. Run the full floor at the commit.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - **Never commit a red floor.**
- Stage by name BEFORE the floor. Never use `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- If your budget runs short, stop after the last fully landed record and report where.

## Report

- per record: what it is (with evidence), the decision and its four answers, and the codemod if any;
- GF2's anchor;
- each gate's mutation RED;
- any golden that changed beyond the allowed changes;
- the floor `Summary` line, verbatim;
- the SHA(s).
