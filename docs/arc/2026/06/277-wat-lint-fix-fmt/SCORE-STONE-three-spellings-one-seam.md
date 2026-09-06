# SCORE — STONE: three head spellings, one seam

No commit. Floor and clippy left to the orchestrator.

One function folds three head spellings onto the **clojure target**. Fmt rules compare `wat.core/defn`, not `:wat::core::defn`. Dropping a keyword flavor is the deletion of one arm.

---

## RELAND — Named is canonical for every node, so every reader of Named had to move

The seam was accepted. The first strike migrated fmt rules only. `Named.name` is canonical for **every** node, so grep programs and recorded migrations that still compared FQDN strings were silently dead. `--check` stayed green. `wat_grep::*` stayed green (`"<-"` is identity). Proven dead: `can-raise.wat` with `":wat::core::first"` on `wat/deporder.wat` → **0** matches; with `"wat.core/first"` → **9**.

### The same codemod, 11 files

`fmt-head-fqdn-to-clojure.wat` over the 11 named files: **CHANGED=11**, second run **0**.

```
wat-scripts/grep/       5 files, 13 compares
wat-scripts/fixes/      5 recorded migrations, 20 compares
wat-scripts/scratch-pad/probe-four-homes-census.wat  2 compares
```

### starts-with prefixes (not in the 11, required by the gate)

