# SCORE — STONE 255.10: the keyword requirement IS the defect

**Executed against `f1bef5944`** (brief drawn at `fe4c0923d`; `a51ba7dea` drew the stone, `a75a1e788`
recorded the SEAM, `f1bef5944` corrected the discriminator mid-stone). ⚠ **HEAD advanced under me
during the session** — `a51ba7dea` → `f1bef5944`. Measured: `git diff --stat fe4c0923d..f1bef5944 --
src/` is **EMPTY**, so every `src/` number below is unaffected and the baseline binary built at the
start of the session stays valid. **Every number in this document was produced this session.**

## VERDICT — **LANDED**

Six cures across five `src/` files, each with a file or test that changes state. **All 8 `defsurface`
residue files close.** Delta **18 → 4** on the committed list, **RECOVERY 0**. Floor
**5963/5963**, clippy 0, census `no STOP-8`. **No `.wat` in the live tree changed.**

⭐ **The two biggest findings are NOT in the brief, and neither was visible to 255.9's census frame:**

1. ⛔⛔ **`macros/eval.rs::validate_pure_total` — the DEFAULT-DENY F5 expand-time purity wall was
   BYPASSED ENTIRELY by the symbol spelling.** A symbol-headed form fell straight through to the
   `_ =>` "recurse into children" arm; the allow-list never ran. Measured on two binaries built this
   session. 255.9's census classified this site **LOUD**; it is **SILENT**. §1.5 (a).
2. ⛔ **`macros/expand.rs::is_callable_form` — the arc-143 CODE-vs-DATA discriminant.** A computed
   unquote with a symbol head silently became literal data: spliced un-evaluated, so an
   expand-time-only verb surfaced as a runtime error. ⚠ **This site is invisible to
   `grep "Some(WatAST::Keyword"`** — it is a `matches!` inside a closure. §1.3.

⚠ **Three corrections to the brief, all measured** — see §6. ⛔ **One self-inflicted regression,
caught by the delta and cured** (`vector_splice_symmetry.wat`, §2 (e)). ⛔ **The first floor ran on a
tree I rebuilt underneath it; disclosed and re-run clean** — §3.6.

---

## 0. THE BUILDER'S MID-STONE CORRECTION

Row 3 of the non-vacuity gate (*"a variant tag `:Ok` is still data"*) was **STRUCK** by the builder
while this stone was in flight: a variant tag in a `defenum` **declaration** is a NAME being minted,
not data. **The correction arrived before I had written any test**, so nothing was reworked and
nothing was removed. **This SCORE asserts NOTHING about variant tags, in either direction**, and the
non-vacuity test carries a comment saying so. The gate below therefore has **three** rows, not four.

---

## 1. THE REACHABILITY CENSUS — whole

### 1.1 The frame, and why the brief's frame is too narrow

The brief's probe is `grep -rn "WatAST::Keyword(" src/` → **634**. **Measured at `f1bef5944`: 665**,
in 65 files. (`src/` is byte-identical to the brief's draw commit, so this is a plain miscount, not
drift.) But the raw count is not the census frame either — most of those are *constructions*, and,
more importantly, **a keyword REQUIREMENT does not have to be spelled `WatAST::Keyword` at all.**

Measured at `f1bef5944`, by the **shape** the requirement wears:

| shape | probe | count | example |
|---|---|---|---|
| **A** — AST slot read | `Some(WatAST::Keyword` | **83** | `surface.rs:729` — 255.9's whole frame |
| **B** — predicate inside `matches!` | `matches!(… WatAST::Keyword` minus A | **32** | ⛔ `expand.rs::is_callable_form` — **CURED HERE; INVISIBLE TO A** |
| **C** — bare dispatch arm | `^\s*WatAST::Keyword(x, _) =>` | **122** | `runtime.rs::eval_list` (already reads both) |
| **D** — a STRING test demanding `:` | `starts_with(':')` | **24** | ⛔ `render.rs::eval_keyword_node` — **CURED HERE; NO AST MATCH AT ALL** |

⭐ **Two of this stone's six cures are outside shape A.** 255.9's census read all 82 of shape A and
was right to; it simply could not see B or D. `[[A PATTERN THAT MATCHES A SUBSET IS NOT A CENSUS]]`,
demonstrated on the immediately preceding stone's frame — and one of the two invisibles is a
**security wall**.

### 1.2 The classifier: pipeline order, reused from 255.9

255.9's rule is the method, re-verified and used unchanged
(`freeze.rs::startup_from_source` steps 1–9 / `freeze/env.rs::build_env`):

