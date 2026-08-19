# SEAM — the ONE live breadcrumb. As of 2026-08-18 (end of day). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `0f44ff7c`.** Run **`git log --oneline 0f44ff7c..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`. **`mcp__wat__eval` LIES** — use `./target/release/wat`.

```
floor .......... 4772/4772, 0 FAIL, 19 skipped   (own invocation)
clippy ......... 0          ignores ......... 13
stash@{0} ...... the lifecycle strike. NEVER drop.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …` — `MemorySwapMax=0` is load-bearing. **Read exit codes directly, never through a pipe.**

## ★ THE FRONTIER — 118.B6b IS DRAWN AND BRIEFED. STRIKE-READY. DO NOT RE-DERIVE IT.

**Retire `:wat::core::foldr`.** Builder ruled it. Artifacts committed:

```
DESIGN-STONE-118.B6b-retire-foldr.md     the ruling, the prior art, three traps
BRIEF-STONE-118.B6b-retire-foldr.md      four rooms, five steps, four STOPs
EXPECTATIONS-STONE-118.B6b.md            rows 6-8 are the ORCHESTRATOR's
```

**The reasoning, so you do not rebuild it:** our `foldr` is *correct* — `(foldr - 0 [1 2 3])` → `2`.
Correctness was never the defect. It is literally `xs.iter().rev()` + accumulate — `reverse` then
`foldl` — wearing a name borrowed from Haskell, where the verb is distinct **only because it is lazy**
(`foldr f z (x:xs) = f x (foldr f z xs)` puts the recursion in an ARGUMENT, so `foldr (||) False
(repeat True)` returns at once). wat is strict; that property cannot exist here. Clojure and Ruby,
both strict, both declined to name it. The replacement is `(reduce f init (reverse coll))` — and
`reduce` **already IS `foldl`** (`wat/seq.wat:308`, since B7), so nothing is renamed.

⚠ **The Haskell/Clojure/Ruby claims are from KNOWLEDGE, not fetched.** Everything wat-side is measured.

## ★ ARC 118 ROUTE B IS COMPLETE — B1 through B5

```
c90647d4 B4-i  nth widened   8f5252a0 B4-0  nth native + wat oracle
8c28ace2 B4-ii 44-site codemod, self-hosted, idempotent
71c7e4ea B4-iii THE WALL — first/rest/empty?/nth all refuse a Stream
c7b11901       refusals compose and name the door
29dc5862 B5    the drain goes native (529ms -> 22ms)
```

`:wat::stream::next` → `NextOutcome<T>` is the only way a Stream yields anything.

## ★★ MEASUREMENTS — do not re-derive

```
walk shapes     next-only n+1 · empty?+next 2n+1 · empty?+first+next 3n+1  (walk C uses NO rest)
nth quadratic   (nth s i) i=0..5 -> 21 FORCED = n(n+1)/2   vs next-walk 7 = n+1
B5 drain        529/532ms -> 22/20ms (~25x). Decomposed: round-trip 87%, map's closure 13%.
B5 retention    drain ~90 B/elem vs native concat ~94 — retains NOTHING on top of its output
HashSet/conj    O(n^2) MEASURED: 12/53/223/875ms at n=2k/4k/8k/16k (x4 per doubling)
                109/NOTE-hashset-conj-full-clones-per-insert.md. Builder: "we chase it later."
```

## ⛔ TWO SEAM LINES I WROTE THIS MORNING WERE STALE — I "re-verified" them wrongly

**The previous seam claimed "map/filter/foldr over a Stream" was an open gap.** Measured: `map`,
`filter`, `take`, `drop`, `remove` **all accept a Stream and run** — proven end to end. Only `foldr`
refused, and it is now being retired. The gap was one verb, not three.

**It also claimed `infer_map`/`infer_filter` "promise `or Stream<T>` while `mappable()` refuses it."**
All five sites carrying that string are in `infer_map`/`infer_filter`/`infer_foldl`/`infer_take`/
`infer_drop` — **every one of which accepts a Stream.** The messages are accurate.

★ **I "re-verified" that one by grepping that the string EXISTED — which confirms the words, not the
falsehood.** Checking what an instrument says instead of what it means, during the very act of
claiming re-verification. `[[feedback_two_instruments_agreeing_is_not_corroboration]]`

## THE STILL-OPEN — re-measured today, not inherited

- **B6b** — drawn, briefed, strike-ready. The frontier.
- **`ordered()`'s header is stale** (`seq_container.rs:277`): says it gates `reverse`/`take`/`drop`/
  `concat`; has exactly TWO live consumers, `concat` (`eval.rs:763`) and `reverse`
  (`transform.rs:51`). `take`/`drop` moved off it in 118.2a — `infer.rs:1070` records the move.
  **Folded INTO B6b's scope** (same file as `mappable()`'s stale header). Not a routed-around
  capability; a comment that outlived its subject.
- **`dorun` is `(into [] coll)` with the result binned** — builds a Vector to discard it. B5 made the
  waste cheap without removing it. A consumer question.
- **`HashSet/conj` O(n²)** — noted in 109 with the numbers. Parked by ruling.
- **The class census** — "a growing collection threaded through a lazy walk". `distinct` was the one
  we found; nobody looked for siblings.
- **255's (a)/(b) fork** — unruled since 2026-08-13. See `255/NOTE-promotion-is-not-relocation-…`.
- **Tasks #106/#107/#108** — `defn` missing from `is_declaration_form`; a macro body reaching only
  intrinsics; clause→intrinsic promotion dropping free-variable unification.
- **Task #58** · **the rete deadline** (`spec_equals_native` 73.3s of a 120s kill).

## ⚠ ARC 118 IS NOT INSCRIBED, DELIBERATELY

Route B is complete; the arc is not, until B6b lands. **INSCRIPTION = DONE** (FM 11) — do not write
one over an open stone.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **TWO INSTRUMENTS AGREEING PROVES NOTHING IF THEY SHARE AN INPUT.** A form-tree census and a grep
> both said 44 sites; I had fed both the same path list and `tests/` (~900 `.wat` files) was not in it.
>
> ⚠ **A BORROWED NAME IMPORTS ITS HOME LANGUAGE'S SEMANTICS.** `foldr` was correct on every input and
> still wrong to have. `109/NOTE-a-borrowed-name-can-import-a-property-the-language-lacks.md`.
>
> ⚠ **A PLAN SKETCHED N STONES AHEAD NAMES AN UNMEASURED SHAPE.** B1 wrote B5 as "delete the two
> redundant verbs"; they were `into`'s own implementation and the stone shipped the opposite.
>
> ⚠ **A GATE READ IN THE SAME BLOCK THAT ACTS ON IT IS NOT A GATE.** Twice this arc. Write, LOOK, commit.
>
> ⚠ **RIDERS DO NOT RUN THE FLOOR.** FM 19 amended: four riders backgrounded it and delivered nothing.
> Give them `--check`, a probe, a scoped `nextest -E`. The orchestrator measures centrally, once.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
