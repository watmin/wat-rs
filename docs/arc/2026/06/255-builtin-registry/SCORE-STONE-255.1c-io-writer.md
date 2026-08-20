# SCORE — STONE 255.1c-io-writer

Every row re-run by the orchestrator's own hand. Rider runtime **10.0 min** against a predicted
20–35. It ran no build and tripped no STOP.

| # | what | result |
|---|---|---|
| 1 | writer arms gone | ✅ 0 |
| 2 | the stays-block survived | ✅ Temp ×4, read-file/list-dir ×2 |
| 3 | the foreign family survived | ✅ `:wat::stdlib::sources` ×1 |
| 4 | thirteen rows | ✅ 13 |
| 5 | registry 107 → 120 | ✅ 120 |
| 6 | checker untouched | ✅ empty diff |
| 7 | tests untouched by the rider | ✅ empty diff |
| 8 | builds | ✅ 18.6s |
| 9 | five registry gates | ✅ 5/5 |
| 10 | ★ the gate can FAIL | ✅ `writeln` `@ret` i64→nil → RED, named row + both readings; reverted byte-identical |
| 11 | probe population held | ✅ **29** · 28 · 1 · 0 |
| 12 | floor | ✅ **4818/4818, 0 FAIL, 71.4s** |
| 13 | clippy `-D warnings` | ✅ 0 |
| 14 | rustfmt adds nothing | ✅ `io/writer.rs` clean; `purity.rs` 40 before, 40 after — four of mine found and fixed |
| 15 | goldens | ✅ 25247 → **25209**, `:col 17` untouched |

## Every trap the EXPECTATIONS named was cleared

The rider transcribed all 18 `@arg` and 13 `@ret` from `check.rs`, not from the neighbouring row:
`writeln`→`i64` beside `println`→`:()`; `write`→`i64` beside `write-all`→`:()`;
`to-string`→`Option<String>`; and `new` carries **zero** `@arg`. The perturbation targeted `writeln`
precisely because it was the likeliest copy, and the gate named it.

Delta measured from the **two** hunks (`@@ -6441,37 +6441,12` and `@@ -6497,19 +6472,6`):
44 removed, 6 added, net **38** — predicted 39 ± 2. Nothing from the stays-block appears as a
deletion line.

## ★★ THE FLOOR'S RED WAS THE THIRD INSTANCE OF ONE FAILURE CLASS TODAY — AND THE OLDEST

```
rete::purity::completeness_gate::every_dispatched_verb_is_classified_or_disposed
  panicked at src/rete/purity.rs:2297:9:
  the dispatch scan found only 400 verbs — the `fn dispatch_*` anchors have drifted and
  this gate is measuring nothing. Fix the anchors; do NOT lower the floor.
```

Its own non-vacuity guard, firing exactly as built. But **the anchors had not drifted.** The gate
draws its population by scanning `runtime.rs`'s dispatch blocks — *"it IS the source of truth for
what verbs exist"*, said a comment arc 255 exists to falsify. The `:wat::io::` carve moved 23 verbs
into the registry and took the count 423 → 400, one below the floor.

**This is the same defect as the probe fixed at `b9e28b946`, one layer deeper — and it is older.**
`git log -S` on a time verb lands on `25c1f4521`, the home #2 carve, whose own commit message reads:

> *"THE FLOOR RED, and it was a GOOD red — the ledger RATCHET: '41 verb(s) in KNOWN_UNREVIEWED are
> no longer unreviewed'. It named all 41 precisely because it freezes NAMES, not a count. Deleted
> those lines per the builder's ruling; unreviewed debt 214 → 173."*

**Those 41 were never reviewed.** They left the scan's sight when the home carved. The ratchet was
right to name them and the *disposition* was wrong: the honest answer to a population that shrank is
to ask why it shrank, not to delete the names that fell out of it. Home #1 did the same for
`Bytes::to-hex`/`from-hex`, and the reflect rows went the same way. **48 verbs of real purity debt
were booked as progress.**

The memory entry `[[feedback_a_gate_freezes_names_never_a_count]]` was written about this exact gate
and it held — the gate froze names and named all 41. What it could not do is tell a verb that was
*ruled on* from a verb that *left the room*.

### What shipped here

