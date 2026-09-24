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

### ⭐⭐ RULING 2026-09-23 — EXACTLY ONE WAY TO DO THINGS

> **Builder:** *"we need to have exactly one way to do things.. us having two write paths is clearly an
> indicator of sloppy code"*

⛔ **Two paths to the same thing is a defect in its own right**, not a style choice. It is the same
instinct as the earlier EDN ruling (*"i strongly view this as a highlander... there must be only one
source of truth"*), now stated generally.

**What it settled immediately** (weighed with the four questions, `FINDINGS-INTUERI-the-name-families.md`):

1. ⭐ **Service lifecycle → option E (generic).** One `wat.service/start` (and `stop`, `hibernate`, …)
   dispatched on the service — the way `wat.spawn/Locus` is already dispatched via `extend-type`
   (47 uses) — **not** a `svc/start` minted per service (option D = N copies of one path).
   User operations own the service's namespace; everything the macro generates leaves it, so a user op
   and a lifecycle method **cannot** share a name. ⚠ **E's Simple cell is UNMEASURED:** `start` is typed
   per service (its state and handle types) — whether ONE generic `start` can carry them needs a probe
   before E is drawn. ⛔ **The ruling also binds E itself: the per-service `svc/start` must be RETIRED,
   never kept beside the generic one** — that would be the two paths this ruling forbids.
2. ⭐ **One name per message type → keep `<S>/<Op>Request`** (e.g. `StdOut/WriteRequest`, the name the
   checker already enforces, `types.rs:3647`) and **retire the alias `<S>::<op>/Request`**.
   ⚠ The alias exists to re-attach the surface's type parameters (`types.rs:4137-4142`); retiring it
   needs another way to do that — **unmeasured.**

**What it implicates next** — every dual path in the tree is now a named defect: the dual
keyword/symbol acceptance (the terminal cut), `/` vs `::` as two joins at one position, `ns_to_wat_path`
vs `reconstruct_call_path` as two identity reconstructions (255.8's open hole).

### ✅ 255.12 LANDED 2026-09-22 — ⭐ **THE 8d-ii BLOCKER IS CURED.** Delta 4 → 3

Floor **5986/5986** (after a captured RED), clippy 0, census `no STOP-8`, **RECOVERY 0**, 0 live
`.wat`. **Sites: 3 of 4 cured + 1 found outside the brief; site 2 REFUTED and reported.**

⛔⛔ **THIS STONE CURED IN THE *PERMISSIVE* DIRECTION** — the first since 255.9 to do so. Denotation-
aware equality makes walls **accept more**. ⭐ **The adversarial row was the whole stone and IT CAUGHT
THE CURE'S FIRST DRAFT.** Orchestrator proved the guard load-bearing by deleting the carve-out:

```
carve-out DELETED  → (defrecord :p::Inf [n <- :wat::type::Infer]) + [n <- :wat::core::Infer] → rc=0 ⛔⛔ ACCEPTED
carve-out RESTORED →                                                                            rc=1 DuplicateType
legitimate cross-spelling case                                                                  rc=0 ✅ still passes
```

⭐ **The carve-out already existed inside `check::format_type_path` — A RENDERER.** Every other
consumer of `type_denotation` was collapsing `Infer` silently. ⭐ **`type_defs_same` is `==` over
DENOTATION-NORMALIZED defs, not a field walk**, so `nature`/`purity`/`restrictions`/`type_params`/
variant names/field order stay byte-compared — **that shape is what makes the permissive direction
survivable.**

⭐⭐ **SITE 3 = THE 8d-ii STOP, PROVEN ON TWO BINARIES, WITHOUT CONVERTING THE STDLIB:**
`(:wat::edn::validate 42 :wat::type::i64)` was `Invalid {:expected ":wat::core::i64" :got "Integer"}`
— ⛔ **a self-contradicting refusal** — now `Valid`, while a String is ⭐ **still `Invalid`**.
⚠ **Neither state is visible to `--check`.**

⛔ **SITE 2 (`Ngram`) REFUTED, NOT CURED.** The blocker is **not** `is_reference()` — the cross arm
compares `canonical_identity` and `"<-"` ≠ `":-"`, returning false first. **The orchestrator's lead
was wrong**, refuted three ways. The collision is the `include_str!`-baked copy meeting the converted
on-disk one (**path-independent, proved**). Curing it = teaching a blind walker `<-` ≡ `->` ≡ `:-`
inside a template ⇒ **two macros emitting different surface forms become one.** ⭐ **Correctly
refused — 8d-ii's job / BUILDER'S CALL.**

⛔ **THE FLOOR RED — arm 2 is a FINDING, NOT A FLAKE.** `harvest_cost.rs:337` is a **wall-clock
apportionment bound** and ⭐ **the test's own comment concedes it**: *"wall clocks on a shared
runner."* Not this diff (timed path = `PVec::iter`/inline closure/`PMap::from_pairs`/`Value::clone`,
none of the 8 changed files; last touched by a different stone). ⛔ **"Timing" is NOT a disposition
here:** a ratio over single-iteration wall clocks sampled during a 5,986-test parallel floor **cannot
be a gate. IT WILL FIRE AGAIN — it needs a stone.**
`[[feedback_a_wall_clock_ratio_is_not_a_gate]]`.

### ⭐⭐ BOTH INSTRUMENT DEBTS CLOSED — after three stones

- **`scripts/replay/delta.sh`** — RECOVERY standing, non-zero **exits 9**. ⭐⭐ **It caught a defect
  in ITSELF on run 1:** `wat` reads stdin, so a `while read … < LIST` loop was eaten after **10 of
  179** files **and printed a plausible delta.** Now `xargs` + `/dev/null` stdin, and it **refuses to
  report a partial measurement.**
- **`src/freeze/pass_order.rs`** — 14 passes announce themselves; a unit test pins the order. ⭐ **It
  has FAILED once** (moving `resolve_references` ahead of normalize reds it — **and the mutated
  program still froze clean**). Carries its own non-vacuity (`expand < normalize < check`) and
  ⭐⭐ **the instruction that makes it a gate: do NOT re-order the array to match the code until you
  have re-read every "post-step-7 ⇒ unreachable" disposition.** Corrected the documented pipeline:
  `normalize_stored_function_bodies` runs **twice**.
  ⛔ **It pins ORDER, not the CODE-vs-DATA half that actually misled 255.9/255.10** — that needs a
  **post-normalize residue census. A STONE.**

⭐ **`Infer`'s uniqueness, narrowed by the orchestrator:** 4 other candidate markers (`Seq`,
`Stream`, `HolonAST`, `Thermometer`) are **refused by the ANNOTATION WALL before the comparator
runs**. ⭐ **`Infer` is dangerous because it PASSES that wall with no `core` counterpart**;
`INFER_TYPE_PATH` is the only such constant (4 carve-out sites). ⛔ **A narrowing, not a proof.**

