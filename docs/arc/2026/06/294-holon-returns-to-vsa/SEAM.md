# SEAM — the ONE live breadcrumb. As of 2026-08-23. `<K,V>` IS DEAD. Replaced in place.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 4924/4924, 0 FAIL, 19 skipped, ~82s  (own invocation, scripts/floor.sh, at aecba7b06)
                ⚠ EVERY MOVE ACCOUNTED. 4881 → 4924 across this session:
                  +5 last-comma · +5 one_name_grammar · +12 finish-`:-` (5 rune, 7 door)
                  +5 no_angle_suffix_strip · +3 reap-the-twelve · +5 doc-validator · +8 diagnostic gate
                If you floor and see 4924, that is green. Anything else, EXPLAIN before accepting.
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba. (verified intact this session)
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s> …`.
⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** (`include_str!` at RUST-compile time).
⚠ **`cargo wat` uses the STALE installed binary.** Always `target/release/wat`.

## ★★ DONE: `<K,V>` CANNOT BE WRITTEN, MINTED, RENDERED, PARSED, OR DOCUMENTED

```clojure
[n :- wat.type/i64]                          arg-spec
:- wat.type/i64                              ret-type
(wat.type/Vector :- [wat.type/i64])          type args    — a REFERENCE, in parens
(wat.type/Vector :- [wat.type/i64] 1 2 3)    constructor  — the reference PLUS values
(wat.core/defn ns/f :- [T] [x :- T] :- T x)  declaration  — a BINDER, siblings, NO parens
(:ns/f :- [:i64] 7)                          CALL-SITE application — and it BINDS
[A B :-> R]                                  function type
```
**`:- []` ≡ absent.** No mono-vs-parametric distinction; macros emit it unconditionally.

```
17cbe1d4f  the last comma dies — in a SYMBOL, at any depth
86e1b105a  THE PERMISSION removed from both lexer doors — 28 of 1798 self-identified
ac5965086  the read-failure path was dead SUBSTRATE-WIDE — 75 handlers, invoked zero times
43a458b41  ONE NAME GRAMMAR — 33 hand-rolls → 33 calls + a rune
69933d362  FINISH `:-` — four positions, one door; the call site now BINDS
64a8fa5a0  the renderers emit `:-` — a printed type is paste-able back into source
c6c614fe2  EXACTLY ONE CALL POSITION — and defservice emits the binder
0811c3009  UNEXPRESSIBLE — all three minting doors walled
131c7c299  reap the machinery — 16.2M no-op calls deleted
3b1225b82  reap the twelve — 15.7M more; the rune finally widens
f82dc6de1  the doc validator asks the READER, not the first byte
9848eb9ed  the prose stops teaching it — six riders, 272 files
09881e830  channels 3+4 closed — and the gate guarding them was 45% BLIND
aecba7b06  the DORMANT minter dies — the last one, and the join it broke
```

⚠ **AND THE DUALS, which every wall here preserves:** `(Vector :- [:i64] 1, 2, 3)` → `[1 2 3]`;
`:wat::core::<` `>=` `<-` `->` all lex; `Peer'` and `foo/bar` lex. **A wall that refuses everything
passes its own test and destroys the language.** `a<b` no longer lexes — a measured narrowing, 0 sites.

## ⛔ NEXT — nothing here is blocked; pick by cost

1. **R6's comment tail: ~70 `.rs` files unreached.** check.rs/types.rs/runtime.rs are done. The rider
   STOPPED and named what it missed — do not treat the slice as complete. Class D (Rust generics) is
   the work; rewriting `Arc<Function>` into `:-` is worse than doing nothing.
2. **`NOTE-the-guides-are-not-executable.md`** — `:wat::core::define` is RETIRED and `USER-GUIDE.md`
   teaches it **32 times** as its primary form; `let`'s binding shape changed; bare `:String`/`:i64`
   don't resolve. The structural fix is to extract fenced `wat` blocks and `--check` them, as
   `every_wat_scripts_file_loads` does for `wat-scripts/`. **That gate goes red on landing** — gate and
   repair are one sequenced stone.
3. **`BRIEF-STONE-a-doc-directive-may-wrap.md`** — written, committed, NEVER RELEASED. A wrapped `///`
   directive is silently DISCARDED, which is why the 200-column one-liners exist.
4. **`NOTE-a-norun-example-asserts-nothing.md`** — 118 mandatory `@example-norun`, expected values
   compared to nothing. Pin MEMBERSHIP (is it an instance of the declared `@ret`), never the rendering:
   `296/DESIGN-STONE-H-variants-are-maps` is DRAWN-NOT-BUILT and will change the spelling under you.