```
2. collect_entry_file (config)   ← raw symbols
3. resolve_loads                 ← raw symbols        (255.9's two cures)
3b. extract_rete_defn_names      ← raw symbols        ⛔ THIS STONE
4. register_defmacros → expand_all ← raw symbols      ⛔ THIS STONE (×3)
5. register_types                ← raw symbols        ⛔ THIS STONE (×2)
6. register_defines (+ methods)
7. normalize_symbol_refs   ←←← SYMBOL → KEYWORD, HERE
8. check_program
9. freeze → run
```

> Anything at or after step 7 **in a CODE position** cannot see a symbol head. Data positions are
> never normalized (`resolve/boundary.rs`), so a symbol inside quoted data reaches runtime raw.

⭐ **Every one of this stone's six cures is at step 3b, 4 or 5 — strictly before normalize.** That is
the census doing its job: the reachable window is steps 2–6, and all six defects are inside it.

### 1.3 The census table — the sites a converted form can reach

Disposition key: **SILENT** = declines with no diagnostic · **LOUD** = errors · **BACKSTOPPED** =
declines silently but a second wall refuses anyway · **UNREACHABLE** = a symbol cannot arrive.

| site | shape | stage | symbol arrives? | disposition | evidence |
|---|---|---|---|---|---|
| **`types/surface.rs:729`** — `:messages` slot-1 NAME | A | 5 | **YES** | ⛔ **SILENT→FALSE RED** → **CURED** | §2 (a), §3.1, §3.2 |
| **`types/surface.rs:1003`** — post-arrow TYPE ref | A | 5 | **YES** | ⛔ **SILENT (wall stops collecting)** → **CURED** | §2 (b), §3.1 |
| **`freeze/env.rs:742/748`** — rete `defn` head + name | A | 3b | **YES** | **LOUD** (`UnresolvedReference`) → **CURED** | §2 (c), §3.2 |
| **`freeze/env.rs:769`** — rete `defn` head REWRITE | A | 3b | **YES** | same wall, other half → **CURED** | §2 (c) |
| **`edn/render.rs::eval_keyword_node`** — NAME input | **D** | 4 (macro eval) | **YES** | **LOUD** (`MalformedForm`) → **CURED** | §2 (d), §3.2 |
| ⛔⛔ **`macros/eval.rs::validate_pure_total`** — F5 allow-list head | A | 4 | **YES** | ⛔⛔ **SILENT — THE GATE NEVER RAN** → **CURED** | §1.5 (a), §2 (e) |
| ⛔ **`macros/expand.rs::is_callable_form`** — code-vs-data | **B** | 4 | **YES** | ⛔ **SILENT — code became data** → **CURED** | §1.5 (b), §2 (f) |
| `types/surface.rs:617/618/808` — `:nature` / `:features` MARKERS | A | 5 | YES | ⭐ **CORRECT AS IS** — a marker slot must reject a symbol | §3.1 row 2 |
| `types/surface.rs:680` — `:messages` MARKER peek | A | 5 | bare kw, never converted | ⭐ **CORRECT AS IS** | §3.1 row 2 |
| `types/surface.rs:443` — option KEY (`:max-request-bytes`) | A | 5 | MARKER | ⭐ **CORRECT AS IS** | read |
| `types/surface.rs:650` — `:nature` VALUE (a NAME) | A | 5 | YES | **already reads both** (`WatAST::Symbol` arm present) | read |
| `freeze/env.rs:347` — decl-name of a type form | A | 5 | YES | **already reads both** | read |
| `runtime.rs::eval_list` head dispatch | C | 9 | YES | **already reads both** (`is_reference()` → `reconstruct_call_path`) | read — ⭐ **this is the executor the two cured gates now agree with** |
| `macros/expand.rs::is_quasiquote_form` / `quasiquote_inner` / `match_unquote` | B | 4 | **NO** — `` ` ``/`~` are reader-synthesized and the codemod SKIPS synthesized leaves | **UNREACHABLE** for the sugar; LOUD for the explicit spelling | 255.9 probe B, re-read |
| `macros/eval.rs::validate_quasiquote_template` | A | 4 | same | same | read |
| `load/loader.rs` ×2 (255.9's cures) | A | 3 | YES | **already cured** | 255.9 |
| `types.rs` ×5, `rete/purity.rs` ×2, `rete/kernel/{stratify,arm}.rs` ×4 | A | 5–9 | YES | **already read both** | read |
| `resolve/walk.rs:87`, `check.rs` ×8 (code position), `closure_extract.rs` ×3, `holon/ast.rs:1034`, `kernel/serve.rs` ×4 | A | 7–9 | ⛔ **NO — normalize ran first** | **UNREACHABLE** | §1.2 |
| `resolve/normalize.rs` ×3, `resolve/quote.rs`, `rete/purity.rs` ×9, `rete/kernel/stratify.rs` ×2, `rete/expr_ir/mod.rs` ×4, `runtime.rs` match-pattern sites | A | 7–9, DATA position | YES (data is never normalized) | **LOUD** | 255.9 probe C, unchanged |
| `freeze.rs::refuse_mutation_forms`, `runtime.rs::refuse_mutation_forms_in` | A | 9 | YES | ⚠ **BACKSTOPPED** — refusal survives, diagnostic does not | 255.9 §1.5 A — **still open, still recommended** |
| `load/loader.rs::parse_verify_algo` / `parse_payload_interface`, `runtime.rs:14272/14343` | A | 3/9 | YES | **LOUD** | 255.9 §4 (a) — **still open** |
| `runtime.rs:12674/14723/19970` (session / `run()` paths) | A/C | 9 | plausibly | ⚠ **UNMEASURED** | 255.9 §4 (c) — **still unmeasured; I did not probe them either** |
| `macros/tests.rs:146`, `runtime.rs:15850`, `freeze/env.rs:850/902`, `freeze.rs:1894` | A | — | test code | **UNREACHABLE** | read |

⚠ **The UNREACHABLE rows are a PIPELINE-ORDER ARGUMENT, NOT A PROBE — and 255.9 said so first. IT IS
STILL TRUE.** I read `freeze.rs`'s step list and `normalize.rs`'s contract and reasoned from them.
**Re-order any pass relative to `normalize_symbol_refs` (step 7) and every UNREACHABLE row above goes
live SILENTLY.** There is no gate on that ordering beyond
`expand_runs_before_register_defines_phase_order`. ⭐ **Recommended as a stone for the second time.**

### 1.4 How the 18 residue files classify (measured, not assumed)

Pre-cure delta on the committed list, each of the 18 read:

| class | files | seed |
|---|---|---|
| `defsurface :messages` false red | **8** | `surface.rs:729` (+ `:1003` behind it) |
| rete `defn` head | **4** | `freeze/env.rs:742` |
| `keyword-node`'s input | **3** | `render.rs::eval_keyword_node` |
| ⚠ **NOT this class — `wat.type/X` denotation** | **2** | `TypeEnv::register_validated`'s `==` |
| ⚠ **NOT this class — a `.wat` substring swap** | **1**† | `wat/core.wat`'s `kwargs-type-slot-swap-head` |

⭐ **The brief's 8/4/3 table is EXACTLY right.** † The macro-call-site file counted in the brief's
"other" bucket turned out to be `is_callable_form` and is cured; the two arc-170 probes surfaced only
after the `keyword-node` cure unblocked them (they were counted in the 3).

### 1.5 The two probes that found what the brief does not look at

**(a) ⛔⛔ THE F5 DEFAULT-DENY PURITY WALL WAS OPEN.** Two `.wat` files differing only in spelling,
run on the **pre-cure** and **cured** binaries, both built this session from the same tree:

```wat
;; o.wat (symbol)                         ;; p.wat (keyword)
(wat.core/defmacro probe/pb [] :- wat/WatAST     (:wat::core::defmacro :probe::pb [] -> :wat::WatAST
  (wat.kernel/println "EXPAND-TIME EFFECT"))       (:wat::kernel::println "EXPAND-TIME EFFECT"))
