# SCORE — STONE 251.8d-ii (SIXTH DRAW): the acc-fold head

Branch `main`, drawn against `28fc7e26d`. Parent:
`BRIEF-STONE-251.8d-ii-SIXTH-the-acc-fold-head.md`. **Not pushed.** Every number below was produced
this session on a binary I built; `cargo build --release` ran after **every** `src/` edit and each
row names which binary produced it.

## VERDICT — **LANDED (the cure). STOP (the conversion).**

⛔⛔ **THE BRIEF CONVICTED THE WRONG ADDRESS FOR THE THIRD DRAW RUNNING — and this time it said it
had read the code.** It sent this draw to `src/rete/kernel/arm.rs::compile_acc_fold` for the fifth
draw's **239** converted-floor `ArityMismatch` failures. ⭐ **Those failures cannot come from that
function.** They are `#wat.check/CheckErrors` — raised at **freeze** time by the type checker.
`compile_acc_fold` runs at **fire** time and raises `RuntimeErrorKind::MalformedForm`. Two different
passes, two different error types, and the brief's own quoted evidence carries the tell.

⭐ **THE REAL ADDRESS, MEASURED — by converting exactly ONE stdlib file and rebuilding:**

```
printf '["wat/rete/syntax.wat"]\n' | ./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat
cargo build --release
./target/release/wat --check <a 9-line file holding ONE accumulate rule>
  → #wat.check/ArityMismatch ":wat::rete::acc::count: expected 1 argument(s); got 0"
```

`wat/rete/syntax.wat` holds `defrule`, whose template is
`(:wat::rete::make-rule ~name-str (:wat::core::quote ~when-vec) (:wat::core::quote ~then-vec))`.
Converted, it emits **`(wat.core/quote …)`** — a `WatAST::Symbol` head. All four `make-rule`
descents tested that head with a **keyword-only** `matches!`, so the `:when` argument was neither
recognized as a quote nor re-spelled; a surviving `wat.core/quote` head then misses
`check.rs::infer_list`'s keyword-only arm, and **the quoted condition vector is type-checked as
CODE**. A zero-argument acc-form is an arity error the instant it is read as a call.

⭐ **ONE DOOR, SIX SITES, AND THE PROBE THAT FAILS WITHOUT IT NEEDS NO CONVERSION** — the shape is
writeable by hand on an unconverted tree:

```
  PRE-CURE   4 × ":wat::rete::acc::count: expected 1 argument(s); got 0"
  POST-CURE  8 / 8 rows, 1 1 1 1 1 1 0 0
```

⭐ **THE LEDGER: 229 → 223, six rows gone entirely, and the shape mix is the real number.**
A **101 → 97** · B **3 → 1** · C 0 · E **125 → 125**. ⛔ **Crate-wide shape B is now ONE site**, and
the calibration was **re-anchored onto it**, not deleted.

⭐ **AND THE 283 MOVED: the converted floor is 283 → 161.** `comm` on the two sorted failing-test
sets: **127 fixed, 5 NEW** — ⛔ **not a strict subset, and the 5 are the finding.** All five carry
one arm, and a two-binary experiment on the converted tree isolates them to **one of the six cured
sites**: `expand_make_rule_then`, whose cure **re-arms a `:then` macro expansion the conversion had
silently switched OFF** (§7.3). ⛔ **I kept the cure** — a disarmed wall is worse than a red, and the
landable floor is green with it. **161 is still RED, so the conversion does NOT land**; `wat/`
restored (`git checkout -- wat/` + rebuild — the sixth proof of that recovery).

⭐ **The class the brief was drawn for is GONE, not reduced:** `:wat::rete::acc::{count,sum,max,min,all}:
expected N argument(s)` was **198 occurrences** in the fifth draw's converted log and is **0** in mine.

Landable floor **6008 / 6008 GREEN**, 327.9 s · clippy **0** (not cached) · census **`no STOP-8`**,
213 = 213 · delta **NEW 3 = baseline 3, RECOVERY 0**. ⛔ **The FIRST floor went RED on two lint
gates — both MINE, both at one line — captured verbatim in §6.1 and cured.**

⛔ **FIVE CORRECTIONS TO THE BRIEF (§8), and the first is the twenty-fourth in the series.**

---

## 1. ⭐ THE ADDRESS — what the brief claimed, and what the code does

### 1.1 The brief's mechanism, quoted and refuted

> ⭐ **The source is a ZERO-ARG call in BOTH spellings** — `(:wat::rete::acc::count)` →
> `(wat.rete.acc/count)`. **The missing argument is INJECTED by the builtin arm**, and the
> converted head misses that arm, falls to `_`, and is compiled as a **user fold**.

⛔ **Every clause of that is wrong, and they fail together.**

| the brief says | measured |
|---|---|
| the converted source spells the acc head `(wat.rete.acc/count)` | ⛔ **NO.** The failing fixtures (`ACCUM_AXIS_WORLD`, `src/rete/kernel/tests/mod.rs:94`) are **Rust string constants** — the codemod never touches them. Their acc head is `(:wat::rete::acc::count)`, keyword-spelled, on both sides of the conversion. The error's own `:callee` says so: `":wat::rete::acc::count"` |
| "the missing argument is INJECTED by the builtin arm" | ⛔ **Nothing injects an argument.** `wat/rete/acc.wat:31` declares `:wat::rete::acc::count` as a 1-arg fn (`els`). A rule's acc-form legally passes **zero** because the whole `:when` vector is **quoted DATA** and is never type-checked as a call at all |
| the head "falls to `_` and is compiled as a user fold" | ⛔ That is `compile_acc_fold`'s **fire-time** behaviour and it produces `MalformedForm`, not `ArityMismatch`. The 239 are freeze-time check errors |

⭐ **The actual defect is that the `:when` vector STOPPED BEING DATA.**

### 1.2 The chain, each link driven

```
wat/rete/syntax.wat:222   (wat.core/quote ~when-vec)        ← converted defrule template
        ↓ macro expansion (freeze step 4)
resolve/normalize.rs:537  normalize_make_rule_when
        matches!(qitems.first(), Some(WatAST::Keyword(h,_)) if h == ":wat::core::quote")
        ⛔ FALSE for a Symbol head → the argument is returned UNTOUCHED, head not re-spelled
        ↓ (freeze step 7)
check.rs:2706             infer_list:  `if let WatAST::Keyword(k, head_span) = head`
        ⛔ a Symbol head never reaches the `":wat::core::quote"` arm (`check.rs:3506`)
        ↓
the `:when` vector is walked as CODE → `(:wat::rete::acc::count)` is an arity error
```

