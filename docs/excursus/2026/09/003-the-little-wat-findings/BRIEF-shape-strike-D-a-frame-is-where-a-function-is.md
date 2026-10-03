# BRIEF — shape strike D: a frame is a function and where it is, innermost first, tail calls named

Excursus 003. Design: `AUDIT-the-shape-of-an-error.md` F6, its § RULING 2026-09-27 item 2, and its
§ RULING 2026-10-03 item 1 (tail calls). This builds on `22f97d1db`.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

## Why: today's frames mislead, driven at C-114

The C-114 repro is `:user::grow` doing `(:wat::core::+ n MAX)` at line 3, called from `:user::main`
at line 6. Its frames today:

```
{:symbol ":wat::i64::+"  :span wat/core.wat:66   :kind Wat}   ← D4's synthesized raise frame
{:symbol ":wat::core::+" :span c114.wat:3        :kind Wat}
{:symbol ":user::main"   :span src/freeze.rs:1646 :kind Wat}  ← a Rust span on a wat frame
{:symbol "<rust>"        :span src/numeric/arith.rs:93 :kind Rust}
```

- **Wrong pairing.** Each frame pairs (*callee*, *where it was called from*). Clojure and Java pair
  (*function*, *where inside it execution is*). Read the conventional way, the trace says `main`
  called `+` at line 3, but line 3 is inside `grow`.
- **`grow` is missing.** `+` is in tail position, so `replace_top_frame` (`src/value/frame.rs:64`)
  overwrote its frame.
- **A synthesized entry.** D4's raise frame names an intrinsic (`:wat::i64::+`) that never had a
  frame.
- **A Rust span on a wat frame.** `:user::main`'s span is the Rust site that invokes main.
- **The Rust frame is in the wrong place.** It is listed outermost (D3's "user first") but is the
  innermost activation.
- **`kind` is derivable.** Wat spans carry an `end` and Rust spans don't (D1, verified in step 4),
  and `kind` already disagrees with the span on `main`'s frame.
- **`<rust>` is a placeholder** where a function name belongs.

## Target

1. **`Frame` is `{fn at}`**, where `at` is the location **inside** `fn` where execution is (or where
   it made the next call). `kind` goes. `fn` is mandatory: no placeholder.
2. **Innermost first, in true order.** The innermost frame is the activation that raised:
   - for an intrinsic raise, `{fn <the intrinsic's wat name> at <the Rust raise site>}`, e.g.
     `{fn ":wat::i64::+" at src/numeric/arith.rs:93}`;
   - then `{fn ":wat::core::+" at wat/core.wat:66}`, then the user frames.
   - D4's synthesized raise frame is gone: the raise site is the innermost frame's `at`.
   - The Rust call site of `:user::main` disappears, because it is the `at` of no wat function.
3. **Tail calls are named, not lost (the ruling).** When `replace_top_frame` overwrites a frame, the
   surviving `FrameInfo` keeps, at O(1) cost:
   - the **entry call site**: the original call span in the real (non-tail) caller, kept from the
     first frame of the tail chain;
   - the **last tail caller**: the replaced callee whose body contains the surviving call span, so
     it owns that location;
   - a **count** of the frames the chain collapsed.

   For C-114 the trace must read, innermost first:
   ```
   {fn ":wat::i64::+"   at src/numeric/arith.rs:93}
   {fn ":wat::core::+"  at wat/core.wat:66}
   {fn ":user::grow"    at c114.wat:3   …tail…}     ← reconstructed from the last tail caller
   {fn ":user::main"    at c114.wat:6}              ← from the entry call site
   ```
   Choose the wire spelling for "tail-collapsed" (for example a `tail-elided <- i64` on the frame,
   0 when there is no collapse) by the four questions, and name it. **The invariant: no frame ever
   attributes a location to a function that does not contain it, and every collapse states how many
   frames are missing, and where.**
4. **The fn name for the innermost Rust activation.** `RuntimeErrorKind::op()` (step 4, exhaustive)
   names it for the 12 kinds that carry an op.
   - For the others, measure which honest name is in hand at the raise: the form head the
     evaluator applied, or the intrinsic being executed.
   - If some raises have **no** honest name, list them and STOP. Do not invent a placeholder. The
     builder's ruling removed `<rust>` as a placeholder.
5. **D4 follows.** The primary `:location` is still derived as the innermost frame whose `at` is in
   user source. Its semantics are unchanged; it just reads the new shape. G0–G5 from step 4 stay
   green.
6. **The cap is unchanged** (innermost 32 + outermost 8, `frames-elided`), applied to the new list.

## Gates (each mutation-proven in RELEASE)

- **GD1, C-114's trace is exact.** Assert the four frames above, field for field.
  - Mutation: drop the tail-caller record. RED, because `grow` vanishes.
- **GD2, a tail chain.** Use `f` → (tail) `g` → (tail) `h` → raise, called from `main`. The trace
  names `h` with its raise, `g` at its tail call (the last tail caller), the count (1, for `f`'s
  collapsed frame), and `main` at the call into `f`.
  - Mutation: count wrongly. RED.
- **GD3, a non-tail call has no tail marker**, and its frames are unchanged in meaning.
- **GD4, constant space.** A tail-recursive loop of 10⁶ iterations raises at the end. `CALL_STACK`
  depth stays O(1): assert the depth, and the trace holds a single collapse count of about 10⁶.
  - Mutation: push instead of replace. RED, or a timeout, whichever comes first; say which.
- **GD5, no synthesized or placeholder frame.** No golden frame carries `<rust>`, a `:kind`, or a Rust
  span on a wat function's `at` that is not a raise site. Use a lint that parses the goldens.
  - Its anchor must be RED on today's goldens; report the count.

## Goldens

Every golden with `:frames` changes. Recapture with `UPDATE_EDN=1`, in the foreground, and **read
every diff**. Allowed changes:
- the reshape to `{fn at}`;
- the reorder to innermost-first, true order;
- tail markers;
- the removal of D4's synthesized frame, of `kind`, and of main's Rust call site.

Report anything else.

## Scope fence

- **IN:** items 1–6 and the gates.
- **OUT:**
  - E (`EvalError`), F (the domain `Fault`s);
  - the post-F strikes ruled 2026-10-03: retire provenance, the startup-message type, declarable
    `char`, the `LoadOther` rename;
  - the stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build, recapture or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- Iterate with targeted runs. Run the full floor only at a commit. `wat/` files are `include_str!`'d,
  so rebuild after editing them.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - On a timeout: surface it with its history; do not widen `.config/nextest.toml`.
  - **Never commit a red floor.** "Unrelated to my change" is not a disposition.
- Stage by name BEFORE the floor. Never use `git add -A`. No `cargo fmt`.
- Multi-site `.wat` edits go through the wat-fix codemod.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- **If your budget runs short, stop at a clean boundary and report where.** "The `FrameInfo` tail
  record plus GD1–GD4 at the Rust level, on a green floor, before the wire reshape" is a valid
  boundary.

## Report

- the wire spelling for tail collapse, and its four-questions answer;
- the innermost-Rust naming census (which kinds name themselves how, and any STOP list);
- C-114's trace before and after;
- each gate's mutation RED;
- GD5's anchor count;
- any golden that changed beyond the allowed shapes;
- the floor `Summary` line, verbatim;
- the SHA(s).
