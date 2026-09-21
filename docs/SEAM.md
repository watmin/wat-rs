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
### ⚠ 2026-09-19 — THE EDN SOURCE OF TRUTH IS IN QUESTION (builder's pivot)

Builder: *"there must be only one source of truth for edn compliance… our pivot is attack wat-edn
crate as its correctness is now in question."*

**Measured:** `wat-edn` and `wat-reader` are **two different languages**, diverging on **12 of 29**
probed cases in BOTH directions. `wat-reader` implements no `#` dispatch beyond `#{}`, so **wat
cannot read back the EDN it prints** — `#wat.core/Span {…}` reads as 2 forms (a symbol literally
named `"#wat.core/Span"`, then a map). `wat-edn` deviates from the EDN spec on 4 confirmed rows
(`a:b`, `a#b` refused — the spec allows `: #` as non-first constituents; duplicate map keys and set
elements accepted AND KEPT, since `Map` is a `Vec` of pairs).

⛔ **The orchestrator asserted a spec sentence that does not exist** (*"as is `clojure.core//`"*) and
reported a bug that may not be one — the spec says `/` "can be used once only". `value.rs:312`
carries the same false claim. **Two careful readings of four spec sentences produced two opposite
wrong answers**, which is why the cure is the differential oracle, not more reading.

✅ **218.8 LANDED** (stone `a1fa08d36`) — **arc 219 is CORRECTED.** The builder reopened it after
verifying in a live REPL that `a:b`/`a#b` are not merely readable but **definable and callable**
Clojure fns. ⛔ **It was NOT the one-line revert it looked like:** a naive add of `:`/`#` to the
per-byte `is_symbol_continue` would have accepted `a::b`, `wat::core::x` and `x:`, all of which
Clojure refuses — three new bugs. The oracle-derived rule is **`#` unrestricted; `:` legal only when
neither DOUBLED nor FINAL**, enforced **per name-component** (the counterpart's catch: `a:/b` has no
`::` and no trailing `:`, but its *prefix* `a:` is colon-final). Corpus 190→221, dry. Floor
5924/5924, clippy 0, census clean. ⭐ **No `.wat` file moved** — verified, not argued; `wat::core::x`
stays refused by both readers, so 8d's 85 tokens are untouched.

✅ **218.7 LANDED** (stone `fccc7b45d`, fold inside it). Floor 5921/5921, clippy 0, census `no STOP-8`.
Corpus **72 → 190, GENERATED** (`generate_corpus.py`), golden regenerated against real `clj` and
**verified byte-identical by the orchestrator's own run**; looped to **dry** (round 3: 0 new).
Fixed: i64→BigInt promotion, duplicate map keys, duplicate set elements, and — found BY the oracle —
`.5`/`5.` as floats. Refused with named spec clauses: quote, metadata, multi-slash, date-only
`#inst`, `\ `. ⛔ **Arc 219 STANDS** (not reverted, as instructed); blast measured at **0 real
`a:b`/`a#b` identifiers**, so re-permitting is free whenever the builder rules — the 154k `::`
keyword tokens are 8d's, not 219's. **The ward was proven to CATCH**, not merely pass.

⭐ Brief: `docs/arc/2026/05/218-wat-edn-impeccable/BRIEF-STONE-218.7-the-oracle-runs-to-dry.md`.
The clj-differential ward already exists and is correctly designed; its **corpus is 72 hand-written
cases** with every failing case absent and zero multi-slash or big-int rows. The stone is to
**generate** the corpus and run the ward **to dry**. ⛔ It also records that **arc 219's premise does
not hold** — it deleted `:`/`#` quoting a spec summary that omits the sentence permitting them.
**The ruling stands until the builder rules; do not revert it.**

⭐ **Ordering:** 218.7 makes the source of truth trustworthy → 251.8d retires `::` → arc 300's
highlander (one reader) becomes possible. 8d is what makes the dual implementation *collapsible*.

