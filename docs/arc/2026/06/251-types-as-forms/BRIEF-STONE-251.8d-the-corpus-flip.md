# BRIEF — STONE 251.8d: the reader/printer flip (THE CORPUS FLIP)

**Drawn 2026-09-19 against `main` @ `00f841ca3`** (floor 5920/5920, clippy 0, census `no STOP-8`).
Design: `DESIGN-STONE-251.8-symbol-proper.md` §251.8d. Live breadcrumb: `docs/SEAM.md`.

**Why now:** 8c closed the check hole and the first-slash ruling landed. The builder's standing
direction — *"the last outstanding item for non-edn compliant is our keywords having `::` in them"* —
makes this the last stone in the clojure/EDN compliance line.

---

## ⛔⛔ READ THIS FIRST — THE TOOL ALREADY EXISTS, AND THE STONE IS NOT WHAT THE DESIGN SAYS

The design calls 8d *"a spelling change over a substrate that already type-checks both"* and sizes it
as *"the 1392-file corpus flip."* **Both halves are wrong today, and the work is somewhere else.**

### ⭐ The codemod is WRITTEN, RECORDED, AND WORKS

`wat-scripts/fixes/to-faithful-clojure.wat` (plus `-net`, `-rete` siblings) drives
`:wat::fix::fix-text` (`wat/fix.wat:339`). **Measured on a copy, not read:**

```
(:wat::core::defn :u::f [xs <- (:wat::core::Vector :- [:t::New])] -> :t::New (:t::New/make xs))
→ (wat.core/defn u/f    [xs :- (wat.core/Vector :- [t/New])]    :- t/New   (t.New/make xs))
```

Comment-faithful, span-edit based, and **idempotent** (second run: 0 changes, verified). So **8d is
not "write the migration." It is "make the migration survive the corpus."**

### The real corpus numbers (measured 2026-09-19 — the SEAM's were WRONG)

| | count |
|---|---|
| tracked `.wat` files | **2,190** |
| with `::` call heads (`(:wat::`) | **2,140** |
| with REAL faithful-clojure call heads (`(wat.`) | **55** (~2.5% done) |
| raw `::` occurrences | **304,196** |

⛔ **The SEAM said "169,845 `::` across 2,440 of 2,483 files." Every one of those numbers is wrong.**
Do not carry them forward. ⚠ And a first pass here said *"80 files already converted"* — **48 of
those 80 were `wat.core/` inside COMMENTS and `#wat.core/Result.Ok` EDN TAGGED LITERALS**, not code.
That is the campaign's dominant defect recurring inside this very brief's drafting
(`[[feedback_ask_the_tool_that_owns_the_fact]]`). **The 55 figure counts `(wat.` call heads only.**

---

## ⛔ THE MEASURED BLOCKERS — 7.5% of files kill the run, in THREE distinct classes

Run on an 80-file spread of **copies** (never the tree), one file per process so a death does not
mask the rest: **6 of 80 failed = 7.5%.**

⚠ **THE 7.5% IS A SKEWED SAMPLE — DO NOT PLAN AGAINST IT.** A corpus-wide census (all 2,140 files,
class-A guard live) is running; at **272 files** it reads **268 OK / 4 class-B / 0 class-A / 0
class-C ⇒ ~1.5%**, projecting **~32 files**, not ~160. The 80-file sample was drawn with
`awk 'NR%27==1'`, which over-selected `probe_*` and `scratch-pad/` files — the densest users of macro
and reader-macro syntax. **A stride sample over a sorted path list is not a random sample**; the
sort correlates with directory, and directory correlates with the defect. Take the final numbers
from the census table at the foot of this brief, not from here.

⭐ **They are not one bug.** The design named only the first and called it *"tracked separately."*