`core-numerics-ops.wat` / `rete-numerics-ops.wat` compare `starts-with?` against `":wat::core::i64::"` etc. The fold turns `:wat::core::i64::+` into `wat.core.i64/+`; a trailing-`::` prefix is identity under `canonical_head_name` (by `wat_keyword_to_clojure_symbol`'s own None). The same-codemod rewrite would not touch them. Hand-moved the four prefixes to `wat.core.i64/` / `wat.core.f64/` / `wat.rete.core.i64/` / `wat.rete.core.f64/`. Without that, the gate over `wat-scripts/grep/*.wat` would go red on two programs the 11-file list did not name.

### STOP-3 — left alone, and why

`wat-scripts/fixes/to-faithful-clojure-rete.wat` and `to-faithful-clojure-net.wat` compare `"<-"` and `"->"`. Bare data; the fold is identity. Not rewritten.

### Each grep program matches (not `--check`)

| program | target | matches |
|---|---|---|
| can-raise | `wat/deporder.wat` | **9** (RELAND's proof) |
| head-position | `wat/deporder.wat` | **3** |
| bare-variant-constructors | `wat/spawn.wat` | **4** |
| defined-twice | `wat/bracket.wat` | **2** |
| unwrap-of-lookup | live corpus `HashMap/get` is gone | **0** on `wat/grep.wat` |
| all 7 | `tests/cli/grep_smoke_target.wat` | **3, 2, 1, 1, 1, 1, 1** |

### The gate

`tests/cli/grep_programs_still_match.rs` — every `wat-scripts/grep/*.wat` is driven via `wat --grep` over `tests/cli/grep_smoke_target.wat` (two same-name defns, `first`, `Option/expect`+`HashMap/get`, `Some`/`Ok`/`Err`, `:wat::core::i64::+`, `:wat::rete::core::i64::+`). Asserts a non-zero `#wat.grep/Match` count. Vacuity-guarded: the program list must not be empty.

`every_wat_scripts_grep_program_matches_a_known_target` **PASS**.

Un-arm the canonical fold (or revert one comparison to FQDN) and this test goes red. `--check` of the same files would not.

### STOP-5 — four corpus files still byte-identical

deporder, spawn, fmt, io vs the pre-seam capture: **IDENTICAL**.

---

## Row 1 — it builds

`cargo build --release` clean. `wat/grep.wat` is frozen; the release `wat` used below is newer than the Named-line edit.

## Row 2 — all three spellings get the SAME ruled shape

Same form, ≥2 defn args, ≥2 let binders. `IDEMPOTENT=true` on each.

FQDN / dotted / clojure — args one per line with `<-` aligned; let binders paired:

```
(HEAD_DEFN :fix::two
  [a <- :wat::core::i64
   b <- :wat::core::i64]
  -> :wat::core::i64
  (HEAD_LET
    [x a
     y b]
    x))
```

Dotted lives as `spelling-dotted.wat.bad`: the checker does not treat `:wat.core/defn` as `defn` (Doctrine 1 on the type keywords). The formatter still rules it. FQDN and clojure `--check` **0**. The clojure target is what `--check`s; the printer's dotted keyword is a format-only fixture.

## Row 3 — identical modulo the head token

Strip `defn`/`let` spellings and comments from the three formatted outputs: **byte-identical**.

## Row 4 — two-argument `defn` is the discriminator

Every spelling fixture has two args and two let binders. Row 6's deletion proves the discriminator: without the FQDN arm, FQDN collapses to `[a <- :wat::core::i64 b <- :wat::core::i64]` on one line (fallthrough). With the arm, one arg per line.

## Row 5 — the canonical form is the clojure target

Rules compare `"wat.core/defn"`, `"wat.core/let"`, `"wat.core/if"`, … `grep` for `string::=` / `string::not=` against `:wat::core::` in `fmt/rules/` is **0**.

## Row 6 — DROPPING A FLAVOR IS ONE DELETION (demonstrated, restored)

Deleted the FQDN arm of `canonical_head_name`. Rebuilt. **No rule file touched** (rules hash `e348d23c…` identical).

| fixture | with arm | FQDN arm deleted |
|---|---|---|
| clojure `wat.core/defn` | ruled | **UNTOUCHED** (byte-identical) |
| FQDN `:wat::core::defn` | ruled | **CHANGED** — args and binders collapse to one line (generic fallthrough) |

Restored the arm. FQDN fixture byte-identical to the pre-deletion capture.

## Row 7 — three-spelling knowledge does not leak into the rules

`grep -c ':wat::core::' wat-scripts/fmt/rules/*.wat` = **0** on every file.

## Row 8 — the canonicaliser is ONE function

**Home:** `src/edn/render.rs::canonical_head_name`, beside `wat_keyword_to_clojure_symbol` — the comment there already forbids re-encoding the `::`/`/` grammar in wat.

**Exposure:** `:wat::grep::canonical-name` (`src/intrinsic/grep.rs`).

**Call site:** `wat/grep.wat` Named construction. `ast-name` stays verbatim (Written still keys off source width). Named.name is the canonical spelling the rules match on.

Three arms:

```
":wat::core::defn"  →  wat_keyword_to_clojure_symbol  →  "wat.core/defn"
":wat.core/defn"    →  strip leading colon            →  "wat.core/defn"
"wat.core/defn"     →  identity                       →  "wat.core/defn"
```

Bare data is identity (`:else`, `:-`, `<-`, `x`). Not a second name parser: arm 1 is the existing door (`identifier::leaf`/`path`). Unit tests: `three_spellings_fold_to_the_clojure_target` **PASS**, `bare_data_and_binders_are_identity` **PASS**. Smoke: `"wat.core/defn | wat.core/defn | wat.core/defn"`.

## Row 9 — FQDN corpus is a no-op

Captured formatted output of the five files **before** the strike (`/tmp/fmt-before-seam`). After: **byte-identical** for deporder, spawn, fmt, io. `wat/grep.wat` is the file this stone edited (Named line); its source moved, so it is not in the identity set.

## Row 10 — a real doc-row example now gets the RULED shape

`render_one` on `step_payload.rs:19-62` emits the example in **dotted** spelling. Formatted:

```
(:wat.core/do
  (:wat.core/defrecord :probe/StepPayloadExampleTemp
    [celsius <- :wat.core/i64])
  …
  (:wat.core/let
    [rules (:wat.rete/collect-rules :probe)
     session (:wat.rete/compile rules)
     …
```

Defrecord's **name rides**. Let binders **paired**, one per line — not the fallthrough explosion.

## Row 11 — the 614

`N=614 CHANGED=614 INLINE=282 OVER120=0 WORST=104`.

## Row 12 — no comment lost

`28` · `85` · `429` · `157` · `45` both sides.

## Row 13 — idempotent

spelling-fqdn, spelling-clojure, spelling-dotted: **IDEMPOTENT=true**.

## Row 14 — hygiene

`'col'` **0** · `'120'` **0** · `ClaimedUnder` **0**.

## Row 15 — wat-scripts load · wat-grep

`every_wat_scripts_file_loads_on_the_current_runtime` **1 passed**.
`wat_grep::*` **7 passed**.

## Rows 16–17 — floor / clippy (ORCHESTRATOR)

Not run.

---

## Codemod (R21)

`wat-scripts/fixes/fmt-head-fqdn-to-clojure.wat`. Dry-run on `/tmp` copies: **CHANGED=12** (siblings + table have no head FQDN). Second dry-run **CHANGED=0**. Apply to `wat-scripts/fmt/rules/*.wat`: **CHANGED=12**, second run **0**. A pre-migration file reports 1+.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| three-arm smoke | `wat.core/defn` × 3 |
| spelling fixtures, heads stripped | **byte-identical** |
| FQDN arm deleted | clojure untouched; FQDN fallthrough; rules hash unchanged |
| FQDN arm restored | FQDN fixture restored |
| four corpus files vs pre-strike capture | **IDENTICAL** |
| step-payload dotted example | defrecord name rides; let binders paired |
| 614 | **OVER120=0 WORST=104** |
| comments | 28 · 85 · 429 · 157 · 45 |
| load gate | **1 passed** |
| `wat_grep::*` | **7 passed** |
| `grep -c ':wat::core::'` over rules | **0** |

## Honest deltas

- Dotted `:wat.core/defn` is **not** a checker special form. The fixture is `.wat.bad` so the load gate does not type-check it. The formatter still rules it. The clojure target `--check`s. This stone is the formatter seam, not a checker alias table.
- Named is canonical for **all** nameable nodes. Bare data is identity (`<-`, `:else`, `:-`). The first strike migrated fmt only; grep programs and recorded migrations that still compared FQDN strings were dead while `--check` and `wat_grep::*` stayed green. Reland: same codemod over those 11 files, starts-with prefixes on the two numerics censuses, and a match-count gate over every `wat-scripts/grep/*.wat`.

## The board

```
✅ if · the test rides its head
✅ cond · clauses align
✅ three spellings, one seam     canonical = wat.core/… ; drop a flavor = delete one arm
✅ Break.kind is an enum
✅ Node.kind is an enum
✅ :then checks declared field types
✅ fence refuses match it cannot prove
✅ variant-name
   E · where a TRAILING comment goes UNRULED, and a width cost
```

---

## ORCHESTRATOR VERDICT — 2026-09-06 (after the RELAND)

**ACCEPTED, with one orchestrator fix.** Floor **5192 run, 5192 passed, 0 FAILED**. Clippy **0**.

| what | my own re-run |
|---|---|
| ★★★ the dead rules are alive | `can-raise` on `deporder` **0 → 9** · `head-position` **3**. The exact A/B that proved them dead |
| ★★★ **the gate DISCRIMINATES** | reverting the load-bearing compare in `head-position` drops its match count to **0** and the gate goes **RED**. See the correction below |
| ★★★ no FQDN name-compare survives | `string::=` against `":wat::` across `grep/` + `fixes/` + the probe = **0** |
| ★★★ row 6 · dropping a flavor | (from the first strike, unchanged) FQDN arm deleted → clojure fixture byte-identical, FQDN collapses to fallthrough, **no rule file touched** |
| ★★★ STOP-5 · the corpus | `deporder` · `spawn` · `io` · `fmt` all **IDENTICAL** under my per-line capture |
| ★★ row 2 · the spellings | FQDN and clojure produce the same ruled shape — args one per line, `<-` aligned |

## ⛔ THE FLOOR CAUGHT A RED, AND IT IS A GATE DOING EXACTLY ITS JOB

```
FAIL wat intrinsic::tests::checker_skip_debt_is_named_and_frozen

  NEW — registered but absent from `CheckEnv` … `doc_arg_ret_types_match_checker_scheme`
  is silently skipping these and verifying nothing about their @arg/@ret docs:
      [":wat::grep::canonical-name"]
```

The stone's new intrinsic was reachable from wat and **never type-checked**. The diagnostic named
both remedies and the better one: register it, rather than park it on the debt ledger. Registered in
`src/check.rs` beside `:wat::string::trim` — its declared shape is exactly `String -> String` — so the
doc gate now verifies its `@arg`/`@ret` for real. **Floor after: 5192/5192.**

★ Sixth time this arc that the floor or clippy found what a targeted run could not.

## ⚠ AND A CORRECTION TO MY OWN VERIFICATION — I NEARLY REPORTED THE GATE AS BROKEN

My first sabotage of the new gate `sed`-replaced `"wat.core/defn"` in `head-position.wat`. **That
string does not appear in that file.** The edit changed nothing, the gate passed, and I wrote *"the
gate did not catch it"* — one step from publishing a false finding about the very artifact the
reland demanded.

The load-bearing comparison is `"wat.core/first"`, the one `:hp::calls-first` uses to emit its Match.
Reverting **that** drops the count to 0 and the gate fails.

⚠ **Third no-op sabotage of this session** — after the self-consistent rename of `:wat::fmt::spaces`
and the global `"block"`→`"align"` flip. Every one produced a green I briefly read as evidence.
`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`

## ⛔ MY RELAND'S CENSUS WAS INCOMPLETE, AND I HAD THE MISSING DATA IN HAND

The reland scoped **11 files / 35 comparisons**, from a `string::=` grep. The true scope was **13**:
`core-numerics-ops.wat` and `rete-numerics-ops.wat` ask the same question with
`string::starts-with?` against a trailing-`::` prefix, which the fold leaves as identity and the
codemod would never touch. **The strike found them because the new gate went red on two programs my
list did not name.**

★ **And I had measured those sites minutes earlier** — they are the census in
`[[NOTE-a-namespace-question-must-not-be-asked-with-a-string-prefix]]`, written, committed, and
never folded back into the reland's scope. Two censuses of the same class in one session, joined by
nobody. `[[feedback_a_census_of_a_name_must_ask_every_rendering]]`

## The residue, named rather than swept

**15 `starts-with?` FQDN prefixes remain, all in `wat-scripts/fixes/`** — spent recorded migrations.
They are dead by the same mechanism, and the disposition is deliberate:

- the repo's bar for `fixes/` is `every_wat_scripts_file_loads` — a **parse** gate, not a match gate;
- they are already applied, so no consumer re-runs them for effect;
- and `[[NOTE-a-namespace-question-must-not-be-asked-with-a-string-prefix]]` rules that rewriting a
  prefix literal is a **stem-patch**. Churning fifteen of them now buys a spelling that 251.8b's
  proper `ns-of` API will replace anyway.

⚠ **The honest cost of leaving them: an already-applied codemod reports `CHANGED=0`, and a
rules-dead one reports `CHANGED=0`.** For `fixes/` that ambiguity is now permanent until the
namespace API lands. It is written down here so it is a known, chosen residue rather than a
discovery.

## The board

```
✅ three spellings, one seam       canonical = wat.core/… ; drop a flavor = delete one arm
✅ and every reader of Named moved  13 files · a match-gate, sabotage-proven
   E · where a TRAILING comment goes UNRULED, and a width cost
   ns-of / in-ns?                  NOTE filed in arc 109; blocked on 251.8b (derived → stored)
   15 starts-with? prefixes in fixes/  known residue; stem-patches by the NOTE's own ruling
```
