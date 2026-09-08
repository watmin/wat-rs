# SEAM — the ONE live breadcrumb. 2026-09-07. ⛔⛔ **THE TREE IS DIRTY AND NOTHING RUNS.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), then run the commands below and read this whole file before you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## ⛔⛔ FIRST — THIS IS THE MOST DANGEROUS STATE THIS SESSION HAS PRODUCED

```bash
git status --porcelain     # expect 6 DIRTY: 4 src/ + 1 probe .rs + 1 SCORE .md
git log --oneline -1       # expect db547fa67 or later
./target/release/wat tests/types/probe_arc296_enum_map_ctor__control.wat; echo "EXIT=$?"
```

```
floor .... 5238 run: 2447 passed, 2773 FAILED, 18 TIMED OUT, 18 skipped     FLOOR EXIT=100
control .. run EXIT=3     ← ⛔ NO WAT PROGRAM STARTS. THE STDLIB DOES NOT LOAD.
```

⛔ **DO NOT COMMIT.** ⛔ **DO NOT REVERT — THE WORK IS CORRECT.** ⛔ **DO NOT "FIX" THE FLOOR.**

**Arc 296 STONE M landed in the working tree, UNCOMMITTED, and it is RIGHT.** It refuses positional
enum construction — and the stdlib is written in the positional form, so the loaded world now fails
to check. That is **one cause with everything downstream of it**, verified, not assumed:

```
every :reason in the floor log   "positional variant construction is retired; write
                                  `(:ns::E::V {:field value …})` or `(… {})` for a unit variant"
the 18 TIMEOUTS                  14 are wat_mcp::* + 2 sigterm + 2 process-label — tests that SPAWN
                                 a wat process. The child now dies at startup, so the parent waits
                                 30s for a handshake that never comes. NOT a second defect.
```

## ⛔ THE WAY OUT IS WRITTEN DOWN — `wat/fix.wat:23` THE STASH-DANCE

The codemod that fixes this is itself a `.wat` program, **and no `.wat` program can run.** That
chicken/egg is documented, and its header says a prior self abandoned the tool because the dance was
not written down. It is now:

```bash
1.  git stash push -m "stone M" src/check.rs src/runtime.rs src/types.rs src/record/construct.rs
2.  cargo build --release                    # OLD checker (accepts positional) + any NEW fix verb
3.  printf '["pathA" …]\n' | cargo wat ./wat-scripts/fixes/<the-fix>.wat    # EVERY path; a missed
                                                                            # file breaks the build
4.  git stash pop
5.  cargo build --release && scripts/floor.sh
```

⚠ Dry-run step 3 on a `/tmp` COPY first and `diff` it. ⚠ **`git stash` is load-bearing here — do
NOT drop it.**

## WHAT IS UNCOMMITTED (all of it correct, none of it committable alone)

```
src/types.rs               EnumDef::variant_fields — unit variant -> empty slice
src/record/construct.rs    try_eval_enum_map_ctor + enum_runtime_value
src/runtime.rs             intercept in dispatch_keyword_head_value before Function lookup
src/check.rs               infer_enum_map_ctor before scheme lookup; refuse positional
tests/types/probe_arc296_enum_map_ctor.rs      the four #[ignore] removed
docs/…/296…/SCORE-STONE-M-the-enum-ctor-is-a-map.md    grok's score, untracked
```

**The corpus migration is NOT "the next stone" — it is the OTHER HALF OF THIS ONE.** They were drawn
sequentially and that framing was wrong the moment the refusal reddened the loaded world. Section 7's
atomic-commit pattern governs: A dirty, B against the dirty tree, ONE commit when green.

**The worklist starts at the 659 stdlib sites `--check` already named** (grok's SCORE has the
per-file and per-enum tables): `journal.wat` 160 · `sqlite-store.wat` 116 · `cache.wat` 96 ·
`span.wat` 91 · `stdio.wat` 90 · `mem.wat` 80 — and by enum, `RecvOutcome` 125 · `Store::Reply` 44 ·
`Journal::Reply` 32 · `Outcome` 25. **Then every positional site in `tests/`, `wat-tests/`,
`wat-scripts/` — the floor will name them.**

## ★★★ WHERE THE CRUSADE IS

```
296 H/J/K/L   enum wire · 26 enums into wat + a wall · aliases + an ORACLE · type-of      ✅
109 the arm   108 → 38 → 16 → 7 → 0. 1869 files by codemod.                              ✅
251.8b · 251.9  Identifier stores (ns,name) · a symbol-headed declaration DECLARES       ✅
109 kwargs    assertion-failed! takes kwargs. 453 files. bare None 5818 → 650 (−5168)    ✅
296 M         the enum ctor is a MAP; positional REFUSED                    ⛔ IN TREE, UNCOMMITTED
```

## ⛔ QUEUED — the order is the builder's and it is RULED