```

| binary | `p.wat` (keyword) | `o.wat` (symbol) |
|---|---|---|
| **pre-cure** | `MalformedDefmacro … RefusedInMacro` — **the gate fires at DEFINITION** | ⛔ **the gate NEVER FIRES.** The form reached `runtime::eval` at expand time and stopped only because *stdio services were not running*: `ProgramBodyEvalFailed → MacroEvalRuntimeFailed → ServiceNotRunning` |
| **cured** | identical | ✅ **`MalformedDefmacro … RefusedInMacro`, the same wall, the same stage** |

⛔ **The pre-cure stop is incidental.** `println` needed a service that was absent; an impure verb
that needs nothing would have **executed during expansion**. Arc 249 stone O's whole property —
*"fails at definition, not silently at first use"* — was spelling-dependent.
⭐ **A lying diagnostic became a true one, at a security wall.**

**(b) ⛔ A COMPUTED UNQUOTE SILENTLY BECAME DATA.** Bisected on the real corpus file
`tests/macros/probe_arc278_macro_call_site.wat`, converted: restoring the keyword spelling of
**only** `(wat.kernel/macro-call-site)` inside the `` `~ `` made the file clean, and restoring only
the macro CALL sites did not. Minimal 5-line repro:

```wat
(wat.core/defmacro probe/hf [] :- wat/WatAST  `~(wat.kernel/macro-call-site))
```

| | keyword inner | symbol inner (pre-cure) | symbol inner (cured) |
|---|---|---|---|
| `--check` | rc 0 | ⛔ rc 1 `Frame/line: parameter #1 expects :wat::core::Record; got :wat::WatAST` | ✅ rc 0 |
| what happened | evaluated at expand time → a `Frame` | ⛔ **spliced UN-EVALUATED**; shown directly: `(wat.core/write-forms ~(wat.kernel/macro-call-site))` raises at RUNTIME *"only valid inside a macro body / at expand time"* | evaluated at expand time |

