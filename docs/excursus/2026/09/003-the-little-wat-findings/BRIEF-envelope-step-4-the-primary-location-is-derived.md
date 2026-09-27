# BRIEF — envelope step 4 (D4): the primary `:location` is the innermost line of the user's program

Excursus 003. Design: `DESIGN-the-error-envelope-and-its-frames.md` § D4 (RULED). This builds on step
3c (`6489b23a8`). It retires the-little-wat's **C-114** location complaint (its `FINDINGS.md`,
§ C-114): an `i64` overflow in `(:wat::core::+ …)` "locates at `wat/core.wat:66` rather than at the
line that overflowed."

**Read `wat-rs/CLAUDE.md` in full first.** Every count and citation below is a claim; verify before
relying on it. STOP on a contradiction. Where the call is small and the intent is clear, make it and
name it.

## What is true today (driven at `6489b23a8`, not read)

A user program whose `:user::grow` does `(:wat::core::+ n 9223372036854775807)` at its line 3, col 3
dies with:

```
:error #wat.runtime/IntegerOverflow {… :location #wat.core/Span {:file "wat/core.wat" :line 66 :col 62 …}}
:frames [ {:symbol ":wat::core::+" :span {:file "<user file>" :line 3 :col 3 …} :kind Wat}
          {:symbol ":user::main"   :span {:file "src/freeze.rs" :line 1591 …} :kind Wat}
          {:symbol "<rust>"        :span {:file "src/numeric/arith.rs" :line 93 …} :kind Rust} ]
```

The user's line is already in the frames: frame 0's call site is where the user called `+`. The
primary `:location` names the stdlib line instead.

Three facts from that output shape the design:
1. **A `:Wat` frame can sit on a non-wat file.** `:user::main`'s call site is `src/freeze.rs`, the
   Rust site that invokes main. So "user source" cannot mean "not stdlib". It must be a **positive**
   fact: a file the loader read with user privilege.
2. **The raise site must not vanish.** If `:location` moves to the user's line, `wat/core.wat:66` is
   still true and still useful ("the stdlib and Rust frames are still there, one level down", D4).
3. **Tail calls.** `:user::grow` is absent: `+` is in tail position, so its frame replaced grow's.
   D4 still lands on the right line here, because the call site survives the replacement. The elision
   itself is OUT of scope; it is a separate, open question for the builder.

## What to build

1. **Record which files are user source, from privilege, never from a path prefix.**
   `Privilege` (`src/resolve/registration.rs:28`) is recorded per registered NAME today, not per file.
   - Find where the loader reads each file: the stdlib list in `src/load/stdlib.rs`, the entry
     program, `load-file!`, `read-string`/eval sources, and the REPL.
   - Record the set of files read with **user** privilege, somewhere a `RuntimeError` can consult at
     construction (the frozen world, or whatever the frames capture already reaches).
   - Say where you put it and why that is the source of truth.
   - **Measure:** can a user program be loaded from a path string that equals a stdlib file's path
     (e.g. a user file at relative path `wat/core.wat`)? If the two can collide, a path-string set is
     ambiguous; STOP and report. If the loader refuses it, cite the refusal.
2. **Derive the location at construction** (`RuntimeError::new`, `src/value/signal.rs`, where step 2
   captures frames):
   - If the raise span is in user source, **keep it** and add nothing.
   - Otherwise the primary `:location` is the call-site span of the **innermost `:Wat` frame whose file
     is user source**. The raise span becomes the **innermost frame**, with `kind :Wat` when it is in
     wat source and `symbol` = the op or function that raised, so it survives exactly once.
   - If no frame is in user source (a stdlib error with no user caller, or a startup path), keep the
     raise span. Name which producers reach that arm.
   - **Invariant:** the raise span appears **exactly once** across `:location` ∪ `:frames`.
3. **Assertion panics.** `Failure`s built from an `AssertionPayload` carry frames too.
   - **Measure** where an `assert-eq` from `wat/test.wat` (or any assertion helper in stdlib) locates
     today.
   - If it locates inside the stdlib helper, apply the same derivation. If it already locates at the
     user's line, say so and leave it.
4. **The wire and every consumer follow automatically:** `Failure.error.location`, `Failure/location`,
   and the `LociDiedError/message` headline. Confirm that nothing re-derives the location downstream a
   second way.

## Gates (each mutation-proven in RELEASE)

- **G1, C-114 is retired.**
  - A fixture that overflows via `(:wat::core::+ …)` from a user function. Assert:
    - `:location` is the fixture's own file and line;
    - the innermost frame is the raise site in `wat/core.wat`;
    - the Rust frame is still present.
  - Mutation: disable the derivation. G1 must go RED.
- **G2, a user-raised error is untouched.** Integer division by zero written directly in user code:
  `:location` is the raise span, and no raise frame is added (the exactly-once invariant).
  - Mutation: always add the raise frame. G2 must go RED.
- **G3, privilege, not prefix.** A user program whose path string begins with `wat/`, e.g. run from a
  temp dir holding `wat/foo.wat`, with the overflow from G1. Its `:location` must be its own line.
  - Mutation: replace the privilege lookup with `file.starts_with("wat/")`. G3 must go RED.
  - If step 1 found that such a path collides with a stdlib path, this gate is where it shows. STOP
    there rather than build around it.
- **G4, no user frame.** Drive one real producer of the "no user frame" arm, if one exists, and
  assert that the location is kept. If none exists, say so; do not hand-build one.

## Goldens

Recapture with `UPDATE_EDN=1`. **Read every diff.** Report any change beyond two shapes:
- `:location` moving to a user line;
- the raise frame appearing innermost.

C-114's own wording in the-little-wat is not ours to edit. Report the before/after for its repro.

## Scope fence

- **IN:** the user-source record, the derivation, assertion panics (if measured to need it), G1–G4,
  and the recapture.
- **OUT:**
  - tail-call frame elision;
  - the `"<rust>"` symbol convention;
  - `EntryFormFailure`/`BadReturn` retirement;
  - the raw `RuntimeError` writer's retirement;
  - check-time errors (no frames).

## Discipline

- **One cargo process at a time.** Check with `pgrep -x cargo`, never `pgrep -f`.
- Use `cargo nextest run --release`, never `cargo test`.
- The floor:
  - Run it in the foreground, or `nohup` plus a foreground
    `until grep -qE '^ *Summary' <file>; do sleep 30; done`.
  - Never end your turn while a build or floor runs.
  - The floor is **0 failed, 0 timed out**. On any red, **surface it before any re-run**: capture it
    verbatim and name the arm.
- Stage every path by name before the floor. Never use `git add -A`.
- No `cargo fmt`.
- Multi-site `.wat` edits go through the wat-fix codemod.
- New test files carrying wat-looking strings need their rune.
- A deliberately failing measurement `.wat` goes in the session scratchpad, not `wat-scripts/`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.
- Push to `origin/reason/little-wat-findings`.
- If your budget runs short, stop at a clean boundary and report where.

## Report

- where the user-source record lives, and why;
- the collision measurement;
- the assertion-panic measurement;
- each gate and its mutation RED;
- the "no user frame" producers;
- any golden that changed beyond the two shapes;
- C-114's repro, before and after;
- the floor `Summary` line, verbatim;
- the SHA(s).
