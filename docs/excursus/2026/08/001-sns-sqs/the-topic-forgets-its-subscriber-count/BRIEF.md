# BRIEF — the topic forgets its subscriber count

Delete `nsubs` from `:demo::topic::Record`. It is the last fossil of the expansion `e0c552bf0`
removed from `publish`: a `:durable` field supplied at every construction site and read by nothing,
duplicating a fact the **worker** owns correctly as `(:wat::core::count subs)`. This is a corpus
migration across several `.wat` files, so it goes through a recorded `wat/fix.wat` codemod.

**Read in order:**

1. `docs/excursus/2026/08/001-sns-sqs/the-topic-forgets-its-subscriber-count/DESIGN.md` — the three
   forms, and the trap.
2. `wat-scripts/fixes/alarm-after-to-delay.wat:1-27` then `:78-97` — **your shape.** The only
   recorded fix that matches kind `"symbol"`, it carries three forms in one rule set, and it
   disambiguates a binder symbol by parentage (`?hi 0` head, `?ti 1` type). Its header names the
   same discipline this stone needs: *FORM, not token.*
3. `wat-scripts/fixes/declare-queue-drop-knobs.wat:1-50` — the **inverse** operation: kwarg pairs on
   a `:durable` record's constructions, anchored on the head keyword, idempotent by presence check.
   Copy its two-entry-point layout (`:user::grep` / `:user::main`) and its usage header verbatim in
   shape.
4. `wat/fix.wat:243` `fix-text-deletion-edit`, `:1002`/`:1006` `node-start-offset` /
   `node-end-offset`, `:367` `fix-text-apply` — your primitives. `:505` `strip-arrow-ascription` is
   the worked composition that deletes `->` **plus** its type; a binder triple is that move with one
   more token.
5. `wat-scripts/topic/sns-fanout.wat:71` (the binder), `:217-222` (the multi-line construction whose
   `:nsubs` value is the accessor), `:425` (**the worker's `let` binding — this must survive**).

**Sketch** — two rules, one rule set:

```
rule 1  kwarg pair    keyword ":nsubs", parent's head is ":demo::topic::Record"
                      -> delete [start-offset(keyword) .. end-offset(value node)]
                         (this also takes the accessor at :220, which is that pair's value)

rule 2  binder triple symbol "nsubs", parent kind "vector",
                      the vector's grandparent head is ":wat::service::defservice",
                      index-1 of that form is ":demo::topic"
                      -> delete [start-offset(symbol) .. end-offset(type keyword)]
```

Then: **census first, diff, apply.** `wat --grep` prints every match unapplied; diff a `/tmp` pilot
copy before touching the corpus; then apply with **every** path on stdin. Count occurrences, not
lines — the finder emits one long line and `grep -c` undercounts.

**Blast radius, stated as a property rather than a list:** the codemod's own finder is the census,
and the corpus gate plus the floor name anything it missed. No change to `wat/`, to `src/`, to
`mem.wat`, or to the store. **Anything the gate or the compiler forces IS in radius — make the
change and name it in the SCORE.** Do not work around a forced change to stay inside a boundary.

⚠ Do not trust a site count from the DESIGN or from me. My own single-line grep said 19
constructions and missed the multi-line one at `:219-221`; the finder is the instrument.

**STOP-1** — if the finder cannot separate the topic's `:durable` binder `nsubs` from the worker's
`let` binding at `:425` by **parentage**, STOP and report the fact-shapes you tried. Do not fall
back to matching on the symbol name, and do not special-case a line number.

**STOP-2** — if `wat --grep` reports a `:nsubs` site in a file the DESIGN's exemplars did not
anticipate, STOP and report the full list before applying. My file set came from greps that have
already been wrong once this session.

**STOP-3** — if deleting the field turns out to need a new generic in `wat/fix.wat`, STOP and report
what is missing and why the existing primitives do not compose. **Promotion to `wat/` is the
builder's ruling, so surfacing that gap is a complete and valuable result, not a failure.**

**STOP-4** — on any red in the corpus gate or the floor: do **not** re-run. Capture the whole log,
name the exact failing arm, and surface it. A green re-run destroys the only evidence, and a
`debug_assert!` panic is a real failure.

**Write your SCORE to disk** at
`docs/excursus/2026/08/001-sns-sqs/the-topic-forgets-its-subscriber-count/SCORE.md`, graded row by
row against EXPECTATIONS.md, and leave everything uncommitted — the grading and the commit are mine.
For the shape of a good SCORE, copy
`docs/excursus/2026/08/001-sns-sqs/the-inbox-holds-messages-not-pairs/SCORE.md`.

**Read the floor's Summary line**, never a piped exit code: `cargo … | tail` returns `tail`'s
status, and `grep -c` exits 1 on zero matches. Run it via `scripts/floor.sh`.