---

## 2. THE SIX CURES — each through the identity door, each with its evidence

**Diff: 5 files, `+241 −14`** (bulk = doc comments and one new test). **ZERO `.wat`.**

| # | site | door | ⭐ THE EVIDENCE THAT CHANGED STATE |
|---|---|---|---|
| **(a)** | `types/surface.rs:729` — `:messages` slot-1 | `form_match::canonical_identity_of` | **8 corpus files broken→clean** (§3.2). Unit test **FAILS** without it (§3.1). |
| **(b)** | `types/surface.rs:1003` — post-arrow type | `types::parse_type_node` | ⛔ **With (a) alone the wall goes SILENT on a genuinely broken surface** — a targeted revert of (b) only made the non-vacuity test fail with *"a type reachable from `:messages` but absent from it is a located error in EITHER spelling"* and the surface parsing **clean**. (b) is what stops a false red becoming a FALSE GREEN. |
| **(c)** | `freeze/env.rs:742/748/769` — rete `defn` | `canonical_identity_of` | **4 corpus files broken→clean** (§3.2). |
| **(d)** | `render.rs::eval_keyword_node` — the NAME string | `edn::render::canonical_identity` | **`wat-tests/rete/differential-fuzz-rules.wat` broken→clean**; it also unblocked 2 arc-170 probes from a macro-internals error to their own (different, reported) defect. |
| **(e)** | `macros/eval.rs::validate_pure_total` — F5 head | identity + `types::other_join_spelling` | §1.5 (a): the wall went from **not firing** to firing, in both spellings, with the same located reason. |
| **(f)** | `macros/expand.rs::is_callable_form` — code-vs-data | `is_reference()` (matching `eval_list`) | **`tests/macros/probe_arc278_macro_call_site.wat` broken→clean**; and `` `~(wat.kernel/println …) `` went **rc 0 → rc 1 `RefusedInMacro`**, i.e. the F5 wall now also fires on the quasiquote path in both spellings. |

**Why these doors and no others.** `canonical_identity_of` is the door 255.1/255.2/255.6/255.7/255.9
already routed five subsystems through, and `parse_type_node` is the substrate's own *"keyword,
namespaced symbol, parametric form, `[arg… :-> ret]` bracket"* type reader — using it means a
spelling added there reaches `:messages` for free. **A per-site spelling test was rejected**: that is
the shape that cost 255.4 thirty-two reds, and the injected `CLAUDE.md` names it by name.

⛔⛔ **(e) TAUGHT ME THE RECURRING CLASS ON MYSELF, AND THE DELTA CAUGHT IT.** My first version of (e)
used `canonical_identity` alone. `wat.core.Option/expect` is `:wat::core::Option/expect` — a
`Type/method` surface op — but `canonical_identity` renders it `:wat::core::Option::expect`, because
which join is right depends on whether `Option` is a TYPE, and this gate has no `TypeEnv`. Result:
**`tests/macros/vector_splice_symmetry.wat` went clean → REFUSED**, a regression *I* introduced, on
the delta, in this session. The substrate already owns the answer — `types::other_join_spelling`,
whose doc says *"callers ask the registry which spelling it holds and use this only when the primary
misses"*. The gate now consults **both** renderings; default-deny survives because a head is legal
only if SOME rendering of it is on the list. ⭐ *"A string comparison with one side normalized and the
other not"* — the injected `CLAUDE.md`'s named class, fourth instance in arc 278/255, this one mine.

**Restraint in (e) and (f):** both are restricted to `Identifier::is_reference()` — a **bare** symbol
head stays exactly what it was (a local callable / substituted data). That keeps arc-029's `,,X`
outer-pass path and every local closure unchanged.

---

## 3. THE GATES

### 3.1 The non-vacuity rows — THREE, in ONE test

`src/types/surface.rs::tests::a_name_slot_reads_both_spellings_a_marker_slot_reads_one_and_the_wall_still_fires`

```
test types::surface::tests::a_name_slot_reads_both_spellings_a_marker_slot_reads_one_and_the_wall_still_fires ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1320 filtered out
```

| row | what it pins | how |
|---|---|---|
| **1** ⭐ | a NAME slot reads both spellings, **same identity** | the same peer surface twice; `assert_eq!` that the request param resolves to `":t::Echo::Req"` **and** that the two spellings agree |
| **2** ⛔ | `:messages` / `:features` / `:nature` are **still MARKERS** | three surfaces, one per marker spelled as a symbol (bare and namespaced); each must still be a located `MalformedDecl` **whose full REASON is asserted byte for byte** — the kind alone would pass on any malformation, so the row pins that the failure is *the clause is missing* |
| **4** ⭐⭐ | the wall **still FIRES on a genuinely undeclared type, in BOTH spellings** | a `:messages` record whose field references `:t::Bad::Oops`, absent from `:messages`; `assert_eq!` on the **whole reason**, byte for byte, both spellings |
| ~~3~~ | — | ⛔ **STRUCK by the builder mid-stone.** Nothing asserted about variant tags, either way. |

⭐ **THE GUARD HAS FAILED ONCE — TWICE, ACTUALLY** (`[[A GUARD IS NOT A GUARD UNTIL IT HAS FAILED
ONCE]]`). Both by targeted revert of the cure under it, re-run, restore:

```
# revert (a)+(b):
panicked at src/types/surface.rs:1365:
  the faithful-Clojure spelling must parse …: MalformedDecl … surface :t::Echo feature `echo`
  references protocol type :t::Echo::Req which is not declared in this surface's :messages
        ↑ THE FALSE RED, reproduced by the test itself