⛔ **`resolve/boundary.rs` is NOT the broken link.** `normalize_form` already reads a list head
through `crate::declare::parse::head_fqdn` and re-spells a symbol boundary head
(`rewrite_boundary_head`) — with a comment that states this exact hazard:

> *"`head_fqdn` classifies a symbol head (`wat.core/match`) as the boundary, but the checker
> matches the keyword node. Leaving the symbol in place made every match arm infer as a value
> vector."*

The `make-rule` descents **intercept the argument before `normalize_form` ever sees it**, so the
generic boundary path never runs on it. The door existed one frame up; four sites did not take it.

### 1.3 ⚠ AND `compile_acc_fold`'S SYMBOL ARM IS UNREACHABLE FROM THE WAT SURFACE TODAY

Driven, before writing a line of cure — a `defrule` with a symbol-spelled acc head, on the
**pre-cure** binary:

```
(?n :- (wat.rete.acc/count) :from (:sx::Reading (?g :- :g)))
  → #wat.kernel/AssertionFailure "compile-condition: accumulator expr is not total —
     ':wat::core::length' is not total"   at wat/rete/compile.wat:609
```

`wat/rete/compile.wat:578` — the **acc-form fence**, in wat — is
`is-builtin (:wat::string::starts-with? acc-hd-nm ":wat::rete::acc::")`, a **keyword-only prefix
test on `(:wat::core::ast-name acc-hd)`**. A symbol-spelled acc head is not "builtin", so the
four-axis fence runs on it, walks `acc::count`'s body and refuses on `:wat::core::length`. The head
never reaches `compile_acc_fold`.

⛔ **I cured `compile_acc_fold` anyway, and state plainly why it is not theatre:** the function
genuinely reads both payloads raw (255.13's shape B — the value is *not* already a keyword, which is
the ground on which the FIFTH draw correctly refused to "cure" `effectful_by_prefix`), the cure
changes its behaviour at its own boundary — driven, §3 — and it is required the moment 8d-iii cures
the wat fence. ⚠ **What it does NOT do is move the converted floor**, and §7 reports that.

---

## 2. THE CURE — one language fact, six sites, every Keyword arm byte-identical

### 2.1 The `make-rule` quote boundary (four descents, one fact)

`resolve/boundary.rs`'s own doc says these three descents *"cannot drift on which heads open the
code region"*. They had not drifted from **each other** — all four were keyword-only — they had
drifted from the door their own enclosing pass uses.

| site | before | after |
|---|---|---|
| `resolve/normalize.rs::normalize_make_rule_when` | `matches!(… Keyword(h,_) if h == ":wat::core::quote")` | `head_fqdn` + **`rewrite_boundary_head`** — recognized AND re-spelled |
| `resolve/normalize.rs::normalize_make_rule` (`:then`) | `out.extend(iter) // data, as-is` | `normalize_quote_head_only` — the MARKER is re-spelled, every argument byte-identical |
| `resolve/walk.rs::check_make_rule_when` | keyword-only | `head_fqdn` |
| `macros/expand.rs::expand_make_rule_when` / `expand_make_rule_then` | keyword-only | `canonical_identity_of` — the door `expand_form` **already uses** twelve lines above for the `make-rule` head itself |

⭐ **`head_fqdn` and `canonical_identity_of` are both members of 255.13's eight-function door set,
derived by fixpoint from its two seeds — not comparators I minted.** Both return a `Keyword`
payload unchanged, so the keyword side cannot move (255.13 §2.2, the DUAL-ARM RULE).

⚠ **The `:then` marker is a SEPARATE read, and whether it needed a separate cure is MEASURED in
§3.1a — not asserted here.** It is not covered by the `:when` cure (different code path, different
argument), and the pre-cure fixture failure does NOT name it (§3.1).

⛔ **`:wat::core::forms` and `:wat::holon::literal` share `Boundary::AllData` with `quote` and are
NOT taught to these four descents.** Every measured `make-rule` producer quotes with `quote`; a
`forms`-quoted `:when` was not a shape before this stone and is not one after it. Teaching the
descents a wider head set than the corpus produces would be a widening nothing drives.

### 2.2 The acc-fold head (`rete/kernel/arm.rs`, the brief's address)

```rust
let head_identity: std::borrow::Cow<'_, str> = match items.first() {
    Some(WatAST::Keyword(k, _)) => std::borrow::Cow::Borrowed(k.as_str()),
    Some(WatAST::Symbol(s, _)) => {
        std::borrow::Cow::Owned(crate::edn::render::canonical_identity(s.as_str()))
    }
    _ => { /* unchanged */ }
};
let head: &str = head_identity.as_ref();
```

⛔ **`compile_user_fold_programs` takes the SAME door, and it is not optional.** That fn skips
builtins by prefix and compiles everything else as a user fold; `compile_acc_fold` keys the builtin
table. Curing one and not the other would make them **disagree about which heads are builtin** — a
symbol-spelled `wat.rete.acc/count` would be sent to `lower_named_rete_fn` as a user rete-defn by
one and compiled as `AccFold::Count` by the other. 255.13 §4.3 named both; the brief named only one.

### 2.3 ⭐ `var_key` did NOT need the cure — MEASURED, not assumed

The brief: *"A SECOND dual-raw read is ~15 lines above, inside the `var_key` closure … Measure
whether it needs the same cure or is already reached canonically — do not assume."*

⭐ **It needs neither.** `var_key` reads `items.get(1)` — the acc-form's `?var` argument, a rete
binding key, never a namespaced name. `canonical_identity` is the **identity function** on it: no
`::`, no leading `:`, no `/`, so `edn/render.rs`'s first three branches all fall through to
`s.to_string()`. A door there would move no byte.

⛔ **And it is not a reading — it is a row.** `the_var_key_is_identical_in_both_spellings` drives
`(wat.rete.acc/sum ?v)` and `(:wat::rete::acc::sum ?v)` on two verbs and asserts
`operand_keys() == [Value::String("?v")]` for all four, plus the `Keyword` arm (`:v`) so a future
door on `var_key` cannot move it silently. ⚠ That test is **red pre-cure** — not because `var_key`
is wrong, but because the HEAD read above it sent `wat.rete.acc/sum` to the user-fold arm.

⛔ **`var_key` is also not in the ledger and could not be** (§8.2).

---

## 3. ⭐ THE NON-VACUITY ROWS — two tiers, thirteen rows, driven on two binaries

### 3.1 Tier 1 — the wat surface, `tests/rete/probe_arc251_8d_make_rule_quote_boundary.{rs,wat}`

Eight hand-written `:wat::rete::make-rule` calls — **never a `defrule`** — because the shape under
test is exactly what a converted `defrule` template emits. Each entry point compiles its namespace's
rules, inserts facts, fires **natively** and returns the number of derived facts.