1. `dispatch_verbs` now scans the `#[wat_intrinsic]` homes as well as `runtime.rs`. A verb dispatched
   through the registry is still dispatched; it just is not a literal arm any more. Population
   **448**, and it grows as the carve proceeds instead of shrinking.
2. The 48 names are **restored** to `KNOWN_UNREVIEWED`, with the history written above them. Nothing
   is classified — 255.3 owns that. The debt is honest again: 173 → 221 is not a regression, it is
   the number that was always true.
3. The stale comment calling `runtime.rs` the source of truth for what verbs exist is corrected.
4. **Non-vacuity checked on my own fix**: removing `:wat::time::now` from the ledger makes the gate
   fail and name it. The restored gate has teeth.

★ This makes the carve's remaining stones safe. Without it, every future home would have drained the
same gate a little further, and it could only have screamed once more — at 400 — before its floor
was the only thing left to lower.

## Categories — thirteen rows, weighed at the bodies

`Resource ×4 · Projection ×2 · Io ×7`. The straddle is real.

- **`open-file` / `from-fd` → `:Resource`.** `OpenOptions::…open(&path)` and `libc::dup(fd)`.
- **the write/print family → `:Io`.** Real pushes; `print`/`println` discard the count, `writeln`
  returns it, which is exactly why the schemes disagree.
- **`flush` → `:Io`.** Real for `RealStdout`/`RealStderr` (`io.rs:230`/`271`), documented no-op for
  the in-memory and pipe backings.
- **`to-bytes` / `to-string` → `:Projection`, and the rider argued rather than asserted it.** The
  input is the writer HANDLE; the bytes are a component of its state, matching the axis prose
  (*"returns a COMPONENT of a compound value that was already there"*) almost verbatim. The brief's
  own predicted phrasing — "hand back a form of what was written", i.e. `:Transform` — is written in
  as the live counter-argument. `Pure` + `Nondeterministic` follows `HandlePool::finish`, and the
  rider used the **corrected** pairing that `resource.rs:254` records the orchestrator imposing on a
  previous rider, rather than repeating that mistake.
- **`close` → `:Resource`, and the rider corrected my brief.** I grouped `close` with `flush` under
  "administer". The body says otherwise: `PipeWriter::close` (`io.rs:759`) swaps fd→-1 and calls
  `libc::close(2)` — it genuinely **releases**, the `close'` shape, not `signal`'s. `flush` is the
  true administer. The rider split them and said the brief undersold `close`. Correct.

### `new` — the second filed classification strain, and it pairs with `rewind`

Landed `:Resource` with the thinness documented in both the module doc and the row comment.
`StringIoWriter::new()` is a syscall-free heap allocation, structurally identical to `reader.rs`'s
`from-bytes`/`from-string` — which are ruled `:Transform` precisely because a `StringIoReader` has no
lifetime tracked outside value scope. The only reason `:Transform` cannot apply is that `new` is
**nullary**: there is no input for the output to be a form of.

So the taxonomy has no home for *"mints a fresh in-memory value from nothing."* That is the second
strain this family has produced, and the two are a matched pair:

| row | strain |
|---|---|
| `IOReader/rewind` | a handle operation that moves no bytes — `:Io` cannot hold it |
| `IOWriter/new` | a nullary mint with no OS resource — `:Transform` cannot hold it |

Both filed, neither resolved, per the builder's standing rule. `:Mutate` is unavailable
(`runtime-meta.wat:169` records it REFUSED).

## Honest deltas

- **My room map was off by one.** I gave the stays-block as `6476–6498`; the boundary comment starts
  at 6475. It cost nothing because the rider deleted by exact text match rather than by line number
  — and it reported the correction instead of quietly absorbing it.
- **`new`'s runnable `@example` is the rider's own flagged unknown.** It reused `reader.rs`'s proven
  `(:wat::core::Vector :u8)` constructor rather than guessing at `Option` rendering. It passes
  `purity_mandated_examples` and the floor.
- **Fourth boundary-attribution error of the session, mine.** I `comm`'d rustfmt line anchors between
  HEAD and my tree — my 48-line ledger insert had shifted every anchor below it, so ten "new" lines
  were the same lines moved. Redone by CONTENT, the real answer was four, and all four are fixed:
  40 nonconformances before, 40 after. Same class as the `@arg` miscount an hour earlier.
