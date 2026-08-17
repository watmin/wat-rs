# SEAM — the ONE live breadcrumb. As of 2026-08-17 (late). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

⚠ **THE MARKER IS A DIFF INSTRUCTION, NOT A PASS/FAIL.**

> **Written against `0d651715`.** Run **`git log --oneline 0d651715..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it landed after this text and **outranks every line below.**

⚠ `git status` FIRST. A dirty tree never says WHO — `pgrep -af 'cargo|nextest'`, and check whether a
rider ended its turn mid-floor (**FM 19**; resume with `SendMessage`, do not adopt its work). Never
run cargo while a rider holds `target/` (**FM 18**).

⚠ **`mcp__wat__eval` LIES** — long-lived server, pre-rebuild substrate. Use `./target/release/wat`.
⚠ **`target/release/wat` may be STALE** — check its mtime against `src/`. It prints a warning.

```
floor .......... 4707/4707, 0 FAIL, 0 TIMEOUT   (verified on my own invocation)
clippy ......... 0
#[ignore] ...... 13
stash@{0} ...... the lifecycle strike. NEVER drop.
grok ........... in sync; two pushes today, both floored by us
```

## ★ WHERE WE ARE — the collection/stream tier, and it is the active front

Chain A–E's join half is **done** (279.3 `8b2cdbf2`, 279.4 `94142919`). Everything since is the
**stream tier**, and it turned out to hold two independent defects that had to be found before
`Seqable` could be built at all.

**JUST LANDED — 118.11a (`0d651715`): the pull primitive.**

```wat
(:wat::stream::next s) -> :wat::stream::NextOutcome<T>
   :Item      [value <- T  rest <- :wat::stream::Stream<T>]
   :Exhausted []
```

**One force per call, measured with a control** (two `next` → two forces). Purely additive: the memo
is untouched, nothing migrated. **That was deliberate** — see stone B.

## ★★ THE MEASURED CHAIN — why this exists. Do not re-derive it.

```
the walk protocol is  empty? → first → rest        THREE forces of ONE cell
  ⇒ USER CODE RUNS 3× PER ELEMENT                  measured: 15 calls for 5 elements
  ⇒ patch: cache the cell (forced: OnceLock)       measured: restores exactly 5 for 5
  ⇒ the cache links every cell to its tail
  ⇒ the head pins the whole realized chain
  ⇒ +297 B/element, linear                         measured: memo-off == eager mapv, within
                                                   200 KB on 326 MB. The memo IS the overhead.
```

**Neither arm ships:** memo-on is silently wrong for effectful `f`; memo-off OOMs (585 B/elem → 10M
items ≈ 5.8 GB). `next` is the only shape that gets both, because one force is **structural** —
nothing to dedupe, so no cache to retain.

Leaves of the same root, all still open: **`dorun` is `(do (into [] coll) nil)`** — the verb that
exists to avoid the allocation pays it in full · **`length` on a Stream type-checks then RAISES** ·
**`first` on an exhausted Stream returns bare `nil`**.

## THE FRONTIER — stone B, and its acceptance is TWO numbers

Migrate the 7 `-stream` twins + the drain verbs from three-call to `match`, **then** delete the memo.
Order is load-bearing: the memo cannot die while anything still walks with three calls.

> **ACCEPTANCE: `f` runs exactly N times for N elements — AND — the per-element delta reaches the
> 288 B eager baseline.** Both, or it has not landed. Instrument:
> `/usr/bin/time -f 'maxRSS=%M KB'` at 100k/250k/500k/1M.

⚠ **O(1) is a PREDICTION, not a result.** My last prediction here — that removing the memo alone
reaches O(1) — was **wrong**; it reached eager parity. Predictions in this area have a poor record.

## THEN `Seqable` — and it is downstream, not parallel

`118.3-B` (`a15f4ea9`) already landed the mechanism: a concrete type now satisfies a **parametric**
surface (the 2×2's fourth cell; the bug was a **string compare** where unification was needed).
`Seqable` itself is a ~20-line `defsurface` + four `extend-type`s once the protocol is fixed.

**Minting it before stone B would freeze the broken three-call protocol into a user-facing type.**

The split brain it closes, counted: **3 ways** to write a sequence verb (native · wat+twin · wat
armless) **+ 7 twins** → **1**. `keep`'s four `defclause` arms have **byte-identical bodies** — that
is the missing type rendered as code. Full argument: `118.6` (assessment) · `118.7` (user forms).

## ⛔ WHAT I GOT WRONG TODAY — six times, and the pattern is one thing

**I kept planning against a substrate further along than my plan, because I read artifacts instead
of running them.**

1. **Three "Seqable blockers" were stale** — refuted by a test green a MONTH before they were written.
2. **`measurable() => false`** — I inferred what it gated instead of testing. `empty?` *works*;
   `length` type-checks then raises.
3. **A `--check` on declarations** with no call site → I reported "the full design type-checks."
4. **I preserved `seqable->stream`** instead of asking which of two names survives.
5. **I designed the thread-per-stage architecture that arc 118 ALREADY KILLED** — the epitaph
   (`stdlib.rs:226`) was 11 lines from a comment I had read that same day.
   `[[feedback_read_the_epitaph_before_you_build_on_prior_art]]`
6. **I predicted removing the memo gives O(1)** — it gives eager parity.

★ **Every resolution came from a RUN, not a read**: a 20-line probe, a four-point RSS series, a
throwaway variant build, a printing function. **Zero came from reading more carefully.**

## THE STILL-OPEN

- **Stone B** — the migration + the memo. The frontier.
- **`dorun` · `length` · `first`** — leaves of B, not separate stones.
- **`Seqable`** — after B.
- **The rete deadline is eroding:** `spec_equals_native` went 48.7s → **73.3s** on one grok push —
  61% of its 120s kill. The durable fix (split it per where-family) is named in `.config/nextest.toml`
  and is looking less like "later".
- **255's (a)/(b) fork** — unruled since 2026-08-13; `BRIEF-STONE-255.1b-i` marked *"must not be
  struck as written."* Gates the registry carve.
- **The `Var`-gate** excludes concrete surface instantiations (`[s <- :Seqable<i64>]`); a surface
  with >1 type param is untested.
- **#101** Thermometer-as-record · **#102** `#holon {:a "b"}` broken · **#103** symbol→keyword
  (RULED NOT NOW) · **#91** HolonAST census.

---

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and **the better it reads, the more it will feel like continuing rather than waking. That
> feeling is the failure.**
>
> **`git status` FIRST. Then `git log --oneline 0d651715..HEAD`.** Everything in that range outranks
> this file.
>
> ⚠ **A BLOCKER NOTE IS A CLAIM WITH A DATE ON IT.** One sat for two months, quoted everywhere,
> refuted by a test already green when it was written.
>
> ⚠ **AND SO IS AN EPITAPH — read it before you build.** Twice today I designed on machinery without
> asking what it replaced and why. The second time I rebuilt an architecture this project had already
> built and annihilated.
>
> ⚠ **DO NOT DESIGN THE STREAM TIER FROM READING.** Six wrong plans, six corrections, every single
> one from a measurement. The instruments are cheap and they are already written down above.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