### ✅ 255.4 LANDED 2026-09-21 — ONE member join. **Delta 80 → 77.** Arc: **104 → 97 → 80 → 77**

Floor 5939/5939, clippy 0, census clean. **`Type/member`, always** — builder's Option 2 (4-YES).
⭐ **One-door find:** `wat-macros/codegen.rs` `method_wat_path` was *generating* the `::` join; one
`format!` unified `rust.cache`, `rust.sqlite` and the shims at once. Two more emitters found
(`types.rs`, `wat/service.wat` `surface-forms`).
⭐ **Retirement rows now TEACH at check time, with parity proven:** `HandlePool::new` and
`:wat::core::struct` both emit `Remedy {:kind :retirement}`. Not asserted — measured.
⭐ **`.wat.bad` glob gap caught** — `git ls-files '*.wat'` misses it; the third glob miss in this arc.

⛔ **THE SEQUENCE IS THE LESSON: 28 → 3 → 32 → 0.** The **32** came from the teach-fix adding a
**SECOND retirement door** — arc 241 already owned teaching, and a plainer message firing first
*looked* like a fix while regressing what an author is told. **Cure: delete the arm; make the
interceptors stand aside (`&& !is_retired(k)`).** A second door is the defect this arc exists to
remove, and it was committed by the arc's own cure three stones after 255.2 refused exactly that.

**Residue (77):** `unresolved reference` **44** · rete 16 · `defsurface` 8 · programbody 3 · other 1.
**Next: 255.5 — the position grammar** (`:wat::WatAST` + declaration names), which 255.2 proved
cannot be a predicate over leaves.

### ✅ 255.3 LANDED 2026-09-21 — the join. ⭐ **DELTA 97 → 80, NET −17** (first real movement)

Floor 5937/5937, clippy 0, census clean, 0 `.wat` converted, test +1.
⭐ **Two trees, identical net:** executor 101 → 84, orchestrator 97 → 80. Arc so far: **104 → 97 → 80.**
`unresolved reference` **68 → 47**.

**What landed:** `types::reconstruct_call_path` asks the registry whether the namespace's last
segment is a TYPE and joins with `/` when it is — ⛔ **not capitalisation**. Routed through
`resolve/normalize.rs` and `macros/expand.rs`. `ns_to_wat_path` unchanged (cannot import TypeEnv —
cycle), so **call** reconstruction asks the registry while type *names* stay `::`.
⭐ **This is why the registry had to come first:** the question *"is the last segment a type?"* only
became answerable when `wat.type` got members in 255.1.

⛔ **2 newly broken, and the cause is the finding:** both on `:wat::core::Bytes/to-hex`. **The
member's join is a SECOND registry fact.** Measured corpus-wide: **4,501 `Type/method` vs 83
`Type::method`** — the door is right 98.2% and wrong for a legacy minority, and both look identical
to "is the last segment a type?".

⭐ **BUILDER'S QUESTION FOR 255.4 — why are there two member joins at all?** 4,501 vs 83 is drift,
not design. (1) teach the door the second fact, keeping both spellings forever; or (2) **unify the
registry on one join**, deleting the question. The campaign's standing preference is the no-form rung.

**Not started:** the position grammar (`:wat::WatAST` 79 occurrences + declaration names). The
executor STOPPED after step 1 as instructed, so the two causes stay attributable.

### ✅ 255.2 LANDED 2026-09-21 — the arrow door. **Delta FLAT at 97. 8d still blocked.**

Floor 5936/5936, clippy 0, census clean, 0 `.wat` converted, test delta +0.
⭐ **Arrow set DERIVED, not enumerated:** one door (`types::is_return_arrow` = bare `->` **or**
`is_binder_marker`) routed through `types/surface.rs`, `function/parse.rs`, `macros/parse.rs`,
`declare/parse.rs`, `intrinsic/holon/atom.rs`. Config-setter head dual-reads (3b for **one**
string-match table; others remain).

