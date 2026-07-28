# BRIEF — `Stopped`, not `Shutdown` (the intueri rename, arc 170)

## The work

Commit `3e297846` shipped five names flagged PROVISIONAL, pending an intueri cast. The cast
has ruled. This brief applies the verdict and deletes the provisional notes so they cannot
become the next stale lie.

Two rulings, one gated on a probe.

## RULING A — `Shutdown` → `Stopped`, at the wat-visible layers ONLY

wat already has a word for this fact and it is not "shutdown": `(:wat::kernel::stopped?)`
(registered `src/check.rs:16594`) is the primitive every wat program that cares about SIGTERM
already calls, and it means exactly *"has a stop been requested?"*. A second word for one fact
in one audience's vocabulary is the synonym anti-pattern.

It is also more honest. Nothing is shutting down when this variant is produced — a stop has been
*requested*, and the program decides. The fixture says so itself
(`tests/cli/wat_cli__sigterm_blocked_on_stdin.wat`: *"SIGTERM is a flag the program observes, not
a kill"*), and the human-facing message already written at `stdio-primes.wat:270` says
*"a stop was requested while blocked reading stdin"* — not "shutdown". The prose was already
right; the variant name was not.

Rename **only** these three:
- `:wat::io::…::ReadFrameOutcome::Shutdown` → `::Stopped` (`src/types.rs`)
- `:wat::kernel::ReadFrameOutcome::Shutdown` → `::Stopped` (`src/types.rs`)
- `:wat::kernel::StdIn::ReadLineResponse::Shutdown` → `::Stopped` (`stdio-primes.wat`)

plus every wat consumer: `stdio-primes.wat`, `wat-scripts/demos/repl/repl.wat`,
`tests/cli/wat_cli__sigterm_blocked_on_stdin.wat`, `tests/services/probe_arc170_stdio_prime.wat`.

**The Rust side KEEPS `Shutdown`** — `FramedRead::Shutdown`, the `NextLine`/`LineRead` variant,
`RecvError::Shutdown`, `LineResult::Shutdown`, `trigger_shutdown`, `SHUTDOWN_BROADCAST_READ_FD`.
Rust's vocabulary is uniformly `shutdown` and `FramedRead::Shutdown` deliberately mirrors
`channel/transfer.rs`'s `LineResult::Shutdown`, which is the file the poll was copied from.
**The rename boundary IS the Rust/wat boundary — that is where the audience changes, so it is
where the vocabulary is allowed to change.** That translation happens at ONE site in
`src/io.rs` (where the `FramedRead` variant becomes the wat outcome); it needs a WHY comment
saying so, or the next reader reads the mismatch as sloppiness.

## RULING B — owner-qualify the duplicated type name ⚠ GATED ON A PROBE

`3e297846` introduced the **only hand-written duplicate base name in the wat type vocabulary**:
`:wat::io::ReadFrameOutcome` and `:wat::kernel::ReadFrameOutcome` are *structurally identical*
(`Frame [text] | Eof | Shutdown`, same purity, same field name). At a match arm a reader sees two
30-character keywords differing in one buried middle segment, with byte-identical variant sets.
Nothing at the point of use tells them apart.

Ruled: `:wat::kernel::ReadFrameOutcome` **keeps** the short name (its verb is
`:wat::kernel::read-frame`; verb and outcome agree, and it is the surface wat programmers meet),
and the plumbing one becomes **`:wat::io::IOReader::ReadFrameOutcome`** — owner-qualified, the
same shape as the existing `:wat::kernel::StdIn::ReadLineResponse`.

**⚠ PROBE THIS FIRST, BEFORE ANY RENAME WORK.** Every Rust-registered builtin in `src/types.rs`
currently uses a THREE-segment path; there are zero four-segment ones. The mechanism is proven
for wat-*declared* types (`StdIn::ReadLineResponse::Shutdown` resolves fine) but NOT for
`register_builtin` + variant-keyword resolution. Register a throwaway four-segment builtin enum,
build, and construct/match one of its variants from a `.wat` fixture.

- Probe passes → apply Ruling B.
- **Probe fails → STOP. Apply Ruling A only, report the exact error, and leave the duplicate
  name in place.** Do NOT invent a different name to route around it; the cast ruled this shape
  and a substitute would be an unruled name.

## Also in scope — the provisional notes and three stale comments

**Delete all six PROVISIONAL notes** once the names land (`src/io.rs`, `src/types.rs` ×2,
`stdio-primes.wat` ×3). Left behind they become exactly the stale-comment lie this arc keeps
finding.

**Three comments are now Level-1 lies** — each inside a block `3e297846` edited:
1. `src/types.rs:1015` — `// Two variants, and only two:` sits ~40 lines above a THIRD variant.
2. `src/check.rs:17293` — `// this one always answers the same two-variant outcome` — it has three.
3. `src/services/verbs.rs:224` — a doc comment opening with the word *honest* that omits the
   outcome the commit exists to add.

## OUT of scope — named so it is not smuggled in

- `:wat::kernel::LociDiedError::Shutdown` → `Stopped`. Ruled as **debt**, ~8 wat sites, its own
  wat-fix codemod. Do not touch it here.
- `StdIn::ReadLineResponse`'s `read-line`/`:Line [line]` naming (a frame can span several
  physical lines, so `Line` mumbles). Real, larger, its own stone.
- `as_raw_fd_for_poll` on `WatWriter` having no poll caller. Real, separate.

## Blast radius

`src/types.rs` · `src/check.rs` · `src/io.rs` · `src/edn_shim.rs` (the `NextLine` → `LineRead`
type rename, ruled: it names the request rather than the answer, and `LineRead`/`FramedRead`
pair exactly) · `src/services/verbs.rs` (comment only) · `wat/kernel/services/stdio-primes.wat` ·
`wat-scripts/demos/repl/repl.wat` · `tests/cli/wat_cli__sigterm_blocked_on_stdin.wat` ·
`tests/services/probe_arc170_stdio_prime.wat` · `tests/comms/probe_ioreader_read_frame.{rs,wat}`.
No new files. Nothing under `src/kernel/`. No behaviour change whatsoever — this is a rename.

## STOP triggers — ship nothing and report

- **STOP-1.** The four-segment probe fails → Ruling A only, as above.
- **STOP-2.** Any rename requires a behaviour change to compile → STOP. This is a pure rename;
  if it isn't, the premise is wrong and I want to know before it ships.
- **STOP-3.** A wat consumer of these variants exists outside the blast-radius list → STOP and
  name it. Enumerate consumers by grep across `*.wat` AND `*.wat.bad`/`.disabled`/`.expr`/
  `.intueri` — a single `*.wat` glob has silently missed 243 files in this repo before.

## Gate

`cargo build --release --all-targets` — clean, zero warnings. The orchestrator weighs the floor
centrally.

## Definition of done

Build clean, and a report naming: whether the four-segment probe passed, every renamed site,
the six deleted PROVISIONAL notes, the three fixed comments, and any STOP hit.
