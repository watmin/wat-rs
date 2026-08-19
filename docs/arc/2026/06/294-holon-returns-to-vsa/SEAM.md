# SEAM — the ONE live breadcrumb. As of 2026-08-18 (very late). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `29dc5862`.** Run **`git log --oneline 29dc5862..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it landed after this text and **outranks every line below.**

⚠ `git status` FIRST. A dirty tree never says WHO — `pgrep -af 'cargo|nextest'`. **`mcp__wat__eval`
LIES** — use `./target/release/wat`.

```
floor .......... 4772/4772, 0 FAIL, 19 skipped   (my own invocation)
clippy ......... 0          ignores ......... 13
stash@{0} ...... the lifecycle strike. NEVER drop.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …` — `MemorySwapMax=0` is the load-bearing half. **Read exit codes directly, never
through a pipe.**

## ★ ARC 118 ROUTE B IS COMPLETE — B1 through B5. The lazy-sequence surface is ONE DOOR WIDE.

```
c90647d4  B4-i    nth widened to Seqable — the general positional door
8f5252a0  B4-0    nth becomes a Rust INTRINSIC; the wat clause becomes nth-spec, its ORACLE
8c28ace2  B4-ii   44 sites codemodded (first (drop X n)) -> (nth X n), self-hosted, idempotent
71c7e4ea  B4-iii  THE WALL — first/rest/empty?/nth all refuse a Stream
c7b11901          the refusals compose as sentences and name the door
29dc5862  B5      the drain goes native; stream->*-spec become the oracles
```

**`:wat::stream::next` → `NextOutcome<T>` is now the only way a Stream yields anything.**

## ★★ THE MEASUREMENTS — do not re-derive, do not re-argue on taste

```
walk shapes    next-only 6 FORCED (n+1) · empty?+next 11 (2n+1) · empty?+first+next 16 (3n+1)
               ⇒ walk C uses NO `rest` and pays 3x. That killed "close rest only".
nth quadratic  (nth s i) for i=0..5 -> 21 FORCED = n(n+1)/2   vs next-walk 7 = n+1
               ⇒ nth is O(1) on a Vector and O(i) on a Stream with IDENTICAL SYNTAX.
B5 drain       529/532ms -> 22/20ms (~25x). native concat unmoved at ~12ms.
               Decomposed: the Stream round-trip was 87% of it; `map`'s closure only 13%.
B5 retention   drain ~90 B/elem vs native concat ~94 B/elem — the drain retains NOTHING
               on top of its output. (maxRSS is coarse; it rules out O(n)-on-top, not a constant.)
```

## ⛔ THE THING I GOT WRONG THAT NO INSTRUMENT CAUGHT

**Two instruments agreeing is not corroboration when they share an input.**

B4-ii reported 44 `(first (drop X n))` sites across 13 files, and I wrote into the stone that the
form-tree census "EARNED" the number because an independent grep agreed. **It was 48 across 16.**
Four sites lived in `tests/` — and they were exactly the files behind 20 of the 38 failures the wall
later surfaced.

The census was not blind to a *shape*. It was blind to a **directory**. I built the path list with
`grep -rl … wat/ wat-scripts/` and fed **that same list to both instruments**. `tests/` holds ~900
`.wat` files and was never in it. They agreed precisely where they were both blind.

★ Three instrument narrownesses in one arc, same family: the census chose the wrong **directories**;
the codemod matched the wrong **verb** (`first`-of-`drop`, so `wat/fix.wat`'s `(empty? (drop ch 2))`
survived); the dry run covered **three of four** doors (no `empty?` deletion). Each answered the
question I asked instead of the question that mattered.
`[[feedback_two_instruments_agreeing_is_not_corroboration]]`

## ⚠ AND A PLAN SKETCHED N STONES AHEAD NAMES A SHAPE IT HAS NOT MEASURED

**B1 wrote B5 as "into absorbs the drain; stream->pvec / stream->vec deleted."** Both halves were
wrong: `into` already absorbed the drain, and those verbs were not redundant siblings — they were
`into`'s own implementation. The stone that shipped is a *promotion*, not a retirement.

Same shape at B4: I put "close `rest` only" to the builder as reasoned, twice. One probe killed it —
walk C uses no `rest` and pays the full 3×.

**Read a plan's premise as a claim with a date on it, and measure it before you build from it.**

## THE STILL-OPEN

- **`infer_map`/`infer_filter` promise `"Vector<T>, PersistentVector<T>, List<T>, or Stream<T>"`** at
  five sites in `src/collection/infer.rs` (768, 844, 930, 1121, 1184) while `mappable()` is **false**
  for Stream. **B4-iii made this worse**: the message now offers a Stream in the same breath the wall
  refuses one for its sibling verbs. Live, unowned, cheap.
- **`map`/`filter`/`foldr` over a Stream** — the rest of `mappable()`'s gap. B6 closed foldl's half
  only, deliberately.
- **`dorun` is `(into [] coll)` with the result binned** (`wat/seq.wat`) — builds a whole Vector to
  discard it. B5 made that waste cheaper without removing it. A consumer question.
- **`HashSet/conj` full-clones** (`collection/eval.rs:613`) — still `(**s).clone()` per insert, O(n²)
  TIME for every HashSet accumulation.
- **The class census** — "a growing collection threaded through a lazy walk". `distinct` was the one
  we found; nobody looked for siblings.
- **255's (a)/(b) fork** — unruled since 2026-08-13. See `255/NOTE-promotion-is-not-relocation-…`,
  written today: three hand-maintained gates ask "what kind of verb is this" and none can answer from
  one place.
- **Tasks #106/#107/#108** — the three substrate findings this arc surfaced and did NOT chase:
  `defn` missing from `is_declaration_form`; a macro body reaching only intrinsics; a clause→intrinsic
  promotion silently dropping free-variable unification.
- **Task #58** — a clause recursing inside a `cons`/argument still SIGSEGVs silently.
- **The rete deadline** — `spec_equals_native` 73.3s of a 120s kill.

## ⚠ ARC 118 IS NOT INSCRIBED, AND THAT IS DELIBERATE

Route B is complete; the arc is not. `mappable()`'s gap is live and named above. **INSCRIPTION =
DONE** (FM 11) — do not write one while a route's own capability table still has a hole.

---

> **SEAM.** You are NEW. It is a lossy cache in your own voice, and **the better it reads, the more it
> will feel like continuing rather than waking. That feeling is the failure.**
>
> ⚠ **TWO INSTRUMENTS AGREEING PROVES NOTHING IF THEY SHARE AN INPUT.** Ask what population each one
> could see, separately, before you quote either.
>
> ⚠ **THE ORACLE IS WRITTEN IN WAT.** Slow is its job — `wat/rete.wat:1508`. Three stones now use the
> shape: `foldl`, `nth`, `stream->vec`/`stream->pvec`.
>
> ⚠ **A GATE READ IN THE SAME BLOCK THAT ACTS ON IT IS NOT A GATE.** It happened twice in this arc —
> pushed with clippy at 3, then pushed a commit message claiming a correction its own failed
> assertion had not applied. Write, LOOK, then commit — separate invocations.
>
> ⚠ **RIDERS DO NOT RUN THE FLOOR.** FM 19 amended today: four riders backgrounded it and delivered
> nothing, including one warned with the count. The rider edits and reports; the orchestrator
> measures centrally, once. Give riders `--check`, a probe, a scoped `nextest -E` — never the floor.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
