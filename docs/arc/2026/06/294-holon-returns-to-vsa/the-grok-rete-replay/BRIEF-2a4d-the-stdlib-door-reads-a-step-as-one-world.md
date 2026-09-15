# BRIEF 2a4d — the stdlib door reads a step's stdlib files as ONE world; record #155's hand wrap

> Found by the orchestrator verifying batch 4a (finding 23). Batch 4a is green on the floor; these are
> two METHOD flaws in how it got there. 4b (#160–#211) touches no stdlib file; the door gap recurs at
> #221 #377 #379 #398 #438 #440.

## Why — measured

1. **The stdlib door reads one file at a time against HEAD's snapshot.** 4a's three new outcome enums are
   declared in `wat/rete.wat` (`FireOutcome :- [T]` :242, `InsertOutcome` :273, `CompileOutcome` :311) and
   used in `wat/fmt.wat`, `wat/grep.wat`, `wat/query.wat`, `wat/rete/oracle/{explain,fire}.wat` — all changed
   in the same steps. Phase (a) converted each user file against HEAD's snapshot (the OLD or absent enum), so
   match-arm left list-form arms UNRESOLVED; the merged stdlib could not load, and grok KEY-FIRST'd the
   leftovers BY HAND (SCORE-7a E9: "R21 exception: wat-fix cannot load"). The STASH-DANCE would not have
   helped: the previous binary lacks the new enum too. The door must see the step's stdlib as one world.
2. **#155 hand-edited 11 chain members** (`rename-core-string-to-string`, `rename-core-{numerics,maps,
   vectors,set-and-list,bigint-rational}-…`, `rename-four-families-…`, `rename-keyword-to-its-home`,
   `rename-math-stat-seq-…`, `rename-rete-numerics-…`, `rename-string-verbs-…`) plus grep/fmt: each
   `(overlay records)` wrapped in its `FireOutcome` match, 7 identical lines a file — necessary (the chain
   cannot load otherwise; disclosed in #155's body, not in SCORE-7a), but an 11-file structural rewrite by
   hand (R21). The ported `wrap-fire-rules-in-fireoutcome` changes nothing on them (probed: `overlay` is not
   a `fire-rules` call site).

## The shape

1. **The door asked once per set.** The codemods already run once per `convert.sh` set; for the set's
   `wat/` files, `wat/fix.wat`'s `enum-fields` asks `:wat::runtime::declared-stdlib-types` over the forms of
   ALL of them together (each divergent declaration replaced in the copy, as today), and every file of the
   set is answered from that one world. Derived from the set, never a list.
2. **The recorded migration after the fact:** `wat-scripts/fixes/wrap-overlay-in-fireoutcome.wat` — its
   fixture's `before.pre` is a chain member at `18eb21a71`, `after.post` the same file at #155; it must
   reproduce #155's edit byte-for-byte on all 11 (and grep/fmt's overlay sites). Content unchanged; the
   method recorded. A follow-up commit (nothing in #155 is wrong or red).
3. **4a's hand KEY-FIRSTs, proven:** re-run phase (a) of #155 (the largest) at `#154` with the new door
   (patch stashed onto a detached #154, build, `convert.sh`), merge as the step did, and compare with the
   committed stdlib files. Identical proves the hand edits; any difference is a finding, not a fix-up.

## The bar

- A fixture: two stdlib files in one set, one declaring a CHANGED enum, the other matching its NEW
  variants — converted KEY-FIRST, no UNRESOLVED, no hand touch. RED under the mutation that asks per file.
- `wrap-overlay-in-fireoutcome`'s fixture replays; applied to the 11 at `18eb21a71` it yields #155's text.
- #155's stdlib re-conversion equals the committed files.
- run5 unchanged (0 losing; chain vs main ≥ 1370); floor + clippy.
