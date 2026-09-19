# SEAM — THE ONE LIVE BREADCRUMB. 2026-09-19. ⛔ **YOU ARE ON `main`.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice,
> which is why it will feel like *continuing* rather than *waking* — **and that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy), read
> `docs/COMPACTION-AMNESIA-RECOVERY.md`, then run the commands below before you touch anything.

> ⭐ **THIS FILE IS NOT ARC-SCOPED, AND THAT IS DELIBERATE.** The live breadcrumb used to live inside
> whichever arc was active. **No arc is active right now** — the replay closed and the next arc is the
> builder's ruling, unmade. An arc-scoped breadcrumb would have to be moved the moment that ruling lands;
> this one does not. When an arc is ruled live, either point this file at its seam or keep using this one.

## FIRST — RUN THESE. DO NOT READ THE NUMBERS BELOW.

```bash
git rev-parse --abbrev-ref HEAD          # expect main
git rev-parse HEAD                       # FRESHNESS PROBE — compare to the stamp below
git status --porcelain                   # expect empty
git log --oneline -12
ls -dt docs/arc/2026/*/*/ | head -6      # which arcs moved most recently
git grep -hcE '^\s*#\[ignore' -- 'src/*.rs' 'tests/*.rs' | paste -sd+ | bc   # the ignore ledger, live
```

**Stamp: written against HEAD `d01e638f375324d9f17c32c8e5f659868795f846`.** `main` == `origin/main`.
Floor at that tree: **5918 passed / 22 skipped**, clippy 0, census `no STOP-8`.
⚠ A one-commit docs-only gap is normal — the commit that writes this stamp lands after it.

---

## ⭐ WHAT JUST HAPPENED — the grok-rete replay is MERGED

**651 of 651 commits replayed and landed on `main`** (fast-forward, 841 commits, no merge commit).
The branch `replay/grok-rete` points at the same commit; `merge/grok-rete` (the first, **rejected**
whole-merge) can be retired.

Verified before merging, each independently re-derived rather than inherited:
- all **651** cherry-pick trailers two-sided — **0 mismatches**;
- the last step cites grok's tip `37528f6e0`;
- the record gate green over **`#153→#651` in one run** (499 consecutive steps). ⚠ It cannot reach #1:
  the five-verbatim-line record convention **did not exist before #153**;
- **end cross-check: ZERO files grok has that this tree lacks**; 49 files carry content deltas, all
  attributed (`SCORE-8` + its correction) — **D=0, E=0**, nothing dropped, nothing unexplained.

**The method is written down:** `docs/arc/2026/06/294-holon-returns-to-vsa/REPLAY-PLAYBOOK.md` — the
invariants, the batch loop, the tooling, a pre-flight checklist, and the six failure modes that bit.
**Read it before the next branch.** The full record is `294-holon-returns-to-vsa/the-grok-rete-replay/`.