# revert (b) ONLY:
panicked at src/types/surface.rs:1433:
  a type reachable from :messages but absent from it is a located error in EITHER spelling —
  255.10 must not turn a false red into a false green: SurfaceDef { name: ":t::Bad", … }
        ↑ THE FALSE GREEN. Row 4 is not decoration; (b) is what earns it.
```

Rows 2 and 4 both assert the **full reason** with `assert_eq!`, never `contains` — a loose check
would pass on a wall that named the **wrong** message, or on a fixture that was malformed for an
unrelated reason. ⚠ **Row 2 was strengthened mid-stone**: it originally asserted only the error
KIND, which a fixture broken for any other reason would have satisfied
(`[[AN ACCEPTANCE ROW A DEFECT CAN SATISFY IS NOT A ROW]]`).

### 3.2 ⭐ The 8 `defsurface` residue files — **8 of 8 CLOSE**

Converted copies, `wat --check`, pre-cure binary vs cured binary, both built this session:

| file | conv rc pre | conv rc post |
|---|---|---|
| `tests/services/probe_arc272_rs1_state_must_be_record.wat` | 1 | **0** |
| `tests/services/probe_arc278_peers_bijection_case3_form_ok.wat` | 1 | **0** |
| `wat-scripts/probes/arc-170/probe-compound-upcast.wat` | 1 | **0** |
| `wat-scripts/probes/arc-170/probe-m1-dump-forms.wat` | 1 | **0** |
| `wat-scripts/probes/arc-278/s2s-process-probe.wat` | 1 | **0** |
| `wat-scripts/scratch-pad/probe-arc278-let-tail-service-reaped.wat` | 1 | **0** |
| `wat-scripts/scratch-pad/probe-arc278-tco-drops-caller-env.wat` | 1 | **0** |
| `wat-tests/service-parametric-bare-messages.wat` | 1 | **0** |

The rete class closes **4 of 4**, the `keyword-node` class **3 of 3** (two of the three then surfaced
a *different*, reported defect — §4 (b)).

⭐⭐ **THE BRIEF'S CENTRAL CLAIM, REPRODUCED ON PURPOSE-BUILT PROBES.** A correct surface and a
genuinely-broken one, each in both spellings:

| probe | keyword | symbol, PRE-cure | symbol, CURED |
|---|---|---|---|
| `good.wat` — correct | rc 0 | ⛔ **rc 1 "EchoRequest is not declared"** (FALSE RED) | ✅ rc 0 |
| `undeclared.wat` — `Oops` genuinely absent | rc 1, names **`:p::Echo::Oops`** | ⛔ **rc 1 naming `:p::Echo::EchoRequest`** — a **correctly declared** type; the true defect is never reached | ✅ rc 1 naming **`:p::Echo::Oops`**, **byte-identical to the keyword diagnostic** (diffed, modulo path) |

### 3.3 Delta — **18 → 4**, with the RECOVERY column

`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`, 179 paths, sha256
`33ede76cbb9b8809ecebc7656db3d4733769df54c2641b7e6d798773f1cbc3e5` — **used as committed, never
rebuilt** (`[[USE THE FILE. NEVER REBUILD BY INDEX.]]`). All 179 present (`missing=0`). ⚠ **Originals
and converted were BOTH checked from copies under scratch** — the asymmetry that manufactured a
phantom regression in the 255.9 weigh.

| binary | orig clean | conv clean | **NEW (clean→broken)** | ⭐ **RECOVERY (broken→clean)** |
|---|---|---|---|---|
| **pre-cure** (`f1bef5944` src) | 160/179 | 143/179 | **18** | **0** |
| **cured** | 160/179 | **156/179** | ⭐ **4** | ⭐ **0** |

**Gate ✅ 4 < 18.** ⭐ **RECOVERY = 0 — no file went broken→clean under a pure spelling change**, so
nothing in this diff disarmed a checker. ⭐ **160/179 reproduces 8d-ii-THIRD's and 255.9's
pre-conversion baseline exactly**, and `join`ing the two runs shows **0 rc changes on the ORIGINAL
side across all 179** — the cures are invisible to the keyword spelling, as intended.

⭐ **255.9 recommended RECOVERY as a standing column; this is the first stone to print it as one, and
it is the column that would have caught eight stones of forged greens.** It also caught *me*: the
intermediate measurement that exposed my own `vector_splice_symmetry` regression was this same table
(§2 (e)).

### 3.4 Census — `no STOP-8`

⚠ **`[[DIFF LIKE AGAINST LIKE]]` — baseline comparability CHECKED BEFORE USE.** The newest artifact
is **not** always the right one: `.census/2026-09-22T03-34-24Z.txt` has **343** failures because it
was captured under the 8d-ii **converted-stdlib** tree. I used `.census/2026-09-22T05-21-06Z.txt`
(2202 files, **212** nonzero) after verifying `git diff 4b6088454..f1bef5944 -- src/` is **EMPTY** —
so that artifact was produced by a binary whose `src/` is identical to this stone's parent.

```
baseline: .census/2026-09-22T05-21-06Z.txt   files=2202   nonzero 212
this stone: .census/2026-09-22T06-32-56Z.txt  files=2202   nonzero 212
$ ./scripts/replay/census.sh --diff .census/2026-09-22T05-21-06Z.txt .census/2026-09-22T06-32-56Z.txt
census-diff: no STOP-8                                                       (rc 0)
```

**Zero rc changes in either direction across all 2202 files** (`join … | awk '$2!=$3' | wc -l` = **0**), and the nonzero count is identical to the baseline. Expected: nothing in the live tree is converted, so the cures are invisible to it — ⛔ **which is also exactly why this gate could not have caught the defects this stone cures** (§5.7).

### 3.5 Clippy

```
$ cargo clippy --all-targets --workspace -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.22s
clippy exit=0        (no warnings; re-run on the frozen tree, exit 0 again)
```

### 3.6 Floor — ⛔ **THE FIRST RUN WAS CONTAMINATED BY ME. DISCLOSED, THEN RE-RUN CLEAN.**

#### Run 1 — `.floor/2026-09-22T06-16-37Z/` — GREEN, **but do not count it**

```
     Summary [ 322.953s] 5963 tests run: 5963 passed (10 slow), 22 skipped
