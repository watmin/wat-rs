# SEAM — the ONE live breadcrumb. As of 2026-08-19 (end of day). Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.

## GROUND FIRST

> **Written against `ec11f6ac`.** Run **`git log --oneline ec11f6ac..HEAD`**. Empty → nothing moved.
> Non-empty → every commit in it outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`. **`mcp__wat__eval` LIES** — use `./target/release/wat`.

```
floor .......... 4819/4819, 0 FAIL, 19 skipped   (own invocation)
clippy ......... 0          ignores ......... 13
stash@{0} ...... the lifecycle strike. NEVER drop.
```

⚠ **RUN EVERYTHING CAPPED.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0
timeout <s> …` — `MemorySwapMax=0` is load-bearing. **Read exit codes directly, never through a pipe.**

## ★★ THE FRONTIER — THE KERNEL CARVE. Every one of its 44 verbs now has an honest row.

`255.1c-taxonomy` **STRUCK** (`ec11f6ac`). `Category` went 10 → 15 and its subject line stopped lying.
The kernel carve was blocked on exactly that and is now unblocked. **Do not re-derive the taxonomy —
it cost two `intueri` casts and a builder override.**

```
:Resource   custody of a handle — acquire, release, ADMINISTER   (name ADOPTED from task #68)
:Message    payload to/from a locus you hold a handle to — NOT :Io
:Ambient    process-global state, BOTH directions                (rejected :Signal-as-Clock-sibling)
:Project    the inverse of :Combine — returns a COMPONENT        (not :Accessor — an agent noun)
:CheckGate  refuses a call site at check time; runtime is identity
```

**The 44 kernel verbs decompose into seven concerns** (`255/DESIGN-STONE-255.1c-kernel-stdio.md`) —
stdio ✅ carved as home #3; **concurrency · networking · signals · errors · handles/capability ·
misc** remain. ⚠ That table is NOT exhaustive: `signal`, `address-wire?`, `require-wire-address`,
`macro-call-site` sit in no row. Its own arithmetic (46) does not match its stated 49, and today's
dispatch count is 44. **Bookkeeping drift, not a defect** — nothing breaks by carving a subset; that
is the state of the other ~395 verbs too.

★ **A carve rider must RE-DERIVE each verb's Category from its BODY.** The last home's rider had its
`Io`/`Reflection` first pass overruled and re-derived; that is the discipline working, not a failure.

## ★ THE 255 ROAD — the builder's chain, grounded. This is WHY, and it is not obvious.

```
255 registry  →  the INVENTORY (every form, by name, with confidence)
   →  the flip:  :wat::core::+  →  wat.core/+
                 HashMap<K,V>   →  (wat.type/HashMap [wat.type/K wat.type/V])
      →  ONLY THEN can the two EDN encoders merge
```

**Builder:** *"we cannot [kill the two EDN paths] until we annihilate the edn illegal parametric and
illegal keyword syntax... the registry is necessary to force us to inventory every form so we can
mass fix with confidence."*

**The scale, measured** (`251/CENSUS-the-illegal-edn-form-classes.md`):

```
colon-quoted symbol   79,253 occurrences · 6,552 DISTINCT spellings · 1,263 .wat files
angle parametric       2,945, of which 951 COMMA-BEARING
double-slash           NO VALIDATED COUNT — the pattern was refuted, so no number was written
```

★ **The migration unit is 6,552, not 79,253**, and the top 12 spellings are 26,138 occurrences —
**33% of the corpus.** So it is *head-by-codemod, tail-by-name*, and the registry is what lets the two
compose (it can hold two spellings at once).
★★ **The 951 comma-bearing are the dangerous ones:** after the flip `(f HashMap<K,V>)` reads as
VALID EDN and **silently changes arity 2→3.** Which is why **255.1b-iv** (kill the blanket-accept)
must precede the flip — today an unregistered `:wat::` head type-checks clean, so a mass rename would
ship half-broken and silent. Task **#110**, two doors measured (`resolve/walk.rs:257` +
`check.rs:5568`); 255's record names only the first.