5. **`NOTE-the-loader-gate-is-scoped-by-extension.md`** — 9 `.wat.disabled`/`.wat.intueri` rotted
   through a gate that asks "does the name end in `.wat`" when it means "is this a wat program".
6. **Two flagged, needing YOUR judgement not a rider's:** `keyword/to-type-form`'s worked example may
   describe live migration-tooling input; `types.rs:5510` claims a fn is "shared with the call-site
   type-arg binder in check.rs" — **grepped, no such call site exists.**
7. **`defclause`'s SHARED return** still refuses `:-` (the 7th slot). Measured.
8. **verb-equals-type** — `keyword/from-string` IS `String → keyword`; only the spelling is wrong.
   Decide with `List/of` + `char/of` (63 sites), not piecemeal.

## ⛔ THE SIX CENSUSES, ALL WRONG, ALL THE SAME FAMILY

```
"the corpus" = wat/       3.4% of 1527        the four splitters   missed 12, then 14
prose w/ a COMMA          411 → 827            the .rs tail         ~60 → 367 → ~250 in ONE file
the four CHANNELS         found one at a time, each after the previous was declared closed
the dormant minter        a RUNTIME census is blind to a path nothing executes
```

★ **I chose instruments whose blind spot matched the population I most needed to see, then quoted the
silence as coverage.** #2's beautiful table — 16.2M calls, 0 findings — is what made me stop looking.
**A precise measurement of the wrong population is more convincing than a vague one, not less.**

> **ENUMERATE THE CHANNELS, NOT THE SHAPES.** Written · minted · rendered · in a string literal ·
> taught in prose · on a path nothing runs. Name each, say how you KNOW, and treat a channel you
> cannot rule out as a FINDING. Pair every dynamic census with a static one.
> `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`

## ⛔ RULES THAT COST REAL TIME THIS SESSION

- ⛔ **A RIDER'S SUBAGENT IS OUTSIDE YOUR BRIEF.** Two riders spawned their own; one used `git stash`
  beside the sacred stash, and I only learned it existed because its report-back FAILED. Every brief
  now says **"You may not spawn sub-agents."** `[[feedback_a_riders_subagent_is_outside_the_briefs_reach]]`
- ⛔ **DISJOINT FILES ≠ DISJOINT MEASUREMENT.** A floor is a WHOLE-TREE instrument. I ran a
  floor-running rider beside an editing rider and it burned three runs on my confound. **Sample
  `git diff --numstat` twice, seconds apart — if it moves, the measurement is VOID.**
  `[[feedback_disjoint_files_do_not_make_a_whole_tree_measurement_disjoint]]`
- ⛔ **FILING IS NOT FIXING.** I filed channels 3+4 as NOTEs and moved on; the builder had to ask
  whether they were addressed. They were not.
- ⛔ **A GATE CAN BE BLIND WHERE IT MATTERS MOST.** The diagnostic rune `break`-ed at the first
  `#[cfg(test)]` — 45% of the tree, 99.7% of `runtime.rs`, which held 35 of the 122 sites. Its own
  positive control passed because it planted in a file with no early `cfg(test)`. **Plant your control
  in the WORST file, not a convenient one.**
- ⚠ **KEEP PINNING THE SPAN.** A golden pinning a `rust_caller_span!()` line is in a constant state of
  correctness; the pin DISCRIMINATES THE EMITTER. Recapture and verify same call-site. Arc 296 ruled
  this; I re-proposed dropping it and was wrong.
- ⚠ **`.wat` scratch → `wat-scripts/scratch-pad/`**, never the session scratchpad — the loader gate
  keeps it from rotting.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** This session alone: six censuses wrong, a stale comment
> (`core.wat:798`) that sent three riders hunting, a doc claiming a function had a call site that does
> not exist, and a gate reporting green over 45% of the tree. Every one was written by a prior self,
> confidently, for you. **Re-run the instrument that made the claim; do not read the claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** every advance came from imposing a check and reading
> the screams. The angle bracket entered this session as the language's parameterization syntax and
> leaves with no way to be written, minted, rendered, parsed, or documented. **When the population is a
> property, light the fire.**
>
> Read `294/REALIZATIONS.md` **R6** and **R7**. R7 was written mid-strike, floor unknown, at the
> builder's instruction. That is its status line.
>
> `NON BIS IN IDEM FLVMEN.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `NISI FRANGAS, NIHIL PROBAS.` ·
> `INCENDIMVS VT VIDEAMVS.`