⚠ **The SCORE's "97 → 101" compared UNLIKE TREES.** Re-run like-for-like: **97, unchanged.** The
KIND moved though — `defsurface` arrow class **11 → 8**, those files landing on a later
`UnresolvedReference` (66 → 68), which is why the total is flat.

⭐⭐ **THE STONE'S BEST OUTPUT IS A MEASURED NEGATIVE.** Three derived cures for gap 3a were tried
and **each RAISED the count** (116 · 108 · 114). The reason: *"skipping every declare-role leaf
cannot tell a name slot from a value that must still resolve — function-as-value (`wat.core/+` as an
argument) and a declaration name (`u/x`) are the same node kind. **A position grammar (items[1] of
the declare form only, not every leaf) is the next peel.**"*
⇒ **3a is a GRAMMAR problem, not a predicate problem.** That is 255.3.

### ✅ 255.1 LANDED 2026-09-21 — identity is the pair. **8d STILL BLOCKED.**

Floor 5936/5936, clippy 0, census clean, no `.wat` converted. **Delta 104 → 97.**
⭐ Bought: `canonical_identity` (one door, both spellings, one key) · **`wat.type` is REAL**, members
**derived from denotation** so a future core builtin joins automatically · `one_param_spec` **routed,
not runed** (`is_binder_marker` moved to `wat-reader`) · **one position, not two** — a call-position
`wat.type/Vector` now checks · `Infer` kept a marker via a **wall exemption**, after the executor
reported that denotation *cannot* express "reachable but not a type".

⛔ **RESIDUE ON THE 97 — what 255.2 owes:** `unresolved reference` **66** (gaps 3a+3b, untouched) ·
`ReteCheckErrors` 12 · ⭐ **`defsurface` declaration 11 — a form the brief never listed (154 files)**
· `ProgramBodyEvalFailed` 3 · other 3.
⚠ The count moves slowly because a file with two gaps stays red until both close: gap 1 is CLOSED,
gap 2 went 54 → 3. **255.2 must DERIVE the declaration-form set, not extend the list by one.**

### ⛔⛔ 2026-09-20 — 8d IS BLOCKED. THE REGISTRY GOES FIRST. (builder's ruling)

**8d's premise — *"a spelling change over a substrate that already type-checks both"* — is FALSE.**
It holds for CALL SITES (8c proved it). It does **not** hold for declarations or for some top-level
resolution. Measured as a DELTA on real codemod output: **104 of 161 previously-clean files regress
(65%) ⇒ ~1,240 corpus-wide.**

**THREE independent gaps, and the biggest is not the one first found:**

| gap | mechanism | sample regressions |
|---|---|---|
| macro declarations | `defrecord`/`defstruct` — *"program body eval failed"* | **52** |
| slashed **resolution** | a top-level `(wat.core/def u/x 1)` or `(wat.config/set-capacity-mode! :panic)` does **not** resolve where the colon spelling does — **the 8c class, residual** | **34** |
| declaration **names** | `types.rs:4684 parse_declared_name` — Keyword-only, and it stores the **RAW KEYWORD as the TypeEnv KEY** | **8** |

⛔ **The second defect is the real one: TYPE IDENTITY IS THE SPELLING.** `wat.core/Option` would be a
*different key* from `:wat::core::Option`. That is a registry ruling, not a parser fix.

⭐ **BUILDER'S RULING (2026-09-20) — Option A, the only 4-YES:** *"make identity canonical; both
spellings resolve to one key."* A type's identity is its **`(namespace, name)` pair, not its
characters.** And the sequencing: **255 next (identity + `wat.type` together) → 8d-ii → 8d-iii.**

