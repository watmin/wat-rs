# SCORE — STONE 255.13: the heresy ledger

**Executed against `600abe8c3`.** Every number in this document was produced this session, by an
instrument that lives in the tree and that anyone can re-run (`cargo nextest run --release -E
'test(keyword_heresy)'`, ~4 s). ⛔ **`src/` is BYTE-IDENTICAL to HEAD** — `git diff --stat HEAD --
src/` is empty. ⛔ **Not one `.wat` was converted.** The diff is `Cargo.toml` (+2 dev-deps) and one
new file, `tests/lint/keyword_heresy_ledger.rs`.

## VERDICT — **LANDED**

⭐ **THE LEDGER READS 232.** 93 shape A (keyword-only dispatch) · 14 shape B (dual-read, keyword-keyed)
· 0 shape C · 125 shape E (type path without the denotation door), across 29 files and 111 functions.
Plus **68 sites whose provenance no call site resolved**, counted apart and never folded in, and
**61 sites the discriminator can PROVE are cured.**

⭐ **The calibration gate holds: every site this arc cured is absent from the ledger; the open one is
present, and the discriminator found its REAL HOME by data flow.**

⛔⛔ **AND THAT IS THE TWENTY-SECOND CORRECTION. The brief's mandatory calibration target —
`src/collection/transform.rs:304-340` — CONTAINS NO KEYWORD COMPARISON AT ALL.** Measured: **zero**
`":wat::` string literals between lines 290 and 360, and every such literal in the whole file is an
`OP` label, a `#[wat_intrinsic]` attribute, or message text. That range is `sort$native`'s **CALL**
of the purity gate. The comparison 251.8d-ii's fourth draw convicted lives in **`src/rete/purity.rs`**,
and the lint names it there without being told: `classify_expr` reads its head as
`Keyword(k) => k.as_str()` / `Symbol(id) => id.as_str()` **with no door** (shape B), and the
inter-procedural pass carries that provenance into `intrinsic_meta` and `effectful_by_prefix`, whose
tables are keyword-keyed. **12 of the 14 shape-B sites in the entire crate are in that one file.**
That is the `wat.core/<` failure, and this is the first instrument in the arc to point at it
mechanically rather than by reading a stack trace.

⭐ **The lint has FAILED once, by construction, and §6 shows the whole output.** It also carries a
permanent, self-contained negative control (`the_discriminator_convicts_a_synthetic_heretic_and_clears_its_cure`)
that asserts a conviction for each of the four shapes **and** a `cured` verdict for the same shapes
routed through their doors — so a refactor that quietly makes the analyzer answer `cured` to
everything cannot pass.

Gates: ⛔ **the first floor went RED — three tests, all three MINE, all three captured verbatim in
§8.1 and cured; the second floor went RED on a second offender inside one of them; the third is
GREEN 5993/5993** · clippy **0** · census **`no STOP-8`**, 0 rc changes across 2209 · delta
**3 = baseline 3**, ⭐ **RECOVERY 0**.

---

## 1. ⛔ THE BRIEF'S NUMBERS ARE BOTH WRONG, AND SO IS ITS TARGET

| the brief says | measured here | what the gap was |
|---|---|---|
| `":wat::…"` literals in `src/` — **7 233** | **5 976** | a count of a different thing; mine is `grep -o '":wat::[^"]*"' src/ --include=*.rs \| wc -l` |
| of those, comparison/dispatch shapes — **~872** | ⛔ **364 decision sites** | the ±3-line window counted `OP` labels, `#[wat_intrinsic]` attributes and `format!` arguments as comparisons. It also counted each `match` ARM (450 lines matching `":wat::…" =>`) as a separate site; the **decision** is the `match`, and there are 28 of them in the ledger |
| of a 354-line subset, no normalizer within ±3 lines — **340** | ⛔ **232** heretical, **61** provably cured, **68** unresolved | proximity cannot see a `let` chain, a `match` scrutinee, or a callee |
| ⭐ **MUST flag `src/collection/transform.rs:304-340`** | ⛔ **IMPOSSIBLE — there is no comparison there** | §1.1 |

### 1.1 The convicted site is not where the brief points

`src/collection/transform.rs:304-340` is the body of `eval_vec_sort_by`'s purity gate: it binds
`comparator_label`, picks a `ClassifyCtx`, loops `[(Axis::Pure, "pure"), (Axis::Deterministic,
"deterministic")]`, calls `find_axis_violation_ctx` / `classify_native_fn`, and formats a
`MalformedForm`. **No string is compared to a keyword literal anywhere in it**:

```
$ awk 'NR>=290 && NR<=360' src/collection/transform.rs | grep -c '":wat::'
0
```

The 8d-ii FOURTH draw's own capture names the two halves separately:

> `comparator ":wat::core::Fn" is not pure: "wat.core/<" is not proven pure`

`wat.core/<` is the **head** that failed to prove, and it failed inside `classify_expr`'s walk —
`src/rete/purity.rs`. The chain the discriminator reconstructs, unaided:

```
rete/purity.rs::classify_expr        general-list arm:
    Some(WatAST::Keyword(k, _)) => k.as_str(),      ← canonical
    Some(WatAST::Symbol(id, _)) => id.as_str(),     ← `wat.core/<`, RAW      ⇒ provenance = BothAst
        ↓ head
  head_ok(head, …)
        ↓
  intrinsic_meta(head)          matches!(head, ":wat::core::+" | …)          ⇒ shape B   purity.rs:525
  effectful_by_prefix(head)     head.starts_with(":wat::kernel::") …         ⇒ shape B   purity.rs:2117-2124
```

**The lint's calibration test asserts `intrinsic_meta` is in the ledger AND that its shape mix
contains `B`** — not merely that it is flagged. Shape B is the claim that a *symbol-spelled* head
reaches a *keyword-keyed* table; without it the row would prove nothing about that defect.