[floor] exit=0
```

⛔ **I ran `cargo build --release` while this run was in flight**, to rebuild after a cosmetic
simplification of `is_callable_form`. nextest had already compiled its own binaries, but
`target/release/wat` — which corpus tests invoke — was replaced underneath the run. **The change was
behaviour-preserving (verified: the five discriminating probe files return the same rc before and
after), but a green whose binary changed mid-run does not describe the tree I am committing.**
Reported, not relied on. `[[A GREEN MUST DISCLOSE WHAT BOUGHT IT]]`.

#### Run 2 — `.floor/2026-09-22T06-24-…Z/` — GREEN, **and ALSO stale, by the same mistake**

```
     Summary [ 334.218s] 5963 tests run: 5963 passed (9 slow), 22 skipped
[gates] floor exit=0
```

⚠ After Run 2 started I strengthened non-vacuity row 2 (§3.1) from an error-KIND assertion to a
full-REASON one — a `#[cfg(test)]`-only change, but a `src/` change nonetheless. **Reported, not
relied on, for the same reason as Run 1.**

#### Run 3 — THE GATE — on the FROZEN tree, nothing touched during it

```
     Summary [ 318.569s] 5963 tests run: 5963 passed (10 slow), 22 skipped
[floor] exit=0. Log kept at .floor/2026-09-22T06-33-40Z/ regardless — a green run is evidence too.
```

⛔ **I made the same process error twice** — rebuilding under a running floor, then editing test
source under the next one. Neither green was thrown away silently; both are above. **The lesson is
the cheap one and I am writing it down: FREEZE THE TREE BEFORE THE FLOOR, not during it.** Doctests:
exit 0 in all three runs. No `ARM.txt` was written in any run (the script writes one only on a red).

### 3.7 ⛔ NOT ONE `.wat` CONVERTED

```
$ git status --porcelain
 M src/edn/render.rs
 M src/freeze/env.rs
 M src/macros/eval.rs
 M src/macros/expand.rs
 M src/types/surface.rs
$ git status --porcelain -- '*.wat'     (empty — 0 lines)
```

Every conversion in this SCORE ran on copies under `/home/john/.claude/jobs/edaabf97/tmp/`.

