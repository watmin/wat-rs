# SCORE — STONE 255.1c-io-reader

Every row below re-run by the orchestrator's own hand against the disk. The rider ran no build.
Rider runtime **8.9 min** against a predicted 25–40 — the room map to the line is what bought that.

| # | what | result |
|---|---|---|
| 1 | IOReader arms gone from `runtime.rs` | ✅ 0 |
| 2 | IOWriter arms untouched | ✅ 13 |
| 3 | ten rows in `io/reader.rs` | ✅ 10 |
| 4 | registry 97 → 107 | ✅ 107 |
| 5 | `src/check.rs` not edited | ✅ empty diff |
| 6 | `tests/` not edited by the rider | ✅ empty diff (the 5 goldens are the orchestrator's own step) |
| 7 | `stdlib::sources` survived the cut | ✅ 1 |
| 8 | builds | ✅ clean, 18.5s |
| 9 | doc/scheme gate green | ✅ + `purity_mandated_examples`, `all_see_fqdns_resolve`, the purity census, `yields_type_matches_fn_arg_param` — 5/5 |
| 10 | ★ **the gate can FAIL** | ✅ perturbed `read-all-string`'s `@ret` → RED, naming the row and both readings; reverted byte-identical |
| 11 | probe unchanged | ✅ 29 population · 28 scheme · 1 bespoke · **0** blanket-accepted |
| 12 | floor | ✅ **4818/4818, 0 FAIL, 19 skipped, 73.7s** |
| 13 | clippy `-D warnings` | ✅ 0 |
| 14 | goldens | ✅ 5 bumped 25277 → 25247, `:col 17` untouched |
| 15 | rustfmt | ✅ `io/mod.rs` + `io/reader.rs` clean; repo-wide count 2573 before and after |

## The arm, when the floor went red — captured before anything was re-run

The first floor was **4813/4818, 5 failed**. All five were the pinned goldens, and the assertion was
the line number and nothing else:

```
assertion `left == right` failed: EDN data mismatch (Probe 1: NotCallable must surface type name …)
  actual   :location #wat.core/Span {:file "src/runtime.rs" :line 25247 :col 17 …}
  expected :location #wat.core/Span {:file "src/runtime.rs" :line 25277 :col 17 …}
```

Every other byte identical. 25277 − 25247 = **30**, and `git diff src/runtime.rs` is a **single hunk
at 6441** — 30 arm-lines removed, a 4-line comment replaced by 4 lines — entirely above the pin. Both
conditions of the standing step met by reading the hunks, never `--numstat`.

## ★ THE PROBE FAILED ITS OWN NON-VACUITY TEST, AND IT WAS MY INSTRUMENT

Re-run after the carve, `PROBE-255.1c-io-every-verb-is-scheme-enforced.sh` reported
**19 verbs, 0 fell through** — and 0 is the passing value. It enumerated its population by grepping
`runtime.rs` dispatch arms: **the exact thing every stone in this campaign deletes.** Ten verbs left
the corpus it could see, and it printed a clean bill for the nineteen that remained.

Left alone it would have decayed to zero verbs and a perfect score. Fixed to enumerate the **union**
of arms-in-`runtime.rs` and names-registered-via-`#[wat_intrinsic]`, plus a hard floor that screams
if the population drops below 29. Re-run: **29 · 28 · 1 · 0 — identical to pre-carve.** The carve
moved dispatch; it did not move typing.

> An instrument that draws its population from the thing under migration measures less every stone
> and reports better every stone. `[[feedback_a_control_that_answered_the_first_question_cannot_answer_the_second]]`

## The categories — ten rows, weighed at the bodies by my own read

`Transform ×2 · Resource ×3 · Io ×5`. The straddle the design demanded is real; this home could
falsify the metadata contract and did not.

- **`from-bytes` / `from-string` → `:Transform`.** `Arc::new(StringIoReader::from_bytes(bytes))` —
  no syscall, no fd. The axis is *"the OUTPUT IS A FORM OF THE INPUT"* and the reader yields exactly
  those bytes back, in order. The softest of the eight uncontested calls; `:Resource` was the rival
  and loses because a `StringIoReader` has no lifetime tracked outside value scope.
- **`open-file` / `from-fd` → `:Resource`.** `OpenOptions::open` and `libc::dup(fd)` each claim a
  fresh kernel-tracked fd. Textbook acquisition.
- **the five reads → `:Io`.** I nearly filed these as wrong: `:Io`'s prose contrasts itself with a
  target *"the caller holds a handle to"*, and all five take a caller-held `IOReader`. **Re-reading
  the axis, the contrast is drawn against `:Message` — a peer, a typed value with another LOCUS
  behind it.** A stream handle is not a peer. The five stand.

### `rewind` — the row that would not classify, and the rider was right to say so

Landed `:Resource`, contested, with both arguments written into the module doc rather than tidied
away. Verified at all three impls by my own read: `RealStdin::rewind` (`io.rs:179`) is an
unconditional `Ok(())`; `StringIoReader::rewind` (`368`) sets `s.cursor = 0`; `PipeReader::rewind`
(`582`) unconditionally raises *"pipe fds are not rewindable"* without ever calling `lseek(2)`.
**No implementation moves a byte**, so `:Io`'s *"the effect IS the point"* cannot hold. What remains
is administering a handle the caller already holds without acquiring or releasing it —
`:Resource`'s third disjunct, the shape `kernel/resource.rs` gives `signal`. `:Mutate` is not
available: `runtime-meta.wat:169` records it as REFUSED for `allow`/`deny`.

Its `@Determinism` diverges from the read family for the same reason and correctly: the reads are
`Nondeterministic` (content is ambient), `rewind`'s outcome is fixed by the backing type alone.

**This is the classification-failure the taxonomy is being held open to collect.** It is filed, not
resolved — per the builder's standing rule that we continue with the names we have and seek failures
to classify as we move forward.

## Honest deltas

- **The family claim is supported but not ironclad, and the rider said so unprompted.** Every row's
  doing is legibly *"bytes crossing, or a handle for bytes crossing, the process boundary"*; no row
  needed a category foreign to that subject. `rewind` is the load-bearing counterexample — the one
  row where no bytes cross at all, grouped in only because it operates on the same handle type.
- **I nearly filed a false finding against the rider.** My hand-rolled `@arg`-vs-`params.len()`
  counter said `read-frame` documented two args against a one-param scheme; the eleventh `@arg`
  string was inside a `//` maintainer comment *quoting* the tag. Ten rows, eleven `@arg` lines,
  `read` is the one with two params — every row matches. Third boundary-attribution error of the
  session, on a class already named. `[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`
- **I ran `git stash push -u` for a rustfmt baseline** with the lifecycle strike in `stash@{0}`.
  It popped clean and the strike is verified intact (298 insertions, `wat/service.wat`) — but the
  number was one I did not need: the per-file check had already shown both new files clean.
- The rider flagged its runnable `@example`s for the two `Pure` rows as its own highest risk. They
  pass `purity_mandated_examples` and the floor.
