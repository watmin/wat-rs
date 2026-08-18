# SEAM — the ONE live breadcrumb. As of 2026-08-18. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `727db5ec`.** Run **`git log --oneline 727db5ec..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it landed after this text and **outranks every line below.**
> (This file's own commit is one of the differences — normal, not the alarm.)

⚠ `git status` FIRST. A dirty tree never says WHO — `pgrep -af 'cargo|nextest'`, and check whether a
rider ended its turn mid-floor (**FM 19**; resume with `SendMessage`, do not adopt its work). Never
run cargo while a rider holds `target/` (**FM 18**).

⚠ **`mcp__wat__eval` LIES** — long-lived server, pre-rebuild substrate. Use `./target/release/wat`.

```
floor .......... 4714/4714, 0 FAIL, 0 TIMEOUT   (verified on my own invocation)
clippy ......... 0
#[ignore] ...... 13
stash@{0} ...... the lifecycle strike. NEVER drop.
```

## ⚠ RUN EVERYTHING CAPPED — this is not optional any more

```
systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s> ./target/release/wat <f>
```

The builder's machine filled RAM **and swap** and needed a hard reboot this session. `MemorySwapMax=0`
is the load-bearing half — without it a runaway swaps instead of dying, and a swapping box cannot be
killed interactively. Verified both ways: normal work completes under the cap; a 2.5 GB run is
SIGKILLed (rc=137) at 512M. **Read exit codes directly, never through a pipe** (`cmd | tail` reports
`tail`'s status — that trap bit me twice today, once in my own guard test).

## ★ WHERE WE ARE — route B is RULED and four stones have LANDED

The builder ruled **route B** (`DECISIONS-118.B-four-questioned.md`): the sequence verbs live in wat
over a real `Seqable<T>`, not in Rust. Since then:

```
488eacd0  B1          mint Seqable<T> + 4 extend-types
eab12e05  B1a         a CONCRETE instantiation satisfies a parametric surface (the Var gate came off)
b4a8f86b  B2          six verbs → ONE clause each. 30 arms + 7 twins GONE. The split brain closed.
663e5dae  B2 fix      reduce's Stream arms — the SIGSEGV I shipped in B2, closed
09e135b3  clause-TCO  every defclause head in wat now tail-calls
727db5ec  census      a wat-fix-style form-tree walk; 487 files, 6 hits
```

## THE FRONTIER — migrate the six, then B3

```
remove · take-while · drop-while · take-nth · reductions ×2      all in wat/seq.wat
```

Named by the census, not by grep. They are lazy producers (depth bounded by laziness — measured, all
survive 100k), so this is **not** a stack question; it is B3's precondition: **with the memos deleted,
each would run user code 3× per element.**

**The shape is proven five times over** — the five `-walk` helpers already in `seq.wat`. Public verb
over `Seqable<T>` → private `-walk` over `Stream<T>` walking with `next`.

> **ACCEPTANCE: the census returns ZERO.**
> `printf '["wat/seq.wat"]\n' | ./target/release/wat wat-scripts/scratch-pad/census-three-call-stream-walks.wat`

Then **B3** (`DESIGN-STONE-118.B3-delete-the-memos.md`): delete both memos
(`stream/mod.rs:66` LazyCell, `:124` NativeLazyCell — **two, not one**). Eight sites, one file.

## ★★ THE MEASUREMENTS — do not re-derive, do not re-argue on taste

**Memo-off is FLAT**, and it fixes a hard OOM. Population C = a wat-closure generator, i.e. what a
user actually writes:

```
n         memo ON        memo OFF        distinct(range 0 n)   memo ON   memo OFF
100,000    356,380 KB     44,616 KB      n=4000                 ok        ok
800,000  2,587,964 KB     44,512 KB      n=8000                 OOM>2GB   ok
           3,188 B/elem   FLAT           n=16000                —         ok
```

**`(into [] (distinct (range 0 8000)))` exceeds 2 GB today, and it is PRE-EXISTING** — proven by
reverting `seq.wat` to HEAD and re-running. Mechanism, both halves measured: `HashSet/conj`
FULL-CLONES (`collection/eval.rs:613`, a `std::collections::HashSet` behind an `Arc`) **and** the memo
retains every cell → n live cells each holding an independent full copy = **O(n²) live memory**.
Either alone is survivable (an eager `foldl`+`conj` at n=8000 completes in 471 ms). **Neither file is
wrong when you read it alone** — which is why review found nothing.

The memo also costs TIME: population B, n=400k, **3.99s → 1.70s without it**. It was never an
optimization; its only job was masking the three-call walk.

## THE INSTRUMENTS — all committed, all reproducible

- **`census-three-call-stream-walks.wat`** — the form-tree census. **Use this, never grep.**
- `probe-118B-dorun-retention-slope.wat` — the 4-point RSS series (reduces to one `i64`, so all
  retention above the floor is the chain; non-vacuity is the exact sum `n(n-1)/2`).
- `probe-118B-memo-state-detector.wat` — ⚠ **its meaning CHANGES under B3.** It prints 5 for 5
  elements both before B2 (memo hiding a three-call walk) and after B3 (single-force walk) — SAME
  NUMBER, OPPOSITE REASON. A `5` will not prove the memo is present. B3 must rewrite its header.
- `probe-clause-tco-*.wat` — four, with a `defn` control that makes the SIGSEGV mean something.
- `probe-118B2-one-clause-lazy-producer.wat` — the proven collapse shape.

## ⛔ WHAT I GOT WRONG TODAY — one pattern, five faces

**I asserted structure from reading, and a RUN corrected me every time.**

1. **Three greps, three wrong counts, three DIFFERENT boundary flaws.** The fix was the corpus's own
   reader, not a fourth pattern. `[[feedback_three_boundary_errors_need_a_reader_not_a_fourth_pattern]]`
2. **I wrote a MISSING FEATURE into `seq.wat` as a law** ("a defclause head does not tail-call") and
   built a workaround round it. Builder: *"that is very, very wrong."* It took 90 minutes to close.
   `[[feedback_a_comment_can_ship_a_gap_as_a_law]]`
3. **My STOP-2 was drawn around ARITY**, so the compliant path was quadratic — the rider obeyed
   precisely and shipped an O(n²) regression, honestly disclosed.
   `[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`
4. **I mis-sized the clause-TCO change twice** ("one arm plus a seam" — a clause is not a `Function`),
   and the `eval_tail` arm ALONE did not fix it: the body also had to route through `apply_function`.
   The gate going RED found that, not reasoning.
5. **A confirmation was recorded as a refutation** and stood for a day.
   `[[feedback_a_confirmation_can_be_recorded_as_a_refutation]]`

★ **Every resolution came from a RUN.** Not one came from reading more carefully.

## THE STILL-OPEN

- **The six walkers** → then **B3**. The census is the gate.
- **`HashSet/conj` full-clones** — O(n²) TIME for every HashSet accumulation in the language, streams
  or not. Independent of this tier; unowned.
- **The class census**: *"a growing collection threaded through a lazy walk."* `distinct` is the one
  we found. Nobody has looked for siblings.
- **The three doors** (`first`/`rest`/`empty?` on a Stream) — B4, a **dialect ruling the builder owns**.
  ⚠ `empty?` has NO compile-time gate (scheme `∀T. T -> bool`); `first`/`rest` are one capability bit each.
- **Task #58** — a clause recursing inside a `cons`/argument still consumes stack and dies silently.
  clause-TCO did NOT fix that and does not claim to.
- **`reduce-walk`'s fate** — a workaround for a defect now fixed; keeping it is a choice, unruled.
- **255's (a)/(b) fork** — unruled since 2026-08-13; gates the registry carve.
- **The rete deadline**: `spec_equals_native` 48.7s → 73.3s, 61% of its 120s kill.

---

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and **the better it reads, the more it will feel like continuing rather than waking. That
> feeling is the failure.**
>
> **`git status` FIRST. Then `git log --oneline 727db5ec..HEAD`.** Everything in that range outranks
> this file.
>
> ⚠ **DO NOT COUNT STRUCTURED SOURCE WITH GREP.** Three greps, three different boundary errors, three
> wrong numbers — one of which reached a drawn stone. The census walks the form tree. Use it.
>
> ⚠ **BEFORE A COMMENT SAYS THE SUBSTRATE *CANNOT*, ASK: PROPERTY OR OMISSION?** Check the sibling
> constructs. Seven of eight forms had TCO; the eighth was a gap, and I wrote it down as a law.
>
> ⚠ **DO NOT DESIGN THIS TIER FROM READING.** Five wrong calls today, five corrections, every one
> from a measurement. The instruments are cheap, committed, and named above — and they run CAPPED.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