| ns | row | PRE-CURE | POST-CURE | what it pins |
|---|---|---|---|---|
| q1 | `kw-quote` | — | **1** | ⛔ control — the keyword side may not move |
| q2 | ⭐ `sym-quote` | ⛔ **ArityMismatch, line 44** | **1** | **THE CURE** — both quotes symbol-spelled |
| q3 | ⭐ `sym-when-kw-then` | ⛔ **ArityMismatch, line 62** | **1** | the `:when` door alone |
| q4 | ⭐ `kw-when-sym-then` | — | **1** | the `:then` door alone — §3.1a |
| q5 | ⭐ `sym-quote-where-passes` | ⛔ **ArityMismatch, line 98** | **1** | ⛔ **CODE STAYS CODE** — a `(where (wat.rete.i64/> ?n 1))` body inside a SYMBOL-quoted `:when` is still normalized and still evaluated |
| q6 | `kw-quote-where-passes` | — | **1** | control for the row above |
| q7 | ⭐ `sym-quote-where-filters` | ⛔ **ArityMismatch, line 138** | **0** | ⛔ **the non-vacuity of the row above** — the same body with a FALSE gate derives nothing, so `…-passes` cannot be passing on a `where` that never ran |
| q8 | `kw-quote-where-filters` | — | **0** | control |

⚠ **The pre-cure column is honest about its granularity:** `call_beside_value` freezes the whole
fixture, so one bad row poisons every row. The pre-cure failure names **exactly four**
`ArityMismatch`es — **one per symbol-quoted `:when`** (`q2`, `q3`, `q5`, `q7`), and **not one for
`q4`**, whose `:when` is keyword-spelled. ⛔ **So the four `—` rows are red only by association, and
`q4`'s `:then` door needs its own measurement (§3.1a).** Verbatim, from the pre-cure binary (`src/resolve/{normalize,walk}.rs` and
`src/macros/expand.rs` reverted to `HEAD`, rebuilt):

```
call_beside_value: fixture beside ".../probe_arc251_8d_make_rule_quote_boundary.rs" failed to
freeze: #wat.check/CheckErrors {:message "4 type-check errors" … [#wat.check/ArityMismatch
{:message ":wat::rete::acc::count: expected 1 argument(s); got 0" :location … :line 44 …}
… :line 62 … :line 98 … :line 138 …]}
```

⛔ **And the conditions are still DATA — proven by the cure rows themselves.** Every `:when` holds
`(:q2::Group (?g :- :g))` and `(?n :- (…) :from (…))`, neither of which is a legal call. If the cure
had opened the quoted region to normalization or to the checker, `sym-quote` would fail exactly the
way the pre-cure binary fails — which is the failure it is built to detect.

### 3.1a ⭐ THE `:then` DOOR — measured on the CONVERTED stdlib, because nothing else can see it

⛔ **`q4` is NOT a witness for the `:then` cure.** Measured: with `normalize_quote_head_only`
reverted, `q4` is still **1** and the whole eight-row probe is still green. A cure with no witness
is a cure I should not ship, so I went and found the witness where it lives — on a **converted**
stdlib, which is the only tree that emits a symbol-spelled `:then` marker at all.

⭐ **`scripts/replay/census.sh` over all 2 211 tracked `.wat`, twice, on the converted stdlib, one
`src/` line apart:**

```
:then cure OFF   246 files fail --check
:then cure ON    240 files fail --check
diff: SIX files, all rc 1 -> rc 0, NONE the other way
  tests/rete/probe_enum_name.wat
  tests/rete/probe_then_cond_still_works.wat
  wat-scripts/scratch-pad/277-head-kind-census.wat
  wat-scripts/scratch-pad/277-layout-shape-probe.wat
  wat-scripts/scratch-pad/probe-brief-get-is-total-by-fallback.wat
  wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat
```

⭐ **Six files, strictly in the fixing direction, from one line.** That is the `:then` door's
evidence, and it is the reason `normalize_quote_head_only` ships. ⚠ It is also the reason the row in
§3.1's table reads `—` rather than a claim: **the wat probe cannot see this cure, and saying so is
the row.**

### 3.2 Tier 2 — beside the door, `src/rete/kernel/arm.rs` `mod acc_fold_head_identity_tests`

`compile_acc_fold` is `pub(crate)` and `AccFold` is an internal enum, so the four rows the brief
demands are asserted **on the struct** — the variant, and `RuntimeErrorKind::MalformedForm`'s
`head`/`reason` fields — never on a rendered string. `#[cfg(test)]`, so the ledger (a `src/`
instrument that excludes test modules) does not count them.

| # | row | PRE | POST | what it pins |
|---|---|---|---|---|
| 1 | `a_builtin_acc_head_compiles_the_same_in_both_spellings` | ❌ | ✅ | ⭐ **THE CURE** — `(wat.rete.acc/count)` → `AccFold::Count`, and `…/all` → `AccFold::All` so the row is not one verb wide |
| 4 | `the_var_key_is_identical_in_both_spellings` | ❌ | ✅ | ⭐ `?var` resolves to the same binding key in both spellings, two verbs, plus the keyword-arg arm |
| 2 | `a_genuine_user_fold_is_not_swallowed_by_a_builtin_arm` | ✅ | ✅ | ⛔ **THE ADVERSARIAL ROW — green on BOTH binaries** |
| 3 | `an_unknown_head_with_no_program_is_refused_in_both_spellings` | ❌ | ✅ | ⛔ the WALL first, the identity second |
| — | `the_malformed_shape_arms_are_unchanged` | ✅ | ✅ | the not-a-list / no-head diagnoses stay distinct |

⭐ **Row 2 is the stone and it is built adversarially.** Four heads, not one:

```
:user::my-fold          a plain user keyword head
user.app/my-fold        a plain user SYMBOL head
wat.rete.acc2/count     ⭐ a NEIGHBOUR of the builtin namespace — canonicalizes to
                           :wat::rete::acc2::count, which is in no arm
wat.rete.acc/median     ⭐ canonicalizes INTO the builtin namespace but is not a listed verb
```

