# BRIEF — STONE 255.1c-io-reader (home #12, strike 1 of 3)

Carve the ten `:wat::io::IOReader/*` verbs out of `runtime.rs`'s literal dispatch into a new
`src/intrinsic/io/` directory, registering each with `#[wat_intrinsic]`. This is the first strike of
the `:wat::io::` family; it mints the directory and the doc pattern that strikes 2 (`writer`, 13
rows) and 3 (`fs`, 6 rows) will copy. Design:
`DESIGN-STONE-255.1c-io-the-family-that-the-gate-can-actually-see.md`.

**Your role: you write the text. The orchestrator builds, floors, and clippies.** Your turn ends when
the ten rows are on disk and your report is written — that is the deliverable, and it is complete
without a build. Report what only you can know: which body decided each `@Category`, what surprised
you, which row took an argument.

## Read in order — the rooms are mapped, hunt for nothing

1. **`src/intrinsic/kernel/resource.rs`** — THE TEMPLATE. Read the module doc (its "strain report"
   shape) and then one full row: doc-comment block → `//` maintainer comment → `#[wat_intrinsic]` →
   thin fn. Copy this shape exactly.
2. **`src/intrinsic/kernel/mod.rs:1–40`** — the tier claim and the `[`abort`]`-style module list.
   Your `io/mod.rs` is the same shape with one difference stated in "The family claim" below.
3. **`src/runtime.rs:6444–6477`** — your source arms. Lines `6448–6477` are the ten IOReader arms;
   `6444–6447` is a comment introducing **both** IOReader and IOWriter.
4. **`src/check.rs:15715–15815`** — the ten registered `TypeScheme`s. **These are the authority for
   every `@arg` and `@ret` you write.** Per verb:
   `open-file 15720 · from-bytes 15729 · from-fd 15740 · from-string 15749 · read 15758 ·
    read-all 15767 · read-all-string 15776 · read-line 15785 · read-frame 15794 · rewind 15807`
5. **The bodies, in `src/io.rs`** — read each before you assign its `@Category`:
   `from_bytes 875 · from_string 889 · read 905 · read_all 926 · read_all_string 943 ·
    read_line 958 · read_frame 1003 · rewind 1157 · open_file 1237 · from_fd 1319`
6. **`wat/runtime-meta.wat`** — the `Category` defenum, 15 variants, and the prose for each. The
   prose is the ruling, not the variant name.

## The work

**1. `src/intrinsic/io/mod.rs`** — module doc + `mod reader;`.

**2. `src/intrinsic/io/reader.rs`** — ten rows, each: `///` doc block (`@added 1.0.0`, `@Purity`,
`@Determinism`, `@Category`, `@arg` per param, `@ret`, an `@example` or `@example-norun`), then a
`//` maintainer comment naming the deciding line, then `#[wat_intrinsic(":wat::io::IOReader/…")]`,
then the thin fn delegating to `crate::io::eval_ioreader_*`.

⚠ **The delegate call's argument ORDER varies inside this one block.** Four verbs pass
`(args, list_span, env, sym)` — `from-bytes`, `from-string`, `open-file`, `from-fd`. Six pass
`(args, env, sym, list_span)` — `read`, `read-all`, `read-all-string`, `read-line`, `read-frame`,
`rewind`. Copy each call verbatim from its arm; do not normalise them.

**3. `src/intrinsic/mod.rs:374`** — add `mod io;` in alphabetical position (after `mod bytes;`).

**4. `src/runtime.rs`** — delete lines `6448–6477` (the ten arms, nothing else), and rewrite the
`6444–6447` comment so it names only `:wat::io::IOWriter` and points a reader at
`src/intrinsic/io/reader.rs`. A comment that still says "IOReader / IOWriter" over a block holding
only IOWriter arms is a stale map (FM 14 Bucket B) and this stone is what makes it stale.

## The two things this home is FOR — get these right and the stone is right

**A. The docs conform to the checker, never the reverse.** Every `@arg` type and every `@ret` type is
read off the `TypeScheme` at the `check.rs` line above and transcribed. `doc_arg_ret_types_match_
checker_scheme` compares them by `assert_eq!` at every floor, so a transcription slip goes RED —
which is the point, and why this home is worth doing. **You do not edit `src/check.rs`.** If a
scheme looks wrong against its body, that is a FINDING for your report.

**B. Every `@Category` is decided at the BODY, never from the name.** This family straddles the axis,
which is why it was chosen — a pure construction, an fd claim, a projection, and a push at the world
are all in these ten. Read `src/io.rs` at the line given, then quote the deciding line in the `//`
comment beneath the doc block, the way `resource.rs` does. A row you cannot classify is not a
failure — **it is this stone's most valuable output**; write the argument down and name it in your
report. The builder's standing rule: *a naming argument in the abstract is taste; a verb that will
not classify is data.*

## `read-frame` — the one row that is already known to be odd

It has BOTH a registered scheme (`check.rs:15794`, **one** param, `ret` is a `ReadFrameOutcome`, not
`Option<String>` — read the comment there) AND a bespoke arm `infer_ioreader_read_frame`
(`check.rs:2969`) that intercepts first and accepts **one or two** args. Document its `@arg`/`@ret`
from the **registered scheme** (that is what the gate compares against), and give it a `//`
maintainer comment naming `infer_ioreader_read_frame` as the real check-time authority — the shape
`kernel/message.rs` uses for its bespoke-arm rows. Do not mint a second scheme.

## The family claim, for `io/mod.rs`

`kernel/mod.rs` opens *"`:wat::kernel::` is not a family. It is a TIER."* **`:wat::io::` IS a family**
— one subject, bytes crossing the process boundary, asked three ways (`reader` / `writer` / `fs`).
Write that claim, state the three-file decomposition with only `reader` present so far, and carry the
tier-wide *"the bodies do not live here"* sentence: all thirty arms in the block delegate into
`crate::io::`. If your body-reads refute the one-family claim, say so in your report — that
refutation is worth more than the claim.

## Blast radius

`src/intrinsic/io/mod.rs` (new) · `src/intrinsic/io/reader.rs` (new) · `src/intrinsic/mod.rs` (one
line) · `src/runtime.rs` (one deletion + one comment). No new types. No `src/check.rs`. No `tests/`.
No `.wat`.

## STOP triggers — each rejects; none is a fallback

1. **A doc type would not match its `TypeScheme`.** STOP and report the mismatch with both readings.
   Do not adjust `check.rs`, and do not write the type you think is correct.
2. **A body does not delegate to a `crate::io::` fn**, or the fn at the line given is not the one the
   arm calls. STOP and report — the room map is wrong and the rest of it is then suspect.
3. **A row needs a `Category` variant that does not exist in `wat/runtime-meta.wat`.** STOP and
   report the argument. Do not pick the nearest variant, and do not add one.
4. **The deletion at `runtime.rs:6448–6477` would take any line that is not one of the ten arms.**
   STOP and report what is actually there.

## Acceptance criteria

- `src/intrinsic/io/reader.rs` holds exactly ten `#[wat_intrinsic]` rows, one per IOReader verb.
- Every `@arg` and `@ret` transcribes its `check.rs` `TypeScheme`.
- Every `@Category` has a `//` comment quoting the `src/io.rs` line that decided it.
- `grep -c '":wat::io::IOReader/[^"]*" *=>' src/runtime.rs` → **0**.
- `grep -c '":wat::io::IOWriter/[^"]*" *=>' src/runtime.rs` → **13**, unchanged.
- `src/check.rs` and `tests/` show no diff.
