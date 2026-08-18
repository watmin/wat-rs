# SEAM — the ONE live breadcrumb. As of 2026-08-18 (early). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

⚠ **THE MARKER IS A DIFF INSTRUCTION, NOT A PASS/FAIL.**

> **Written against `488eacd0`.** Run **`git log --oneline 488eacd0..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it landed after this text and **outranks every line below.**
> (This file's own commit is one of the differences — normal, not the alarm.)

⚠ `git status` FIRST. A dirty tree never says WHO — `pgrep -af 'cargo|nextest'`, and check whether a
rider ended its turn mid-floor (**FM 19**; resume with `SendMessage`, do not adopt its work). Never
run cargo while a rider holds `target/` (**FM 18**).

⚠ **`mcp__wat__eval` LIES** — long-lived server, pre-rebuild substrate. Use `./target/release/wat`.
⚠ **`target/release/wat` may be STALE** — check its mtime against `src/`.

```
floor .......... 4713/4713, 0 FAIL, 0 TIMEOUT   (verified on my own invocation)
clippy ......... 0
#[ignore] ...... 13
stash@{0} ...... the lifecycle strike. NEVER drop.
```

## ★ WHERE WE ARE — route B is RULED and stone 1 of 5 has LANDED

Builder, 2026-08-17: *"B has been reasoned…. do your measurements and build."*

The collection/stream tier is the active front. The seven `-stream` twins were **already ruled a
crutch** on 2026-07-31 on the builder's own challenge (`278/DESIGN-STONE-seq-traversal-one-door.md`:
*"The twins are a workaround for the missing type, not a pattern"*). That stone chose **native**;
the fork was **re-posed and re-ruled** because its premise had expired (below). **Route B won.**

```
B1  mint Seqable<T> + 4 extend-types              ✅ LANDED 488eacd0
B2  collapse each verb to ONE Seqable clause,     ← THE FRONTIER
    body walking with `next`; the 7 twins AND
    `seqable->stream` die here
B3  delete BOTH memos. measure.
B4  close the three doors (first/rest/empty? on Stream → unrepresentable)
B5  `into` absorbs the drain; stream->pvec / stream->vec deleted
```

**The split is chosen so NO BODY IS WRITTEN TWICE.** An earlier sketch migrated the 8 walkers to
`next` and *then* collapsed them into `Seqable` clauses — the same bodies, two stones.
`[[feedback_do_not_defer_content_on_mechanisms_difficulty]]`

**A name dies in the stone that removes its last caller** — that is why `seqable->stream` survived B1
and dies in B2. It is not the preserve-and-extend reflex; it is the ordering rule.

## ★★ THE MEASUREMENTS — do not re-derive, and do not re-argue on taste

**Memo-off is FLAT.** Population C = a wat-closure generator, i.e. the builder's Ruby-Enumerator
idiom, i.e. what a user actually writes. Every sum exact at every point in both columns:

```
n            memo ON        memo OFF
100,000      356,380 KB      44,616 KB
400,000    1,312,572 KB      44,400 KB
800,000    2,587,964 KB      44,512 KB
             3,188 B/elem    FLAT (−0.15 B/elem over an 8× range) — 58× smaller at 800k
