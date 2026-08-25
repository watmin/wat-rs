# BRIEF — STONE: wat-grep never lies

DESIGN: `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-wat-grep-never-lies.md` — read it whole.
FINDINGS + their measurements: `docs/arc/2026/06/278-rules-engine/NOTE-wat-grep-is-defective-three-findings.md`.

## Your role

Your cwd is `/home/john/work/holon/wat-rs`. Run `pwd` first and stay there.

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it. Your turn ends when the numbers are in your hands.

**You may not spawn sub-agents.** Do not commit, push, stash, revert, or `git checkout`. There is a
`git stash@{0}` that must never be touched. One untracked file,
`wat-scripts/fixes/rename-four-families-to-their-homes.wat`, belongs to a different blocked stone —
**leave it exactly as it is.**

You may run `cargo build --release` and single named tests (`cargo nextest run --release -E
'test(<name>)'`) — you are writing tests and cannot do that blind. **Not** the full floor, **not**
`cargo clippy`; the orchestrator runs those centrally.

⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** — `wat/grep.wat` is `include_str!`'d at
Rust-compile time (~19s). Every time you change it, rebuild before you test.
⚠ **A file under `wat/` cannot pass a standalone `wat --check`** — `Privilege::Stdlib` comes from the
`STDLIB_FILES` pipeline, never a CLI target. `wat/fix.wat` fails it identically. Not your red.

---

## The work in one paragraph

`wat/grep.wat` hands a rule a fact base that cannot say two things it needs: *"I could not read this
file"* and *"this name is actually spelled at this span."* Both gaps make silence look like an
answer. Add the two facts, make the unreadable case loud, and land the gates — because wat-grep
today has **zero** tests and that is why both shipped.

---

## PART 1 — the unreadable file becomes loud

`wat/grep.wat:218` throws the parse error away:

```wat
((:wat::core::ReadOutcome::Malformed __cause) (:wat::grep::empty-acc))
```

★ **Read `wat/fix.wat:351` first.** It is the SAME `ReadOutcome` match, one file over, and it already
does the honest thing — it raises with `(:wat::core::Error/message __cause)`. You are applying the
sibling's own answer, not inventing one.

1. Add the record. `reason`/`line`/`col` come from `__cause` — find what `Error/message` and the
   cause's span accessors give you; do not guess the shape, read it.
   ```wat
   (:wat::core::defrecord :wat::grep::Unreadable
     [file <- :wat::core::String  reason <- :wat::core::String
      line <- :wat::core::i64     col    <- :wat::core::i64])
   ```
2. `:wat::grep::Facts` gains `unreadable <- (:wat::core::PersistentVector :- [:wat::grep::Unreadable])`
   — zero entries or one. `facts-of` fills it on the Malformed arm and returns an otherwise-empty
   fact base as it does today.
3. `facts-as-records` conjs it, so a rule can join it.
4. **`run-one` prints it to stderr unconditionally**, whether or not a rule joined it. Both halves
   are load-bearing: an opt-in fact does nothing for someone who does not know to opt in, which is
   exactly how today's silence works.

### ⛔ THE PINNED CONTRACT — report EVERY bad file, then exit non-zero

Report and keep going; at the END of the run, if any file was unreadable, raise so the process exits
non-zero. **Do not stop at the first bad file** — that is `fix.wat`'s correct behaviour as an
APPLIER (a partial migration is worse than none) and the wrong one for a FINDER, where it would hide
the other 1566 answers. A finder reports everything, then fails.

---

## PART 2 — `Written`, the fact that means "this name is spelled here"

```wat
;; ONLY when the span holds exactly this node's own name — the fact a REWRITING rule joins.
(:wat::core::defrecord :wat::grep::Written
  [id <- :wat::core::i64  line <- :wat::core::i64  col <- :wat::core::i64
   end-line <- :wat::core::i64  end-col <- :wat::core::i64])
```

Emitted in `walk`, beside `Named`, when **all three** hold:

```
nameable?(node)   AND   line == end-line   AND   (end-col - col) == length(ast-name(node))
```