### ✅ 255.11 LANDED 2026-09-22 — the wall audit. ⛔⛔ **A LIVE CAPABILITY ESCAPE WAS FOUND AND CLOSED**

Floor **5968/5968** (no red), clippy 0, census `no STOP-8` (212 = 212), delta **4 = 4**, RECOVERY 0,
**0 live `.wat`**. **26 walls · 5 FAIL-OPEN · 4 cured · 1 reported.**

⛔⛔ **THE ORCHESTRATOR'S OWN FINDING WAS WRONG AND ITS INSTRUCTION WAS DANGEROUS.** It said
`✅ :restricted-to — INTACT` / `⛔ Do not re-litigate`. **Had that been obeyed the escape would still
be in the tree.** Now retracted: `FINDING-RETRACTED-the-whitelists-were-NOT-intact.md`.

```
PRE-CURE  (:wat::core::quote (:my::kernel::restricted-fn 7))  → rc=1  DefRestrictedCallerNotAllowed
PRE-CURE  (:wat::core::quote (my.kernel/restricted-fn   7))  → rc=0  ⛔⛔ ESCAPED
POST-CURE both spellings                                      → rc=1, same error
POST-CURE a PERMITTED caller, symbol mention                  → rc=0  ✅ not over-restricted
```

⭐ **WHY THE FOUR EARLIER PROBES PASSED: A BACKSTOP, NOT THE WALL.** `normalize_symbol_refs` (step 7)
canonicalizes **CODE** positions, so a symbol *call head* arrived already rewritten. ⛔ **A DATA
position is never normalized**, and **arc 198's own ruling: a restriction governs MENTION, not head
position.** ⛔⛔ **The Rust-side `#[restricted_to(…)]` fences drain into the SAME
`binding_metadata` map read by the SAME walker** — pre-cure, from `:user::` code:
`wat.io.IOWriter/from-fd` **rc=0**, `wat.kernel/close` **rc=0**. All verified by the orchestrator on
two binaries.