```
1  296 M's SECOND HALF   the corpus migration by wat-fix, via THE STASH DANCE. Blocks everything.
2  Option/Result         bare-name retirement: None 650 · Some 633 · Err 561 · Ok 365 = 2209
   completion            THREE spellings legal at once (:None · :wat::core::None ·
                         :wat::core::Option::None); 62 Rust sites across 10 files keep the bare
                         one alive; the 4 `builtin_variant` arms (match_arm.rs:159) must DIE so
                         the bare spelling is UNREPRESENTABLE, not merely unused.
3  {:keys} on defrecord  A MEASURED 4-cell asymmetry — defstruct gets :keys, defrecord does not,
                         because arc 257.2's probe only ever exercised defstruct. The measurement
                         IS the acceptance test. Builder ruled: ITS OWN STONE.
4  variant <: enum       ONE ENTRY in `subtype_edges` (types.rs:542, a GENERAL map), refused by a
                         parse-time wall that admits only `:Name <: <nature root>` (types.rs:304).
                         ⚠ FIRST ACT IS A MEASUREMENT, NOT A DESIGN: can `Shape.Circle` be a type
                         WITHOUT becoming a second way to spell a record? Likely a NARROWING for
                         parameter/dispatch position only.
5  :wat::* whitelist     RE-MEASURED 2026-09-07: 143/833 FAIL (17%), 35 names — NOT the NOTE's
                         578/599 (96%) and 121. Every special form (`fn` `def` `match` `quote`
                         `do` `derive`) is at ZERO. Four families; rete numerics are ~87% of sites
                         at ~11 names. THE NOTE'S FORCED ORDERING NO LONGER HOLDS.
   head_of residual      runtime.rs:12232, Keyword-only, in eval-with-defs!. Unreachable ONLY
                         because macro templates emit keyword heads (core.wat:1348/1351/1393; 51
                         stdlib template sites). THE HEAD MIGRATION REWRITES THOSE. Needs a probe.
   assertion-span        the kwargs refusal points at the MACRO (assertion.wat:34), not the caller.
```

## ⚠ RULINGS — do not re-litigate

- **The enum ctor is a MAP, and only that.** `(:ns::E::V {:field v})` / `(… {})`. Positional dies.
- **A variant is named the same way in all three places** — wire, pattern, ctor.
- **The match arm is `[Variant {:k v} body]`, KEY-FIRST.** `{a :x}` is `let` order.
- **`:keys` is for ONE-SHAPE aggregates** (record · struct · eventually variant). **Match is for
  many-shape.** Destructuring is total; a match arm is a test that can fail. Do not unify them.
- **A type wat uses is DECLARED IN WAT** — categorical, not "if it has bitten".
- **A golden pinning a stdlib line: RECAPTURE, KEEP PINNING.** Do NOT extend the normaliser.
- **⛔ SIDE BRANCHES DO NOT SERVE US** — 3 parked, 15 days, 0 merged.
- **⛔ THE RECORD'S SETTLED NUMBERS EXPIRE.** 296 R20 `HAERESIS EST ITERVM ROGARE`. Before a number
  becomes a premise for an ORDERING, a SCOPE CUT, or a REFUSAL — re-derive it.

## ⛔ THE FAILURE PATTERN — NOW ELEVEN

**I COUNT SOMETHING CORRECTLY AND SAY THE WRONG THING ABOUT WHAT I COUNTED.** Never caught by me:

```
25/26 · "no live producer" · "17 .wat.bad" (11) · "4 failures" (108) · nested arms · "[~@ {" ·
"teach the arm reader"
  ⑧ THE BISECT — a first-bad commit names where a symptom became VISIBLE, never a cause.
  ⑨ THE FRAMING — "5816 :wat::core::None" reported as neutral; it was the ILLEGAL spelling's size.
  ⑩ THE PIPE, 3rd in a day — read a stone's own load-bearing exit code through `| head -5`.
  ⑪ ★ THE CONTROL-DERIVED BAR WENT VACUOUSLY GREEN. Every probe today derived its bar from a
    control run in the same test, precisely so no literal could deceive it. Stone M's refusal
    reddened THE CONTROL — three rows passed on `1 == 1`. THE PEER CAUGHT IT, NOT ME. And my
    EXPECTATIONS carried a contradiction I never saw: row 1 (`control EXIT=0`) and STOP-5 (don't
    migrate the corpus) CANNOT BOTH HOLD.
    ⛔ THE FIX, OWED: measure FIXTURE-LOCAL errors, never the shared exit code.
```

★ **THE CURE: GO TO THE SOURCE, NOT THE BRIEF.** `parse.rs`'s own doc stopped a wrong "7,816" from
shipping. `EnumValue`'s own comment collapsed three stones into one finding. The disk answers; the
note remembers.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Re-run the commands. Do not read the numbers.
>
> ⛔⛔ **NOTHING RUNS. FOUR `src/` FILES ARE DIRTY AND UNCOMMITTED AND THE WORK IS CORRECT.** The
> instinct on waking to a dead toolchain is to revert. **REVERTING IS A LOSS.** `wat/fix.wat:23`
> is the way out and it was written for exactly this.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`
