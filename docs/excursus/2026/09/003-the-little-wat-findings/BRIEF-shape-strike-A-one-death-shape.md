# BRIEF — shape strike A: one death shape, and a `Failure` that carries only what it means

Excursus 003. Design: `AUDIT-the-shape-of-an-error.md`: F1, F2, F7, and its § RULING 2026-09-27
(removal-biased: *"we'll add stuff back in later if we choose to"*). This builds on `c144178e5`.

**Read `wat-rs/CLAUDE.md` in full first.** Every count and citation is a claim; verify before relying
on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it and name
it.

## Target

1. **One death shape (F1).** A peer that dies from an unhandled assertion reports
   `[#wat.kernel/LociDiedError.Panic {:failure #wat.kernel/Failure {…}}]`, the same chain every other
   death uses. `#wat.kernel/AssertionFailure` is retired: its declaration
   (`wat/kernel/diagnostics.wat`), its registration (`src/types.rs`, near `:2427`), and its
   hand-built writer (`src/panic_hook.rs`: `payload_to_edn`, `assertion_failure_envelope`,
   `write_assertion_failure`). The payload's `thread` and `upstream-chain` go with it (removal-biased;
   0 wat readers).
2. **`Failure` is `{error frames frames-elided}` (F2).** `actual`/`expected` leave `Failure`.
   - An assertion's `Failure.error` becomes the assertion's own record, carrying `actual`/`expected`.
     The natural home is the existing `:wat::runtime::AssertionFailed` record (step 3a: floor plus
     `actual`/`expected`); an assertion is the same concept whichever path raised it. If that record
     does not fit the panic path, say why before minting another.
   - `:wat::kernel::Failure/actual` and `/expected` stay as **derived** accessors, the way
     `Failure/message` is derived today. They read `actual`/`expected` off `error` when it is an
     `AssertionFailed`, and return `None` otherwise. Their 7 wat call sites need no edit:
     `tests/comms/probe_arc209_structured_peer_death.wat:28,31`,
     `tests/comms/wat_arc113_cross_fork_cascade.wat:37,40`, `wat-tests/test.wat:98,99,157`.
3. **Retire the dead (F7).**
   - `:wat::kernel::StartupError` (the `{message}` defstruct at `wat/kernel/diagnostics.wat:107`; its
     registration is at `src/types.rs:3071`). Constructed nowhere, read nowhere. Verify both before
     deleting.
   - `LociDiedError.EntryFormFailure`: no producer anywhere (measured in 3b).
   - `LociDiedError.BadReturn`. Its producer is a **runtime guard**: `src/process/verbs.rs:~254` and
     `:~321` fire when `:user::main` returns non-nil. The type checker refuses that program first, so
     the guard never fires, but **a guard that never fires is not dead code**. Keep it, and route it
     to `LociDiedError.Panic`, whose `Failure.error` is a `Fault` saying `:user::main` returned
     `<type>`, not nil. Reaching it means an internal invariant broke, which is a panic's meaning.
     Delete `process_died_error_bad_return*` once nothing calls them.

## Measure first, and report

- **Every path by which an assertion panic reaches stderr or a parent:**
  - the main thread of a top-level `wat` run (the hook, `src/panic_hook.rs:~90`);
  - a fork/`spawn-program` child;
  - a thread peer.

  After this strike each must emit **exactly one** death line, in the chain shape. Say how you
  proved "exactly one": no hook line plus envelope line, and no empty stderr.
- **`AssertionPayload.location` is an `Option<Span>`, and the floor's `location` is mandatory.** When
  is it `None`? Derive it the D4 way (the innermost user frame). If there is none, use the raise
  site. Name what happens in the no-frame case.
- `message_only_failure` (Rust, and `:wat::kernel::message-only-failure` in `wat/spawn.wat`) and
  `failure_value_from_assertion_payload` build `Failure` with 5 fields; update both.
- The doc of `assertion_failure_envelope` says it is used by a crash-send site in `kernel/spawn.rs`.
  Grep found no such caller. Confirm, and fix any stale doc you touch.

## Gates (each mutation-proven in RELEASE)

- **GA1, one death shape.** An unhandled `assertion-failed!` in `:user::main`, driven through the
  binary (EDN-over-stdio), and the same through a thread peer. stderr parses as a vector of
  `LociDiedError`s, the head is `Panic`, its `failure.error` is the assertion record with `actual`
  and `expected` set, and there is no `AssertionFailure` tag anywhere.
  - Mutation: restore the hook's old render. GA1 must go RED.
- **GA2, the derived accessors.** `Failure/actual` on an assertion death returns `Some`, and on a
  runtime-error death returns `None`.
  - Mutation: make the accessor read a stored field that no longer exists, or always return `None`.
    GA2 must go RED.
- **GA3, the guard kept.** Measure whether the BadReturn guard can be reached. If it can be driven,
  drive it and assert `Panic`. If it provably cannot, say so, and do not fake a test for it.

## Goldens

Recapture with `UPDATE_EDN=1`, and **read every diff**. The expected shapes:
- `AssertionFailure` → `LociDiedError.Panic`;
- `:actual`/`:expected` leave `Failure` (they appear on assertion error records only).

Report anything else.

## Scope fence

- **IN:** F1, F2, F7 as above.
- **OUT:**
  - the floor's `causes` (strike B);
  - `provenance` (C);
  - `Frame` (D);
  - `EvalError` (E);
  - the domain `Fault`s (F);
  - tail calls.

## Discipline

- **One cargo process at a time.** Check with `pgrep -x cargo`, never `pgrep -f`.
- Use `cargo nextest run --release`, never `cargo test`.
- The floor:
  - Run it in the foreground, or `nohup` plus a foreground
    `until grep -qE '^ *Summary' <file>; do sleep 30; done`.
  - Never hand back while a build or floor runs; **wait for it**.
  - The floor is **0 failed, 0 timed out**. On any red, **surface it before any re-run**: capture it
    verbatim and name the arm.
- Stage every path by name before the floor. Never use `git add -A`.
- No `cargo fmt`.
- Multi-site `.wat` edits go through the wat-fix codemod.
- New test files carrying wat-looking strings need their rune.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.
- Push to `origin/reason/little-wat-findings`.
- If your budget runs short, stop at a clean boundary and report where.

## Report

- the assertion-to-stderr paths, and your proof of exactly one line each;
- the `location: None` answer;
- the BadReturn guard's reachability;
- each gate's mutation RED;
- any golden that changed beyond the named shapes;
- the floor `Summary` line, verbatim;
- the SHA.