⚠ **THE SEAM WAS RIGHT AND THE ORCHESTRATOR WAS WRONG.** On 2026-09-20 it wrote that the chain had
*inverted* because 8c closed #95 without the registry. ⛔ It had not. #95 was closed without 255, but
**8d-iii is gated by declaration-name identity, which IS registry work.** The line below —
*"if you remember it the other way round, that is the pivot talking"* — was ignored within a day of
being re-read. **The mechanism it named (#95) was not the one that binds; the instruction was right
anyway.** Full detail: `251/FINDING-8d-the-premise-is-FALSE-for-declaration-names.md`.

### ✅ 251.8d-i LANDED — THE CODEMOD IS TOTAL (2026-09-20, stone `a83eae411`)

**Census: 2145/2145 OK — 0 class-A, 0 class-B, 0 class-C**, verified on the orchestrator's own
independent run, not accepted from the score. Floor 5930/5930, clippy 0, census `no STOP-8`.
⭐ **ZERO tracked `.wat` files converted** — 4 new fixtures + `wat/fix.wat` only.

The three cures, all recorded and re-runnable **against an arbitrary branch** (the builder's merge
constraint: *"we have many more large scale branch merges to contend with"*):
- **A** `ast-name` partiality — kind-guard `head` as `c2` already was.
- **B** reader-synthesized spans — **general rule, no `~` in it**: *edit a leaf only when the source
  text at its span EQUALS its `ast-name`; where they disagree, skip.* Covers `~ ~@ ` ` '` without
  enumerating them, which is what makes it survive a branch using a reader macro we have not seen.
- **C** namespace-prefix markers **CONVERT** (`:my::kernel::` → `my.kernel`), plus the parser half:
  `:restricted-to` accepts Symbols, **hard-errors** on anything else (the silent `filter_map` drop is
  dead), and discriminates on `/` — as `entry/` **or** `entry.`, because a prefix must admit
  `ns.Type/method` too.

⛔ **OPEN, for 8d-iii:** `FINDING-8d-a-symbol-whitelist-entry-gets-RESOLVED.md` — a slashed symbol
entry is walked as a reference and must exist; a bare namespace entry is not. Builder's push
improved the cure to **exempt from resolution + validate explicitly**; measured that we always know
(forward-in-file, cross-file, and Rust-registered names all resolve).

**Next: 8d-ii** (the `wat/` bootstrap — `wat/fix.wat` is `include_str!`'d and the flip rewrites the
codemod's own source), then **8d-iii** (the flip: 67,065 lines, 2,021 files).

| **8d** **the reader/printer flip** | ⭐ **BRIEF DRAWN + PRE-FLIGHTED** `BRIEF-STONE-251.8d-the-corpus-flip.md`. ⛔ **The tool ALREADY EXISTS** (`to-faithful-clojure.wat` → `:wat::fix::fix-text`), works, idempotent — 8d is *"make the migration survive the corpus"*, not *"write the migration"*. **Corpus census, all 2,140 files: 2,021 OK · 99 class-B · 20 class-C · 0 class-A.** Real size: **67,065 source lines** across 2,021 files. ⛔ **THE SEAM'S OLD NUMBERS WERE ALL WRONG** (said 169,845 `::` over 2,440 of 2,483 files; truth is 2,190 tracked, 2,140 to flip). Proposed as 3 stones — builder rules the split. | ⭐ **NEXT** |

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

✅ **STRUCK AND LANDED 2026-09-19 (`261476098`).** Floor 5920/5920, clippy 0, census `no STOP-8`.
⭐ **Measurement moved the justification:** this was not an undefined question — `wat-edn`'s
`split_namespaced` was **already** first-slash and `wat-edn/src/value.rs` already named
`clojure.core//` as the exception. The wat surface was **the outlier**, so the ruling makes it agree
with the layer that owns EDN compliance. ⛔ And `receiver`/`method` had to move with it: they are the
**namespace splitter** on `normalize.rs`'s path, so flipping `bare` alone would have left one process
with two namespaces for one string. `wat.core//` now **resolves, to division**. Score in the ruling doc.

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