| class | what fires | count | status |
|---|---|---|---|
| **A — `ast-name` partiality** | `ast-name requires a Symbol, Keyword, or StringLit node`, `wat/fix.wat:74` | 2/6 | ✅ **CURED by a one-line guard — confirmed** |
| **B — reader-synthesized span** | `fix-text-apply: … claims old-text ":wat::core::unquote" (19 char(s)) but the source there is "~(wat.keyword/from-" — refusing to splice` | 3/6 | ⛔ **OPEN — needs design** |
| **C — `keyword::to-symbol` refusal** | `not a convertible call-head/reference keyword (bare data keyword or namespace-prefix marker)`, `wat/fix.wat:281` | 1/6 | ⛔ **OPEN — 1 sample, class unsized** |

### Class A — cured, and the cure is confirmed, not proposed

`annotated-if?` (`fix.wat:63-80`) calls `(ast-name head)` with **no kind guard**, two lines above
where it correctly guards `c2` with `(= (ast-kind c2) "symbol")`. A one-line asymmetry.

**Mechanism confirmed with a negative control:**
- `((wat.core/fn [] 1) 2 3)` — list in head position, ≥3 children → **raises**
- `((wat.core/fn [] 1) 2)` — same shape, 2 children → **passes**, because the `(< (length ch) 3)`
  guard short-circuits before `ast-name` is reached

⭐ That short-circuit is why the class hits only ~7.5% and looked mysterious for a whole arc.
`wat.core/if` **is lazy** (verified: an untaken `(:wat::i64::/ 1 0)` branch never ran), so a kind
guard genuinely prevents the call.

**The cure, applied and measured:** guard `head`'s kind exactly as `c2`'s is guarded. After a
**rebuild** (see the bootstrap trap below), both class-A files pass — including **`wat/sqlite.wat`,
a stdlib file.** ⚠ The executor should re-derive this rather than paste it: the guard was written to
test the class, not to be the final shape.

### Class B — ⛔ the guard is CORRECT and the RULE is wrong. Do not weaken the guard.

`wat/fix.wat:225-227` documents this hazard in its own words, before it ever fired:

> *"anywhere a rule's belief comes from a NAME (a keyword's ast-name, a captured 'old' value) that
> could, for a **reader-synthesized node**, sit at a DIFFERENT span than the name implies (the
> char/of corruption this stone exists to stop: a synthesized keyword's Span covers 2 raw source
> chars while its ast-name claims an 18-char canonical token)."*

`~` is a **reader macro**: **1 source char**, but the reader synthesizes a node whose `ast-name` is
the **19-char** `:wat::core::unquote`. The rule computes `old-text` from the name, so the edit claims
19 chars where 1 exists, and `fix-text-apply` refuses to splice.

⛔ **"refusing to splice" is the stone that exists to stop exactly this corruption. A cure that
silences it, widens its tolerance, or fills `old-text` from the span is the defect the guard was
built to catch** — and `fix-text-span-text`'s own doc comment says filling `old-text` that way
"makes fix-text-apply's check compare a slice against itself — vacuous, catching nothing."

**The cure must teach the RULE about reader macros**, so the edit's extent is the SOURCE token
(`~`, `~@`) rather than the canonical name. Candidate sizing: **162 files contain `~`, 35 contain
`~@`.** ⚠ Both are raw `grep -lF` counts over whole files — **they include comments and strings and
are an UPPER BOUND, not the class.** Size the class from the census, not from these.
⛔ Do **not** count `'` this way: 1,095 files match, and they are overwhelmingly apostrophes in prose
and primed names (`:sort'`), not quote reader macros.

### Class C — unsized, 1 sample. **Measure before curing.**

`fix.wat:281` routes a head keyword through `keyword::to-symbol`, which refuses with *"bare data
keyword or namespace-prefix marker."* One sample (`probe-198-ctor-macro-span.wat`). **Do not design
a cure from n=1.** The census (below) gives the real count and the distinct spellings; if it turns
out to be one or two files with a genuinely unconvertible head, **relocating or runing them is a
legitimate answer** — but say which, and why, per file.

### ✅ Failure is FAIL-SAFE — the one piece of good news

On every failing class the file is left **byte-unchanged** (verified by `diff` on a copy). The
codemod refuses rather than half-writes. **No corruption risk; the risk is a stalled run.**

---

## ⛔ THE BOOTSTRAP TRAP — the codemod is its own input, and it is COMPILED IN

Two facts that together decide the stone's shape:

1. **`wat/fix.wat` is `include_str!`'d into the binary** (`src/load/stdlib.rs:353`), as are
   `wat/core.wat`, `wat/sqlite.wat`, `wat/seq.wat` and the rest of `wat/`. **Editing `wat/fix.wat`
   changes nothing until `cargo build --release`.**
   ⚠ This brief's own first attempt at the class-A cure "failed" for exactly this reason — the patch
   was live on disk and absent from the running binary. **If a cure appears not to work, rebuild
   before theorising.**
2. **The corpus flip rewrites `wat/fix.wat` itself.** It is `.wat`, it is in the corpus, and it is
   the tool. `[[feedback_a_tool_is_never_its_own_input]]` is the recorded precedent — *"2026-09-11,
   the chain migrated its own codemods."*

⭐ **Therefore the `wat/` stdlib cannot be flipped by the same pass as the rest of the corpus.** Read
`wat/fix.wat`'s header STASH-DANCE note (lines 22–53) before touching it. The converted stdlib must
still compile under the checker that exists at that moment.

### Operational: batch is fast and fragile; per-file is slow and safe

- **Batch** (one process, N paths — the documented usage): fast, but **exits on the first failure and
  the remaining paths are never attempted.** With ~160 failures spread through 2,140 files, a naive
  batch run stops almost immediately.
- **Per-file** (one process per path): fault-isolating, but **~2.7 s/file ⇒ ~96 min for 2,140.**
  ⚠ Measured only after launching it — `[[feedback_time_a_tool_on_one_item_first]]`, again.

**Neither is the answer as-is.** A resumable driver (skip-on-failure, record the failures, exit 0)
is probably wanted — but that is a decision for the executor to make and disclose, not a mandate.

---

## THE WORK — proposed as THREE stones, and the split is a QUESTION for the builder

⛔ **This brief does not assume 8d is one stone.** One pass that fixes three defect classes,
bootstraps the stdlib, and rewrites 2,140 files cannot be reviewed, and a red anywhere in it loses
everything before it.

### 8d-i — MAKE THE TOOL TOTAL (no corpus writes at all)

Cure A, B and C. **Acceptance: a dry run over all 2,140 files, on COPIES, with ZERO failures** — and
the count of files the tool would change, which is the flip's real size.
⛔ **Not one tracked file is written in this stone.** The gate is the census going to 0.

### 8d-ii — THE BOOTSTRAP: flip `wat/` (the `include_str!`'d stdlib), stash-dance, rebuild

Small file count, all the hazard. Floor green at the end, which proves the substrate runs on a
converted stdlib. ⛔ If this cannot be done without a checker change, **STOP AND REPORT** — that is a
finding, not a thing to smuggle in.

### 8d-iii — THE CORPUS FLIP

The remaining ~2,085 files, through the tool, recorded. Idempotence re-proven at corpus scale
(second pass changes 0 files).

⚠ **The ordering is the orchestrator's proposal. The builder rules it.** A single-stone strike is
defensible if the builder wants it; this brief's claim is only that the three phases have different
failure modes and different review shapes.

---

## ⭐ THE ACCEPTANCE ROWS THE FLIP ALREADY OWES

`NOTE-a-keyword-literal-means-two-things.md` (measured 2026-09-11) is **not optional reading** — it
records four rows the flip inherited, with row 1 load-bearing:

```
1. a KEYWORD literal types as :wat::core::keyword ALWAYS — registry membership must not change it
2. a SYMBOL naming a unit variant resolves to that variant's constructor
3. defservice's :arms and user code read a bare keyword IDENTICALLY
4. whatever (:u::Op.Mark {}) means today has ONE post-flip spelling; the retired ones are REFUSED
```

> *"Row 1 is the load-bearing one. It is also the row that makes this finding falsifiable: if, after
> the flip, a keyword's type still depends on a lookup, the flip did not separate the two meanings —
> it only moved them."*

**Each must be PROVEN by a test, not asserted.**

## One surface fact worth knowing before you start