## ★ ARC 118 IS INSCRIBED (`ba3bd70c`). Opened as arc 004 on 2026-04-20. Closed 2026-08-19.

**R5 — QVAESTIO SVBSTITVTA LEGEM ELVDIT.** Arc 004 shipped a thread-per-stage CSP pipeline AS "lazy
sequences" and deferred the real thing with *"in-process lazy chains haven't been demanded by a
caller"* — while carrying **Lesson 1: "Absence is signal"** in the same file. The law was not broken;
it was EVADED by substituting the question. **Written. Do not re-derive.**

## GROK'S RETE IS MERGED (`c800d7d5`) — and the grid is NOT a time series

`compiled_where` shipped (task #49 closed); `src/rete/` is a compiler now. ⚠ All 30 grid verdicts read
`:wall-winner :us` — but the runs set `GRID_SKIP_ORACLE=1` (the spec fire *was* the bulk of wat-wall)
**and** `fanout.wat` was edited 3× since 2026-08-01. **Do not diff two dated grid files and report a
trend.**

## THE STILL-OPEN

- **The kernel carve** — the frontier. Pick a concern, re-derive from bodies.
- **#110** the blanket-accept (255.1b-iv) · **#109** the rete right fold (HELD; 255 owns purity)
- **#107** macro bodies reach only intrinsics — ⚠ its control CANNOT live in `wat-scripts/` (a file
  there must LOAD); rebuild it in `tests/` first
- **#108** promotion drops free-variable unification. ⚠ `255/NOTE-promotion-is-not-relocation` says
  measuring the other 141 arms is *"one grep away."* **IT IS NOT** — `StreamContainer::of_type` is 4
  sites in `infer.rs` + 6 in `check.rs`, and the arms classify several ways. Needs a form-reader.
- **#91 HolonAST is 1014 mentions in `src/`** — `HolonRepresentable` is 0, so "holon-rep is done" is
  TRUE and "the holon/wat separation is done" is FALSE. Read `294/RULING-holonast-and-hologram-…` first.
- **Stone A** (`255/CHAIN-…`) — reduced to ONE structural move (16 arms → a `const TAG`). Its other
  three defects closed by 294; its blocker (the `EdnRepresentable` name) evaporated. ⚠ `PORTABLE` is
  **already built** by 294.m as a REGISTRY QUERY, better than the design's const.
- **296 is OPEN** (INSCRIPTION absent) · **295** unblocked by 118's close, not started

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **A REJECTED OPTION RETURNS IN NEW CLOTHES.** I proposed "ambient vs addressed" as a Category
> axis while *quoting* the header that lists "where its input comes from" as REJECTED. It was the
> output-side mirror of exactly that. **Check your own proposal against the reject list you are
> citing.** `[[feedback_a_rejected_option_returns_in_new_clothes]]`
>
> ⚠ **A SCOPED FILTER CAN BE BLIND TO ITS OWN SUBJECT.** My `nextest -E` did not match the test I had
> named as the stone's acceptance row. The rider caught it. A green from a filter that cannot see the
> load-bearing test is worth nothing.
>
> ⚠ **HAND-MAINTAINED LISTS, FOUR TIMES IN ONE DAY** — the three purity ledgers, the kernel
> decomposition table, the "one grep away" claim, and `Category`'s four mirrors under a header saying
> *"there is no second list."* **That class is the whole reason 255 exists.**
>
> ⚠ **A WARD THAT ONLY HEARS THE CASE *FOR* IS NOT INDEPENDENT.** Both `intueri` casts were told my
> hypotheses were mine and to refute them freely; both did, and both were right to.
>
> `NON BIS IN IDEM FLVMEN.` · `QVAESTIO SVBSTITVTA LEGEM ELVDIT.` · `MVRVS AVCTOREM NON NOVIT.`