---

## 4. SITES I REPORTED RATHER THAN CURED — each with its reason

**(a) ⚠ `wat/source.wat` and `wat/holon/Ngram.wat` — NOT THIS STONE'S CLASS.** They fail converted
with `DuplicateType :wat::source::File` / `DuplicateMacro :wat::holon::Ngram`. **Measured, isolated to
four one-line probes:** the spelling is irrelevant — `(:wat::core::defrecord :wat::source::File [path
<- :wat::type::String …])`, in the **keyword** spelling, fails **identically**, while the same file
with `wat.core/String` (symbol) is **clean**. The divergence is `wat.type/X` vs `wat.core/X`, not
keyword vs symbol. The site is `TypeEnv::register_validated` (`types.rs:995`), whose `Existing` is
raw `Some(e) if e == &def` — while the substrate already owns `types::type_exprs_same`, a denotation
-aware equality whose own doc says *"a converted `wat.type/i64` stores `:wat::type::i64` and must
still match."*
⛔ **NOT CURED, deliberately:** this would make a **duplicate-declaration wall** more permissive —
turning `Divergent` into `Equivalent` is exactly the "red into a false green" direction the brief
forbids, at the one gate that decides what counts as a benign re-declaration. **A wall's population
is the builder's call.** ⭐ **Recommended as the next stone; it is 2 of the remaining 4.**

**(b) ⚠ `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and `…/probe-kwargs-peer.wat` — the
defect is in a `.wat` FILE, and this is a `src/` stone.** After cure (d) unblocked them they fail with
8 `TypeMismatch`es (`expects (:wat::kernel::Address :- [:probe::Echo::Op …]); got (… :- [:S :R])`).
**Located:** `wat/core.wat:1087/1155` calls `(kwargs-type-slot-swap-head child "wat::kernel::Peer"
"wat::capability::TypedCapability")` — a **`::`-spelled substring** searched inside a name that, once
converted, reads `wat.kernel/Peer`. The swap silently does nothing. ⭐ **This is the injected
`CLAUDE.md`'s named recurring class — a string comparison with one side normalized and the other not
— and the file's own comment says the eight call sites "swap … by `ast-name` +
`string::contains?`/`split`+`join`".** ⛔ **Not cured:** the fix belongs in `wat/core.wat`, and the
structural fix needs a wat-level canonical-identity verb, **which does not exist** — measured:
`:wat::keyword::` exposes only `to-string`, `from-string`, `to-symbol`, `to-type-form`,
`to-type-form-colon`. ⭐ **Recommend minting `:wat::keyword::canonical-identity`** (the Rust door is
already `edn::render::canonical_identity`) so the eight sites can compare identities, not text.

**(c) ⛔ VARIANT TAGS — out of scope by the builder's own correction.** My census found no `src/`
site where a variant tag's spelling is load-bearing *for this stone's cures*, and the codemod does not
convert them today. **Nothing asserted, nothing cured, nothing pinned.** Needs the ruling the builder
has already opened.

**(d) 255.9's open list is UNCHANGED and I did not narrow it.** The `:wat::verify::*` markers in
`digest-load!`/`signed-load!` and their `runtime.rs:14272/14343` siblings (LOUD, 0 corpus users); the
two mutation walls (BACKSTOPPED — *the refusal survives, the diagnostic does not*; widening a security
wall's population is the builder's call); `runtime.rs`'s session/`run()` paths (**unmeasured, and I
declare them unmeasured — I built no session-level probe either**).

**(e) ⚠ THE EXPLICIT QUASIQUOTE SPELLING, still an 8d-iii blocker.** 255.9 measured
`:wat::core::unquote` on **103 lines across 34 `.wat` files**. `` ` ``/`~` sugar survives conversion
(reader-synthesized leaves are skipped); the explicit spelling does not. Cure (f) does **not** touch
that — `is_quasiquote_form` and `quasiquote_inner` still test the keyword `:wat::core::quasiquote`
exactly, and are correct to for the sugar. **Reported, unchanged.**

---

## 5. ⛔ WHAT MY GREEN CANNOT SEE

1. ⛔ **The UNREACHABLE half of my census is an argument, not a measurement** — the same limit 255.9
   named and asked to have said again if still true. **IT IS STILL TRUE.** Re-order any pass relative
   to step 7 and every UNREACHABLE row goes live silently. **There is still no gate on that ordering.**
2. ⛔ **My census frame is wider than 255.9's and still not total.** I classified shapes A (83) and
   the two B/D sites the failures led me to. **I did not read all 32 shape-B sites or all 24 shape-D
   sites.** Two of six cures were in exactly those unread populations, found by a failing FILE, not by
   the frame — so the honest reading is that the frame follows the evidence, not the other way round.
   ⭐ **A shape-B/D sweep is a real remaining exposure and I am naming it as one.**
