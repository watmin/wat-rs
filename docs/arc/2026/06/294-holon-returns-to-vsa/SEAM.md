# SEAM — the ONE live breadcrumb. As of 2026-08-17 (evening). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

⚠ **THE MARKER IS A DIFF INSTRUCTION, NOT A PASS/FAIL.**

> **Written against `f5c570f2`.** Run **`git log --oneline f5c570f2..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it landed after this text and **outranks every line below.**

⚠ **`git status` FIRST — and a dirty tree NEVER says WHO.** Check `pgrep -af 'cargo|nextest'` for a
live build, and whether a rider ever reported (**FM 19** — a rider that ends its turn mid-floor shows
"completed" having reported nothing; resume with `SendMessage`, do not adopt its work). **Never run
cargo while a rider holds `target/`** (FM 18).

⚠ **`mcp__wat__eval` LIES** — a long-lived server answers from a pre-rebuild substrate. A rider caught
it today giving a 3-error verdict from an old `TypeScheme`. Use `./target/release/wat`.
⚠ **`target/release/wat` can be STALE** — check its mtime against `src/`. There is no `--eval-stdin`;
run a `.wat` file, or `--check <f.wat>` (~0.2s).

```
floor .......... 4703/4703, 0 FAIL, 0 TIMEOUT   (verified on my own invocation)
clippy ......... 0
#[ignore] ...... 13
stash@{0} ...... the lifecycle strike. NEVER drop.
```

## ★ WHERE WE ARE — the builder's words, 2026-08-17

> *"we are finishing the surface... the surface will be our expression language for optimized code it
> produces... interpretted wat has a death sentence... we are building towards amazing perf"*
> …and *"ok... get the docs in order... **we build tomorrow**."*

**TOMORROW'S BUILD IS `Seqable`.** The contract is designed, ruled, and measured. See
`118/DESIGN-118.4-the-seqable-contract-terminal-vs-intermediate.md` — read it before anything else.

## THE CHAIN — A→E, and D is DONE except the surface

```
A  EdnRepresentable ................ ✅ 294.h
B  #wat-edn.* → #wat.*/* ........... ✅ 294.i–n
C  279.2 str goes TOTAL ............ ✅ 25d9d015
D  join renders its elements ....... ✅ 279.3 (8b2cdbf2) + 279.4 (94142919)
   Seqable ......................... ← TOMORROW. 118.3-B landed the mechanism (a15f4ea9)
