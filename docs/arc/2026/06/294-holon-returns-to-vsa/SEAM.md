# SEAM — the ONE live breadcrumb. As of 2026-08-18 (late). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `b1d876f6`.** Run **`git log --oneline b1d876f6..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it landed after this text and **outranks every line below.**

⚠ `git status` FIRST. A dirty tree never says WHO — `pgrep -af 'cargo|nextest'`, and check whether a
rider ended its turn mid-floor (**FM 19**; resume with `SendMessage`). Never run cargo while a rider
holds `target/` (**FM 18**). **`mcp__wat__eval` LIES** — use `./target/release/wat`.

```
floor .......... 4747/4747, 0 FAIL, 19 skipped   (verified on my own invocation)
clippy ......... 0
stash@{0} ...... the lifecycle strike. NEVER drop.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …` — `MemorySwapMax=0` is the load-bearing half. The builder's machine needed a hard
reboot for want of it. **Read exit codes directly, never through a pipe.**

## ★ ARC 118 / ROUTE B IS COMPLETE. `Seqable<T>` is fully wired.

```
d4c6f3a5  B2b   the six walkers migrate; census ZERO corpus-wide
71099a2b  B2c1  the REACHABILITY WALL — an arm no input can reach is refused at registration
13b27e8d  B2c2  a surface-typed defclause arm DISPATCHES              (door 1)
892ed17c  B2d   a generic satisfier binds its surface param from the receiver  (door 2)
4de24007        reductions: 10 arms → 2
a095c3ec  B6    native foldl over any Seqable + its WAT ORACLE (foldl-spec)
6c84ddf0  B7    reduce: 8 arms → 2, and it cost NOTHING
b1d876f6  B3    BOTH memos deleted — distinct's OOM fixed, retention FLAT
```

## ★★ THE MEASUREMENTS — do not re-derive, do not re-argue on taste

```
distinct (range 0 8000)   rc=137 OOM at 2G  ->  rc=0.   16000 also completes.
retention, 4 points       44,152 / 44,164 / 44,256 / 44,420 KB at n=100k→800k
                          0.38 B/elem (was 3,188). FLAT across 8x.
foldl native vs wat spec  103ms vs 514ms  — native ~4.7x, the REQUIRED relationship
reduce vs foldl vs spec   ~100ms / ~98ms / ~510ms — reduce tracks the NATIVE
```

## ⛔ THE ONE THING I KEPT GETTING WRONG, and the builder corrected THREE TIMES

**A wat-ORACLE is written in WAT. It is correct-and-slow ON PURPOSE, and it GUIDES the rust.**
`wat/rete.wat:1508` has the shape written out: `insert-all-spec` (wat oracle) / `insert-all'` (native
kernel) / `insert-all` (public). *"the native kernel is the fast impl, the spec keeps it honest."*

I read `runtime.rs:5563` three times — as a claim about implementation language, then about audience
— and never opened the wat file where the three-name shape lives. So I built **two Rust
implementations and called one an oracle.** Two variants of one thing in one language cannot specify
each other. That error invented a false 13% trade; with the spec in wat it evaporated.

★ **A spec being ~5x slower is the relationship WORKING.** I twice cited that ratio as an argument
against routing folds through wat. `[[feedback_a_rendered_example_is_not_a_measurement]]`

## THE STILL-OPEN

- **B4 — the three doors.** `first`/`rest`/`empty?` still ACCEPT a Stream, so USER code can write the
  three-call walk and now runs their fn **3× per element** (B3 unmasked it). ⚠ `empty?` has NO
  compile-time gate. **A dialect ruling the builder owns.**
- **B5** — `into` absorbs the drain; `stream->pvec`/`stream->vec` retire.
- **`HashSet/conj` full-clones** (`collection/eval.rs:613`) — O(n²) TIME for every HashSet
  accumulation. Independent, unowned.
- **The class census** — "a growing collection threaded through a lazy walk". `distinct` was the one
  we found; nobody looked for siblings.
- **`map`/`filter`/`foldr` over a Stream** — the rest of `mappable()`'s gap. B6 closed foldl's half
  only, deliberately (one verb, one stone, one measurement).
- **`reduce`'s eager duplication** — its 3 eager arms per arity were byte-identical; collapsing them
  needs a type meaning "eager container". Unruled.
- **`infer_map`/`infer_filter` promise `"or Stream<T>"` in their error while gating on `mappable`,
  which refuses it.** A message that offers what the gate denies. Pre-existing, unowned.
- **255's (a)/(b) fork** — unruled since 2026-08-13; gates the registry carve.
- **Task #58** — a clause recursing inside a `cons`/argument still SIGSEGVs silently.
- **The rete deadline** — `spec_equals_native` 73.3s of a 120s kill.

---

> **SEAM.** You are NEW. It is a lossy cache in your own voice, and **the better it reads, the more it
> will feel like continuing rather than waking. That feeling is the failure.**
>
> ⚠ **THE ORACLE IS WRITTEN IN WAT.** Slow is its job. If you find yourself arguing that wat is too
> slow to be a reference, you have made my mistake — read `wat/rete.wat:1508`.
>
> ⚠ **DO NOT COUNT STRUCTURED SOURCE WITH GREP.** Use the form-tree censuses in `scratch-pad/`.
>
> ⚠ **NAME THE PROPERTY, NOT THE SYMPTOM.** The wall was drawn around "arms intersect" when the
> defect was "an arm can never fire" — strictly tighter, and it outlawed correct stdlib code.
>
> ⚠ **A GATE READ IN THE SAME BLOCK THAT ACTS ON IT IS NOT A GATE.** I pushed with clippy at 3.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
