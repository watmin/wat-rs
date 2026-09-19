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
296/298  EDN errors ──► 255  registry ──► 251  clojurification        278  rete: ✅ COMPLETE
```

⭐⭐ **278 IS COMPLETE — BUILDER'S RULING, 2026-09-19.** *"278 should be assumed to be complete… that
was the whole 9 day grind of getting grok-rete merged into main."* The 2026-08-13 banner said 278 would
*"resume once we have the clojure syntax conversion complete"*; **that is superseded** — the replay WAS
278's completion, not a prerequisite for it. The tooling 251 needed (a rules engine classifying by
POSITION, an extractor turning source into facts, a diagnostic naming the cause not the call site) all
landed and is on `main`. ⛔ **Do not re-open 278 expecting pending work.**

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

## ⭐ THE LIVE WORK — RULED 2026-09-19: finish the clojure/EDN compliance

**Builder:** *"we need to keep working on moving the codebase to clojure compliant syntax — the last
outstanding item for non-edn compliant is our keywords having `::` in them… for not being clojure
compliant… we need to make all call heads a symbol."*

Those two items are **already designed**, as the last two stones of arc 251's campaign
(`251-types-as-forms/DESIGN-STONE-251.8-symbol-proper.md`, 466 lines, DRAWN 2026-08-13):

| stone | what it is | state |
|---|---|---|
| **8a** the vocabulary at zero offenders | `$bound` reserved; `namespace()`/`reference?` as the ONE DOOR | ✅ landed |
| **8a-ii** the binder namespace unforgeable | refused at the READER — no-form, not a check | ✅ landed |
| **8b** invert the normalizer | `Identifier` stores `(ns, name)` — ✅ `71c9f2f58`. ⚠ **The INVERSION half may still be owed**: `resolve/normalize.rs` still reads as Symbol→Keyword ("normalize all namespaced symbol refs"). **Confirm before drawing 8c.** | ⚠ partial |
| **8c** **close the check hole (#95)** | ✅ **LANDED `bc93125aa`, pushed.** Floor 5919/5919, clippy 0, census `no STOP-8`. ⛔ **The REAL cause was neither the design's nor the brief's:** `check_program` walked **pre-normalization `FunctionBody` snapshots**, so the class was **every namespaced-symbol call inside a function body**. Fix: normalize stored bodies after residue normalization — one path, no second dispatch. Verified on main: `(user/f "boom")` → `TypeMismatch ":user::f: parameter #1 expects i64; got String"`, **byte-identical to the colon spelling**. | ✅ |
| **8d** **the reader/printer flip** | `::` retires as a reference spelling; a colon means keyword, full stop; the corpus flip lands here | ⭐ **NEXT — UNBLOCKED** |

### ✅ THE HOLE IS CLOSED (8c landed 2026-09-19) — and BOTH prior diagnoses were wrong

⛔ **Recorded because the orchestrator was wrong twice and the counterpart was right.** The 2026-08
design said *"args, arity and return are ALL unchecked"* because `infer_list` gates on
`if let WatAST::Keyword`. The orchestrator's brief then said most of that was already closed and the
residue was one shape (`format`'s `:v`) — **and that measurement was a PROBE ARTIFACT**: four rows read
`rc=1` as "caught" while a *different* error fired, in the same document that warned against exactly
that mistake. **The counterpart killed the hypothesis and found the true cause**: `check_program` walked
**pre-normalization `FunctionBody` snapshots**, so normalized residue was checked while stored function
bodies kept raw `Symbol` heads — the class being **every namespaced-symbol call inside a function body**.
⛔ **The lesson, seventh instance: check WHICH error fired, never the exit code.**

### The pre-8c evidence, kept for the record (measured 2026-09-19, before the fix)

The design's cut stands: *"the corpus flip… must not begin while a dotted call head is unchecked."*
**But the design's headline — "args, arity and return are ALL unchecked" — is STALE.** Measured by hand
at HEAD, callee `(:user::f [n <- i64] -> i64)`:

| shape | colon head | slashed head | |
|---|---|---|---|
| statement · let-bound · plain nested arg · `assertion-failed!` kwarg | rc=1 | **rc=1** | ✓ same — **already closed** |
| **`format` `:v` value** — result lands in a type-unconstrained position | rc=1 | **rc=0** | ⛔ **the residue** |
| unresolvable head `(user/nope 1)` | — | rc=1 `UnresolvedReference :path ":user::nope"` | ✓ the normalizer fires |

⛔ **So "the checker never sees a Symbol head" is NO LONGER the defect** — `resolve/normalize.rs`
converts `Symbol → Keyword` and resolution works. **The residue is narrow**: arg/arity validation is
skipped when nothing downstream constrains the call's result type. **8c still comes first** — an
unchecked shape is an unchecked shape, and 8d must not run over one — but it is a smaller stone than the
design implies, and its brief says so rather than inheriting the 2026-08 framing.

### ⭐ QUEUED BEHIND 8c — the builder's namespace ruling (2026-09-19)

> *"everything to the left of the first `/` is the namespace, everything to the right is the name…
> pathological names are tolerated gracefully… **the first slash separates namespace from name**."*

`RULING-the-first-slash-separates-namespace-from-name.md`. **Today the code does the opposite** —
`Identifier::bare` splits on `rfind('/')`, the LAST slash, and its own comment calls that *"today's
split"*. On `wat.core//` that yields `["wat.core/", ""]`, an **empty name**; the ruling yields
`["wat.core", "/"]`.

⭐ **Measured, and it migrates NOTHING: of 6,292 slashed identifiers in code positions, 15 have 2+
slashes, and all 15 are file paths in string literals — ZERO real identifiers.** The rules agree on
everything that exists; the ruling defines the pathological case that is currently accidental.
⚠ A first count said 3,984 and would have made this look large — it was matching file paths in
**comments**. Strip comments, restrict to code, read the survivors.

⭐ **UNBLOCKED — 8c landed.** It was held only to keep two hands out of the identifier/resolve
neighbourhood at once. **Its own small stone; can go before or after 8d, builder's call.**

### The scale of 8d, measured

**169,845 `::` occurrences across 2,440 of 2,483 tracked `.wat` files**, plus 826 Rust files carrying
`:wat::`. This is R21's codemod path at a scale beyond anything in the replay — **read
`294-holon-returns-to-vsa/REPLAY-PLAYBOOK.md` §6 before drawing it**, and note the design already calls
it *"a spelling change over a substrate that already type-checks both"*, which is only true after 8c.

### What 8c may still need from elsewhere

⚠ **Unresolved, and it decides 8c's brief:** 251's seam says *"255 closes #95 because `type_sig` was
ruled day-one"*, while the design says **8c** closes it. Both can be true — 8c makes the checker SEE a
Symbol head; 255's registry supplies what to check it against. **Measure this before drawing the brief**;
do not assume either doc is the whole story. The rest of the chain (296/298 → 255) is unchanged and still
blocks the registry.

---

**Nothing is in flight. The tree is clean, `main` is green and pushed, and no batch is running.**