All four must compile to `AccFold::User` **and** carry the right operand key. ⛔ **They are green on
the PRE-cure binary too** — which is what makes them a test of the wall rather than a test of the
cure (255.12's discipline).

⭐ **Row 3 asserts the WALL before the identity.** `is_some()` first, so the equality below can
never be satisfied by two spellings that are both silently admitted; then the located reason
(`"user acc fold has no compiled Program — setup should have refused"`) and the head. Pre-cure,
verbatim:

```
assertion `left == right` failed: THE CURE: the refusal names the IDENTITY, not the written text
  left: "not.a.real/fold"
 right: ":not::a::real::fold"
```

and it closes with a **positive control** — `refusal((wat.rete.acc/count)) is_none()` — so the two
refusal rows cannot pass on a door that refuses everything.

⭐ **Pre-cure summary, verbatim: `5 tests run: 2 passed, 3 failed`.** The two passes are exactly the
two wall rows.

---

## 4. ⭐ THE LEDGER — 229 → 223, and the shape mix is the number that moved

`cargo nextest run --release -E 'test(keyword_heresy)'` — **4 / 4 green** at the new freeze
(1.15 s), on the post-cure binary.

### 4.1 The ratchet's own output, verbatim

```
⭐ THE LEDGER SHRANK — 229 → 223. This is the good direction, and the
ratchet is deliberate: the frozen census below has to be tightened by hand so the number
can never drift back up silently. Update `LEDGER_TOTAL` and the rows named here.

CURED:
  (none)
GONE ENTIRELY:
  − src/macros/expand.rs  fn expand_make_rule_then  was 1 [Ax1], now 0
  − src/macros/expand.rs  fn expand_make_rule_when  was 1 [Ax1], now 0
  − src/resolve/normalize.rs  fn normalize_make_rule_when  was 1 [Ax1], now 0
  − src/resolve/walk.rs  fn check_make_rule_when  was 1 [Ax1], now 0
  − src/rete/kernel/arm.rs  fn compile_acc_fold  was 1 [Bx1], now 0
  − src/rete/kernel/arm.rs  fn compile_user_fold_programs  was 1 [Bx1], now 0
```

### 4.2 ⛔ THE SHAPE MIX, NOT JUST THE COUNT

Computed over every `FROZEN_LEDGER` row (the fifth draw's own warning: a count-only ratchet can be
walked past sideways).

| | fifth draw | **this draw** |
|---|---|---|
| **A** — keyword-only | 101 | **97** |
| ⛔ **B** — dual-raw | 3 | ⭐ **1** |
| **C** — symbol-raw | 0 | 0 |
| **E** — type-path raw | 125 | **125** |
| **LEDGER_TOTAL** | 229 | ⭐ **223** |
| frozen rows | 110 | **104** |

⭐ **Four of the six are shape A and two are shape B.** The A→0 direction is the one the terminal
cut needs and it is the one nobody had moved: 255.9 dispositioned `rete/kernel/arm.rs` as *"no
(already both) — fine"*, and the four `make-rule` descents were never named by any stone.

⛔ **Shape E did not move, and it is now 56% of the ledger.** 125 sites, 78 in `check.rs`. The
terminal cut needs A+E at zero.

### 4.3 ⭐ THE CALIBRATION — re-anchored, and it FAILED FIRST, exactly as the brief predicted

The brief: *"Curing this WILL red `the_discriminator_separates_the_cured_from_the_open`. That
failure is correct and is NOT permission to delete the row."* ⭐ **It is right, and it fired —
verbatim, before I touched the test:**

```
thread 'keyword_heresy_ledger::the_discriminator_separates_the_cured_from_the_open' panicked at
tests/lint/keyword_heresy_ledger.rs:1098:10:
compile_acc_fold must be in the ledger — it is the shape-B calibration anchor
```

⭐ **That IS the "has failed once by construction" evidence for the re-anchor**, and it cost nothing
to obtain because the cure produced it.

How it moved:

- ⭐ **Six new CURED rows**, keyed per DECISION as 255.13 requires — `compile_acc_fold`
  (`:wat::rete::acc::count`), `compile_user_fold_programs` (`:wat::rete::acc::`),
  `normalize_make_rule_when` / `check_make_rule_when` / `expand_make_rule_when` /
  `expand_make_rule_then` (each `:wat::core::quote`). A revert of this stone now goes red as
  **"THE DISCRIMINATOR CONVICTED A SITE THIS ARC ALREADY CURED."**
- ⛔ **The shape-B anchor moved to `rete/purity.rs::walk_rete_defn_callees`** — the **last** shape-B
  site in the crate, the rete-defn cycle detector the FIFTH draw named and deliberately did not cure.
- ⭐ **The OPEN table still has three rows** (`effectful_by_prefix`, `classify_expr`,
  `value_matches_type_by_name`), so the open side is not empty and the calibration still proves both
  directions.
- ⛔ **And I wrote down what happens when B hits zero**, in the test: the honest move is a
  **synthetic** anchor in `the_discriminator_convicts_a_synthetic_heretic_and_clears_its_cure`,
  which already constructs one shape-B heretic — **not** the deletion of the assertion. A future
  hand that cures `walk_rete_defn_callees` will read that before it reaches for `git rm`.

---

## 5. ⭐ THE CENSUS THE CODE FORCED — every other reader of the same language fact

Measured with `grep -rn '":wat::core::quote"' src/ --include=*.rs`, then each hit read. The question
is not "who compares this string" but **"who decides a data boundary with it"**.

| site | verdict |
|---|---|
| `resolve/boundary.rs::quote_boundary` | ⭐ **already cured at every caller** — `normalize_form` and `walk::check_form` both feed it through `head_fqdn` / the head-node match; it is the `make-rule` descents that bypassed it |
| `resolve/normalize.rs::normalize_make_rule_when` · `resolve/walk.rs::check_make_rule_when` · `macros/expand.rs::expand_make_rule_when` / `…_then` | ⭐ **CURED by this stone** (§2.1) |
| `resolve/normalize.rs::normalize_make_rule_condition` · `resolve/walk.rs` · `macros/expand.rs` — the `is_where_form` guards | ⛔ **STILL KEYWORD-ONLY. Reported, not cured.** A converted *corpus* (8d-iii) spells the condition `(wat.rete/where …)`, and these three would miss it. The stdlib does not emit a `where` from a template, so nothing in §7's conversion reaches them — but the next draw will |
| `check.rs:3506` `":wat::core::quote"` arm, inside `infer_list`'s `if let WatAST::Keyword` | ⛔ **shape A, `check.rs`, 78 shape-E + 37 shape-A siblings — OUT OF SCOPE** and the reason this cure had to re-spell the head rather than teach the checker a second spelling |
| `rete/purity.rs:1241` / `:2040` — the quote/quasiquote DATA guards | ⛔ the FIFTH draw's §9.4 finding, **still the builder's call** (255.11 §4(a)): teaching a *data* guard a second spelling makes it swallow more as data, which is the false-green direction |
| `rete/expr_ir/mod.rs:624` · `macros/eval.rs:189` · `runtime.rs:6888` / `:15883` · `intrinsic/special/quote.rs` · `intrinsic/mod.rs` | not boundary decisions on a raw head (an already-canonical `core` name, a registration key, an error-shape test) |

---

## 6. EVERY GATE ON THE LANDABLE STATE, WITH ITS EVIDENCE

| gate | landable state (`wat/` untouched) |
|---|---|
| `cargo build --release` after every `src/` edit | ✅ exit 0, 21.7–22.9 s each, `Compiling wat` shown |
| ⭐ **`scripts/floor.sh`** | ✅ **GREEN — 6008 / 6008**, 10 slow, 22 skipped, **327.923 s**, exit 0 · `.floor/2026-09-23T00-06-10Z/` |
| floor denominator | 6002 (fifth draw) **+ 6 of mine** (5 unit + 1 probe) = **6008** ✓ |
| ⚠ floor duration | **327.9 s** — the healthy band (326.7 / 328.5 / 320.1 s on disk). ⚠ *A fast floor is a symptom*: `every_wat_scripts_file_loads_on_the_current_runtime` is `SLOW [>300.000s]` on its own, and it ran |
| ⚠ `harvest_wrap_split` | **PASS** [0.205 s] (1569/6008) — named either way, per the standing bookkeeping exception (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`) |
| clippy `-D warnings --all-targets --workspace` | ✅ **0 warnings, rc 0** — and **NOT cached**: `touch src/rete/kernel/arm.rs src/resolve/normalize.rs` first, **12.85 s** elapsed |
| census | ✅ **`census-diff: no STOP-8`, rc 0** · `.census/2026-09-23T00-14-46Z.txt` · **2211 paths** (2210 + my committed fixture) · **213 failing = 213 failing** |
| ⭐ `scripts/replay/delta.sh` | ✅ **NEW 3** = baseline 3 (`probe-c1-clean-surface`, `probe-kwargs-peer`, `wat/holon/Ngram.wat`), ⭐ **RECOVERY 0**, exit 0 · ORIG-CLEAN 160/179, CONV-CLEAN 157/179 · list-sha `33ede76c…f1cbc3e5`, paths=179 missing=0 · `.delta/2026-09-23T00-13-31Z/` |
| ⭐ ledger | ✅ **4 / 4 green at 223** (`-E 'test(keyword_heresy)'`, 1.15 s) |
| `wat/` clean at commit | ✅ `git status --porcelain` named no `wat/` path |

### 6.1 ⛔ THE FIRST FLOOR WENT RED — TWO TESTS, BOTH MINE, ONE LINE

⛔ **Not re-run. Captured whole, from `.floor/2026-09-22T23-58-12Z/ARM.txt`.**

```
     Summary [ 322.414s] 6008 tests run: 6006 passed (9 slow), 2 failed, 22 skipped
        FAIL [   0.077s] ( 217/6008) wat::lint one_variant_separator::only_identifier_rs_spells_the_variant_separator
        FAIL [   0.256s] ( 219/6008) wat::lint one_name_grammar::only_identifier_rs_parses_a_name
```

**(a)** verbatim:

```
thread 'one_variant_separator::only_identifier_rs_spells_the_variant_separator' (1818504) panicked at
/home/john/work/holon/wat-rs/tests/lint/one_variant_separator.rs:260:5:


🔥🔥🔥 A SECOND VARIANT SEPARATOR — 1 site(s) spell the `::` between an enum and 
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

src/rete/kernel/arm.rs:1611  [DATA]  let leaf = head.rsplit("::").next().unwrap();

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**(b)** verbatim:

```
thread 'one_name_grammar::only_identifier_rs_parses_a_name' (1818442) panicked at
/home/john/work/holon/wat-rs/tests/lint/one_name_grammar.rs:127:5:


🔥🔥🔥 A SECOND NAME PARSER — 1 site(s) hand-roll one of the five name-grammar
call-shapes (`rfind("::")`, `rsplit("::")`, `rfind('/')`, `rsplit_once('/')`,
`strip_suffix('\'')`) OUTSIDE `crates/wat-reader/src/identifier.rs`. A name is an atom;
structure encoded inside it must be parsed exactly ONE way, or two parsers WILL disagree
(STONE-one-name-grammar, arc 109 — the census found 33 that already had).

THE FIX — route through the door's accessors (methods on `Identifier`, free functions on
`&str` for sites holding a raw keyword/symbol string):

  leaf(name)        the last `::` segment        :wat::cache::Lru  -> Lru
  path(name)        everything before the leaf   :wat::cache::Lru  -> :wat::cache
  receiver(name)    everything before the `/`    :S/mk             -> :S
  method(name)      everything after the `/`     :S/mk             -> mk
  prime(name)       is the name primed?          :sort'            -> true
  deprimed(name)    the name without its `'`     :sort'            -> :sort

If the hit is genuinely NOT a wat name (a filesystem path, a URL, an EDN tag, a doc
string) — earn a co-located, same-line `// rune:lint(one-name-grammar) — <reason>`.
A rune reason of "it's just this one site" does NOT earn its standing — that site is a
FIX (route through the door), not a rune (excusare — the reason must earn it).

Offenders:

src/rete/kernel/arm.rs:1611   rsplit("::")

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

⭐ **The exact arm: both gates fired on ONE line — `src/rete/kernel/arm.rs:1611`, a
`head.rsplit("::").next().unwrap()` I had written in a TEST HELPER** to derive `sum` from
`":wat::rete::acc::sum"`. Two independent gates, one hand-rolled name parser, and the irony is
exact: **this stone is about a name read two ways, and my own fixture read one two ways.** Cured by
routing through the door (`wat_reader::identifier::leaf`), with the `rune:lint(one-variant-separator,
namespace)` the second gate's own message prescribes for a namespace/leaf split. Both gates green on
the next build; the floor that follows (§6) is a **different tree**, not a re-run.

---

## 7. ⭐ THE CONVERSION — 283 → 161, and it does NOT land

⛔ **Operational discipline, stated because the arc has paid for its absence:** the 64 paths came
from `git ls-files | grep -E '^wat/.*\.wat$'` (⚠ **not** `git ls-files 'wat/**/*.wat'`, which
returns 31 of 64), were passed as one explicit EDN vector, and **no `cargo` command and no
`git add` ran while the codemod was writing `wat/`.**

### 7.1 The numbers

| | fifth draw | **this draw** |
|---|---|---|
| conversion | 64/64, rc 0, ~1330 s | ✅ **64/64, rc 0** (`CONVERSION_RC=0`); `git status` = exactly those 64 `wat/` paths and nothing else |
| `cargo build --release` on the converted tree | 22.84 s | ✅ **exit 0, 23.14 s** |
| binary starts | ✅ | ✅ (`--check` of my own fixture: rc 0) |
| ⭐ **floor** | ❌ **283 / 6002**, 453.01 s | ❌ ⭐ **161 / 6008**, **456.687 s**, exit 100 · `.floor/2026-09-23T00-39-37Z/` |
| ⚠ `harvest_wrap_split` | PASS | **PASS** [0.137 s] (1569/6008) — named either way |

```
     Summary [ 456.687s] 6008 tests run: 5847 passed (18 slow), 161 failed, 22 skipped
```

⚠ 456.7 s is the *converted-but-red* band (the fifth draw's was 453.0 s), not the 24 s
"stdlib-did-not-load" symptom.

### 7.2 ⭐ FIXED / NEW — and it is **NOT** a strict subset this time

`comm` over the two sorted failing-test-NAME sets, extracted with a parser that strips the **entire**
`FAIL [   0.836s] (1523/6002) ` prefix including a padded index (the regex that manufactured 138
phantom regressions during the last weigh). It reproduces the fifth draw's **283** exactly from its
own `clean.log`, which is the control that says the parser is right.

```
only in the FIFTH draw's 283  (i.e. FIXED here) : 127
only in mine                  (i.e. NEW)        :   5
```

⛔ **FIVE NEW. The fifth draw's was 133 fixed / 0 new; mine is 127 / 5, and the 5 are the finding.**

| the 5 NEW |
|---|
| `wat::rete probe_arc278_D11_nested_then_field_types::every_well_typed_nested_then_value_still_compiles_and_derives` |
| `wat::rete probe_arc278_match_arm_is_not_a_call::a_correct_constructor_in_a_match_arm_body_still_fires` |
| `wat::rete probe_arc278_nested_wall::a_correctly_spelled_nested_constructor_still_compiles_and_fires` |
| `wat::rete probe_constructor_meta_surface_audit::nested_surface_aggregate_constructor_now_works_via_native_kernel` |
| `wat::rete probe_constructor_meta_surface_audit::nested_surface_aggregate_constructor_now_works_via_oracle` |

⭐ **All five carry ONE arm, verbatim and identical:**

```
compile-condition: then expr is not a rete primitive — ':wat::core::kwargs-construct' is not a
rete primitive; a then admits only :wat::rete:: ops
  :location #wat.kernel/Location {:file "wat/rete/compile.wat" :line 797 :col 35}
  :frames [ {:file "wat/rete/compile.wat" :line 1120 :symbol ":wat::rete::then-item-fence"}
            {:file "wat/rete/compile.wat" :line 1180 :symbol ":wat::rete::compile-rule"} … ]
```

and the whole block of one of them, untruncated:

```
        FAIL [   0.948s] ( 732/6008) wat::rete probe_arc278_nested_wall::a_correctly_spelled_nested_constructor_still_compiles_and_fires
  stdout ───

    running 1 test
    test probe_arc278_nested_wall::a_correctly_spelled_nested_constructor_still_compiles_and_fires ... FAILED

    failures:

    failures:
        probe_arc278_nested_wall::a_correctly_spelled_nested_constructor_still_compiles_and_fires

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 521 filtered out; finished in 0.94s
    
  stderr ───

    thread 'probe_arc278_nested_wall::a_correctly_spelled_nested_constructor_still_compiles_and_fires' (2703116) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_nested_wall.rs:74:5:
    the control must run — every nested field name in it is real
    #wat.kernel/AssertionFailure {:thread "main" :message "compile-condition: then expr is not a rete primitive — ':wat::core::kwargs-construct' is not a rete primitive; a then admits only :wat::rete:: ops" :location #wat.kernel/Location {:file "wat/rete/compile.wat" :line 797 :col 35} :actual nil :expected nil :frames [#wat.kernel/Frame {:file "wat/rete/compile.wat" :line 1120 :symbol ":wat::rete::then-item-fence"} #wat.kernel/Frame {:file "wat/rete/compile.wat" :line 1180 :symbol ":wat::rete::compile-rule"} #wat.kernel/Frame {:file "tests/rete/probe_arc278_nested_wall_ok.wat" :line 22 :symbol ":wat::rete::compile-all"} #wat.kernel/Frame {:file "tests/rete/probe_arc278_nested_wall_ok.wat" :line 40 :symbol ":nwo::fire"} #wat.kernel/Frame {:file "src/freeze.rs" :line 1542 :symbol ":user::main"}] :upstream-chain nil}
    [#wat.kernel/LociDiedError.Panic {:message "compile-condition: then expr is not a rete primitive — ':wat::core::kwargs-construct' is not a rete primitive; a then admits only :wat::rete:: ops" :failure #wat.core/Option.Some {:value #wat.kernel/Failure {:error #wat.core/Fault {:message "compile-condition: then expr is not a rete primitive — ':wat::core::kwargs-construct' is not a rete primitive; a then admits only :wat::rete:: ops" :location #wat.kernel/Location {:file "wat/rete/compile.wat" :line 797 :col 35} :causes []} :frames [#wat.kernel/Frame {:file "wat/rete/compile.wat" :line 1120 :symbol ":wat::rete::then-item-fence"} #wat.kernel/Frame {:file "wat/rete/compile.wat" :line 1180 :symbol ":wat::rete::compile-rule"} #wat.kernel/Frame {:file "tests/rete/probe_arc278_nested_wall_ok.wat" :line 22 :symbol ":wat::rete::compile-all"} #wat.kernel/Frame {:file "tests/rete/probe_arc278_nested_wall_ok.wat" :line 40 :symbol ":nwo::fire"} #wat.kernel/Frame {:file "src/freeze.rs" :line 1542 :symbol ":user::main"}] :actual #wat.core/Option.None {} :expected #wat.core/Option.None {}}}}]

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### 7.3 ⭐⭐ THE 5 NEW ARE ISOLATED TO ONE OF THE SIX SITES — by a two-binary experiment, not a theory

⛔ **I did not classify these and move on.** Two reverts on the converted tree, each rebuilt:

| reverted (converted tree) | the 5 NEW |
|---|---|
| `normalize.rs::normalize_quote_head_only` (the `:then` MARKER) | ❌ **still all 5 fail** — not the cause |
| ⭐ `expand.rs::expand_make_rule_then`'s `is_quote` alone | ✅ **all 5 pass** (`25 tests run: 24 passed, 1 failed`; the 1 is `a_not_knowable_nested_then_operand…`, already in the fifth draw's 283) |

⭐ **THE MECHANISM: `expand_make_rule_then`'s cure ARMS A WALL THAT THE CONVERSION HAD SILENTLY
DISARMED.** Arc 278 built that descent because *"EVERY rete macro was unusable in a `:then`"*. Under
a converted stdlib its keyword-only `is_quote` was **false for every rule in the corpus**, so the
`:then` fact vector was **never macro-expanded at all**. Cured, the expansion runs again — a nested
kwargs constructor expands to `(:wat::core::kwargs-construct …)` — and `wat/rete/compile.wat:797`'s
`then-item-fence` (LAW A, *"a then admits only `:wat::rete::` ops"*) refuses it.

⛔ **I KEPT THE CURE, and the reasoning is the arc's own.** Reverting it would buy five green tests
by leaving the `:then` expansion silently OFF under conversion — a **disarmed wall**, which
255.13 and the fifth draw both name as strictly worse than a red. It would also ship
`expand_make_rule_when` cured and its identical twin not, which is exactly the drift
`resolve/boundary.rs`'s doc exists to prevent. ⚠ **The landable floor is GREEN with it (6008/6008);
these five fail only on a converted tree, and the conversion does not land.**

⛔ **What I did NOT do is diagnose why the fence admits `kwargs-construct` on the unconverted tree
and refuses it on the converted one.** That is the seventh draw's first question, and the ledger
already names a candidate: `rete/purity.rs::is_declaration_derived_construction`, **2 sites, shape
A, keyword-only** — the predicate that decides whether a construction is declaration-derived.

### 7.4 The 161 that remain — CLASSIFIED, NOT DIAGNOSED

⛔ **Stated plainly, as the fourth and fifth draws did: I bucketed every failing block by the error
its stderr carries. I diagnosed the 5 NEW (§7.3) and none of the other 156, and the classes may
interact.** All 161 have a body block in `ARM.txt`; the classifier reads that block.

| n | class |
|---|---|
| 54 | a bare assertion / panic with no typed wat error |
| ⭐ **38** | **a rete fence refusal — `… is not a rete primitive`** (`wat/rete/compile.wat`'s `then-item-fence` / `where` fence / acc-form fence). ⚠ **This is the class my §7.3 five joined, and it is now the largest TYPED class** |
| 33 | `#wat.resolve/UnresolvedReferences` |
| 14 | `#wat.runtime/MalformedForm` |
| 12 | `#wat.check/CheckErrors` (other — arity, unknown name, …) |
| 8 | `#wat.check/TypeMismatch` |
| 2 | `StartupError` (other) |
| **161** | |

⭐ **THE CLASS THE BRIEF WAS DRAWN FOR IS GONE, NOT REDUCED.** Occurrence counts over the whole
converted `clean.log`, same grep on both:

| verb | fifth draw's converted log | **mine** |
|---|---|---|
| `:wat::rete::acc::count: expected` | 75 | ⭐ **0** |
| `:wat::rete::acc::sum: expected` | 48 | ⭐ **0** |
| `:wat::rete::acc::max: expected` | 41 | ⭐ **0** |
| `:wat::rete::acc::min: expected` | 29 | ⭐ **0** |
| `:wat::rete::acc::all: expected` | 5 | ⭐ **0** |
| **total** | **198** | ⭐ **0** |

⚠ **My counts of the fifth draw's log (75 / 48) differ from the brief's (98 / 66)** — the brief
counted over `ARM.txt`, which repeats each failure; these are over `clean.log`, which does not.
Either way the post column is **zero on every verb**.

### 7.5 ⛔ THE CONVERSION IS RESTORED

```
git checkout -- wat/    →  git status --porcelain names no wat/ path
cargo build --release   →  exit 0, 23.32 s
-E 'test(make_rule_quote_boundary) or test(acc_fold_head_identity) or test(keyword_heresy)'
                        →  10 / 10 green on the restored binary
```

---

## 8. ⛔ WHAT THE BRIEF GOT WRONG — five, and the first is the twenty-fourth

1. ⭐⭐ **THE ADDRESS. `compile_acc_fold` cannot produce the failures it was drawn for.** The brief
   starred *"READ BY THE ORCHESTRATOR THIS TIME"* and named
   `src/rete/kernel/arm.rs::compile_acc_fold` for the 239 `ArityMismatch` failures. Those are
   `#wat.check/` errors from the **type checker** (freeze step 8); `compile_acc_fold` is a fire-time
   lowering whose only refusal is `RuntimeErrorKind::MalformedForm` (§1.1). ⛔ **The orchestrator
   verified the FUNCTION SHAPE — a real dual-raw head read, correctly quoted — and inferred the
   CAUSAL LINK from the error text naming `acc::count`.** That is the same *kind* as the
   twenty-second and twenty-third: an address taken from the neighbourhood of a symptom rather than
   from the data flow. The real cause is `resolve/normalize.rs::normalize_make_rule_when`, four
   files away, and it is measured, not argued (§1).
2. ⛔ **"The missing argument is INJECTED by the builtin arm."** Nothing injects an argument
   anywhere in the crate. The zero-arg acc-form is legal because the `:when` vector is quoted DATA;
   the defect is that it stopped being data (§1.1).
3. ⛔ **"255.13's ledger named BOTH [`compile_acc_fold`'s head and `var_key`]; they are the two
   remaining shape-B sites."** 255.13 §4.3 and `FROZEN_LEDGER` name **`compile_acc_fold` and
   `compile_user_fold_programs`** — two different FUNCTIONS. `var_key` holds **no keyword-literal
   comparison at all**, so it is not a decision site and cannot be a ledger row by construction. The
   brief's §"The work" item 2 therefore sent me to measure a site that could not have been there —
   and the site it left out (`compile_user_fold_programs`) is the one that *must* be cured in the
   same edit or the two disagree about which heads are builtin (§2.2).
4. ⚠ **`var_key` needs no cure — measured, and the brief's row 4 is the measurement** (§2.3).
   `canonical_identity` is the identity function on a `?var`.
5. ⚠ **What the brief could not have known, and it changes the disposition of its own address:**
   `compile_acc_fold`'s Symbol arm is **unreachable from the wat surface today**, because
   `wat/rete/compile.wat:578`'s acc-form fence — `starts-with? acc-hd-nm ":wat::rete::acc::"`, in
   **wat**, keyword-only — refuses a symbol-spelled acc head first (§1.3). The cure is still right
   and still driven; it just cannot move a floor by itself.

⭐ **And the brief got the hardest thing right:** it predicted that curing would red
`the_discriminator_separates_the_cured_from_the_open` and forbade deleting the row. It did, and it
was re-anchored (§4.3).

---

## 9. ⛔ WHAT MY GREEN CANNOT SEE

1. ⛔ **`compile_acc_fold`'s cured Symbol arm has NO wat-surface witness.** `wat/rete/compile.wat`'s
   acc-form fence refuses a symbol-spelled acc head before the lowering ever runs (§1.3), so all
   five of its rows are **unit** rows calling `compile_acc_fold` directly with constructed ASTs.
   They prove the function's own boundary and nothing about reachability. ⚠ The fence is itself a
   keyword-only prefix test **in wat**, outside the ledger's reach (`src/` only) and outside this
   stone's scope: teaching a default-deny fence a second spelling is a permissive widening of a
   WALL, which is a different decision from a door on a READ.
2. ⛔ **The `is_where_form` guards are still keyword-only in all three descents** (§5). Nothing in
   the corpus reaches them today because no template emits a `where`; a converted `.wat` **corpus**
   (8d-iii) will, in every rule that has one.
3. ⛔ **`check.rs::infer_list` is still keyword-only on the head** — that is why the cure had to
   RE-SPELL the boundary marker rather than teach the checker a second spelling. Every other symbol
   head that reaches `infer_list` through a path `normalize` does not cover behaves the same way,
   and I did not census those paths.
4. ⛔ **The eight wat rows freeze ONE fixture**, so the pre-cure column is "the file did not freeze,
   naming four `ArityMismatch`es". I cannot say from that run which of the eight rows would have
   passed in isolation; what I can say is which four sites the checker named, and they are exactly
   the four symbol-quoted `:when`s.
5. ⛔ **`normalize_quote_head_only`'s only witness is a CONVERTED-tree census** (§3.1a). The
   landable floor cannot see it, the wat probe cannot see it, and the converted tree it fixes is a
   tree that no longer exists. Six files, rc 1 → rc 0, measured once.
5a. ⛔ **The five NEW converted failures are ISOLATED, not DIAGNOSED.** I proved which of my six
   sites causes them by reverting one guard at a time and rebuilding; I did **not** find out why
   `wat/rete/compile.wat`'s `then-item-fence` admits `:wat::core::kwargs-construct` on an
   unconverted tree and refuses it on a converted one. Naming `is_declaration_derived_construction`
   is a LEAD (§7.3), not a cause.
5b. ⛔ **I cannot say the other 156 hold no second instance of that shape.** 38 of them are rete
   fence refusals and I read exactly five of those blocks.
6. ⛔ **The adversarial search is four constructed heads, not a proof of non-existence.** I could
   not construct a head that `canonical_identity` re-spells INTO the builtin table without already
   being it; that is a statement about my search.
7. ⛔ **The ledger's own blind spots are unchanged** (255.13 §2.3): reachability, a keyword-only
   READ feeding a covered decision, the 68 `D-unresolved` sites, macro-generated code, anything
   outside `src/`. My 223 inherits every one.
8. ⛔ **The census gate could not see my new fixture until it was committed** — `census.sh` reads
   tracked files, so the run before the commit had 2210 paths and the one after has 2211. The row
   in §6 is the post-commit one.
9. ⛔ **`delta.sh`'s RECOVERY column is only meaningful against an UNCONVERTED embedded stdlib**
   (the fourth draw proved that). My delta row is the landable one.

---

## 10. WHAT THE SEVENTH DRAW NEEDS (not started here)

1. ⭐⭐ **THE FIVE (§7.3) FIRST.** They are the only converted-floor failures this arc has ever
   isolated to a single named line by experiment rather than classified by grep, and the question
   they leave is exact: *why does `wat/rete/compile.wat:797`'s `then-item-fence` admit
   `:wat::core::kwargs-construct` on an unconverted tree and refuse it on a converted one?* The
   ledger's candidate is `rete/purity.rs::is_declaration_derived_construction`, **2 sites, shape A**.
   ⛔ Answering it is worth more than the next 100 of the 161, because 38 of those 161 are the same
   fence.
2. ⭐ **The `is_where_form` guards** (§5, §9.2) — the same language fact as this stone's, one level
   down, in the same three descents. Cheap, and it is the first thing a converted corpus hits.
3. ⭐ **`wat/rete/compile.wat:578`'s acc-form fence** — `starts-with? acc-hd-nm
   ":wat::rete::acc::"`, keyword-only, **in wat**. It is what makes this draw's `compile_acc_fold`
   cure unreachable (§1.3), and it is the first keyword-heresy anyone has had to cure in `.wat`
   rather than `.rs`. ⛔ It is a default-deny FENCE, so it is the builder's call, not a rider's.
4. ⛔ **The rete-IR lowering refuses a symbol head outright** — measured on the
   single-file-converted stdlib: `malformed :wat::rete::lower form: call head must be a keyword`
   (`probe_arc278_then_is_an_expansion_boundary`). Every rete macro whose template emits a head
   (`wat/rete/syntax.wat`'s rete-spelled `cond`/`if`) produces symbol heads under conversion, and
   `expr_ir::lower` and `rete/clause.rs`'s clause classifier both read keywords only. **That is the
   next class after this one, and it is bigger than a door.**
5. `rete/purity.rs::walk_rete_defn_callees` — the **last** shape-B site in the crate, and the
   calibration anchor. When it is cured, read §4.3's note before touching the assertion.
6. Shape **E**, 125 sites, 78 in `check.rs` — untouched, and now **56%** of the ledger. ⭐ The
   terminal cut needs A+E at zero, not just B.

## Artifacts

- `.floor/2026-09-22T23-58-12Z/` — the FIRST landable floor, **RED 2/6008**, 322.414 s, exit 100;
  both failures mine, `ARM.txt` holds them whole
- `.floor/2026-09-23T00-06-10Z/` — the landable floor, **6008 / 6008 green**, 327.923 s, exit 0
- `.census/2026-09-23T00-14-46Z.txt` — landable census, 2211 paths, `no STOP-8` vs the fifth draw's
- `.delta/2026-09-23T00-13-31Z/` — NEW 3 = baseline 3, RECOVERY 0
- `.floor/2026-09-23T00-39-37Z/` — the **converted** floor, **161 / 6008 red**, 456.687 s, exit 100;
  `ARM.txt` holds all 161 whole blocks untruncated
- `.census/2026-09-23T00-50-52Z.txt` / `.census/2026-09-23T00-52-22Z.txt` — the `:then`-door
  measurement, converted stdlib, 246 vs 240 (§3.1a)
- `tests/rete/probe_arc251_8d_make_rule_quote_boundary.{wat,rs}` — the 8-row wat truth table
- `src/rete/kernel/arm.rs` `mod acc_fold_head_identity_tests` — the 5 struct-level rows
- `tests/lint/keyword_heresy_ledger.rs` — the ledger at **223**, six new CURED rows, the shape-B
  anchor re-anchored onto `walk_rete_defn_callees`