Converted files spell types in a **`wat.type/`** namespace — `wat.type/nil`, `wat.type/i64`,
`wat.type/bool` — while parametrics keep `wat.core/Vector`. `wat.core/String` does **not** resolve.
⚠ Read off 55 converted files and a pilot conversion; **not a spec.** Confirm against the printer
before relying on it.

---

## The gate

- `scripts/floor.sh` green, clippy `-D warnings --all-targets --workspace` 0 — **the orchestrator's
  row**, run uncontended. Predict the test-count delta from the diff first, then confirm with
  **`cargo nextest list`**.
- `scripts/replay/census.sh --diff` → `no STOP-8`.
- Idempotence: a second full pass changes **0 files**.
- ⛔ Every converted file still `--check`s **no worse than before** — captured as a **delta**, per
  file, never an absolute count (`[[feedback_a_number_assembled_from_two_measurements]]`).

## Doctrine — `wat-rs/CLAUDE.md` does not reach a subagent

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture it whole the **first** time; never re-run.
- ⛔ **R21: the corpus moves BY THE TOOL.** No hand-edited `.wat`, no python, no sed. Any site the
  tool cannot reach is a **STOP and a report**, named with its reason — never a quiet hand-fix.
- ⛔ **DRY-RUN ON COPIES AND `diff`** before any tracked file is written. Every measurement in this
  brief was taken on copies under `$CLAUDE_JOB_DIR/tmp`; the tree was never the experiment.
- ⛔ **ASK THE TOOL THAT OWNS THE FACT.** Test counts → `cargo nextest list`. This brief's drafting
  produced **four** instrument errors, all caught, all of the same family — disclosed below so you
  distrust the right things.
- ⛔ **If this brief contradicts the code or the runner, they win — land what is true and report the
  brief's error.** Two consecutive briefs in this arc led with a claim the executor had to kill.
- Work only in `/home/john/work/holon/wat-rs`. No `git filter-branch`. Do not push.

### ⚠ THE FOUR INSTRUMENT ERRORS MADE WHILE DRAWING THIS BRIEF

Disclosed because they say exactly which of this brief's numbers to re-derive:

1. **Counted comments and EDN tags as code** — "80 files converted"; 48 were `wat.core/` in prose or
   `#wat.core/Result.Ok` tagged literals. True figure **55**.
2. **`$?` read the wrong command's exit** — `printf '%s' "$(basename $p)" "$?"` expands the
   substitution first, so `$?` was `basename`'s. Six failing runs were reported **rc=0**. Real exits
   are 1 and 2. **Capture `rc=$?` on its own line, immediately.**
3. **Edited `wat/fix.wat` and tested without rebuilding** — it is `include_str!`'d; the test measured
   the old binary and I nearly recorded "the guard does not work."
4. **Launched a 96-minute census without timing one item first** — the recorded lesson, repeated.

## Out of scope — affirmatively cut

- **The `.rs` side.** 826 Rust files mention `::` wat spellings; whether any must change is a
  separate measurement, and nothing here depends on it.
- **`ast-name`'s partiality as a CONTRACT question.** The cure here is a caller-side kind guard.
  Making `ast-name` total would weaken a documented round-trip contract
  (`(symbol-node (ast-name sym)) = sym`) and hide caller bugs like class A. **If you believe the
  contract itself is wrong, that is a finding to report, not this stone.**
- **The opaque-clause-table leak** (`(println :wat::core::+)` → `#wat-edn.opaque/clauses nil`) —
  filed in the design, still its own defect.
- **Arc 296's tail and the registry (255).** Neither gates this.


---

# CENSUS — the corpus-wide class distribution ✅ COMPLETE

Run 2026-09-19 over **all 2,140 files with `::` call heads**, on copies, one process per file, with
the class-A guard live. ⛔ **The tree was never written.**

| class | files | share | status |
|---|---|---|---|
| **OK** | **2,021** | 94.4% | converts cleanly |
| **B — refusing to splice** (reader-synthesized span) | **99** | 4.6% | ⛔ open |
| **C — namespace-prefix marker** | **20** | 0.9% | ⛔ open, **one shape** |
| **A — `ast-name` partiality** | **0** | 0% | ✅ **the one-line guard cured the WHOLE class** |
| other | 0 | — | — |

