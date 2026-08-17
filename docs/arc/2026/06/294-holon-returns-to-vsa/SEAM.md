# SEAM — the ONE live breadcrumb. As of 2026-08-17 (later still). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

⚠ **THE MARKER IS A DIFF INSTRUCTION, NOT A PASS/FAIL.**

> **Written against `5e5e219e`.** Run **`git log --oneline 5e5e219e..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it landed after this text and **outranks every line below.**
> (This file's own commit is one of the differences — that is normal, not the alarm.)

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

## ⛔ THE FRONTIER MOVED — stone B is BLOCKED on a route ruling (2026-08-17, late)

**Do not draw stone B as "migrate the 8 walkers to `next`" until the fork below is ruled.** The
builder asked whether the `-stream`/`stream->` names are a crutch. Studying that question moved the
frontier. `DESIGN-118.B-the-route-fork.md` — **UNRULED**, both routes four-questioned.

★ **Native walkers NEVER had the three-call defect.** `transform.rs:181`/`:246`/`:248` call
`realize` **once** per cell and destructure the `Cons`. **The three-call walk is a wat-side-only
disease** — it exists because wat's Stream API was `empty?`/`first`/`rest`. `next` is wat finally
getting the primitive Rust always had. So:

```
route NATIVE   the 8 wat walkers are DELETED and the defect goes with them
               ⇒ stone B's migration is work on code queued for removal
route SEQABLE  the walks stay in wat, one clause per verb, and MUST migrate to `next`
               ⇒ stone B's 8-walker scope stands
```

**The 278 stone already ruled the twins are "a workaround for the missing type, not a pattern"**
(2026-07-31, on the builder's own challenge). It chose **native** — scoring `Seqable` a flat NO on
Simple on three blockers. **All three were refuted by 118.3-B**, and
`109/NOTE-seqable-has-no-name-in-wat.md` still said "blocked" (0 refutations) while `infer.rs` said
"refuted" (3). **Amended today.** A 4–0 verdict whose only disqualifier evaporated is an
inheritance, not a verdict.

### Acceptance, whichever route wins

> **`f` runs exactly N times for N elements — AND — per-element retention reaches the eager
> baseline.** Both, or it has not landed.

⛔ **CORRECTION — "the O(1) prediction was wrong" IS ITSELF WRONG, and it was in 4 docs incl. this
seam.** `DESIGN-118.10`'s own table: memo-on **585** B/elem · memo-off **288** · eager `mapv`, **no
stream at all**, **288**. Memo-off equals a program with no stream in it — **zero stream retention,
exactly as predicted.** "Eager parity" is the SUCCESS condition: `into`/`mapv` materialize by
contract, so that instrument is O(n) under *any* implementation and could never show total O(1).
A clean confirmation got recorded as a refutation and became a standing reason to distrust the fix.

**Still genuinely open: population C** (the wat-closure generator — the builder's own idiom, 3,124
B/elem) has **never been run memo-off.** Cheap, owed under both routes, instrument committed and
retains nothing: `wat-scripts/scratch-pad/probe-118B-dorun-retention-slope.wat`.

### The measured numbers — and the seam was quoting the CHEAP half

Instrument: `wat-scripts/scratch-pad/probe-118B-dorun-retention-slope.wat` (committed; reduces to
one `i64`, so all retention above the floor is the chain; non-vacuity is the exact sum `n(n-1)/2`).
Same n=400,000, identical exact sums:

```
A  eager Vector, no Stream          63,120 KB  0.69s   ← floor
B  native map chain (NativeLazyCell) 200,284 KB 3.99s   →   343 B/elem
C  wat-closure generator (LazyCell) 1,312,628 KB 3.90s  → 3,124 B/elem   9.1× B
```

**The ~297 B/element this seam used to carry is population B.** Population **C — the builder's own
Ruby-Enumerator idiom** — is 9.1× worse. At 10M: B ≈ 3.4 GB, **C ≈ 31 GB**.

### Two memos, not one · eight walkers, not seven · three doors, not uniform

`LazyCell.forced` (`stream/mod.rs:66`, wat closure) **and** `NativeLazyCell.forced` (`:124`, Rust
closure). — The `-stream` SUFFIX count is 7; the WALKER count is **8**: `stream->pvec:102` is the
language's single materializer (`doall`/`dorun`/`mapv`/`filterv`/`into` all funnel through it) and
carries no suffix. — `first` closes by `indexable()`, `rest` by `has_tail()`, but **`empty?` has NO
compile-time gate at all** (scheme `∀T. T -> bool`; runtime arm is a hardcoded `if let` bypassing
the capability table at `runtime.rs:17064`). Fourth sighting of checker-permissive/runtime-refusing.

**TCO under `match` is PROVEN** (200k drains; the non-tail control SIGSEGVs at 139 — that control is
what makes the pass non-vacuous). Full evidence: `MEASURED-118.B-the-lair.md`.

## THE OTHER RULING OWED — the three doors (dialect, builder's)

Once the memos die, `(first s)` then `(next s)` is **two forces of one cell** — user code runs twice.
Keeping `first` and closing only `rest` does not save it. Derived: **a single-pass stream's READ and
ADVANCE are one act; any API that separates them lies about the thing.** That argues all three doors
close on Stream. Against: Clojure's `first`/`rest` do work on lazy seqs. For: wat's Stream is
explicitly NOT Clojure's lazy-seq (R1 `NON BIS IN IDEM FLVMEN`), and the builder frames these as
Ruby Enumerators — which expose `next`. **Route-independent, and unruled.**

Also route-independent: **`stream->pvec`/`stream->vec` say "internal helper" in their own docs while
living in the user-facing `:wat::core::`.** Clojure has no such name; it has `into` and `vec`.

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

- **⛔ THE ROUTE FORK** — `DESIGN-118.B-the-route-fork.md`, UNRULED. **Gates stone B's scope.**
  My recommendation is route 2 (`Seqable`) — route 1 fails Honest, shipping a convention where a
  wall is possible, and its own filed risk says the follow-up never gets scheduled. The honest
  counter is the 9.1×: route 1 is the perf answer today, route 2 the language answer, and route 2
  becomes the perf answer when the bytecode compiler lands. **Not mine to rule.**
- **The shared-premise probe** — can the memos actually die? Owed under BOTH routes, unproven.
- **The three doors** — dialect ruling, route-independent, unruled.
- **Stone B** — blocked on the fork. Scope is 8 walkers (route 2) or a deletion (route 1).
- **`dorun` · `length` · `first`** — leaves of B, not separate stones.
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