⭐ **Cured:** `check.rs::walk_for_restricted_call` (both joins, via `other_join_spelling`) · both
mutation walls (`runtime.rs`, `freeze.rs`) · hygiene Gate E — ⚠ **which needed TWO sites**, because
curing `quasiquote_inner` alone left the wall silent.
⛔ **Reported, NOT cured — `is_quasiquote_form` is a ROUTER**: teaching it symbols would let a
symbol-spelled qq body **skip BOTH** the hygiene check and the F5 gate. **Residual asymmetry (the
KEYWORD path is now the weaker one) is the BUILDER'S CALL.**

⭐⭐ **"A GATE THAT CANNOT MOVE IS NOT EVIDENCE THE DIFF IS SAFE."** Delta 4→4 and census 212→212
prove **nothing** here — no sample file mentions a restricted binding in a data position, hands a
mutation form to `eval-ast!`, or nests an explicit quasiquote. **The evidence is the hand-written
probes and the two-binary pairs.**

⛔ **CORRECTED: 255.9's mutation disposition** ("no mutation head escapes both walls" — **two do**:
`set-redef!`, `set-eval-redef!`; blast radius nil TODAY, ⭐ *"currently harmless is not a
disposition"*) · **255.10's quasiquote row** (LOUD → **SILENT**) · **the orchestrator's seed list**
(counts four NON-walls — three are `#[test]` fns — and ⛔ **MISSED `walk_for_restricted_call`, the
wall that mattered, because it is not named `refuse_*`/`validate_*`. A NAME GREP CANNOT ENUMERATE
WALLS.**)

⛔⛔ **THE DEEPEST FINDING — 255.9 AND 255.10 BOTH WROTE THE PIPELINE RULE AND APPLIED ONLY ITS FIRST
HALF.** *"Post-step-7 ⇒ unreachable"* is true only of the **CODE** half of a wall's input. The
capability wall is a stage-8 site **both stones would have marked UNREACHABLE — and it was live.**

### ⛔ TWO INSTRUMENT DEBTS, NOW THREE STONES OLD

1. **RECOVERY (fail→clean) as a standing column** — printed by hand three stones running. *"One
   `awk` clause; belongs in `scripts/`."*
2. **A GATE ON PASS ORDERING** — recommended three times. ⭐ **Its absence is exactly what let two
   consecutive stones mark a live capability wall unreachable.**

### ✅ 255.10 LANDED 2026-09-22 — ⭐ **DELTA 18 → 4**, and ⛔⛔ **A DEFAULT-DENY SECURITY GATE WAS OPEN**

Floor **5963/5963** (×3, no red), clippy 0, census `no STOP-8` (212 = 212, 0 rc changes),
**0 live `.wat`**. 5 `src/` files. Arc: **104 → … → 49 → 18 → 4.**
⭐ **RECOVERY (fail→clean) = 0** — no forged green.

⛔⛔ **THE F5 EXPAND-TIME PURITY GATE WAS BYPASSED BY THE SYMBOL SPELLING.** Orchestrator built
**both** binaries and measured:

```
PRE-CURE   (:wat::kernel::println …) in a macro body → rc=1  "default-deny F5 gate, arc 249"
PRE-CURE   (wat.kernel/println   …) in a macro body → rc=0  ⛔⛔ CLEAN — the macro was DEFINED
POST-CURE  both spellings                            → rc=1, BYTE-IDENTICAL refusal
```

⛔ Arc 249 stone O's property — *"fails at definition, not silently at first use"* — **was
spelling-dependent**, and 8d-iii would have made the bypass the DEFAULT spelling. ⭐ **Found by
following a failing FILE, not by the census frame** — the argument for the evidence gate over a count.

**Six cures, each with a file that changed state:** `surface.rs:729` `:messages` NAME (**8 files**) ·
`surface.rs:1003` post-arrow TYPE (⛔ without it the wall goes **SILENT** — proven by targeted
revert) · `freeze/env.rs` rete `defn` head (**4 files**) · `edn/render.rs` `keyword-node` input
(**1+2**) · `macros/eval.rs` the F5 gate · `macros/expand.rs` `is_callable_form` (**1**).

⭐⭐ **THE TEST IS THE BEST IN THE ARC.** Row 1 asserts a **byte-identical identity path**, not "both
parse". Row 2 pins **full reasons** for three markers. Row 4 pins the wall naming the **TRUE**
undeclared type in **both** spellings — ⛔ **so the stone provably cannot trade a false red for a
false green.** The guard **has failed twice** under targeted revert.