**119 files block the flip; 2,021 are ready.**

## ⭐ THE FLIP'S REAL SIZE — measured, not projected

Every one of the 2,021 clean files **is changed** by the tool (0 unchanged), rewriting
**67,065 source lines**. ⛔ *That* is 8d's size — not the SEAM's "169,845 occurrences", not the raw
304,196 `::` count, both of which count characters in comments and strings.

## ⭐ ALL THREE CLASSES ARE THE SAME DEFECT WEARING THREE COATS

This is the brief's most useful finding and it was not visible until all three were characterised:

> **In every class, a RULE misclassifies a node, and the downstream primitive CORRECTLY REFUSES.**

| class | the rule's misclassification | who refuses, correctly |
|---|---|---|
| A | `annotated-if?` assumes a list's head is named | `ast-name` — its contract says named nodes only |
| B | the head rule takes `old-text` from the canonical NAME of a reader-synthesized node | `fix-text-apply` — "the rule's belief and the source disagree" |
| C | `head-keyword?` treats *any* `::`-bearing keyword as a call head | `keyword::to-symbol` — "not a convertible call-head/reference keyword" |

⛔ **So the cure is NEVER in the primitive.** Every one of the three is fixed by narrowing a
predicate in `wat/fix.wat` so it stops making a claim it cannot support. **A cure that relaxes
`ast-name`, widens `fix-text-apply`'s tolerance, or makes `to-symbol` accept markers is the defect,
not the fix** — those three refusals are the only reason this corpus was not silently corrupted.

## Class B — 99 files, concentrated where the syntax lives

| directory | files |
|---|---|
| `tests/macros` | 38 |
| `wat-scripts/scratch-pad` | 20 |
| `wat/holon` | 11 |
| `wat` | 8 |
| `tests/wat_lang`, `tests/types` | 3 each |
| `wat/kernel`, `wat-tests/core` | 2 each |
| (remainder) | 12 |

Macro-dense directories, exactly as the `~`-unquote mechanism predicts.
⛔ **8 are in `wat/` itself** and so are `include_str!`'d — they belong to the 8d-ii bootstrap.

## Class C — 20 files, ONE shape, and the cure is a predicate

The refused keywords are **`:my::issuer::` and `:wat::spawn::`** — keywords **ending in `::`**. They
are **namespace-prefix MARKERS used as DATA**:

```wat
{:restricted-to  [:my::issuer::]
 :field-metadata {:secret {:restricted-to [:my::issuer::]}}}
```

`head-keyword?` (`fix.wat:94`) tests only `(contains? name "::")`, so a marker is handed to
`to-symbol` as though it were a call head. **The cure is to exclude trailing-`::` markers from
`head-keyword?`** — they are data, and 16 of the 20 files are `*restricted*` whitelists.

⛔ **But answer this first, because it is a DESIGN question, not a predicate question:** *what does a
namespace-prefix marker become after the flip?* `:my::issuer::` names a namespace, not a value.
Leaving it as a `::` keyword means `::` does not fully retire — which is 8d's whole point. Under the
builder's first-slash ruling the natural spelling is `my.issuer/`, but **that is unmeasured and it is
the builder's call.** ⛔ **Do not pick one silently.** 2 of the 20 are in `wat/` (`spawn.wat`,
`kernel/services/stdio.wat`), so this shape reaches the stdlib.

## What this census changes about the plan

- **8d-i is real and it is small**: three predicate narrowings in `wat/fix.wat`, one already proven.
  Its gate is this census re-running to **0 failures**.
- **The 7.5% sample figure is dead.** True rate **5.6%** (119/2,140). ⚠ And the interim reading at
  272 files said **1.5%** — also wrong, because failures cluster in `tests/`, `wat-scripts/` and
  `wat/`, which sort LAST. **A partial run of a path-sorted census is not an estimate of the whole**
  (`[[feedback_a_totality_claim_is_only_as_good_as_its_sampling]]`). Three different numbers came out
  of three different slices of the same instrument; only the complete run is quotable.