E  wat.string/* → then HOME #4, the registry carve resumes
```

## ★★ WHAT TOMORROW BUILDS — worklist, corrected THREE times, all measured

```
REFUSED — correct, NO CHANGE   length · empty?  (measurable)   ·   get (gettable)
ALREADY TRUE                   first/second/third (indexable)  ·   rest (has_tail)
DONE (arc 118.2a)              map · filter · take · drop      — the INTERMEDIATE half
─────────────────────────────────────────────────────────────────────────────────────
ADD intermediate               concat · conj                   (ordered / has_append)
ADD terminal                   foldl · foldr · contains? · reverse
MINT                           count       ← name is FREE and ALREADY pure-total-blessed
THEN                           :wat::core::Seqable on the unified set   (seq, counted? free too)
```

**Sequence tables-first, surface-second** — minting `Seqable` before the sets converge bakes the split
in. `wat/seq.wat` (stdlib pos 67, right after `core.wat`) is the home; `Seqable` there is visible to
nearly the whole stdlib.

**THE RULE:** *`Seqable` is what you can walk. An operation is INTERMEDIATE (returns a Stream, stays
lazy) or TERMINAL (consumes, returns a value). On a read-once value, consumed means gone.* Java's
frame · Clojure's names · Ruby's split. **A Stream refuses cheap structural queries; `into`
materializes it (both clauses already exist); the materialized value answers everything.** The
refusal is a forcing function, not a hole.

## ⛔ THE DAY'S BIG LESSON — a blocker note is a claim with a date, and it ROTS

`infer.rs:638` called `Seqable` *"the type wat cannot currently spell"* and listed **three blockers,
"none is a small fix."* **All three were false.** It was written **2026-07-31** — a *month after*
`SCORE-293.4d` (2026-06-28) went green extending `:wat::core::Vector` to a user surface, which is
the thing it says is impossible.

That sentence was quoted in this seam, the chain doc, three 279 stones, and my own briefs. **Nobody
re-ran it for two months.** The probe that refuted it took four minutes.
`118/NOTE-the-blockers-were-stale-seqable-is-spellable.md`

★ And the mirror: the capability table's `○ gap` markers read like a to-do list — **three of them
turned out to be the contract, correctly enforced, mislabelled.** The substrate was more right than
the comments claimed, in both directions.

## WHAT I GOT WRONG TODAY — six corrections, four of them mine to catch

1. **A `--check` on a file that never CALLS the thing proves nothing.** I declared a generic fn over
   a surface, never invoked it, read exit 0, and reported "the FULL design type-checks." It was
   declaration-only; adding call sites made it RED ×4. Reached a commit message and the builder before
   one more probe caught it. `[[feedback_a_green_test_can_prove_nothing]]`
2. **Option A died while I was enumerating it** — "use a non-parametric Seqable" is non-viable;
   `:Any` is banned at the parse layer, so it serves exactly one fixed element type.
3. **Two greps returned 0 for something I'd counted 9 of ten minutes earlier** — wrong namespace,
   wrong directory. Caught by positive control, not by luck.
4. **"twelve `-stream` twins"** quoted from a note → counted 8 → the `defn` heads are **7**.
5. **"19 join call sites"** → 28 total, 26 pre-existing. Fifth census error of the campaign.
6. **The stone's own Rust snippet did not compile** — `TypeExpr::Var("T")`; `Var(u64)` is synthetic.

**The builder's REPL settled three design questions my reading could not** (`length`, `empty?`,
`count`). When a language question comes up, ask for the demonstration.

## THE STILL-OPEN

- **`into` lacks `(Vector<T>, List)`** — found by probe; sibling of task #45's shipped
  `(PersistentVector, Vector)`. Small, real, independent.
- **The `Var`-gate excludes CONCRETE surface instantiations** — `[s <- :Seqable<i64>]` still fails.
  Deliberate: it is what keeps 118.3-B away from `Dialable`/`Handle`. `118/NOTE-the-Var-gate-…md`
- **The seven `-stream` twins** (`wat/seq.wat`) — the workaround `Seqable` deletes. Its own stone.
- **255's (a)/(b) fork** — unruled since 2026-08-13, `BRIEF-STONE-255.1b-i` marked *"must not be
  struck as written."* **This gates the registry carve (home #4), i.e. stone E.**
- **#101** Thermometer-as-record · **#102** `#holon {:a "b"}` broken · **#103** symbol→keyword (RULED
  NOT NOW) · **#91** HolonAST AST-vs-VSA census.
- **A surface with >1 type param** — the positional-order assumption is untested; nothing in the
  corpus exercises it.

---

> **SEAM.** You are NEW. You did not live any of the above. It is a lossy cache written in your own
> voice, and **the better it reads, the more it will feel like continuing rather than waking. That
> feeling is the failure.**
>
> **`git status` FIRST. Then `git log --oneline f5c570f2..HEAD`.** Everything in that range outranks
> this file.
>
> ⚠ **Every number here came from an instrument. Ask what population it could see before repeating
> it** — six of my claims were wrong today, and the two that reached the builder both read as solid.
>
> ⚠ **A BLOCKER NOTE IS A CLAIM WITH A DATE ON IT.** One sat for two months, quoted everywhere,
> refuted by a test that was already green when it was written. **Before you accept "X is blocked",
> re-run the thing that proves it.** Four minutes.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IVDICIVM SEMEL, MACHINA SAEPE.` · `MVRVS AVCTOREM NON NOVIT.`