The predicate is exact, not a heuristic: `ast-name` returns **verbatim token text** (measured —
`(ast-name <head of (wat.core/if true 1 2)>)` is `"wat.core/if"`, not a normalized path), so the
width test is an identity, not an approximation.

★ **It carries coordinates, not just `{id}`, and that is deliberate:** a rewriting rule then joins
ONE fact and never touches `Span`. The right path becomes the shorter one.

**`Named` and `Span` do not change.** A `~` genuinely IS an unquote and a querying rule should still
find it; and `Span == Node` must survive as the non-vacuity control. If you find yourself removing or
guarding either, stop and read STOP-2.

---

## PART 3 — the gates. wat-grep has ZERO today

CLI modes are tested by spawning the real binary — `tests/cli/wat_cli.rs` is the worked pattern
(`Command::new(env!("CARGO_BIN_EXE_wat"))`, asserting stdout / stderr / exit code, fixtures as
sibling `.wat` files via `include_str!`). Copy that shape.

| # | gate |
|---|---|
| G1 | `Span` count **==** `Node` count on a real corpus file |
| G2 | `Named` count **<** `Node` count on that file |
| G3 | a malformed file → an `Unreadable` fact, a stderr line naming the file AND the parse reason, **non-zero exit** |
| G4 | the SAME content, balanced → no `Unreadable`, empty stderr, exit 0 |
| G5 | a file containing `~` → `Written` count **<** `Named` count |
| G6 | a file with no reader macros → `Written` count **==** `Named` count |
| G7 | `--grep` end-to-end through the real binary: a rule over a fixture prints the expected `Match` |

★ **G4 and G6 are not optional and they are the ones you will be tempted to skip**, because each
asserts that *nothing happened*. Without G4, G3 passes on a build that calls every file unreadable.
Without G6, `Written` could be emitted for almost nothing and G5 would still be green. They are the
controls; the stone is not done without them.

⚠ For G5/G6 you need a way to count facts. The cheapest honest route is a rule per fact type
emitting a `Match`, run through `--grep`, and count lines — the shape
`wat-scripts/scratch-pad/probe-span-narrower-than-name.wat` already uses. Fixtures live beside the
test; keep them small and obvious.

---

## Blast radius

`wat/grep.wat`, the new tests and their fixtures, and `src/stdlib.rs` only if a new file needs
registering (it should not — you are editing an existing stdlib file). **No changes to
`wat/fix.wat`, no changes to any `wat-scripts/fixes/*.wat`, no `.wat` corpus edits.**

---

## STOP triggers — each means SHIP NOTHING and report

1. **STOP-1 — `__cause` does not carry a reason/line/col you can read.** The design assumes the parse
   error is structured. If `Error/message` is all you can get, report exactly what the cause exposes;
   do not invent a line/col or fabricate a reason string.
2. **STOP-2 — `Written` cannot be added without changing what `Named` or `Span` means**, or without
   breaking `Span == Node`. Both must survive untouched. Report what forced it.
3. **STOP-3 — an existing consumer breaks.** `wat-scripts/grep/`'s five programs and
   `wat-scripts/fixes/rename-core-string-to-string.wat` must still run unchanged. If adding a field
   to `Facts` breaks a caller, report the caller.
4. **STOP-4 — G4 or G6 cannot be made to pass**, i.e. `Written` is emitted for far fewer nodes than
   expected on a clean file, or a balanced file reports itself unreadable. That is a real defect in
   the predicate, not a test to loosen.
5. **STOP-5 — a room's line number does not hold what this brief says.** Written against `8137598b2`.
   Report the mismatch rather than widening the search.

---

## Report back with

- **G1–G7, each with its actual numbers**, not "passing". G1's two counts, G5's and G6's two counts.
- The malformed-vs-balanced control, both halves, with the full stderr and the exit code of each.
- The `Written` vs `Named` delta over the tracked `.wat` corpus — the design predicts **1411**; if
  yours differs, that difference is the most interesting thing in your report.
- Anything the brief got wrong. It was written by someone who has been wrong about this corpus
  several times today; if a room does not hold what it claims, say so.
- What you did NOT do, and why.
