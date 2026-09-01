# HANDOFF — item (b): the batched writer

You are making an oversized batch drain. Today, after a failed size-triggered flush, the span's
buffer can exceed the server's cap and **never** be flushable again — every later write is refused
`RequestTooLarge` and each further log makes it bigger.

Start here, in order:

1. `DESIGN-STONE-the-batched-writer.md` — the scope ruling, the cut rule, the one decision that is a
   data bug in both directions.
2. `BRIEF-item-b-batched-writer.md` — the rooms as exact `file:line`, four STOP triggers.
3. `SCORE-item-c-stone-c-flush-must-speak.md` — its Row 2 section is the finding you are closing.

Four things to hold:

**Do NOT build `Stream`.** The design defines (b) as sugar over item (a)'s stream writer, but
`Stream`, `WriteResult` and any chunker are all absent from the tree, and nothing streams. The span
holds its whole buffer in hand — that is (b)'s own "have-it-all-in-hand" case. Building an
abstraction for its first user is how it ends up wrong for its second.

**★ The written count must be EXACT.** Report fewer than landed and those items get re-sent —
duplicate logs. Report more and they are dropped — lost logs. Neither shows up anywhere except the
gate that counts the buffer afterwards. This is the stone.

**Cut at `>`, not `>=`.** The server rejects at `>` (`service.wat:1779`); the span's `>=` is a
when-to-flush heuristic, not a what-fits rule. Conflating them is what produced the stuck buffer you
are fixing. A chunk sized exactly to the cap is legal and must be sent.

**A single over-cap item must not loop.** It can never fit any chunk. Report it as
`RequestTooLarge{bytes, cap}` and return. An empty chunk that goes round again hangs the flush, which
is worse than the failure it was trying to report.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-item-b-batched-writer.md` when done. It will be graded by re-running.