**Four deliberate divergences from grok ride on `main`**, each ruled and recorded: the `:then`-match
fence stands (#324) · `--check` is a UNIT checker, narrowed to a DECLARED `:user::main` (#388) · the
cured dominance assertion stays ON the floor where grok ignores then deletes it (#472/#498) · #638's
three probe dispositions.

---

## ⛔ THE DEPENDENCY CHAIN — read this before choosing anything

Three threads are live and they are **not independent**. Measured from their own docs, not recalled:

```
   296 / 298  ── EDN errors, Option tagging
        │      "⛔ THE DERIVE SWEEP IS BLOCKED ON THIS — deriving RuntimeError/MacroError
        │       over Option-erasing data would bake in the lie" (255/CURRENT-STATE)
        ▼
   255  ── the builtin registry
        │   closes #95 "because `type_sig` was ruled day-one" (251/SEAM)
        │   owes `wat.type` a real home: today a `strip_prefix("wat::type::")` at exactly two
        │   sites (types.rs:4503, :4702) — an ALIAS, not a namespace. Measured:
        │   `:wat::type::Vector` annotates but is an UNKNOWN FUNCTION.
        ▼
   251  ── types-as-forms / the clojurification
            BLOCKED on #95: a dotted call head is not type-checked AT ALL (args, arity, return),
            because `infer_list` gates its whole call-inference universe on
            `if let WatAST::Keyword` (check.rs:2542, closing :5568).
```

⛔ **THE REGISTRY UNBLOCKS THE CLOJURIFICATION, NOT THE REVERSE.** If you remember it the other way
round, that is the pivot talking — see below.

### ⭐ THE BUILDER ALREADY RULED THE TAIL OF THIS CHAIN — 2026-08-13

From `278-rules-engine/SEAM.md`, in the builder's own words:

> *"back to 251 we go… **we'll resume 278 once we have the clojure syntax conversion complete.**"*

So the sequence is not an orchestrator's guess — it is on record, and the full chain reads:

```
296/298  EDN errors ──► 255  registry ──► 251  clojurification ──► 278  rete resumes
```

⭐ **And the replay just did 278's half of the bargain.** That same banner: *"251 was abandoned FOR
rete; rete is what made 251 executable"* — the tooling 251 needed (a rules engine that classifies by
POSITION, an extractor turning real source into facts, a diagnostic that names the cause instead of the
call site) landed in rete. **All of it is now on `main`.** 278's subsystem is current; what it waits on
is the clojure syntax conversion, which is 251, which waits on 255.

### Why 251 has landed work despite being blocked

The stones that landed (**251.8a** the ONE DOOR · **8a-ii** binder namespace unforgeable at the reader ·
**8b**) are **reader-level**, which #95 does not gate. The type-check half waits on 255. That is why work
happened on 251 while the registry sat.

### The pivot that got us here — and it LANDED

`255/BRIEF-STONE-an-example-is-a-form-not-a-string.md`. Doc-comment examples were held as **source
text**, so a malformed `@example` failed late in a reflection test and a wat declaration had to write an
**escaped string** instead of literal syntax. **Shipped:** `DocExample.expr` is now a `WatAST`
(`crates/wat-doc/src/lib.rs:181`), commit `a4463743a` — *"a malformed one now fails the BUILD"*.

---

## 📋 THE IGNORE LEDGER — the concrete measure of what is owed

**Builder's standing ruling: *"we should have precisely 1 ignore when we're done — the ignore that proves
wat-tests support ignores."*** Today: **17**. History: 31 → 24 → 13 → **17** (the replay moved it back up).

| owed by | count | note |
|---|---|---|
| **255** registry | **6** | 5 are `RE-POINTED arc 255 Stone P3`; 1 says outright *"blocked on arc 255's unbuilt builtin registry"* |
| **296** EDN errors | **4** | P-2a (a variant is not a registered type) ×3, A-2 superseded ×1 |
| boot-clock | 2 | seam/peer `started-at` ordering — unlock named in the reasons |
| **300** EDN source | 1 | the equality-matrix flaw-tracker, RED by design |
| **260** kwargs | 1 | wat has no keyword args; call sites positional |
| orchestrator eyeball | 1 | a `--ignored --nocapture` diagnostic |
| other | 2 | the `-tick` op-ref remove-at idx-shift |

**Ten of seventeen trace to 255 + 296** — the two threads at the head of the chain.

---

## ⛔ STALE DOCUMENTS — do not resume from these

The redirects formed a **cycle that now dead-ends**, because every one pointed at the arc that just
closed:

| file | says | reality |
|---|---|---|
| `251-types-as-forms/SEAM.md` | *"do not resume from this file — go to `255/SEAM.md`"* | 255 redirects onward to 294, which is **done** |
| `255-builtin-registry/SEAM.md` | *"PARKED — go to `294/SEAM.md`"* | 294 is **complete**; ⛔ **now points HERE** |
| `255-builtin-registry/CURRENT-STATE.md` | branch `arc-170-gap-j-v5-deadlock-state`, `FLOOR 0 — 4285 passed` | **2 months stale.** We are on `main` at **5918**. ⛔ **It will mislead you within a minute — it still carries 296's live tail, so read it for THAT and distrust every number** |
| `278-rules-engine/SEAM.md` | the rete arc | its subsystem is what the replay just landed |

⛔ **The lesson, already paid for twice:** a durable pointer must not live in a section that gets
rewritten. This file's ledger accumulates; its stamp rotates. Put claims that must survive in the ledger.

---

## ⭐ WHAT IS **NOT** DECIDED — the builder's ruling

**Which arc resumes is the builder's call, not the orchestrator's** — opening or resuming an arc is a
ruling (`[[feedback_opening_an_arc_is_the_builders_ruling]]`).

What the chain says *mechanically*, offered as a measurement and not a recommendation: **296's remaining
tail sits at the head**. Its own notes call the `Failure`/`ProcessDiedError` de-stringify **"THE real
heresy"** — registered wat types declared `:String` but carrying edn-as-text, so a receiver does
`(edn::read (Failure/message f))`. That is the double-encode 296 R1 named, and the derive sweep is
blocked behind it. Also open in 296: **`deferror`** (modelled, crowned N8, **unbuilt** — grep → 0) and
**the L1/L2 close-gate** (cast the wards on the error subsystem and drive lie+mumble to 0).

**Nothing is in flight. The tree is clean, `main` is green and pushed, and no batch is running.**