```

**And the memo costs WALL CLOCK too** — population B (native map), n=400k: **3.99s → 1.70s memo-off,
2.35× FASTER.** The floor control did not move (0.69→0.71s), so it is not machine state. **The memo
is not an optimization. It is a patch that costs on both axes, and its ONLY function is masking the
three-call defect.**

**Intervention control, run before any of it:** memo ON → `"FORCED"` ×5 for 5 elements; memo OFF →
×15. That is the bypass proving itself, AND the three-call defect made visible. Instruments, all
committed and loader-gated: `wat-scripts/scratch-pad/probe-118B-{memo-state-detector,dorun-retention-slope,match-tco-drain,match-no-tco-control}.wat`.

**TCO under `match` is PROVEN** — a 200k tail drain completes; the non-tail control SIGSEGVs (139).
The control is what makes the pass non-vacuous. B2 rewrites `if`+three-call → `match (next s)`, so
this was the trap that had to be disarmed first.

## ⛔ TWO CORRECTIONS TO THE RECORD MADE TODAY — both were propagating

**1. "The O(1) prediction was wrong" was ITSELF WRONG**, and it sat in FOUR docs including this seam.
`DESIGN-118.10`'s own table: memo-on **585** B/elem · memo-off **288** · eager `mapv` **with no
stream at all** **288**. Memo-off equalled a program containing no stream — **zero stream retention,
exactly as predicted.** "Eager parity" was the SUCCESS condition; `into`/`mapv` materialize by
contract, so that instrument was O(n) under *any* implementation and **could never have shown total
O(1).** A clean confirmation was filed as a refutation and became a standing reason to distrust the
fix. `[[feedback_a_confirmation_can_be_recorded_as_a_refutation]]`

**2. A RULING'S PREMISE EXPIRED WHILE THE RULING STOOD.** Arc 278 scored `Seqable` a flat NO on
Simple over three blockers, and chose native on that basis. All three were refuted by 118.3-B
(`a15f4ea9`) — but `109/NOTE-seqable-has-no-name-in-wat.md`, the artifact that FILES the decision,
carried **0** refutations while `infer.rs` carried **3**. **Amended today**, per claim, original text
preserved. `[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`

## THE FRONTIER — B2, and what it must do

Collapse each lazy verb to **ONE** `defclause`/`defn` over `Seqable<T>`, body walking with
`:wat::stream::next`. Delete the 7 twins, their ~29 byte-identical arms, `seqable->stream`, and
`extract_lazyable_elem` (`infer.rs:665` — its own doc says it *"is exactly what that stone would
delete"*).

⚠ **R21: this is a multi-site structural `.wat` rewrite — it is a wat-fix codemod, NOT hand edits.**
Framework `wat/fix.wat`; copy a recorded migration from `wat-scripts/fixes/`; dry-run on a `/tmp`
copy and `diff` before applying.

> **B2 ACCEPTANCE, both required:** every verb's observable behaviour unchanged (the floor),
> **AND** the twins/arms actually gone (a count, not a claim).
> B3's acceptance is the separate pair: **`f` runs exactly N times** — AND — the per-element slope
> reaches the flat column above.

**The eight walkers** (`Stream<` appears in `wat/` in ONE file): `stream->pvec:102` ★ the drain,
every eager materializer funnels through it · `reduce-stream:180` · `interpose-stream:412` ·
`keep-stream:454` · `keep-indexed-stream:481` · `map-indexed-stream:512` · `dedupe-stream:540` ·
`distinct-stream:571`. **Eight, not the seven the `-stream` suffix counts.**

## TRAPS THAT ARE NOW KNOWN — each cost a run to find

- **A dotted call head is NOT type-checked at all** (task #95, confirmed live today): `--check` on
  `(:wat::core::Seqable/seq …)` **passed before the type existed.** Any gate row over a dotted head
  must be a **RUN**, never a `--check`.
- **`empty?` cannot be walled at compile time** — its scheme is `∀T. T -> bool` and its runtime arm
  is a hardcoded `if let` **bypassing the capability table** (`runtime.rs:17064`). `first` and `rest`
  are one capability bit each (`indexable()` / `has_tail()`). B4 is therefore asymmetric work.
- **TWO memos, not one:** `LazyCell.forced` (`stream/mod.rs:66`, wat closure) and
  `NativeLazyCell.forced` (`:124`, Rust closure). B3 must kill both.
- **A builtin `extend-type` inside the stdlib now has precedent** (B1 set it). Before B1 there were
  zero, and every proof was a user-level file loaded after the whole stdlib.
- **Native walkers never had the three-call defect** — `transform.rs:181/:246/:248` realize ONCE and
  destructure. The disease is wat-side only; `next` is wat getting what Rust always had.

## THE STILL-OPEN

- **B2 → B5**, in order. B3 cannot precede B2: a surviving three-call walker runs user code **3×**,
  measured, not theorized.
- **`dorun` builds a Vector and bins it** · **`length` on a Stream type-checks then RAISES** ·
  **`first` on an exhausted Stream returns bare `nil`** — leaves of B2/B4, not separate stones.
- **`keep-stream`'s `None` arm** recurses in Rust on a long run of drops — pre-existing, unmeasured,
  same silent-SIGSEGV class as tasks #58/#86. Not B2's scope; should not be found by a user.
- **The rete deadline:** `spec_equals_native` went 48.7s → **73.3s** on one grok push — 61% of its
  120s kill. Durable fix named in `.config/nextest.toml`.
- **255's (a)/(b) fork** — unruled since 2026-08-13; gates the registry carve.
- **#101** Thermometer-as-record · **#102** `#holon {:a "b"}` broken · **#91** HolonAST census.

---

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and **the better it reads, the more it will feel like continuing rather than waking. That
> feeling is the failure.**
>
> **`git status` FIRST. Then `git log --oneline 488eacd0..HEAD`.** Everything in that range outranks
> this file.
>
> ⚠ **A BLOCKER NOTE IS A CLAIM WITH A DATE ON IT — AND SO IS THE RULING BUILT ON IT.** Today a
> 4–0 verdict was found standing on three blockers that had all died, in a document nobody updated.
> Before inheriting any decision, check whether its *reasons* are still true.
>
> ⚠ **AN INSTRUMENT THAT CANNOT SHOW THE RESULT WILL REPORT A NULL, AND THE NULL WILL BE READ AS A
> NO.** A correct confirmation spent a day on the record as a refutation. Ask what your probe's own
> scaffolding allocates before you trust what it measured.
>
> ⚠ **DO NOT DESIGN THE STREAM TIER FROM READING.** Every resolution here came from a run — a
> 20-line probe, a four-point RSS series, a throwaway variant build, a function that prints.
> The instruments are cheap, committed, and named above.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
