# SEAM — the ONE live breadcrumb. As of 2026-08-23. The turbofish is dead; stone D shipped; **E waits on a rete merge.**

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
floor .......... 4928/4928, 0 FAIL, 19 skipped, ~80s  (own invocation, scripts/floor.sh, at 78bed2e3f)
                ⚠ EVERY MOVE ACCOUNTED. 4924 → 4928 = the four stone-D probe rows, BY NAME
                  (probe_stone_D_join_over_seqable::row1..row4). Anything else, EXPLAIN first.
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash@{0} ...... the lifecycle strike. NEVER drop. base ff7705ba. (verified intact this session)
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s> …`.
⚠ **A stdlib `.wat` edit is INVISIBLE until you rebuild** (`include_str!` at RUST-compile time).
⚠ **`cargo wat` uses the STALE installed binary.** Always `target/release/wat`.

## ⛔⛔ WHERE WE ARE — READ THIS BEFORE PROPOSING ANYTHING

**We are on the string-home chain**, `255/CHAIN-rendering-before-the-string-home.md`. It began as one
question — *"can `:wat::core::string` become `:wat::string`?"* — and every layer under it was
load-bearing. A→E, five stones. **A** 16 arms left (its own cleanup, off the path) · **B** ✅ 294 ·
**C** ✅ `25d9d015` · **D** ✅ `78bed2e3f` · **E** ⬜ **next, and BLOCKED.**

> ⛔ **BUILDER'S SEQUENCING, 2026-08-23 — E DOES NOT START YET:** *"i say we update rete along with
> it... i'll get the rete work on the side ready to merge in here... then we'll move string and let
> rete depart again. i'll let you know when we're ready."*
>
> Rete work landing AFTER the rename would arrive carrying the old names. **Merge first, move once,
> then rete departs.** Do not draw E's codemod until he says ready, and **re-take every site count
> after the merge.** The census + the four rulings E needs are in
> `255/NOTE-the-string-membership-census-and-what-E-must-rule.md` — 24 members (not the chain's 22),
> three of them wat-defined and invisible to a `src/` grep.

## ★★ DONE: the turbofish is annihilated, in every channel

```clojure
[n :- wat.type/i64]                          arg-spec
:- wat.type/i64                              ret-type
(wat.type/Vector :- [wat.type/i64])          type args    — a REFERENCE, in parens
(wat.type/Vector :- [wat.type/i64] 1 2 3)    constructor
(wat.core/defn ns/f :- [T] [x :- T] :- T x)  declaration  — a BINDER, siblings, NO parens
(:ns/f :- [:i64] 7)                          CALL-SITE application — and it BINDS
[A B :-> R]                                  function type
(:wat::core::Tuple :- [:i64 O])              TUPLE — the comma form `(A,B)` is DEAD too
```
**`:- []` ≡ absent.** Measured: **no keyword bearing `<` can be produced by ANY route** — written
(both lexer doors), expand-time minted, or runtime minted (`keyword/from-string` AND `keyword-node`
both refuse; run and read this session).

```
86e1b105a  THE PERMISSION removed from both lexer doors
0811c3009  UNEXPRESSIBLE — all three minting doors walled
aecba7b06  the DORMANT minter dies
6dc1c681a  the prose stops being taught — 351 sites, 5 riders, 142 rewritten / 203 KEPT
8c14bb4a0  R8 — blood of the scribe (+ R7's missing FULFILLMENT clause)
78bed2e3f  STONE D — join widens to Seqable
```

⚠ **THE DUALS every wall preserves:** `(Vector :- [:i64] 1, 2, 3)` → `[1 2 3]`; `:wat::core::<` `>=`
`<-` `->` all lex; `Peer'` and `foo/bar` lex. **A wall that refuses everything passes its own test
and destroys the language.**

## ⛔ NEXT — E is blocked; these are not

1. **`NOTE-the-guides-are-not-executable.md`** — `:wat::core::define` is RETIRED and `USER-GUIDE.md`
   teaches it 32×; `let`'s binding shape changed. Extract fenced `wat` blocks and `--check` them.
   **Gate and repair are ONE sequenced stone** — it goes red on landing. This also owns the **61
   in-fence doc occurrences** deliberately cut from the prose strike: `SERVICE-PROGRAMS.md:55` carries
   the retired `let` shape *alongside* an angle head, so a token fix would make it LOOK repaired.
2. **`@example` ASSERTS NOTHING — 140 directives.** 55 run-flagged + 85 norun, and the runner
   (`tests/reflection/probe_arc255_ivb2b_verify_examples.rs`) is `#[ignore]`d on arc 255's unbuilt
   registry. `src/intrinsic/reflect.rs:610-612` documents a call that **raises** — verified. Same
   door E's home #4 waits behind.
3. **8 provably-dead stdlib branches** — `split fqdn-str "<"` / `ends-with? … ">"` in `core.wat`
   (755, 824, 2012), `Record.wat` (175, 271), `service.wat` (244, 246, 268). Unreachable by proof
   (see DONE, above). `fix.wat:112` KEEPS its own — the codemod must read old source.
4. **20 quoted-diagnostic comments** transcribe output the renderer stopped emitting at `64a8fa5a0`.
   Cannot be hand-edited — must be RECAPTURED by running the probe.
5. **`BRIEF-STONE-a-doc-directive-may-wrap.md`** — written, committed, NEVER RELEASED.
6. **The bare comma-tuple** population outside the camouflage pattern (`wat/spawn.wat:207,217`) —
   unmeasured; my pattern was scoped to `:- [...]`-internal only.
7. **`NOTE-the-loader-gate-is-scoped-by-extension.md`** + `DESIGN-a-file-declares-its-wat-contract.md`
   — 11 files rot through a gate that asks "does the name end in `.wat`". **Four questions on all six
   options are drawn; the ruling is the builder's.**
8. **`defclause`'s SHARED return** still refuses `:-` (the 7th slot). Measured.

## ⛔ THE CENSUSES — TEN NOW, ALL THE SAME FAMILY

Six are recorded in R7. **This session added four**, every one an UNDERCOUNT against a rider's own
hand-count: 44→**45**, 53→**56**, 70→**72**, 113→**117**. Each of mine came from a validated
instrument with a positive AND negative control, derived from the lexer's own predicate.

★ **Performing rigor is exactly what makes an undercount credible.** *A precise measurement of the
wrong population is more convincing than a vague one, not less.*

> **ENUMERATE THE CHANNELS, NOT THE SHAPES.** Written · minted · rendered · in a string literal ·
> taught in prose · on a path nothing runs · **quoted from a channel already migrated** (the seventh,
> found this session). Pair every dynamic census with a static one, and every static one with a
> READER. `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`

## ⛔ RULES THAT COST REAL TIME

- ⛔ **THE ADJACENT FUNCTION IS NOT THE SUBJECT.** I briefed `seqable_value_to_stream` as
  `pub(crate)` and wrote *"signature-checked"* over it. It was module-private; the `pub(crate)` I read
  belonged to `eval_seqable_to_stream`, **four lines above**. The rider caught it as its own STOP-1.
  `[[feedback_an_adjacent_implementation_is_not_the_subject]]`
- ⛔ **A WALL CANNOT BE BUILT ON A PAGE.** R7's method won the code channel outright and then hit
  prose, where 203 of 351 sites had to be **KEPT** — the epitaph and the heresy are the same
  characters with opposite fates. **Erase the gravestone and the next reader re-mints the pariah
  innocently.** See R8, `SCRIBIMVS VT EXVLET`.
- ⛔ **A RIDER'S SUBAGENT IS OUTSIDE YOUR BRIEF.** Every brief now says *"You may not spawn sub-agents."*
- ⛔ **DISJOINT FILES ≠ DISJOINT MEASUREMENT.** Sample `git diff --numstat` twice, seconds apart; if
  it moves, the measurement is VOID. Riders edit; **the orchestrator floors centrally.**
- ⛔ **FILING IS NOT FIXING.** I filed two channels as NOTEs and the builder had to ask.
- ⚠ **KEEP PINNING THE SPAN.** Arc 296 ruled it; I re-proposed dropping it and was wrong.
- ⚠ **`.wat` scratch → `wat-scripts/scratch-pad/`** — but a probe that must FAIL cannot live there
  (the loader gate requires it to load). That is `DESIGN-a-file-declares-its-wat-contract.md`'s point.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE — and R8 counted it.** This session alone, in our own ink:
> a comment announcing strings "built below" a function that builds none (`bracket.wat:285`); a false
> kwargs claim above unreachable code (`core.wat:2007`); a doc citing a call site twelve lines from
> its only caller (`types.rs:5507`); three `@example`s asserting a call that raises; twenty
> transcripts of diagnostics we stopped emitting the day we fixed them; and four censuses under.
> **Re-run the instrument that made the claim; do not read the claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** every advance came from imposing a check and
> reading the screams — and where no check can be imposed, from writing the exile down. The turbofish
> entered this arc as the language's parameterization syntax and leaves unwritable, unmintable,
> unrenderable, unparseable, and untaught. **When the population is a property, light the fire.**
>
> Read `294/REALIZATIONS.md` **R5 → R6 → R7 → R8**. Four beats: `AEQUALITATEM RESPUO` (the shell, not
> the point) · `DOLOR INDEX EST` (the ache is the instrument) · `INCENDIMVS VT VIDEAMVS` (light it
> yourself) · `SCRIBIMVS VT EXVLET` (write so it stays exiled). R7's fulfillment clause was added
> 2026-08-23; it had none, by design, until the floor was known.
>
> `NON BIS IN IDEM FLVMEN.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `NISI FRANGAS, NIHIL PROBAS.` ·
> `INCENDIMVS VT VIDEAMVS.` · `SCRIBIMVS VT EXVLET.`