3. ⛔ **Cure (f) widens what counts as expand-time CODE.** A `,,X` outer-pass substituted list whose
   head is a namespaced symbol is now EVALUATED where it used to be spliced. I restricted it to
   `is_reference()` and the floor/census/delta are clean, but **the keyword spelling of that same
   list was ALWAYS evaluated** — so the new behaviour is the old behaviour of the other spelling, not
   a new rule. **That is the argument; the floor is the only measurement.**
4. ⛔ **Cure (e)'s two-candidate allow-list check is a WIDENING of a default-deny gate.** A head is
   legal if *either* rendering is on the list. For a keyword head nothing changes (only one candidate
   is produced). For a symbol head it is strictly more permissive than testing one rendering — and
   strictly less permissive than the status quo ante, which tested **none**. **No probe here
   constructs a head whose two renderings disagree in legality;** I could not build one that is also
   reachable. **Unmeasured, and declared so.**
5. ⛔ **"LOUD" still means *a* diagnostic fired, not the *right* one.** Every LOUD row in §1.3 will
   still cost an 8d-iii session pointed at the wrong subsystem. Unchanged from 255.9.
6. **The delta gate remains blind in the direction it was always blind.** RECOVERY is 0 here, which is
   evidence about *this* diff; it is not a gate anyone is forced to print. ⭐ **255.9's recommendation
   that it become a STANDING column is still not implemented, and this stone did not implement it.**
7. **The corpus census only looks at `0 → non-zero`.** A file going `non-zero → 0` reads as an
   improvement. My census diff shows no changes in either direction *because nothing in the live tree
   is converted* — it would not have caught a forged green if something had been.
8. **`--check` is not `run`.** 156 of the 179 converted files check clean (the other 23 = 19 that are
   broken as ORIGINALS too + the 4 named in §4). §3.2's and §1.5's probes are the only places I ran a
   converted program to completion. **The other 154 are `--check`-clean and UNEXECUTED** — a program
   that type-checks under a spelling change can still behave differently at runtime, and 255.9's
   `scan_for_setter` is the proof that it can do so silently.
9. ⛔ **Run 1 of the floor was contaminated by me** (§3.6) and I am reporting Run 2 as the gate. If
   the behaviour-preservation argument for the intervening refactor is wrong, Run 2 is the only thing
   that would catch it — which is why Run 2 exists.

---

## 6. WHAT THE BRIEF GOT WRONG

⭐ **(1) "634 keyword read-sites."** Measured at the brief's own draw commit (`src/` unchanged since):
**665**, in 65 files. More importantly, **the count is the wrong frame** — a keyword *requirement*
also wears a `matches!` predicate and a bare `starts_with(':')` string test, and **two of this stone's
six cures are in those two populations** (§1.1). The brief's warning *"do not cure blind"* is right;
its probe would not have found the security-wall defect.

⚠ **(2) `surface.rs:746` and `surface.rs:1003`.** The `:messages` name read is at **`surface.rs:729`**,
not 746. (1003 is exact.) Named here so the next hand does not chase a line that reads something else.

⭐ **(3) "15 of the 18 failures are three user forms."** ⛔ **Measured: 15 of 18 are three user forms —
but not the three the brief names, and one of the brief's "other" three IS this class.** The
`defsurface`/rete/`keyword-node` split is **exactly 8/4/3 = 15**, confirmed. Of the remaining 3,
`tests/macros/probe_arc278_macro_call_site.wat` is **`is_callable_form`** — a fourth member of the
class, and the one that led to the F5 wall. The other two (`wat/source.wat`, `wat/holon/Ngram.wat`)
are **not this class at all** (§4 (a)), which the brief's framing would have had me cure blind.

⚠ **(4) The brief's own discriminator table was corrected by the builder mid-stone** (§0). Recorded
here because the SCORE must contain what it was executed against, not only what it ended with.

---

## 7. REPRODUCTION

```bash
cargo build --release
cargo test --release --lib types::surface::tests
cargo clippy --all-targets --workspace -- -D warnings
./scripts/replay/census.sh
./scripts/replay/census.sh --diff .census/2026-09-22T05-21-06Z.txt <CURR>.txt
./scripts/floor.sh
# delta: copy the 179 committed paths twice, codemod ONE copy, check BOTH from copies
printf '[…179 abs paths…]\n' | ./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat
```

Scratch (not committed): `/home/john/.claude/jobs/edaabf97/tmp/` — `wat-baseline`/`wat-cured` (the two
binaries), `delta/{orig,conv}`, `{pre,post,post2,post3,post4}.tsv`, `probe/`, `mcs/`, `dup/`.
Committed: the five `src/` files and this SCORE.