⛔ **IT CORRECTED 255.9's CENSUS** — `validate_pure_total` was classified LOUD; it is **SILENT**
(confirmed by the pre-cure `rc=0`). **The frame both briefs used (`grep "Some(WatAST::Keyword"`)
cannot see a `matches!` in a closure or a bare `starts_with(':')` string test. Two of six cures were
outside it.** Re-framed by requirement SHAPE: A=83 · B(`matches!`)=32 · C(bare arm)=122 · D(`starts_with`)=24.
⛔ **A shape-B/D sweep is real remaining exposure.**

⭐ **The committed delta list paid for itself:** it caught the rider's OWN regression
(`vector_splice_symmetry.wat`, cure 5's first version broke `Option/expect`'s member join).

**The 4 survivors, reported not cured:** 2 arc-170 probes (a `::`-spelled **substring** searched in a
converted name — ⭐ **a `.wat`-level fix needing `:wat::keyword::canonical-identity`, which does not
exist**) · `wat/source.wat` + `wat/holon/Ngram.wat` (⛔ **NOT this class** — the `wat.type`
denotation gap in `TypeEnv::register_validated`'s raw `Some(e) if e == &def`).

### ⭐ THE NEXT STONE IS BIGGER AND BETTER-DEFINED THAN "B"

`register_validated` and the two runtime tables that stopped 8d-ii (`edn/render.rs:2508`,
`function/subsume.rs`) are **the SAME `wat.type` denotation gap**. ⭐ **ONE stone: wire type
denotation into every type-identity comparison.** It closes the **8d-ii blocker AND the last 2 delta
files**. ⚠ It makes a **duplicate-declaration wall more permissive** (red → false-green direction) —
**needs a row proving a genuine duplicate is still refused.**

**Then:** the 416-test rete remainder · the shape-B/D sweep · mint `:wat::keyword::canonical-identity` ·
⛔ **RECOVERY as a standing gate** (recommended twice, still not done) · ⛔ **a gate on the step-7
ordering** (recommended twice) · 255.8's wrong-join hole · 8d-ii (4th draw) · 8d-iii.

### ⭐⭐ RULING 2026-09-22 — THE KEYWORD REQUIREMENT IS THE DEFECT

> **Builder:** *"all of the conversions you detailed are correct — we need to break off of the
> keyword requirements onto symbols"*

⛔ **THE SYMBOL SPELLING IS CORRECT WAT. A READER SLOT THAT *REQUIRES* A KEYWORD IS A DEFECT BY
DEFINITION** — not a site to patch when it bites, **a class to eliminate.**

**The discriminator, already implemented in the codemod:**

| keyword | what it is | converts to |
|---|---|---|
| `:wat::core::defrecord`, `:my::svc::Oops` | a **NAME** (carries `::`) | ⭐ symbol — `wat.core/defrecord` |
| `:messages`, `:features`, `:nature` | a **syntax MARKER** | ⛔ stays a keyword |
| `:Ok`, `:Bad` — a variant tag in a **declaration** | ⭐⭐ **A NAME.** ⛔ **RULED 2026-09-22, correcting this brief.** `(wat.core/defenum u/Whatever wat.enum/Pure :- [T] Thing [t :- T] Wut [])` mints `u/Whatever.Thing` / `u/Whatever.Wut` — the tags are **bare symbols**, not keywords | ⚠ **NOT YET** — measured: the codemod leaves `:Ok` alone, while the minted name already renders `wat.core/Option.Some`. **The declaration and what it mints disagree TODAY.** Closing it needs a CODEMOD change — ⛔ **OUT OF SCOPE for this `src/` stone; assert NOTHING about variant tags in either direction.** |
| a map key `{:value 7}` | **DATA** | ⛔ stays a keyword |

⭐ **A slot expecting a NAME accepts both spellings through the identity door; a slot expecting a
MARKER or DATA is untouched.** ⛔ **Both halves are load-bearing — a cure that lets `:messages` be
read as a name is worse than the defect.**

### ⭐ WHAT THE RESIDUE ACTUALLY IS — 83% is ONE HABIT

Measured on the committed sample: **15 of 18** failures are three user forms, and every one — plus
both greens 255.9 found forged, plus the two tables that stopped 8d-ii — is **one slot that
pattern-matches a spelling instead of asking identity.**

| form | the slot | files |
|---|---|---|
| `defsurface :messages` | the message **NAME** (`surface.rs:746`) | **8** |
| a `defn` inside a rete rule | the **head** (identity right, not extracted) | **4** |
| a macro building a keyword from a name | `keyword-node`'s **input** | **3** |
| *(cured 255.9)* `load-file!`, `config/set-*!` | the **head** | — |
| *(open)* the two runtime type tables | the **type path**, `p.as_str()` vs a literal | — |

⛔ **The `defsurface` one is a FALSE RED that HIDES THE REAL ONE** — measured: `surface.rs:746` reads
`Some(WatAST::Keyword(mn,_))`, so after conversion `message_names` is **EMPTY** and every
**correctly-declared** type is reported undeclared, while the genuinely-missing type is never reached.
⭐ This is the arc's standing "`defsurface :messages` 8" residue, **now attributed**.

**Stone: `255-builtin-registry/BRIEF-STONE-255.10-…md`** (`a51ba7dea`). ⛔ **634 `WatAST::Keyword(`
read-sites under `src/` — DO NOT SWEEP.** 255.9's reachability method (pipeline order, step 7) took
82 head sites to **2**; reuse it. ⭐ **Gate on every cure: EVIDENCE, not count** — no cure without a
file or test that changes state; a site that cannot be made to fail is **REPORTED, not cured**.

### ⛔ 2026-09-22 — 8d-ii STOPPED A THIRD TIME, and 255.9 IS DRAWN. ⭐ **A CONVERSION CAN FORGE A GREEN**

⛔ **GROK IS OUT OF CREDITS.** Stones now run on a **fresh Opus rider** (a cold general-purpose agent
reading the brief, **never a fork** — a fork inherits the orchestrator's context and the weigh becomes
self-grading). ⭐ **The swap worked:** the cold rider found a defect nobody predicted, in code the
brief never mentioned, and **stopped rather than forcing a green.**

**8d-ii, third draw: STOP.** ⭐ **The brief's central gate is what caught it** — *"it starts" is not
"it works."* The converted stdlib **converts (64/64), builds, loads, `--check`s green** — and then the
converted codemod **converts nothing** (rc 2, 0 bytes): `(:wat::kernel::readln)` dies in stdin
framing. **Three stones, three layers, each green one layer above the failure.**
Cause: `wat.type/i64` vs `:wat::core::i64` compared as **raw strings** at two runtime sites
(`edn/render.rs:2508`'s hardcoded table, `function/subsume.rs::value_matches_type_by_name`).
255.8's `type_denotation` is wired into `is_subtype`/equatable/orderable — ⛔ **not into the two sites
that decide whether a program RUNS**, including the one in its own file. Floor RED **739**, captured
whole, not re-run; under two diagnostic probes **416** (B ⊂ A, 0 new). Nothing landed.

### ⛔⛔ THE FINDING THAT OUTRANKS THE STOP — and it was OUT OF SCOPE

`src/load/loader.rs::match_load_form` matches **Keyword heads only** (`_ => return Ok(None)`). The
codemod writes `(:wat::load-file! …)` → `(wat/load-file! …)`. **Reproduced in the weigh:**

```
(:wat::load-file! "missing.wat")  → rc=1  "load: file not found: missing.wat"
(wat/load-file!   "missing.wat")  → rc=0
```

⛔ **The converted load form is NEVER FLAGGED** — not malformed, not unresolved. **An honest failure
becomes a PASS.** 26 tracked `.wat` use the six load forms, **0 in `wat/`** — ⛔ **8d-iii converts all
26 and the corpus gets GREENER as its loads vanish.**
⛔⛔ **THE DELTA GATE IS BLIND TO IT:** all 4 load-form files in the 179 sample are `CLEAN → CLEAN`.
**Green before, green after, loads gone.** No count run in eight stones could have seen this.

⭐ **It is a CLASS, not a form.** A crude probe finds **83** keyword-head matchers, ~14 near a silent
fallthrough (`runtime.rs` 4, `macros/expand.rs` 4, `rete/purity.rs` 2, `rete/kernel/stratify.rs` 2,
`load/loader.rs` 2). ⚠ **That is a LEAD, not a census** — producing the real one is 255.9's first job.

### ⛔ THE DELTA SAMPLE RE-RANDOMIZES ITSELF — now FIXED

"Every 12th of the census" is an **index over a GROWING list** (2145 files then, **2152** now), so one
inserted file shifts every later pick. Two reconstructions of "the same" 179 files shared **4 entries**.
⭐ **Three near-disjoint samples returned 18, 18, 16** — the finding is robust, **the instrument was
not.** ✅ **The list is now committed:** `251-types-as-forms/delta-sample-179.txt` (sha `33ede76c…`).
⛔ **USE THE FILE. NEVER REBUILD BY INDEX.**

**`wat/holon/Ngram.wat` answered:** the `DuplicateMacro` is **transitional AND symmetric** — measured
both ways. ⛔ **In 8d-iii the hazard does not vanish, it CHANGES SIDES** (every still-unconverted file
re-declaring a stdlib macro/type shows a spurious duplicate).

### ⭐ THE QUEUE, in order, and WHY this order

1. **255.9 — the keyword-only head** (DRAWN, `3c32ac281`). ⛔ **First, because it is the only one that
   CORRUPTS THE EVIDENCE rather than merely failing.** Every later measurement is untrustworthy until
   a conversion can no longer forge a green.
2. **`type_denotation` at the two runtime string-table sites**, each with a probe that reds without it.
3. **The 416-test remainder** (led by `wat::rete`, 195, losing operand types under the converted stdlib).
4. **255.8's wrong-join acceptance** — `(wat.core.Option.expect …)`, no slash at all, still resolves.
5. **Then 8d-ii** (fourth draw), **then 8d-iii.**

### ✅ 255.8 LANDED 2026-09-21 — a namespace that is also a type. ⭐ **Delta 49 → 18**

Floor **5959/5959**, clippy 0, census `no STOP-8`, **0 live `.wat` converted**.
Arc: **104 → 97 → 80 → 77 → 66 → 61 → 49 → 18.** ⭐ **The largest drop of the arc.**

⭐ **`reconstruct_call_path` was INNOCENT** — its body is byte-identical to the base, verified. A
type annotation never reaches it; the annotation slot rewrites through `ns_to_wat_path` (always
`::`). ⭐ **The brief's own hint was the right one** (*"the bug may be the ROUTING, not the join"*),
and the answer was that it does not reach the join at all. **Sixteenth correction.**

⭐⭐ **THE ONE DOOR LINTS PROVED THEMSELVES.** The stone disclosed an earlier RED floor (66 failed,
`.floor/2026-09-21T23-21-45Z/` with `ARM.txt`, captured whole, **not re-run to green**). What went
red: `one_variant_separator` and `one_name_grammar` — **the first attempt added a second name parser
and the gate caught it.** ⭐ **`tests/lint/` is untouched in the entire diff**: the code was fixed,
not the gate. A negative control (`row3_defclause_still_rejects_non_type_return_slot`) also fired and
**survives explicitly** — bare `n` is still rejected.

⛔ **FINDING THAT GATES 8d-iii — the wrong join is ACCEPTED for a SYMBOL author.** Measured:
`(:wat::core::Option::expect …)` REFUSED ✅ · `(wat.core.Option.expect …)` — **no slash at all** —
⛔ **ACCEPTED**, because identity makes `::expect`, the new `other_join_spelling` flips it to
`/expect`, and the registry holds that. **Contained today** (a keyword author never reaches the
fallback). ⛔⛔ **8d-iii un-contains it**: after 2,076 files become symbols, a wrong-join call head
**silently resolves** instead of raising `UnresolvedReference`. **Pin the registry rows that
legitimately hold a non-canonical join to a LIST; refuse every other flip. Before 8d-iii.**
⚠ Same function consults `other_join_spelling` **twice** (`normalize.rs:708`/`:719`) under two
different authorities — registry-arbitrated, but two consults of one question.

⚠ **THE DELTA SAMPLE IS NOT A COMMITTED LIST.** Reconstructed as *every 12th non-stdlib tracked
`.wat`* → the right **179** files but **164** originals clean vs the SCORE's **161**; no offset
reproduces it. The conclusion survives (my own run: **16** new against 49), but **eight stones have
compared slightly different populations. Commit the list.**

**Residue, disclosed:** `wat/holon/Ngram.wat` converts to `DuplicateMacro`. ⭐ The stdlib-copy
artifact hypothesis was **refuted by control** — an UNCONVERTED copy at the same out-of-tree path is
clean. 1 opened against 31 closed.

⚠ `cargo fmt` ran over the touched files, so the 2,889-line diff is **~2,251 semantic**
(`collection/eval.rs` is **58**, not 958). **Reformat in its own commit, or not at all.**

**Next: 8d-ii re-runs** — its conversion was always proven; only the load failed, and it now works.

### ⛔ 2026-09-21 — 251.8d-ii STOPPED (the stop that BOUGHT 255.8)

**The stdlib CONVERTS and does not LOAD.** 8d-ii's redraw converted all 64 `wat/*.wat`
(`64/64 changed`, dry-run byte-identical to the applied run, `cargo build --release` exit 0) — and
the new binary **dies in startup, before it reads any file**:

```
#wat.type/UnknownNamedType {:message "annotation names unknown type
  :wat::telemetry::Journal::QueryMetricsRequest — not a declared type,
  not a type variable, and not a builtin"}
```

⭐ **Two witnesses, different first names** (pass 2 died on `:wat::query::Store::EnsureSchemaRequest`)
— `validate_named_type_annotations` walks `env.iter()` and returns the FIRST miss, and `HashMap`
order changes per process. **Named, not dismissed as a flake.** `wat/` was restored; recovery proven.

⛔ **CAUSE: 255.3, not the codemod.** A nested type name is **registered** with `::` by
`canonical_identity`/`ns_to_wat_path` and **resolved** with `/` by `reconstruct_call_path`, because
the namespace's last segment (`Journal`) is itself a known type — so the join treats the tail as a
**member**. `canonical_identity` leaves a string already containing `::` alone, so the `/` never
folds back. **Reproduced minimally by the orchestrator in 3 lines:**
`(wat.core/defrecord my.Journal/Req …)` + `[r :- my.Journal/Req]` → `:path ":my::Journal/Req"`;
the `:my::Journal::Req` spelling is CLEAN.

⛔⛔ **`src/types.rs:172-174` — the function's own doc states the rule its body breaks:**
> *"a type name `my.Counter/Req` is not a method. Identity reconstruction stays `::` always."*

⭐ **`Type/member` and `Namespace.Type/NestedType` are the SAME SHAPE.** 255.3 already measured that
last-segment-is-a-type is **necessary and not sufficient**; 255.4 answered one half by unifying the
member join on `/`. **255.8 is the other half.** ⚠ If a type annotation is reaching a function named
for CALL paths, the bug may be the **routing**, not the join. ⛔ A second predicate inside
`reconstruct_call_path` is the shape this arc has rejected four times (255.4 cost 32 reds).

**Stone: `255-builtin-registry/BRIEF-STONE-255.8-a-namespace-that-is-also-a-type.md`** (`6b10437c5`).
Gate: the minimal repro clean **and** `Option/expect` still a member (4,501 sites) in ONE test; a
converted stdlib binary that STARTS, built from copies; **0 live `.wat` converted**. Delta baseline
**49**. **8d-ii re-runs after it** — its conversion is already proven; only the load fails.

### ✅ 255.7 LANDED 2026-09-21 — the validator adopts the door. ⭐ **ReteCheckErrors 16 → 0**

Floor 5957/5957, clippy 0, census clean, 0 corpus `.wat` converted. **Delta 61 → 49.**
⭐⭐ **THE RETE CLASS IS CLOSED.** Arc: **104 → 97 → 80 → 77 → 66 → 61 → 49.**

⛔ **THE CRITICAL CONTROL HELD — "diagnosed, NOT skipped":** an unknown `:then` field is still
`UnknownField`, `:then [42]` is still `ReteCheckErrors`, `(?k <- :k)` is still refused. 255.6 had
just found `check_fence_interior` **fail-opening**, so a widened validator that stopped checking
would have passed a naive control silently. It does not.

⭐ **THIRTEENTH CORRECTION — the brief's 14-line grep was wrong three ways.** It missed `typing.rs`'s
rete-op head (outside the grepped file), misclassified `:135` (a literal-name via `quote_boundary`,
not a Keyword match), and ⭐⭐ **could not see `type_env_name` at all — a SECOND slash-to-`::`
parser**, now replaced with `canonical_identity`. **A grep for one node variant cannot find a helper
that re-implements the door.**

⭐ **Class C was left UNCHANGED on purpose** — kwargs field keys, field refs, the caret node. Asked
to classify before changing, it found a class that must NOT change and pinned it with a fixture.

**Residue (49):** `UnresolvedReference` **23** · ⭐ `TypeMismatch` **11 (new — 4 arrived from the
rete wall; the checker, named not forced)** · `defsurface` 8 · other 7.

### ✅ 255.6 LANDED 2026-09-21 — rete's `:when` adopts the door. **Delta 66 → 61**

Floor 5953/5953, clippy 0, census clean, 0 corpus `.wat` converted.
⭐ **Two independent deltas agreed EXACTLY** — 66 → 61, and the classification matched row for row
(UR 23 · rete 16 · defsurface 8 · programbody 3 · unknown 1). A first for this arc.
**`canonical_identity` is now rete's clause-head / fact-bind-type / constraint-head key** — one
dispatch, Keyword or Symbol, ⛔ **not a second Symbol arm**.
⭐ **Non-vacuity held:** `(?k <- :k)` is **still `MalformedClause`**; the symbol head is legal, the
retired arrow is not.

⛔ **TWELFTH CORRECTION — the brief was wrong.** It said *"ask the registry, `is_known_type`"*, but
`classify_rete_clause` is documented *"Independent of TypeEnv, by SHAPE alone"*. The executor
canonicalized **first** and kept the `::` test, because a namespaced type has `::` after canonical
identity and a field does not. **Same discriminator, spelling-independent, module independence
preserved.**

⭐ **"Measure before touching" found a FAIL-OPEN nobody asked about:** `check_fence_interior` was
Keyword-only and **silently skipped** let/match shadow checks and constraint typing on converted
interiors. **Skipped, not failing.** Wired through the same door.

⛔ **NEXT — the remaining 16 are `:then`, not `:when`:** `validate_then_form` requires
`fact_items[0]` to be a `WatAST::Keyword`. A **fourth** keyword-only slot, sibling of the three this
stone routed. Confirmed **not** synthesised — source literals only.

### ✅ 251.8d-i-b LANDED 2026-09-21 — **the arrow is GONE for rete binds.** Delta baseline now **66**

Floor 5946/5946, clippy 0, census clean.
⭐ **BUILDER'S RULING:** *"the arrow syntax is gone — `<-` and `->` both become `:-`. this is what
becoming a clojure means."* and *"rete moves to `:-` — it is not exception."*
⛔ **The orchestrator had drawn the stone to PRESERVE `<-` at 3,853 rete sites — a design decision it
had no standing to make, inside a codemod-bug brief. The builder caught it. ELEVENTH correction.**

**Surgical dance, measured both ways:** rete-var binds still `<-` = **0**; `fn` param annotations
still `<-` = **7,028, untouched** (those are 8d-iii's). 327 corpus files, by a recorded,
replay-fixtured, idempotent codemod (`rete-bind-arrow-to-binder.wat`). Rete now **refuses** `(?k <- :k)`.

⭐ **The stone's real product is a wall that reaches NODE CONSTRUCTORS.** It shipped a red because
`wat/query.wat` **synthesised** `<-` in a quasiquote where no text rewrite could see it — the same
shape as 255.4's `method_wat_path`. **A codemod rewrites text; a generator builds the node.**
`tests/lint/rete_bind_generators.rs` walks the constructor, with two non-vacuity floors and three
detector self-tests (one positive, two negatives).

⚠ **Delta baseline is now 66, not 64** — this stone moved the live corpus AND the codemod, so the
prior number was measured against a tree that no longer exists. Not a regression.

⛔ **NEXT CLASS — `ReteCheckErrors` 21, declined THREE times and correctly each time:** they fail on
**correctly converted** clauses because **rete's `:when` parser wants a KEYWORD head and a
`::`-keyword fact type**. Independent of the arrow. A **checker** question.

### ✅ 255.5 LANDED 2026-09-21 — the position flag. ⭐ **Delta 77 → 64.** Arc: **104→97→80→77→64**

Floor 5941/5941, clippy 0, census clean, no corpus `.wat` converted.
⭐ **`:wat::WatAST` — 65 occurrences, the arc's single largest path — is GONE**; `unresolved
reference` **44 → 21**.
**The cure was not a new mechanism:** `normalize.rs` already had `also_accept_type` and a general
`is_known_type` acceptance behind it — **set for exactly ONE slot** (`normalize_type_binder_head`).
Annotation slots travelled the general walk with `false`, so a known type was asked the *reference*
question. The stone wires the existing flag to the type slots.
⭐ **Non-vacuity control HELD:** `(wat.time/Instant)`, `(wat/WatAST)`, `(wat.core/i64)` are **still
refused in CALL position**. The widening did not leak.

⛔ **THE ARC'S METHOD, worth carrying forward:** 255.2 moved the count by **zero** and is the stone
that made 255.5 possible — its *negative* result ("a predicate over leaves cannot do this", three
attempts at 116/108/114) stopped a fourth predicate and named the layer. And 255.3 needed 255.1's
`wat.type` members to ask "is the last segment a type?" at all.

**Residue (64):** ⚠ `ReteCheckErrors` **21 — now the largest class, never classified** ·
`unresolved reference` 21 (declaration names, functions, and `not-a-special-form` ×3 which is a
deliberate negative-test name) · `defsurface` 8 (`:messages` slot) · gap-2 tail 4.

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