⚠ The brief is not wrong about the *defect*, only about its address. Both readings agree that
8d-ii's fifth draw is the next cure; they disagree about which file it edits.

---

## 2. THE DISCRIMINATOR

### 2.1 What it is

`tests/lint/keyword_heresy_ledger.rs`, ~1 200 lines, in the shape this repo already uses
(`no_bare_is_err.rs`, `ignore_reason_justified.rs`): a banned shape plus a frozen list that can only
shrink. It parses **every** `.rs` under `src/` with **`syn`** — a real Rust parser, added as a
dev-dependency **because the alternative was another regex**, and this arc has paid for that
instrument twice (255.9's ±3-line probe: 3 false positives in 14; 255.11's name grep: three `#[test]`
fns and one error *constructor* counted as capability walls, and `walk_for_restricted_call` — a live
capability escape — missed). `Cargo.lock` is **unchanged**; `syn` was already resolved for the
proc-macro crates and this adds only the `full`+`visit` features.

**Four passes.**

1. **Parse.** 333 files, **0 parse failures**. A parse failure is a hard assertion, never a silent
   skip — a file the lint cannot parse is a file it silently does not measure. `#[cfg(test)]`
   modules, `mod tests` and `#[test]` fns are excluded (this is a `src/` instrument).
2. **The DOOR SET, derived by fixpoint** — ⭐ **not hand-listed** (`[[a gate over two hand-lists is a
   hand-list]]`). Seeded at exactly **two** roots, `canonical_identity` and `ns_to_wat_path`
   (`src/edn/render.rs`) — the two functions that turn a written spelling into the internal identity.
   A fn joins when it RETURNS a name (`String`/`&str`/`Cow`) and **every** one of its returns is
   door-derived. It converges to **8**:

   ```
   canonical_identity   ns_to_wat_path            ← the two seeds
   canonical_identity_of   head_fqdn   type_denotation   denoted_type_path
   format_type_path   alias_name_token            ← DERIVED
   ```

   The census test pins six of them by name, and pins that **`identity_text` is NOT admitted** — it
   returns the raw text of either spelling (its own doc: *"then `canonical_identity` is the one
   key"*), and admitting it would make shape B unrepresentable.
3. **Provenance taint** — block-scoped, order-sensitive, shadowing honoured — over `let`, `let…else`,
   `if let`, `match` arms (each arm binds from the scrutinee's provenance), closures, and collection
   `push`/`insert`/`extend`. Each value carries one of: `Door` / `KwConst` (proven canonical) ·
   `KwAst` (a raw `WatAST::Keyword` payload) · `SymAst` (a raw `WatAST::Symbol` payload) · `BothAst`
   (joins both, un-normalized) · `TyPath` (a `TypeExpr::Path`/`Parametric` payload) · `Unknown`.
4. **Inter-procedural parameter provenance, by fixpoint over every call site in `src/`**, so
   `fn f(head: &str)` comparing `head` to a keyword literal is judged by what its CALLERS hand it.
   **Converges in 6 rounds** (cap 8; the round count is asserted, so a partial fixpoint cannot pass
   as a fixpoint). 6 189 `(fn, param)` slots resolved.

**Decision shapes detected** (each requires a keyword-spelled literal — `:wat::…`, or the bare
`wat::…` that `TypeExpr::Parametric` stores by documented convention):

| shape | count in the ledger |
|---|---|
| `x == LIT` / `x != LIT` | 172 |
| `match x { LIT => … }` (the MATCH, once, not per arm) | 28 |
| `x.starts_with/ends_with/contains/strip_prefix(LIT)` | 23 |
| `matches!(x, LIT \| …)` | 6 |
| `KEYWORD_CONST_LIST.contains(&x)` | 3 |

### 2.2 The two named rules that carry the discrimination

⭐ **THE DUAL-ARM RULE.** A `match` over a `WatAST` with BOTH a `Keyword` arm and a `Symbol` arm
covers both spellings: **the `Keyword` payload IS the internal identity** (keyword-spelled by
construction — `canonical_identity` returns it unchanged), and the `Symbol` payload is the other
spelling and needs a door. If every `Symbol` arm routes through one, the expression yields a
canonical identity. This is what makes `declare::parse::head_fqdn` a DOOR and `form_match::
identity_text` NOT one — exactly what their own doc comments say, derived rather than asserted.

⭐ **THE COVERAGE RULE** (`KwAst ∨ Door = Door`). The same fact for a value that reaches the decision
through a candidate LIST rather than a match value. 255.10's cure in `macros/eval.rs::
validate_pure_total` builds `head_candidates` by pushing the raw keyword in one arm and
`canonical_identity(id.as_str())` in the other; without this rule that cured site read as a shape-A
heretic. ⚠ **Its failure mode, stated:** a door output and an **unrelated** keyword-only read joined
into one local would read as covered. `SymAst` and `BothAst` are untouched by it.

### 2.3 ⛔ WHAT IT CANNOT SEE

1. **REACHABILITY.** It cannot say whether a symbol spelling can actually ARRIVE at a site. That is
   the pipeline-order argument 255.9/255.10/255.11 made **by hand, per site**, and nothing here
   reproduces it. **The ledger is a COUNTDOWN, not a bug list**: shape-A sites that 255.9 classified
   UNREACHABLE (post-normalize code positions) or LOUD are still counted, because the terminal cut
   retires them regardless of today's reachability.
2. **A keyword-only READ that feeds a COVERED decision.** The unit is the DECISION, so
   `closure_extract.rs:962`'s `match quote_boundary(k)` — a raw `Keyword` payload handed to a callee
   whose *other* callers normalize — is invisible here, and `quote_boundary` itself reads `cured`.
   255.9's hand census of the 82 `Some(WatAST::Keyword` READ sites is the instrument for that half,
   and this lint does not replace it.
3. **The 68 `D-unresolved` sites** — a decision on a `&str` no call site resolved. Counted and
   printed, never folded into the ledger, never claimed clean. The honest reading is "no evidence
   either way", not "fine".
4. **Macro-generated code.** `#[wat_intrinsic]` / `defservice` expansions are not in the token stream
   `syn` reads (`NOTE-the-span-lints-cannot-see-generated-code.md`).
5. **Name-keyed call sites.** Pass C matches callees by function NAME, so a collision makes a
   parameter MORE convicted (a raw argument anywhere poisons the join) — the safe direction, but it
   is an approximation, not resolution.
6. **Anything outside `src/`** — not `tests/`, not `crates/`, not the `.wat` corpus. The corpus
   number (103 213 keyword heads in 2 208 `.wat`) is the brief's and I did not re-derive it.
7. **A comparison with no literal on either side.** `register_validated`'s cured `type_defs_same(e,
   &def)` and `value_matches_type_by_name`'s cured `denoted_type_path(p) == denoted_type_path(t)`
   are invisible *because the cure removed the literal*. The lint therefore cannot prove those two
   cures — it can only fail to convict them (§3, rows marked ∅).

---

## 3. ⭐⭐ THE CALIBRATION — THE GATE WITH TEETH

`the_discriminator_separates_the_cured_from_the_open` is a test, not a paragraph. It is keyed per
**DECISION** — `(file, enclosing fn, the literal compared against)` — never per fn, because
`is_type_equatable` and `validate_pure_total` each hold a cured decision AND a separate raw one, and
a per-fn key would have hidden each behind the other (§4.2).

### 3.1 Cured — **MUST NOT be flagged.** All 13 rows pass.

| site | cured by | verdict |
|---|---|---|
| `load/loader.rs::match_load_form` | 255.9 | ✅ `cured` — `match canonical_identity_of(head)?.as_str()` |
| `load/loader.rs::scan_for_setter` | 255.9 | ✅ `cured` — `.and_then(canonical_identity_of)` then `starts_with` |
| `macros/eval.rs::validate_pure_total` (the head dispatch) | 255.10 | ✅ `cured` — via THE COVERAGE RULE |
| `macros/eval.rs::refuse_expand_only_in_program` | 255.10/255.11 | ✅ `cured` — reads `head_fqdn` |
| `edn/render.rs::edn_to_typed_value_inner` | 255.12 | ✅ `cured` — table matched against `denoted_type_path` |
| `types.rs::from_root_keyword` / `from_marker_keyword` | 255.1 | ✅ `cured` — matched against `canonical_identity` |
| `types.rs::classify_type_decl` / `splice_type_decls` | 255.x | ✅ `cured` — head read through `head_fqdn` |
| `types.rs::is_subtype`, `parse_type_form`, `parse_type_inner` | 255.8/255.12 | ✅ `cured` — through the denotation door |
| `check.rs::is_type_equatable` / `is_type_orderable` (the tables) | 255.12 | ✅ `cured` — matched against `denoted` |
| ⛔ `check.rs::walk_for_restricted_call` | 255.11 | ✅ **no site at all** — the capability wall holds no keyword literal comparison; the calibration row asserts it produces *zero* sites |
| ∅ `types.rs::register_validated` | 255.12 | not a site — the cure replaced `==` with `type_defs_same` |
| ∅ `function/subsume.rs::value_matches_type_by_name` (the Path arm) | 255.12 | not a site — both sides go through `denoted_type_path` |

### 3.2 Open — **MUST be flagged.** All 4 rows pass.

| site | why | verdict |
|---|---|---|
| ⭐ `rete/purity.rs::intrinsic_meta` | the purity/determinism table, keyword-keyed, fed a raw dual-spelling head — **251.8d-ii's defect** | ✅ **3 sites, shape B** (asserted as B, not merely present) |
| `rete/purity.rs::effectful_by_prefix` | the effect-namespace prefix tests, same raw head | ✅ **8 sites, shape B** |
| `rete/purity.rs::classify_expr` | the `quote`/`quasiquote`/`holon::literal` data guard, keyword-only | ✅ 3 sites, shape A |
| `function/subsume.rs::value_matches_type_by_name` | the `Value::Aggregate` arm | ✅ 1 site, shape E — §4.1 |

**The two groups separate cleanly. Nothing in the cured set is convicted; nothing in the open set is
missed.**

---

## 4. ⭐ THREE THINGS THE LEDGER FOUND THAT NO STONE HAD NAMED

### 4.1 It independently re-derived 255.12's own declared blind spot

255.12 §6.4 wrote: *"The `Value::Aggregate` arm of `value_matches_type_by_name` was left raw
(`bare_p == a.class.as_ref()`). A record class cannot live under `:wat::type::` … so I judged it out
of reach — **a reading, not a probe.**"* The discriminator flags
`function/subsume.rs:123` — `bare_p == "wat::core::Record"`, shape E — with no knowledge of that
note. An instrument that rediscovers a hand-declared limit is an instrument that is looking at the
right thing.

### 4.2 `check.rs::is_type_equatable` cured its `Path` table and left two residues beside it

255.12 cured the 15-arm equatable table (`match denoted.as_str()`, line 13625) — measured `cured`
here. **Two decisions in the same function did not get the door:**

- line **13615**, ten lines ABOVE the cured table: `p == ":wat::core::Value"`, raw;
- line **13658**, the `TypeExpr::Parametric` arm: `match head.as_str() { "wat::core::Vector" => … }`
  — a parametric head compared without `parametric_heads_unify`, which is the door 255.12 itself
  built for exactly this (`types.rs:132`). `is_type_orderable` (line 13752) has the identical
  parametric residue.

The cure and the residues sit inside one function, which is precisely why the calibration is keyed
per DECISION and not per function — a per-fn key would have let either fact stand for the other.

### 4.3 Two shape-B sites outside `rete/purity.rs`, named by nobody

`rete/kernel/arm.rs::compile_acc_fold` (line 281) and `::compile_user_fold_programs` (line 429) both
read

```rust
let head = match items.first() {
    Some(WatAST::Keyword(k, _)) => k.as_str(),
    Some(WatAST::Symbol(s, _)) => s.as_str(),
    _ => continue,
};
```

and then `match head { ":wat::rete::acc::count" => … }` / `head.starts_with(":wat::rete::acc::")`.
Same shape as `classify_expr`'s, in the accumulator lowering. 255.9's census listed
`rete/kernel/arm.rs:251/267/425` as *"no (already both) — fine"*: reading both spellings is only half
the cure; **the other half is the door**, and these two do not take it. ⛔ **Reported, not cured.**

---

## 5. THE LEDGER — **232**

### 5.1 By shape

| shape | n | what it means |
|---|---|---|
| **A — keyword-only** | **93** | the dispatch reads ONLY a `WatAST::Keyword` payload; a symbol-spelled name is invisible to it. Correct VALUE, blind dispatch (255.9's class) |
| **B — dual-raw** | **14** | the dispatch reads BOTH payloads and keys on the keyword spelling anyway — it SEES the symbol and mis-keys it. ⛔ **Strictly worse than A**; 251.8d-ii's class |
| **C — symbol-raw** | **0** | a symbol payload compared to a keyword literal, alone. ⭐ **None in the crate** |
| **E — type-path raw** | **125** | a `TypeExpr::Path`/`Parametric` payload compared without the denotation door — 255.12's class: `wat.type/i64` and `:wat::core::i64` are one type and two strings |

Not in the ledger, reported beside it: **61 cured** · **68 D-unresolved** · **3 allowlisted**.
Total decision sites **364**.

### 5.2 By file (all 29)

| n | file | dominant shape |
|---|---|---|
| **115** | `src/check.rs` | E 78 · A 37 |
| 22 | `src/runtime.rs` | A 17 · E 5 |
| 19 | `src/collection/infer.rs` | E 19 |
| ⭐ **17** | `src/rete/purity.rs` | **B 12** · A 5 |
| 10 | `src/collection/seq_container.rs` | E 10 |
| 4 | `src/rete/kernel/stratify.rs`, `src/macros/eval.rs`, `src/closure_extract.rs` | |
| 3 | `src/resolve/boundary.rs`, `src/macros/parse.rs`, `src/macros/expand.rs`, `src/declare/parse.rs` | |
| 2 | `src/rete/kernel/arm.rs` (**B 2**), `src/resolve/walk.rs`, `src/lower.rs`, `src/load/loader.rs`, `src/host/test_runner.rs`, `src/holon/ast.rs`, `src/freeze.rs`, `src/collection/map_container.rs` | |
| 1 | `src/types/defstruct.rs`, `src/rete/expr_ir/mod.rs`, `src/rete/collect.rs`, `src/resolve/normalize.rs`, `src/match_arm.rs`, `src/intrinsic/holon/atom.rs`, `src/function/subsume.rs`, `src/function/eval.rs`, `src/edn/render.rs` | |

### 5.3 ⭐ Three readings the decomposition forces

1. **The migration is ~54% a TYPE-PATH question, not a call-head question.** 125 of 232 are shape E.
   The builder's ruling is about call heads; the ledger says the larger population is 255.12's
   denotation class. Whoever schedules the terminal cut should know the number does not fall to zero
   by converting heads.
2. **`check.rs` is half the ledger** (115 of 232) and it is the stage-8 file 255.9 classified
   *"UNREACHABLE in code position (post-normalize); LOUD inside `match`-arm/quoted data"*. Those
   dispositions are still the right ones; the ledger counts them anyway, because the cut retires the
   shape, not the reachability.
3. **The DANGEROUS class is tiny and concentrated.** 14 shape-B sites in the whole crate, 12 of them
   in one file, and that file is already the next stone's target. **Shape B is the number to watch;
   shape A+E is the number to schedule.**

---

## 6. ⭐ THE LINT HAS FAILED ONCE — THE CONSTRUCTION AND ITS OUTPUT

`src/form_match.rs`, a new function of the exact convicted shape (a head read in both spellings,
compared raw against a keyword literal):

```rust
pub(crate) fn probe_255_13_negative_control(node: &WatAST) -> bool {
    let head = match node {
        WatAST::Keyword(k, _) => k.as_str(),
        WatAST::Symbol(id, _) => id.as_str(),
        _ => return false,
    };
    head == ":wat::core::defn"
}
```

`cargo nextest run -E 'test(the_heresy_ledger_matches)'` — **RED**, verbatim head of the panic:

```
thread 'keyword_heresy_ledger::the_heresy_ledger_matches_its_frozen_census' panicked at
/home/john/work/holon/wat-rs/tests/lint/keyword_heresy_ledger.rs:1208:5:

🔥🔥🔥 NEW KEYWORD HERESY — the dual-support scaffold GREW.
The ledger was 232; it is now 233.

A decision made by comparing a name that can arrive in EITHER spelling against a
keyword-spelled literal, without passing it through the identity door first, is the
defect class arc 255 has convicted eight times (255.9 · 255.10 · 255.11 · 255.12 ·
251.8d-ii). THE FIX IS A DOOR, NEVER A SECOND ARM:

· a form HEAD              → `form_match::canonical_identity_of` / `declare::parse::head_fqdn`
· a bare name string       → `edn::render::canonical_identity`
· a TYPE path              → `types::denoted_type_path` (NOT raw `type_denotation` —
                             it collapses `:wat::type::Infer`, see 255.12 §4)
· a parametric head        → `types::parametric_heads_unify`
· two whole TypeDefs       → `types::type_defs_same`

If the site genuinely cannot arrive in the other spelling, say so ON IT:
`// rune:lint(keyword-heresy) — <reason>`, on the offending line or the one above.

NEW OFFENDING FUNCTIONS:
  + src/form_match.rs  fn probe_255_13_negative_control  1 site(s) [Bx1]
GREW:
  (none)

Every offense, with its operand and the literal it was compared against:
src/check.rs:527  fn is_fn_form_expr  [eq-literal / A-keyword-only]  operand `k`  vs :wat::core::fn
… (232 more)
```

⭐ **It named the function, the count AND the shape (`Bx1` — dual-raw), not just a number.** The
mutation was reverted; `git diff --stat HEAD -- src/` is empty and all four tests are green again.

⭐ **And the failure is permanent, not a one-off.** `the_discriminator_convicts_a_synthetic_heretic_and_clears_its_cure`
runs the analyzer over six synthetic sources and asserts every verdict:

| synthetic source | asserted verdict |
|---|---|
| keyword-only head dispatch | `A-keyword-only` |
| both spellings read raw, then compared | `B-dual-raw` |
| symbol payload vs keyword literal | `C-symbol-raw` |
| `TypeExpr::Path(p) => p == ":wat::core::i64"` | `E-typepath-raw` |
| the dual read with `canonical_identity` on the Symbol arm | ⭐ `cured` |
| the type path through `denoted_type_path` — **with the door chain inline**, so the fixpoint must DERIVE the door two hops from the seed | ⭐ `cured` |
| `const OP: &str = ":wat::core::map"` used in a `format!` message | ⭐ **no site at all** — the negative control for the instrument, so the ledger can never drift back into the orchestrator's 7 233 |

---

## 7. THE ALLOWLIST — frozen, two entries, each with its reason

Keyed by **file + enclosing fn, never by line number**. May only shrink.

| site | reason |
|---|---|
| `src/types.rs::parse_type_inner` | **Not a name dispatch.** `s.starts_with("wat::core::Fn(")` — the open paren is part of the literal, so it tests a RENDERED SURFACE, not an identity. No spelling of a name can reach it. The fn's actual identity decisions (`denoted`) are measured separately and read `cured`. |
| `src/edn/render.rs::type_expr_to_clojure_form` | **A renderer, and the one direction where the keyword spelling is the INPUT by contract** — this fn exists to turn the internal `:wat::…`/`wat::…` identity into the faithful-Clojure surface, so stripping the internal prefix is its whole job. It decides nothing about a name that arrived from source. Its forward partner `wat_keyword_to_clojure_symbol` is the same class. |

A third escape exists and is unused: a per-offense `// rune:lint(keyword-heresy) — <reason>` on the
offending line or the one above, mirroring `no_inlined_edn`'s per-offense marker.

⚠ **A deviation from the brief, stated.** The brief asks for *"one entry per site, each with a
one-line reason"*. The **allowlist** is that, two entries. The **LEDGER** is 232 sites in 111
functions, and it is not an exemption list — it is the countdown. Freezing it per `(file, fn, count,
shape-mix)` is what makes `+1 new / −1 fixed` unrepresentable (`[[a gate freezes names never a
count]]`); each row's **shape tag IS its reason for being in the ledger**, and the four shape
definitions are written once, above the table. 111 hand-written prose lines would have been 111
restatements of four sentences.

---

## 8. THE GATES — evidence, not claims

### 8.1 Floor

#### Run 1 — `.floor/2026-09-22T20-29-33Z/` — ⛔ **RED**

```
     Summary [ 331.283s] 5993 tests run: 5990 passed (10 slow), 3 failed, 22 skipped
```

⛔ **Three tests went red, and all three are MINE** — the new lint file violates three EXISTING
lints. Not one is in `src/`; not one is a flake; none was re-run to clear. `scripts/floor.sh` had
already written `ARM.txt`, and here is each failing test's WHOLE stdout+stderr block, verbatim:

```
        FAIL [   0.142s] ( 145/5993) wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat
  stdout ───

    running 1 test
    test no_inlined_wat_in_tests::tests_carry_no_inlined_wat ... FAILED

    failures:

    failures:
        no_inlined_wat_in_tests::tests_carry_no_inlined_wat

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 363 filtered out; finished in 0.12s
    
  stderr ───

    thread 'no_inlined_wat_in_tests::tests_carry_no_inlined_wat' (1822597) panicked at /home/john/work/holon/wat-rs/tests/lint/no_inlined_wat_in_tests.rs:440:5:


    🔥🔥🔥 INLINED-WAT IN TESTS — 1 file(s) still carry a string literal that wat's own
    reader parses as a form (surface-agnostic: rust-scheme `(:wat::core::…)` AND faithful
    Clojure `(wat.core/…)` both count).

    THE FIX — move the wat into a co-located `.wat` fixture and drive it lint-clean via ONE of
    two idioms. RUBRIC (which to reach for): docs/CONVENTIONS.md § 'Test idioms — EDN-over-stdio
    vs just-eval'. In short:
    • just-eval      — `call_beside_value(file!(), ":user::compute")`: run a fixture's named entry
    fn in-process, inspect its typed Result<Value, RuntimeError>. For a
    VALUE/TYPE claim (a fn's return; a compile-time/freeze property, which
    often needs only `startup_beside(file!())`, no call).
    • EDN-over-stdio — `run-hermetic` runs `:user::main` as a real process; it `println`s its
    result as EDN and the test `edn::read`s it back (lossless round-trip).
    For a PROGRAM claim (a crash/exit + reason, stdio effects, IPC fidelity,
    cross-loci behavior).
    One-line: 'the PROGRAM does X' -> EDN-over-stdio ; 'this VALUE/TYPE is X' -> just-eval.
    A legitimately-inline case (e.g. a parser/reader test) earns a per-site
    `// rune:lint(no-inlined-wat) — <reason>` (the reason must earn it).

    Drive it to ZERO. Literal-hit breakdown so far: 1 format!-driver, 4 faithful-surface,
    0 other parse-body. Offenders:

    tests/lint/keyword_heresy_ledger.rs

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

        FAIL [   0.152s] ( 149/5993) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
  stdout ───

    running 1 test
    test no_loose_string_assert::tests_carry_no_loose_string_assert ... FAILED

    failures:

    failures:
        no_loose_string_assert::tests_carry_no_loose_string_assert

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 363 filtered out; finished in 0.14s
    
  stderr ───

    thread 'no_loose_string_assert::tests_carry_no_loose_string_assert' (1822603) panicked at /home/john/work/holon/wat-rs/tests/lint/no_loose_string_assert.rs:135:5:


    🔥🔥🔥 LOOSE STRING ASSERTIONS — 1 site(s) assert a value with contains/starts_with/
    ends_with where an exact `assert_eq!` belongs. A loose check passes on reordered fields,
    malformed maps, and appended garbage.

    THE FIX (RUBRIC: docs/CONVENTIONS.md § 'Test idioms' -> 'The .edn golden'): a deterministic
    STRUCTURED value goes in a co-located `<probe>__<label>.edn` golden, compared via
    `wat::assert_edn_eq!(actual, include_str!("...edn"))` (parses both sides, structure-exact) —
    capture the whole value, never guess. A scalar -> byte-identical `assert_eq!`. EXEMPT a
    legitimately-loose one (a value that varies per run: path/pid/hash/timestamp, or a targeted
    absence over a large output) with a per-site `// rune:lint(loose-assert) — <reason>`.

    Drive it to ZERO. Offenders:

    tests/lint/keyword_heresy_ledger.rs:951

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

        FAIL [   0.258s] ( 220/5993) wat::lint one_variant_separator::only_identifier_rs_spells_the_variant_separator
  stdout ───

    running 1 test
    test one_variant_separator::only_identifier_rs_spells_the_variant_separator ... FAILED

    failures:

    failures:
        one_variant_separator::only_identifier_rs_spells_the_variant_separator

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 363 filtered out; finished in 0.24s
    
  stderr ───

    thread 'one_variant_separator::only_identifier_rs_spells_the_variant_separator' (1823038) panicked at /home/john/work/holon/wat-rs/tests/lint/one_variant_separator.rs:260:5:


    🔥🔥🔥 A SECOND VARIANT SEPARATOR — 7 site(s) spell the `::` between an enum and 
    its variant OUTSIDE `crates/wat-reader/src/identifier.rs`.

    A variant's fully-qualified name is composed and decomposed in exactly ONE place, or two 
    spellings WILL disagree — nine of these hid for months inside `rsplit_once("::")`, a 
    shape `one_name_grammar.rs` bans for `'/'` and does not name for `"::"`.

    THE FIX — route through the pair:

      compose_variant(enum_path, variant)   -> `{enum}.{variant}`
      decompose_variant(name) -> Option<(&str, &str)>   the exact inverse

    If this site does NOT separate an enum from its variant, add a co-located
    `// rune:lint(one-variant-separator, <category>) — <reason>` on the line or the one
    above, with <category> one of: namespace | type-path | display | edn | not-a-name.
    ⛔ `variant` is NOT a category — a variant site routes through the door.

    Offenders:

    tests/lint/keyword_heresy_ledger.rs:156  [ACCESSOR]  for e in es.flatten() { let p = e.path();
    tests/lint/keyword_heresy_ledger.rs:371  [DATA]  Expr::Path(p) => write!(f, "{}", p.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::")),
    tests/lint/keyword_heresy_ledger.rs:386  [ACCESSOR]  ident == "tests" || attrs.iter().any(|a| a.path().is_ident("cfg") && a.to_token_stream_str().contains("test"))
    tests/lint/keyword_heresy_ledger.rs:394  [ACCESSOR]  if f.attrs.iter().any(|a| a.path().is_ident("test")) { return; }
    tests/lint/keyword_heresy_ledger.rs:398  [ACCESSOR]  if f.attrs.iter().any(|a| a.path().is_ident("test")) { return; }
    tests/lint/keyword_heresy_ledger.rs:519  [ACCESSOR]  if f.attrs.iter().any(|a| a.path().is_ident("test")) { return; }
    tests/lint/keyword_heresy_ledger.rs:523  [ACCESSOR]  if f.attrs.iter().any(|a| a.path().is_ident("test")) { return; }

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**The three arms, each named, each cured — by the exemption form the lint itself prints, never by
weakening the lint:**

| arm | mechanism | cure |
|---|---|---|
| `no_inlined_wat_in_tests::tests_carry_no_inlined_wat` — *"1 file(s) … 1 format!-driver, 4 faithful-surface"* | **The subject of my lint IS wat names written as Rust string literals.** The six synthetic Rust sources in the negative control must each contain a real comparison against `":wat::core::…"`, and the failure message quotes `wat.core/…` in its FIX rubric | one file-wide `// rune:lint(no-inlined-wat)` with the reason spelled out: these literals are DATA for a text analysis, there is no wat runtime in the file, and moving them out would delete the only thing being tested |
| `no_loose_string_assert::tests_carry_no_loose_string_assert` — 1 site, `keyword_heresy_ledger.rs:951` | `mix.contains('B')` in the calibration test | per-site `// rune:lint(loose-assert)`: a targeted PRESENCE check over a shape-MIX summary, deliberately independent of the count — the exact mix is already pinned byte-for-byte by `FROZEN_LEDGER` |
| `one_variant_separator::only_identifier_rs_spells_the_variant_separator` — **7 sites** | ⭐ **6 of the 7 are `syn`'s own `Attribute::path()` and `std`'s `DirEntry::path()`** — an attribute's Rust path and a FILESYSTEM path. The 7th is `.join("::")` rendering a RUST module path into a diagnostic | 7 co-located `// rune:lint(one-variant-separator, …)` runes: `not-a-name` ×6, `namespace` ×1 |

⚠ **Worth recording as a finding about the LINT SUITE, not about this stone:** `one_variant_separator`
classified `e.path()` and `a.path().is_ident("test")` as `[ACCESSOR]` hits. They contain no `::`
between an enum and a variant and no `::` at all — they matched on the METHOD NAME `path`. That is
the same class the injected `CLAUDE.md` names (*"a string comparison with one side normalized and the
other not… a `format!`/`split`/`==` on names is the culprit"*), one level up: a census keyed on a
name rather than on the act (`[[feedback_a_census_predicate_can_name_the_wrong_act]]`). Six runes is
the cost here and it is cheap; a file that used `syn` more heavily would pay more. **Reported, not
cured — it is not this stone's gate.**

#### Run 2 — the fixed tree — GREEN

```
[floor] doctests exit=0
     Summary [ 338.644s] 5993 tests run: 5992 passed (13 slow), 1 failed, 22 skipped
        FAIL [   0.112s] ( 153/5993) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
```

⛔ **NOT green — one left.** Two of the three cures held; the third had a SECOND offender the first
message did not show, because `no_loose_string_assert` reports only the first site per statement
scan: `keyword_heresy_ledger.rs:968`, `!c.doors.contains("identity_text")` — `BTreeSet` membership
on an exact key, not a string match at all. Runed with that reason. ⚠ **Worth naming as a process
fact: fixing all three arms at once did not make the floor green in one step, because one lint's
failure text named one offender and there were two.** A count, again, could not name what it had
not reached.

#### Run 3 — the fixed tree — ⭐ **GREEN, 5993 / 5993**

```
[floor] doctests exit=0
     Summary [ 328.492s] 5993 tests run: 5993 passed (9 slow), 22 skipped
```

⚠ `rete::kernel::tests::harvest_cost::harvest_wrap_split` — the one diagnosed-unsound test
(`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`) — **PASSED** (`0.195s`, run 3). Naming it
because the brief asks to cite it if it fires; it did not.

⭐ **5989 → 5993 is +4, exactly this stone's four tests**, all four green:

```
PASS (  84/5993) wat::lint keyword_heresy_ledger::the_discriminator_convicts_a_synthetic_heretic_and_clears_its_cure
PASS ( 241/5993) wat::lint keyword_heresy_ledger::the_discriminator_reaches_the_tree_and_its_passes_converge
PASS ( 248/5993) wat::lint keyword_heresy_ledger::the_discriminator_separates_the_cured_from_the_open
PASS ( 249/5993) wat::lint keyword_heresy_ledger::the_heresy_ledger_matches_its_frozen_census
```


### 8.2 Clippy

```
$ cargo clippy --all-targets --workspace -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.78s
clippy exit=0
```

⚠ The new file was NOT clippy-clean on first write — five findings (`collapsible_match`,
`type_complexity`, two `dead_code`, `collapsible_else_if`), all fixed, none suppressed with an
`#[allow]` on a real one.

### 8.3 Census

⚠ `[[DIFF LIKE AGAINST LIKE]]` — and the obvious baseline **was not comparable**. The newest census
on disk, `.census/2026-09-22T10-25-28Z.txt`, was taken at 03:26 local; the last `src/` commit,
`ca2ef1311`, landed at **03:31** — *five minutes after it*. Diffing against it would have charged
this stone with 8d-ii's checker change. So the baseline was **re-taken from the stashed HEAD tree**:

```
$ git stash push -u -- Cargo.toml tests/lint/keyword_heresy_ledger.rs docs/…/SCORE-…md
$ cargo build --release && ./scripts/replay/census.sh
  .census/2026-09-22T20-50-39Z.txt  files=2209          ← BASELINE, built from HEAD
$ git stash pop
$ cargo build --release && ./scripts/replay/census.sh
  .census/2026-09-22T20-51-56Z.txt  files=2209          ← this stone

$ ./scripts/replay/census.sh --diff .census/2026-09-22T20-50-39Z.txt .census/2026-09-22T20-51-56Z.txt
census-diff: no STOP-8                                                            (rc 0)

$ join <(sort BASE) <(sort CURR) | awk '$2!=$3' | wc -l
0                                        ← ZERO rc changes, in EITHER direction, across all 2209
$ comm -3 <(cut -d' ' -f1 BASE|sort) <(cut -d' ' -f1 CURR|sort) | wc -l
0                                        ← identical path set (no `.wat` added or removed)
nonzero: 213 → 213
```

⚠ **And the honest reading of that zero: it is a TAUTOLOGY, not a result.** `census.sh` runs
`git ls-files '*.wat'` through `target/release/wat --check`, and this stone touches neither the
`.wat` corpus nor `src/`, so the two binaries differ only in a `[dev-dependencies]` entry the `wat`
binary never links. **The census proves the stone is inert; it proves nothing about the ledger.**
Stating that rather than presenting 2209 unchanged files as evidence for a discriminator.


### 8.4 Delta

`scripts/replay/delta.sh`, the committed 179-path sample
(`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`, sha256
`33ede76cbb9b8809ecebc7656db3d4733769df54c2641b7e6d798773f1cbc3e5`, `paths=179 missing=0`), both
sides checked from copies:

```
  ORIG-CLEAN  160/179
  CONV-CLEAN  157/179
  NEW         3   (orig clean -> converted broken)
  RECOVERY    0   (orig broken -> converted clean)

NEW files:
  wat-scripts/probes/arc-170/probe-c1-clean-surface.wat
  wat-scripts/probes/arc-170/probe-kwargs-peer.wat
  wat/holon/Ngram.wat
[delta] exit=0. RECOVERY 0.
```

**Gate ✅ delta 3 = the brief's baseline 3. ⭐ RECOVERY 0 — no checker was disarmed.** The three are
the same three 255.12 reported and diagnosed: `wat/holon/Ngram.wat` (255.12 §2 — REPORTED as 8d-ii's,
a mixed-dialect artifact of a baked-in stdlib copy) and the two arc-170 probes (a type-parameter
substitution question, a different class). ⚠ Like the census, this is a **null result by
construction** — nothing in `src/` moved. It is the gate the brief asks for and it is satisfied; it
is not evidence for the ledger.


### 8.5 ⚠ The docs-reading lints, re-run AFTER this SCORE landed

Several `tests/lint/` gates read `docs/`, so a green taken before this file existed would have been
a green about a different tree:

```
$ cargo nextest run --release -E 'binary(lint)'      # with SCORE-STONE-255.13 on disk
     Summary [ 249.885s] 364 tests run: 364 passed (1 slow), 0 skipped
```

### 8.6 The lint's own runtime

```
$ cargo nextest run --release -E 'test(keyword_heresy)'
    4 tests run: 4 passed        Summary [ 3.86s ]
```

~3.9 s for four tests that between them parse 333 files twice and run two fixpoints. It is a floor
citizen, not a nightly job.

---

## 9. ⛔ WHAT MY GREEN CANNOT SEE

1. **Every limit in §2.3**, and the first one is the big one: **this instrument does not know
   whether a symbol spelling can reach a site.** 232 is a count of SHAPES, and the stone that
   schedules the terminal cut must not read it as 232 live defects. 251.8d-ii proved exactly one of
   them fires today.
2. **The 68 `D-unresolved` sites are not a clean bill.** They are a measured absence of evidence. If
   a future pass resolves them, the ledger can go UP without anything changing in `src/` — and the
   ratchet will read that as a regression. The next hand to widen the analyzer must re-freeze the
   table in the same commit and say so.
3. **The frozen table is keyed on the enclosing `fn` NAME.** A rename moves a row; a refactor that
   splits one fn into two appears as a `−` and a `+`. Both go red, which is the intended cost, but
   neither is a heresy.
4. **`syn` reads the token stream, so a `#[wat_intrinsic]` body's generated half is invisible** —
   the same blind spot the span lints have (`NOTE-the-span-lints-cannot-see-generated-code.md`).
5. **I did not re-derive the corpus numbers.** 103 213 keyword heads / 2 208 `.wat` / 8 447 embedded
   forms are the brief's, unchecked here. The terminal cut needs both halves and I measured one.
6. **The two shape-B sites in `rete/kernel/arm.rs` are reported from the CODE, not from a run.** I
   did not build a rete accumulator with a symbol-spelled `wat.rete.acc/count` head and watch it
   fall through. Saying so rather than implying a probe.
7. ⚠ **Two of my three floor reds were FALSE POSITIVES OF OTHER LINTS, not defects** (§8.1):
   `one_variant_separator` matched on the METHOD NAME `path` (`syn::Attribute::path()`,
   `DirEntry::path()`), and `no_loose_string_assert` matched `BTreeSet::contains` with a literal
   key. Six of the seven runes I wrote exist to excuse a census that named the wrong act. **I did
   not cure either lint** — it is not this stone's gate — but a future file that uses `syn` more
   heavily will pay the same toll, and the cure is the same one this stone's own discriminator
   applies: ask what the site DOES, not what it is called.
8. **`check.rs`'s 115 rows are unattributed by stage.** 255.9 did that work per site by hand; I
   inherited none of it. A future cure must re-derive reachability per row — the ledger tells you
   WHERE, never WHETHER.

---

## 10. RECOMMENDATIONS

1. ⭐ **8d-ii's fifth draw edits `src/rete/purity.rs`, not `src/collection/transform.rs`.** The cure
   is one door at `classify_expr`'s general-list head read (`canonical_identity_of`, or `head_fqdn`
   if the bare-symbol case must stay a non-head), which retires 12 of the 14 shape-B sites at once
   because `intrinsic_meta` and `effectful_by_prefix` are downstream of that one binding.
2. **`rete/kernel/arm.rs`'s two shape-B sites are the same one-line cure** and should ride along, or
   be their own stone. They are not covered by 8d-ii's brief.
3. **Split the countdown.** Shape B (14) is a defect list. Shape A (93) + E (125) is a migration
   schedule. The terminal cut needs A+E at zero; the next three stones should drive B at zero.
4. **Shape E deserves its own campaign** — 125 sites, 78 of them in `check.rs`, all one door
   (`denoted_type_path`). It is the largest single class in the ledger and 255.12 cured four of them.

---

## VERDICT — **LANDED**

The discriminator is built, derived rather than declared, and it separates the cured from the open
on a table of 17 rows this arc already judged. **The ledger reads 232** — 93 A · 14 B · 0 C · 125 E
— with 68 unresolved counted apart and 61 provably cured. The lint has failed once by construction
and carries a permanent negative control for each shape. **The brief's central calibration target is
corrected: there is no comparison at `transform.rs:304-340`, and the convicted site is
`rete/purity.rs`, reached by data flow.** ⛔ Nothing was cured. ⛔ Not one `.wat` was converted.
