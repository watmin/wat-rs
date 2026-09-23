# SCORE — STONE 251.8d-ii (SEVENTH DRAW): the `:then` item fence

Branch `main`, drawn against `7d02c4ba1`. Parent:
`BRIEF-STONE-251.8d-ii-SEVENTH-the-then-item-fence.md`. **Not pushed.** Every number below was
produced this session on a binary I built; `cargo build --release` ran after **every** `src/` edit
and after every `wat/` change, and each row names which binary produced it.

## VERDICT — **LANDED (the cures). STOP (the conversion).**

⛔⛔ **THE BRIEF'S MEASURED ADDRESS DOES NOT REPRODUCE — the twenty-fifth correction, and a new
species of it.** The named test (`probe_arc278_then_is_an_expansion_boundary::both_fact_form_shapes_expand_their_values`)
**PASSES** with `wat/rete/compile.wat` converted alone. ⭐ **The METHOD is right and I used it**:
that one file does reproduce a failure — **eight** `binary(rete)` tests — but at the **ACCUMULATOR**
fence (line 609, the `is-pure` conjunct), not the `:then` item fence (795, `is-rete`), and
`primitive?` never decides anything.

⭐⭐ **NARROWED TO ONE TOKEN by hunk-level experiment** (four binaries, each a rebuild): with the
whole file converted, putting back **only the `unquote`** at `compile.wat:580` returns
**522 / 0**; leaving only the `unquote` converted reproduces **514 / 8**. The `quasiquote` head is
irrelevant. `src/runtime.rs::match_qq_head` read the marker as a `WatAST::Keyword` payload only,
the unquote never fired, and `fence-call`'s head became a LIST — `'<non-keyword/symbol head>'`.

⭐ **QUESTION ONE, ANSWERED ON A 14-ROW PAIRED PROBE: `primitive?` IS spelling-sensitive, at
exactly two places** — the declaration-derived construction door (the VERB *and* the TYPE) and the
core-structural-guard arm. It is spelling-AGNOSTIC on ordinary call heads and surface constructors.
⚠ And the brief's own expectation in that question is wrong: `(:wat::core::+ 1 2)` must be `false`;
Law A refuses core-spelled computation.

⭐ **THE ERROR MESSAGE LIED AND THREE BRIEFS BELIEVED IT.** `':wat::core::kwargs-construct' is not
a rete primitive` names a spelling **no source file contains**: `classify_expr` re-spells a
`Symbol` head through `canonical_identity` before building the `AxisViolation`. The three emitting
templates (`wat/Record.wat:207`/`:296`, `wat/core.wat:2078`) all emit a **symbol** head once
converted.

⭐ **THREE DOORS, every `Keyword` arm byte-identical** — `match_qq_head` (which also covers
`quasiquote` and `unquote-splicing`), `is_declaration_derived_construction` (both reads), and
`expr_ir::lower_list` (head + constructor type). ⛔ **Door 3 is not scope creep and it is measured**:
door 2 alone moves the error one frame, from the FENCE to the LOWERING.

⛔ **THE WALL DID NOT MOVE.** Law A still refuses core-spelled computation inside a `:then` in
BOTH spellings, with the same located reason — **on the converted tree as well as the landable
one**. The declaration-derived door still refuses an undeclared type, a near-miss verb, a missing
type argument and a non-name, in both spellings. `unquotex` — one letter from `unquote` — still
stays data.

⭐ **THE CONVERTED FLOOR: 161 → 112. `comm` on the two sorted failing-test sets: 49 FIXED, ⭐ 0 NEW
— a strict subset.** ⭐ **The class the brief was drawn for is GONE, not reduced:**
`is not a rete primitive` was **117** occurrences in the sixth draw's converted `clean.log` and is
**0** in mine; `'<non-keyword/symbol head>'` 15 → **0**; `malformed :wat::rete::lower form` 2 → **0**.

⭐ **All five of the sixth draw's NEW now pass, and the mechanism is ISOLATED, not assumed**
(§7.3): reverting MY two rete doors on the converted tree — keeping `match_qq_head` cured and
`src/macros/expand.rs` untouched — brings all five back. The `:then` expansion is still armed.

⭐ **LEDGER 223 → 220**, A **97 → 94**, B **1 → 1**, C 0, E 125. ⛔ **No re-anchoring was needed**:
`walk_rete_defn_callees` is deliberately not cured, so the shape-B calibration anchor stands
untouched.

Landable floor **6011 / 6011 GREEN**, 322.846 s · clippy **0** (not cached, 12.93 s) · census
**`no STOP-8`**, 213 = 213 · delta **NEW 3 = baseline 3, RECOVERY 0**. ⛔ **The FIRST floor went
RED on two lint gates — both MINE, both my own fixtures writing a FORM where they meant a NAME**
(§6.1) — captured verbatim and cured.

⛔ **112 is still RED, so the conversion does NOT land.** `wat/` restored (`git checkout -- wat/`
→ 0 modified paths, + rebuild — the seventh proof of that recovery).

⛔ **FOUR CORRECTIONS TO THE BRIEF (§8).**

---

## 1. ⭐⭐ THE NARROWING — the brief's address does NOT reproduce, and the real one is ONE TOKEN

### 1.1 ⛔ The named test does not fail

The brief's address is starred as *measured, not inferred*:

> | `wat::rete probe_arc278_then_is_an_expansion_boundary::both_fact_form_shapes_expand_their_values`, unconverted | ⭐ **PASS** |
> | convert **`wat/rete/compile.wat` ALONE**, rebuild, re-run | ⛔ **FAIL** |

⛔ **It does not fail.** Reproduced exactly as written — `printf '["…/wat/rete/compile.wat"]\n' |
./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat` (rc 0, `git diff --stat` = one
file, 702+/702−), `cargo build --release` (22.20 s, the converted stdlib verified embedded:
`strings target/release/wat | grep -c 'wat\.rete/then-item-fence'` → **4**, and the keyword
spelling → **0**):

```
Summary [   0.358s] 4 tests run: 4 passed, 6026 skipped
    PASS (1/4) …probe_arc278_then_is_an_expansion_boundary::cond_in_a_then_expands_and_selects_the_right_branch
    PASS (2/4) …probe_arc278_then_is_an_expansion_boundary::the_lhs_path_that_already_worked_still_works
    PASS (3/4) …probe_arc278_then_is_an_expansion_boundary::both_fact_form_shapes_expand_their_values
    PASS (4/4) …probe_arc278_then_is_an_expansion_boundary::a_nested_constructors_value_expands_while_its_head_stays_data
```

⭐ **But the METHOD is sound and the file IS an address** — it just carries a different failure.
The whole `binary(rete)` (522 tests) with `wat/rete/compile.wat` converted alone:

```
Summary [  49.761s] 522 tests run: 514 passed, 8 failed, 0 skipped
    FAIL ( 130/522) wat::rete probe_arc278_8custom_native_differential::differential_custom_empty
    FAIL ( 132/522) wat::rete probe_arc278_8custom_native_differential::differential_custom_fold
    FAIL ( 134/522) wat::rete probe_arc278_8custom_native_differential::differential_custom_fold_value
    FAIL ( 504/522) wat::rete probe_fence_names_the_head::core_op_accumulator_names_law_a_not_total
    FAIL ( 507/522) wat::rete probe_fence_names_the_head::impure_accumulator_names_the_offending_head_and_axis
    FAIL ( 512/522) wat::rete probe_fence_names_the_head::partial_accumulator_names_the_offending_head_and_axis
    FAIL ( 520/522) wat::rete wat_scripts_grid_port_check::every_grid_axis_native_matches_its_oracle
    FAIL ( 521/522) wat::rete wat_scripts_grid_axes_live::grid_axes_run_and_derive_nonvacuously
```

⭐ **All eight carry ONE arm, verbatim and identical, and it is NOT the `:then` fence:**

```
#wat.kernel/AssertionFailure {:message "compile-condition: accumulator expr is not pure —
  '<non-keyword/symbol head>' is not pure"
  :location #wat.kernel/Location {:file "wat/rete/compile.wat" :line 609 :col 46}
  :frames [{:file "wat/rete/compile.wat" :line 1125 :symbol ":wat::rete::compile-condition"}
           {:file "wat/rete/compile.wat" :line 1180 :symbol ":wat::rete::compile-rule"} …]}
```

⛔ **Line 609 is the ACCUMULATOR fence, not line 795's `:then` item fence**, and it fires on the
**FIRST** conjunct (`is-pure`), so `:wat::rete::primitive?` never gets to decide anything. The
brief's class analysis (117 of 161 carrying `… is not a rete primitive`) is about the WHOLE
converted corpus; the single-file address it reports is a different failure entirely.

### 1.2 ⭐ NARROWED TO ONE TOKEN, by hunk-level experiment

`grep -n 'quasiquote\|unquote' wat/rete/compile.wat` on the converted file returns **exactly two
lines** — the accumulator fence's probe template:

```wat
579:  fence-call   (wat.core/quasiquote
580:                  ((wat.core/unquote acc-hd) __acc__))
```

Four binaries, each a full `cargo build --release` + `binary(rete)` (522 tests). Everything else in
the file stays converted; only these two head spellings move. ⚠ These are EXPERIMENTS on a tree
that was restored afterwards — no hand-edited `.wat` is committed by this stone.

| `wat/rete/compile.wat:579-580` | `binary(rete)` |
|---|---|
| fully converted | ⛔ **514 / 8 failed** |
| both markers put back to the keyword spelling | ✅ **522 / 0** |
| `quasiquote` **SYMBOL**, `unquote` keyword | ✅ **522 / 0** |
| `quasiquote` keyword, `unquote` **SYMBOL** | ⛔ **514 / 8 failed** |

⭐ **THE DISCRIMINATING CHANGE IS ONE TOKEN: the `unquote`.** The `quasiquote` HEAD is fine — it
reaches `eval_quasiquote` through the special-form dispatcher, which already resolves both
spellings. The `unquote` INSIDE the template is read by `src/runtime.rs::match_qq_head`, which was
**keyword-only**:

```rust
fn match_qq_head<'a>(items: &'a [WatAST], head: &str) -> Option<&'a WatAST> {
    if items.len() != 2 { return None; }
    if let WatAST::Keyword(k, _) = &items[0] {          // ⛔ Keyword ONLY
        if k == head { return Some(&items[1]); }
    }
    None
}
```

So the unquote never fired, `walk_quasiquote` kept `(wat.core/unquote acc-hd)` **verbatim** as a
sub-list, and `fence-call`'s head became a **LIST** — which is exactly what
`'<non-keyword/symbol head>'` is (`rete/purity.rs::classify_expr`'s general list arm names a head
node that is neither keyword nor symbol that way).

⛔ **`match_qq_head` IS NOT IN THE HERESY LEDGER AND CANNOT BE**: its comparison is `k == head`
where `head` is a **parameter**, and the keyword literal lives at the CALL SITE
(`match_qq_head(items, ":wat::core::unquote")`). A decision with no literal in the deciding
expression is invisible to the discriminator (255.13 §2.3's blind spots, in a shape that list does
not name). The ledger could never have pointed here.

---

## 2. ⭐ QUESTION ONE — IS `primitive?` SPELLING-SENSITIVE? **YES, AT EXACTLY TWO PLACES**

The brief: *"The orchestrator probed it directly and got `false` for all four inputs — including
`(:wat::core::+ 1 2)`, which should be a primitive. The probe's argument shape is therefore wrong…
Re-probe it properly; that is question one."*

⚠ **Two things about that sentence.** The argument-shape diagnosis is right —
`:wat::rete::primitive?` takes `expr <- :wat::WatAST`, so the form must arrive **QUOTED**; an
unquoted form is EVALUATED and the predicate then classifies a value, not a form. ⛔ But the
EXPECTATION in it is wrong: `(:wat::core::+ 1 2)` is core-spelled computation and Law A must refuse
it. `false` is the correct answer for that row, in both spellings.

The re-probe is a committed, durable instrument:
`wat-scripts/scratch-pad/251-8d-ii-vii-is-primitive-spelling-sensitive.wat` (the repo's `.wat`
scratch convention). Each row is a PAIR — the same form in both spellings — so a disagreeing pair
is a spelling-sensitivity whatever the verdict, and an agreeing pair is not.

| row | PRE-CURE | POST-CURE | reading |
|---|---|---|---|
| A `(:wat::rete::core::cond (true 1) (:else 2))` | true | true | |
| B `(wat.rete.core/cond (true 1) (:else 2))` | ⛔ **false** | ⛔ **false** | ⛔ **SPELLING-SENSITIVE — still. §9.1** |
| C `(:wat::rete::i64::+ 1 2)` | true | true | |
| D `(wat.rete.i64/+ 1 2)` | true | true | agnostic — `classify_expr`'s general arm already takes the door |
| E `(:wat::i64::+ 1 2)` | false | false | ⛔ **LAW A holds** |
| F `(wat.i64/+ 1 2)` | false | false | ⛔ **LAW A holds in the other spelling** |
| G `(:wat::core::cond (true 1) (:else 2))` | false | false | ⛔ **LAW A holds** |
| H `(wat.core/cond (true 1) (:else 2))` | false | false | ⛔ **LAW A holds** |
| I `(:wat::core::kwargs-construct :p7::R :x 1)` | true | true | |
| J `(wat.core/kwargs-construct :p7::R :x 1)` | ⛔ **false** | ⭐ **true** | ⭐ **the VERB was spelling-sensitive** |
| K `(:wat::core::kwargs-construct p7/R :x 1)` | ⛔ **false** | ⭐ **true** | ⭐ **the TYPE was too — a SECOND read** |
| L `(:p7::R :x 1)` (surface ctor) | true | true | agnostic — `constructor_meta` is reached canonically |
| M `(p7/R :x 1)` | true | true | |
| N `(:wat::core::kwargs-construct :p7::NotAType :x 1)` | false | ⭐ **false** | ⛔ **THE TIGHTNESS CONTROL — unmoved** |

⭐ **The answer: `primitive?` is spelling-agnostic on ordinary call heads (C/D) and on surface
constructors (L/M), and was spelling-SENSITIVE on the declaration-derived construction door (I/J/K)
and on the core-structural-guard arm (A/B).** The ledger's candidate,
`rete/purity.rs::is_declaration_derived_construction` (2 sites, shape A), is therefore
**CONFIRMED BY MEASUREMENT, not adopted on the brief's word** — and it is confirmed for a
different reason than the brief supposed (§3.1).

---

## 3. ⭐ THE CURES — three reads, one language fact, every `Keyword` arm byte-identical

### 3.1 ⛔⛔ THE ERROR MESSAGE LIED, AND THREE BRIEFS BELIEVED IT

The sixth draw's five NEW all carried:

```
compile-condition: then expr is not a rete primitive —
  ':wat::core::kwargs-construct' is not a rete primitive; a then admits only :wat::rete:: ops
```

⛔ **The keyword spelling in that message is the ERROR's, not the SOURCE's.** When
`is_declaration_derived_construction` declines, the walk falls to `classify_expr`'s general list
arm, which **already** re-spells a `Symbol` head through `canonical_identity` before it builds the
`AxisViolation`:

```rust
Some(WatAST::Symbol(id, _)) => Cow::Owned(crate::edn::render::canonical_identity(id.as_str())),
…
head_ok(&head, axis, …)?      // and AxisViolation::at(at, head, axis) carries THAT string
```

So a `:then` item written `(wat.core/kwargs-construct :T …)` is refused **under the name
`:wat::core::kwargs-construct`**. The brief's class table — *"occurrences naming
`:wat::core::kwargs-construct`: 141"* — counts a keyword-spelled name that no source file contains.

⭐ **Where the symbol spelling comes from, measured:**
`grep -rn "kwargs-construct" wat/**/*.wat` → **three** templates, all inside a quasiquote:

```
wat/Record.wat:207    `(:wat::core::kwargs-construct ~_kc-type ~@call-args)
wat/Record.wat:296    `(:wat::core::kwargs-construct ~_kc-type ~@call-args)
wat/core.wat:2078     `(:wat::core::kwargs-construct ~_kc-type ~@call-args)
```

Converted, all three emit `(wat.core/kwargs-construct …)` — a `Symbol` head. That is why **every**
`:then` holding a nested constructor joined the fence class at once, and why the sixth draw's cure
to `expand_make_rule_then` (which re-armed the `:then` expansion) surfaced exactly five of them.

⚠ Note the `~` reader sugar in those templates is NOT affected: the reader builds an unquote node
itself, as a `Keyword`. The codemod only rewrites the **explicitly spelled**
`(:wat::core::unquote …)`, which is why §1's defect has exactly ONE site in the whole stdlib.

### 3.2 The three doors

| # | site | before | after |
|---|---|---|---|
| 1 | `src/runtime.rs::match_qq_head` | `if let WatAST::Keyword(k,_) = &items[0]` | a `Symbol` arm through **`canonical_identity`** (255.13 door SEED) |
| 2 | `src/rete/purity.rs::is_declaration_derived_construction` | `let Some(WatAST::Keyword(head,_))` **and** `let Some(WatAST::Keyword(type_name,_))` | both through **`canonical_identity_of`** (255.13 door member) |
| 3 | `src/rete/expr_ir/mod.rs::lower_list` | head `Some(WatAST::Keyword(k,_)) => k.as_str()`, everything else refused; and the constructor's `type_kw` the same | both through **`canonical_identity`**, as a `Cow` so the `Keyword` arm BORROWS |

⭐ **Door 1 covers three markers with one edit.** `unquote`, `quasiquote` (the nested-depth case)
and `unquote-splicing` all route through `match_qq_head` / `match_qq_head_named`.

⭐ **Every `Keyword` arm is byte-identical** — a `Keyword` payload IS the internal identity
(255.13's DUAL-ARM RULE), so only the `Symbol` arm needs a door and not one keyword-spelled program
changes or allocates.

### 3.3 ⛔ WHY DOOR 3 IS IN THIS STONE, AND IT IS NOT SCOPE CREEP — IT IS MEASURED

Door 2 alone does **not** make a converted `:then` compile. It moves the error ONE FRAME. Three
binaries, the same probe entry (`:user::sym-ctor`):

| binary | `:user::sym-ctor` |
|---|---|
| HEAD | ⛔ `then expr is not a rete primitive — ':wat::core::kwargs-construct'` — **the FENCE** |
| door 2 alone | ⛔ `malformed :wat::rete::lower form: call head must be a keyword` — **the LOWERING** |
| doors 2+3 | ✅ **derives 1** |

⛔ **The fence and the lowering have to agree about what a head IS, or the `:then` surface has two
answers to one question.** And door 3's refusal was never a WALL: `lower_list`'s very next line is
`resolve_core_name(head)`, and `rete_op_index(head)` keys THE ONE TABLE — everything downstream of
that read already wants the internal identity. A head no arm claims still falls through to the
dispatcher's own by-name refusal at the end of the function. The sixth draw's §10.4 called this
*"the next class … bigger than a door"*; measured, the `lower_list` half of it **is** a door, and
the brief's out-of-scope list does not name it.

---

## 4. ⭐ THE NON-VACUITY ROWS

### 4.1 Tier A — the `:then` surface, `tests/rete/probe_arc251_8d_then_item_identity.{rs,wat}`

Six hand-written `:wat::rete::make-rule` calls with a **nested** constructor in the `:then` item's
operand — exactly what a converted `wat/Record.wat` template emits, writeable by hand on an
UNCONVERTED tree, so this is a two-binary probe with no 22-minute conversion in it. All six rows
are in **ONE** test.

| entry | want | PRE-CURE | POST | what it pins |
|---|---|---|---|---|
| `kw-ctor` | 1 | ✅ 1 | ✅ 1 | ⛔ control — the keyword spelling may not move |
| ⭐ `sym-ctor` | 1 | ⛔ **REFUSED** | ✅ **1** | **row 1** — a legitimate constructor, symbol VERB |
| ⭐ `sym-type-ctor` | 1 | ⛔ **REFUSED** | ✅ **1** | **row 1** — symbol TYPE (a second read) |
| ⭐ `sym-head-and-type` | 1 | ⛔ **REFUSED** | ✅ **1** | both at once — what a converted template emits |
| ⛔ `kw-computation` | REFUSED | ✅ refused | ✅ refused | **row 2** — LAW A, keyword |
| ⛔ `sym-computation` | REFUSED | ✅ refused | ✅ refused | ⭐ **row 2** — LAW A, the OTHER spelling, SAME located reason |

Pre-cure, verbatim (`src/rete/purity.rs`, `src/rete/expr_ir/mod.rs`, `src/runtime.rs` stashed to
`HEAD`, rebuilt — 22.03 s):

```
the :then item fence answered wrongly on 3 row(s):
  :user::sym-ctor → REFUSED, want 1 derived — THE CURE: a symbol-spelled kwargs-construct VERB
      compile-condition: then expr is not a rete primitive — ':wat::core::kwargs-construct' is not a rete primitive; a then admits only :wat::rete:: ops
  :user::sym-type-ctor → REFUSED, want 1 derived — THE CURE: a symbol-spelled TYPE in argument 0
      compile-condition: then expr is not a rete primitive — ':wat::core::kwargs-construct' is not a rete primitive; a then admits only :wat::rete:: ops
  :user::sym-head-and-type → REFUSED, want 1 derived — THE CURE: both at once — what a converted Record.wat template emits
      compile-condition: then expr is not a rete primitive — ':wat::core::kwargs-construct' is not a rete primitive; a then admits only :wat::rete:: ops
```

⛔ **THE TWO LAW-A ROWS ARE THE STONE, AND THEY ARE GREEN ON BOTH BINARIES** — which is what makes
them a test of the WALL rather than a test of the cure (255.12's discipline). The refusal both
spellings receive is byte-for-byte the same and names the head:

```
compile-condition: then expr is not a rete primitive — ':wat::core::if' is not a rete primitive;
  a then admits only :wat::rete:: ops      wat/rete/compile.wat:797
```

⭐ The item's computation is `(:wat::core::if (:wat::core::not false) 1 2)` / `(… (wat.core/not
false) …)` — chosen by measurement, not by guess: `(:wat::core::not true)` and `(wat.core/not true)`
are **pure ∧ deterministic ∧ total** in both spellings and `primitive?`-FALSE in both, so the
refusal can only be Law A, never Total. (Measured with a four-axis probe before the row was
written.)

### 4.2 Tier B — the door's own tightness, `src/rete/purity.rs` `mod declaration_derived_identity_tests`

⛔ **These cannot be written at the wat surface, and that is measured.** A `:then` operand naming an
undeclared type is refused EARLIER, at freeze, by `rete/validate` — which kills the whole fixture
rather than one row:

```
#wat.rete/RhsOperandTypeMismatch  defrule `t7::r`: `:then` insert of `:t7::Out` field `:inner`
  is declared `:t7::Inner`; operand is `:t7::NotAType`
```

⭐ **And its symbol-spelled twin (`wat.core/kwargs-construct` over `t8/NotAType`) was refused under
the CANONICAL name `:t8::NotAType`** — `rete/validate` already reads both spellings, which is part
of why this door's blindness stayed invisible.

Ten rows, one test, on the struct — never on a rendered string:

| row | PRE | POST |
|---|---|---|
| control — keyword verb, keyword type | ✅ | ✅ |
| ⭐ symbol verb | ❌ | ✅ |
| ⭐ symbol type | ❌ | ✅ |
| ⭐ both, and `aggregate-new` takes the same door | ❌ | ✅ |
| ⛔ undeclared type, keyword | ✅ refused | ✅ refused |
| ⛔ undeclared type, symbol | ✅ refused | ✅ refused |
| ⛔ NEAR-MISS verb (`kwargs-constructx`) over a REAL type, keyword | ✅ refused | ✅ refused |
| ⛔ NEAR-MISS verb over a real type, symbol | ✅ refused | ✅ refused |
| ⛔ no type argument at all | ✅ refused | ✅ refused |
| ⛔ argument 0 is not a name (an `IntLit`) | ✅ refused | ✅ refused |

Pre-cure, verbatim — ⚠ obtained by reverting **the function body alone** and keeping the test
module, because the module lives in the same file as the cure and a `git stash` of that file takes
the rows with it:

```
the declaration-derived door answered wrongly on 3 row(s):
  THE CURE — symbol verb: got false, want true
  THE CURE — symbol type: got false, want true
  THE CURE — both, and `aggregate-new` takes the same door: got false, want true
```

⭐ **Six of the ten rows are green on BOTH binaries.** The cure widened the SPELLING, never the
POPULATION.

### 4.3 Tier C — the quasiquote markers, `tests/rete/probe_arc251_8d_unquote_marker_identity.{rs,wat}`

Two tiers in one test, because neither alone is enough: tier 1 asserts the two spellings render
IDENTICALLY (both sides from the fixture — the `.rs` holds no expected wat text of its own, §6.1),
tier 2 asserts the ABSOLUTE property in the fixture, in wat.

| row | PRE-CURE | POST |
|---|---|---|
| `qq-kw-unquote` vs ⭐ `qq-sym-unquote` render as one form | ⛔ `(:foo/bar __acc__)` vs `((wat.core/unquote :foo/bar) __acc__)` | ✅ |
| `qq-kw-splice` vs ⭐ `qq-sym-splice` render as one form | ⛔ `(:head 1 2)` vs `(:head (wat.core/unquote-splicing [1 2]))` | ✅ |
| `kw-unquote-fired` | ✅ | ✅ |
| ⭐ `sym-unquote-fired` | ⛔ **false** | ✅ **true** |
| `kw-splice-fired` | ✅ | ✅ |
| ⭐ `sym-splice-fired` | ⛔ **false** | ✅ **true** |
| ⛔ `kw-not-a-marker-stayed-data` (`unquotex`) | ✅ | ✅ |
| ⛔ `sym-not-a-marker-stayed-data` | ✅ | ✅ |

⛔ **The last two rows are the wall**: `unquotex` is ONE LETTER from `unquote` and must stay inside
the template in both spellings. The cure fires on the MARKER, never on "symbol-headed".

---

## 5. ⭐ THE LEDGER — 223 → 220, shape mix, and NO re-anchoring was needed

`cargo nextest run --release -E 'test(keyword_heresy)'` — **4 / 4 green** at the new freeze
(1.14 s), on the post-cure binary.

### 5.1 The ratchet's own output, verbatim (before I tightened it)

```
⭐ THE LEDGER SHRANK — 223 → 220. This is the good direction, and the
ratchet is deliberate: the frozen census below has to be tightened by hand so the number
can never drift back up silently. Update `LEDGER_TOTAL` and the rows named here.

CURED:
  (none)
GONE ENTIRELY:
  − src/rete/expr_ir/mod.rs  fn lower_list  was 1 [Ax1], now 0
  − src/rete/purity.rs  fn is_declaration_derived_construction  was 2 [Ax2], now 0
```

### 5.2 ⛔ THE SHAPE MIX, NOT JUST THE COUNT

Computed over every `FROZEN_LEDGER` row, mechanically, from the table itself:

| | sixth draw | **this draw** |
|---|---|---|
| **A** — keyword-only | 97 | ⭐ **94** |
| ⛔ **B** — dual-raw | 1 | **1** |
| **C** — symbol-raw | 0 | 0 |
| **E** — type-path raw | 125 | **125** |
| **LEDGER_TOTAL** | 223 | ⭐ **220** |
| frozen rows | 104 | **102** |

⭐ **All three are shape A, which is the direction the terminal cut needs.** ⛔ **Shape E did not
move and is now 57% of the ledger** (125 sites, 78 in `check.rs`).

### 5.3 ⭐ NO RE-ANCHORING — and that is a result, not an omission

The brief: *"Shape B is down to ONE site (`purity.rs::walk_rete_defn_callees`), which currently
carries the calibration anchor — if you cure it, re-anchor per the note already written into the
test; do NOT delete the assertion."*

⭐ **I did not cure it, so the anchor stands untouched and
`the_discriminator_separates_the_cured_from_the_open` is green with its shape-B assertion
unchanged.** Curing a rete-defn CYCLE DETECTOR widens what it recurses into — permissive, on a
default-deny walk, with no fixture — and it is the fifth draw's deliberate deferral. This is the
fourth consecutive permissive cure; it is not the stone on which to make it a fifth.

⭐ **Three new CURED rows**, keyed per DECISION as 255.13 requires, so a revert of this stone goes
red as *"THE DISCRIMINATOR CONVICTED A SITE THIS ARC ALREADY CURED"*:
`is_declaration_derived_construction` × `:wat::core::kwargs-construct`,
`is_declaration_derived_construction` × `:wat::core::aggregate-new`,
`lower_list` × `:wat::holon::literal`.

⛔ **`match_qq_head` gets no CURED row, because it never had a ledger row to cure** (§1.2). A cure
the instrument cannot see is a cure a revert will not be caught by — stated, not papered over.

---

## 6. EVERY GATE ON THE LANDABLE STATE, WITH ITS EVIDENCE

| gate | landable state (`wat/` untouched) |
|---|---|
| `cargo build --release` after every `src/` edit | ✅ exit 0, 21.9–22.5 s each |
| ⭐ **`scripts/floor.sh`** | ✅ **GREEN — 6011 / 6011**, 9 slow, 22 skipped, **322.846 s**, exit 0 · `.floor/2026-09-23T02-06-59Z/` · ⭐ **AND AGAIN ON THE COMMITTED TREE — 6011 / 6011, 323.896 s, exit 0 · `.floor/2026-09-23T02-54-37Z/`**, which is the run the docs-reading lints could see this SCORE in (255.13 §8.5's discipline) |
| floor denominator | 6008 (sixth draw) **+ 3 of mine** (2 probes + 1 unit row) = **6011** ✓ |
| ⚠ floor duration | **322.8 s** — the healthy band (the sixth draw's 327.9 / 322.4 s). ⚠ *A fast floor is a symptom*: `every_wat_scripts_file_loads_on_the_current_runtime` is `SLOW [>300.000s]` on its own and it ran (PASS, 322.840 s) |
| ⚠ `harvest_wrap_split` | **PASS** [0.159 s] (1572/6011), and **PASS** [0.119 s] (1572/6011) on the post-commit floor — named either way, per the standing bookkeeping exception (`278-rules-engine/FINDING-harvest-cost-is-not-a-gate.md`) |
| clippy `-D warnings --all-targets --workspace` | ✅ **0 warnings, rc 0** — and **NOT cached**: `touch src/rete/expr_ir/mod.rs src/runtime.rs` first, **12.93 s** elapsed |
| census | ✅ **`census-diff: no STOP-8`, rc 0** · pre-commit `.census/2026-09-23T02-13-26Z.txt` (2211 paths) and ⭐ **post-commit `.census/2026-09-23T02-53-46Z.txt` (2214 paths — my 3 new tracked `.wat`)**, both **213 failing = 213 failing** vs the sixth draw's landable `.census/2026-09-23T00-14-46Z.txt` |
| ⭐ `scripts/replay/delta.sh` | ✅ **NEW 3** = baseline 3 (`probe-c1-clean-surface`, `probe-kwargs-peer`, `wat/holon/Ngram.wat`), ⭐ **RECOVERY 0**, exit 0 · ORIG-CLEAN 160/179, CONV-CLEAN 157/179 · list-sha `33ede76c…f1cbc3e5`, paths=179 missing=0 · `.delta/2026-09-23T02-14-25Z/` |
| ⭐ ledger | ✅ **4 / 4 green at 220** |
| `wat/` clean at commit | ✅ `git status --porcelain` named no `wat/` path (§7.5) |

### 6.1 ⛔ THE FIRST FLOOR WENT RED — TWO LINTS, BOTH MINE, AND THE IRONY IS EXACT

⛔ **Not re-run. Captured whole from `.floor/2026-09-23T01-54-39Z/ARM.txt`.**

```
     Summary [ 321.874s] 6011 tests run: 6009 passed (9 slow), 2 failed, 22 skipped
        FAIL [   0.055s] ( 144/6011) wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat
        FAIL [   0.101s] ( 143/6011) wat::lint no_inlined_edn::tests_carry_no_inlined_edn
```

**(a)** `no_inlined_edn`, the arm:

```
🔥🔥🔥 INLINED-EDN — 2 site(s) carry a string literal whose content opens with
`#`/`{`/`[`/`(` (EDN-esque, trimmed of leading whitespace).
…
Offenders:

tests/rete/probe_arc251_8d_unquote_marker_identity.rs:82
tests/rete/probe_arc251_8d_unquote_marker_identity.rs:87
```

**(b)** `no_inlined_wat_in_tests`, the arm:

```
🔥🔥🔥 INLINED-WAT IN TESTS — 2 file(s) still carry a string literal that wat's own
reader parses as a form (surface-agnostic: rust-scheme `(:wat::core::…)` AND faithful
Clojure `(wat.core/…)` both count).
…
Drive it to ZERO. Literal-hit breakdown so far: 0 format!-driver, 0 faithful-surface,
5 other parse-body. Offenders:

tests/rete/probe_arc251_8d_then_item_identity.rs
tests/rete/probe_arc251_8d_unquote_marker_identity.rs
```

⭐ **The exact offenders, and the second one is the better find.**

- `unquote_marker_identity.rs` held the expected RENDERINGS as Rust string literals
  (`"(:foo/bar __acc__)"`, `"(:head 1 2)"`, …). Cured per the lint's own rubric — *restructure the
  CODE* — by moving the ABSOLUTE claims into the fixture as wat predicates and having the `.rs`
  compare the two spellings' renderings **to each other**, holding no expected wat text at all.
- ⭐ `then_item_identity.rs` held **`const NAMED: &str = "':wat::core::if'";`** — and `'` is the
  reader's QUOTE SUGAR, so that literal parses as `(:wat::core::quote :wat::core::if')`, a list
  with a keyword head. The gate was right and I did not see it: **this stone is about a name that
  reads two ways, and my own fixture wrote a form while meaning a name.** Cured to the bare
  identity `":wat::core::if"`, which is not a list and pins the same fact.

Both gates green on the next build; the floor in §6 is a **different tree**, not a re-run.

---

## 7. ⭐ THE CONVERSION — 161 → 112, a STRICT SUBSET, and it does NOT land

⛔ **Operational discipline, stated because the arc has paid for its absence:** the 64 paths came
from `git ls-files | grep -E '^wat/.*\.wat$'` (⚠ **not** `git ls-files 'wat/**/*.wat'`, which
returns 31 of 64), were passed as one explicit EDN vector, and **no `cargo` command and no
`git add` ran while the codemod was writing `wat/`** — the codemod ran detached and every later
step waited on a `DONE` line it writes itself, never on a process probe.

### 7.1 The numbers

| | sixth draw | **this draw** |
|---|---|---|
| conversion | 64/64, rc 0 | ✅ **64/64, rc 0** (`CONVERSION_RC=0`), 22 m 37 s; `git status` = exactly 64 `wat/` paths and nothing else |
| `cargo build --release` on the converted tree | 23.14 s | ✅ **exit 0, 22.89 s** |
| binary starts | ✅ | ✅ (`--check` of my own fixture: rc 0) |
| ⭐ **floor** | ❌ **161 / 6008**, 456.687 s | ❌ ⭐ **112 / 6011**, **459.531 s**, exit 100 · `.floor/2026-09-23T02-38-37Z/` |
| ⚠ `harvest_wrap_split` | PASS | **PASS** [0.181 s] (1573/6011) — named either way |

```
     Summary [ 459.531s] 6011 tests run: 5899 passed (19 slow), 112 failed, 22 skipped
```

⚠ 459.5 s is the *converted-but-red* band (the sixth draw's was 456.7 s), not the 24 s
"stdlib-did-not-load" symptom.

### 7.2 ⭐ FIXED / NEW — **49 fixed, 0 NEW, a strict subset**

`comm` over the two sorted failing-test-NAME sets, extracted with a parser that strips the
**entire** `FAIL [   0.836s] (1523/6002) ` prefix including a padded index. ⭐ **The control that
says the parser is right: it reproduces the sixth draw's 161 exactly from its own `clean.log`.**

```
only in the SIXTH draw's 161  (i.e. FIXED here) : 49
only in mine                  (i.e. NEW)        :  0
```

⭐ **AND THE CLASS THE BRIEF WAS DRAWN FOR IS GONE, NOT REDUCED.** Occurrence counts over the whole
converted `clean.log`, the same grep on both logs (⚠ `clean.log`, never `ARM.txt`, which repeats
each failure — this is also why my count of the sixth draw's `kwargs-construct` occurrences is 116
where the brief's was 141):

| verb | sixth draw's converted log | **mine** |
|---|---|---|
| `is not a rete primitive` | **117** | ⭐ **0** |
| `:wat::core::kwargs-construct` | 116 | ⭐ **0** |
| `'<non-keyword/symbol head>'` | 15 | ⭐ **0** |
| `malformed :wat::rete::lower form` | 2 | ⭐ **0** |

### 7.3 ⭐⭐ THE SIXTH DRAW'S FIVE NEW — ALL PASS, AND THE MECHANISM IS ISOLATED

The brief: *"If your cure makes them pass, say WHICH mechanism did it — a wall re-disarmed looks
identical to a bug fixed."*

All five pass. **Three independent pieces of evidence that it is the DOOR, not a disarmed wall:**

1. ⭐ **A TWO-BINARY EXPERIMENT ON THE CONVERTED TREE.** `src/rete/purity.rs` and
   `src/rete/expr_ir/mod.rs` reverted (keeping `runtime.rs` cured), rebuilt in place — **all five
   fail again**, with the sixth-and-only-known arm:

   | reverted, converted tree | the 5 |
   |---|---|
   | my two rete doors OFF | ⛔ **all 5 FAIL** (`25 tests run: 19 passed, 6 failed`; the 6th is `a_not_knowable_nested_then_operand…`, already in the sixth draw's 161) |
   | my two rete doors ON | ✅ **all 5 PASS** |

2. ⛔ **`src/macros/expand.rs` AND `src/resolve/` CARRY ZERO DIFF FROM `HEAD`** — `git diff --stat`
   on both is empty. The sixth draw's `expand_make_rule_then` cure, the thing that re-armed the
   `:then` expansion, is untouched by this stone.

3. ⭐ **THE WALL IS DEMONSTRABLY STILL ARMED UNDER CONVERSION.** All three of this stone's probes
   — including tier A's two LAW-A rows, which require a core-spelled computation in a `:then` to be
   REFUSED in both spellings — pass **on the converted tree**:

   ```
   Summary [   4.348s] 3 tests run: 3 passed, 6030 skipped
   ```

   A disarmed `:then` expansion would make those rows pass for the wrong reason only if the item
   were never expanded at all; but the five that now pass assert **derivation**, not the absence of
   a refusal — `…still_compiles_and_fires`, `…now_works_via_oracle`. An unexpanded nested
   constructor does not fire.

⭐ **So the mechanism is exactly the one the sixth draw's §10.1 asked for:**
`wat/rete/compile.wat:797`'s `then-item-fence` refused `kwargs-construct` on a CONVERTED tree and
not on an unconverted one because the converted `wat/Record.wat` template emits a **`Symbol`** head
and `is_declaration_derived_construction` read a **`Keyword`** payload only. The ledger's candidate
was right; the reason it was invisible is that the refusal printed the canonical spelling (§3.1).

### 7.4 The 112 that remain — CLASSIFIED, NOT DIAGNOSED

⛔ **Stated plainly, as the fourth, fifth and sixth draws did: I bucketed every failing block by the
error its stderr carries. I diagnosed the five (§7.3) and none of the other 107, and the classes may
interact.** All 112 have a body block in `ARM.txt`; the classifier reads that block, one block per
distinct test name.

| n | class |
|---|---|
| 47 | a bare assertion / panic with no typed wat error |
| 33 | `#wat.resolve/UnresolvedReferences` |
| 11 | `#wat.check/CheckErrors` (other — arity, unknown name, …) |
| 11 | `#wat.runtime/MalformedForm` |
| 8 | `#wat.check/TypeMismatch` |
| 2 | `#wat.kernel/AssertionFailure` (other) |
| **112** | |

⭐ **There is no longer a rete-fence class at all.** The sixth draw's largest TYPED class (38 fence
refusals) is zero, and `UnresolvedReferences` — 33 then, 33 now — is untouched and is now the
largest typed class. That is where the eighth draw's single-file loop should be pointed.

### 7.5 ⛔ THE CONVERSION IS RESTORED

```
git checkout -- wat/    →  git status --porcelain | grep '^ M wat/' | wc -l   →  0
cargo build --release   →  exit 0, 23.22 s
-E 'test(then_item_identity) or test(unquote_marker_identity) or
    test(declaration_derived_identity) or test(keyword_heresy)'
                        →  7 / 7 green on the restored binary
```

---

## 8. ⛔ WHAT THE BRIEF GOT WRONG — four, and the first is the twenty-fifth

1. ⭐⭐ **THE NAMED TEST DOES NOT FAIL.** The brief's single strongest claim — starred twice,
   *"THE ADDRESS WAS ISOLATED BY EXPERIMENT, NOT INFERRED … ONE FILE REPRODUCES IT"* — is a table
   with two rows, and the second row is wrong.
   `probe_arc278_then_is_an_expansion_boundary::both_fact_form_shapes_expand_their_values` **passes**
   on a tree with `wat/rete/compile.wat` converted alone, together with the other three tests in its
   file (§1.1, and the converted stdlib is verified embedded in the binary that ran them).
   ⭐ **The METHOD is sound and I used it** — converting that one file DOES reproduce eight
   `binary(rete)` failures — but the brief recorded the wrong one, and the difference is not
   cosmetic: it sent the stone to the wrong FENCE, the wrong CONJUNCT and the wrong PREDICATE.
   This is the twenty-fifth correction, and it is a new species: the first three convicted the
   wrong address by INFERENCE; this one convicted it by a measurement that does not reproduce.
2. ⛔ **THE FENCE IS THE ACCUMULATOR'S, NOT THE `:then` ITEM'S.** The brief names
   `wat/rete/compile.wat:795` (`is-rete (:wat::rete::primitive? item)`) and *"twins at 461 and
   607"*. The single-file failure is at **609** — the accumulator fence's `is-pure`, the FIRST
   conjunct — and its mechanism is `runtime.rs::match_qq_head`'s keyword-only read of the
   `unquote` MARKER inside the fence's own quasiquote template (§1.2). `primitive?` is not
   consulted: by the time the fence runs, `fence-call` has a LIST where a head belongs and fails
   purity.
3. ⛔ **"OCCURRENCES NAMING `:wat::core::kwargs-construct`: 141" COUNTS A SPELLING NO SOURCE FILE
   CONTAINS.** The class is real; the name in it is the ERROR's. `classify_expr`'s general list arm
   re-spells a `Symbol` head through `canonical_identity` before building the `AxisViolation`, so a
   `(wat.core/kwargs-construct …)` refusal prints as `':wat::core::kwargs-construct'` (§3.1). The
   three emitting templates are in `wat/Record.wat` and `wat/core.wat`, and all three emit a symbol
   head under conversion. ⭐ This is why the fence "changed its mind" about a form that looked
   keyword-spelled on both sides — it never saw a keyword-spelled form at all.
4. ⚠ **QUESTION ONE'S EXPECTATION IS WRONG, THOUGH ITS DIAGNOSIS IS RIGHT.** The brief calls
   `(:wat::core::+ 1 2)` *"one that should have passed"*. Law A exists to refuse core-spelled
   computation; `false` is the CORRECT answer, and it is `false` in both spellings on both binaries
   (§2, rows E/F). What was wrong with the original probe is exactly what the brief says — the
   argument must arrive QUOTED — and re-probing with that fixed produced a 14-row answer.

⭐ **And the brief got two hard things right.** Its ledger CANDIDATE,
`rete/purity.rs::is_declaration_derived_construction`, is the real mechanism of the `:then` fence
class — confirmed by probe, not adopted on its word (§2, rows I/J/K). And its warning about the
shape-B calibration anchor was correctly aimed: I did not cure `walk_rete_defn_callees`, so the
anchor stands and the assertion is untouched (§5.3).

---

## 9. ⛔ WHAT MY GREEN CANNOT SEE

1. ⛔ **THE CORE-STRUCTURAL-GUARD ARM IS STILL KEYWORD-ONLY, AND I MEASURED IT AND LEFT IT.**
   `primitive?` row B: `(wat.rete.core/cond (true 1) (:else 2))` is **false** on both binaries,
   while its keyword twin is **true**. `classify_expr`'s guard is
   `matches!(items.first(), Some(WatAST::Keyword(k,_)) if … resolve_core_name(k) …)` — a
   `Symbol`-headed `cond`/`match`/`fn` never reaches it. That is the REFUSING direction (safe
   today), but it means a converted corpus's rete-spelled `cond` inside a `where` or a `:then` is
   refused. It is `classify_expr`'s own ledger row (Ax3) and the calibration's OPEN table names it;
   curing it is a widening of a default-deny arm and is not this stone's.
2. ⛔ **`match_qq_head`'s CURE HAS NO LEDGER ROW AND CANNOT GET ONE** (§1.2). A revert of it will not
   be caught by the discriminator — only by tier C's probe.
3. ⛔ **THE OTHER TWO QUASIQUOTE WALKERS ARE UNTOUCHED.** `walk_quasiquote` carries a
   `rune:solvere(load-bearing-coupling)` saying the depth walk is mirrored in THREE sites
   (`walk_template`, `validate_quasiquote_template`, `walk_quasiquote`). I changed head RECOGNITION
   in one of them. `macros/eval.rs::validate_quasiquote_template` (Ax3) and
   `macros/expand.rs::is_quasiquote_form` (Ax1) are still keyword-only — the brief puts
   `is_quasiquote_form` out of scope — so a `defmacro` body written with an explicitly
   symbol-spelled `unquote` would be VALIDATED and EXPANDED by one rule and EVALUATED by another.
   I did not construct that case and I do not claim it is unreachable.
4. ⛔ **DOOR 3 IS ON TWO READS, NOT ON THE FILE.** `lower_list`'s head and the constructor's type
   argument take the identity door; `lower_expr`'s bare-`Keyword` arm (the zero-arg rete-defn call)
   does not, and I did not census the rest of `expr_ir`.
5. ⛔ **THE THREE TIERS ARE HAND-WRITTEN SHAPES, NOT A CENSUS.** Tier A writes the shape a converted
   `Record.wat` template emits; it does not prove that is the only shape a converted corpus emits.
6. ⛔ **THE ADVERSARIAL SEARCH IS SEVEN CONSTRUCTED REFUSALS AND TWO NEAR-MISS MARKERS.** I could
   not construct a name that `canonical_identity` re-spells INTO an admitted set without already
   being in it; that is a statement about my search, not a proof.
7. ⛔ **THE LEDGER'S OWN BLIND SPOTS ARE UNCHANGED** (255.13 §2.3): reachability, a keyword-only
   READ feeding a covered decision, the 68 `D-unresolved` sites, macro-generated code, anything
   outside `src/`. My 220 inherits every one — and §1.2 adds a shape that list does not name: a
   decision whose literal lives at the CALL SITE.
8. ⛔ **THE CENSUS GATE COULD NOT SEE MY NEW FIXTURES UNTIL THEY WERE COMMITTED** — `census.sh`
   reads tracked files, so the run in §6 has 2211 paths and the post-commit tree has more.
9. ⛔ **`delta.sh`'s RECOVERY COLUMN IS ONLY MEANINGFUL AGAINST AN UNCONVERTED EMBEDDED STDLIB**
   (the fourth draw proved that). My delta row is the landable one.

10. ⛔ **THE 112 ARE CLASSIFIED, NOT DIAGNOSED** (§7.4). 47 of them carry no typed wat error at
    all, so the classifier cannot say more than "something asserted"; I read five blocks in full.
11. ⛔ **`is_where_form`'s keyword-only guards in all three descents are STILL open** — the sixth
    draw's §9.2, untouched here and named out of scope by the brief. A converted `.wat` CORPUS
    (8d-iii) spells a condition `(wat.rete/where …)` and all three will miss it.
12. ⛔ **`wat/rete/compile.wat:578`'s acc-form fence is still a keyword-only prefix test IN WAT**
    (`starts-with? acc-hd-nm ":wat::rete::acc::"`). My §1 cure fixes the fence's PROBE TEMPLATE,
    not the fence's own head test — a symbol-spelled acc head is still refused before
    `compile_acc_fold` runs (the sixth draw's §1.3 finding stands unchanged). It is a default-deny
    FENCE and it is the builder's call.

---

## 10. WHAT THE EIGHTH DRAW NEEDS (not started here)

1. ⭐⭐ **POINT THE SINGLE-FILE LOOP AT `#wat.resolve/UnresolvedReferences`.** It is 33 of the 112,
   it did not move between the sixth draw and this one, and it is now the largest TYPED class. The
   loop that found this stone's address takes ~90 s per file and there are 64 files.
2. ⭐ **THE 47 UNTYPED FAILURES.** They carry no wat error, so no grep classifies them; each one
   needs its block read. That is the honest cost of the remainder.
3. ⭐ **`classify_expr`'s core-structural-guard arm** (§9.1, `primitive?` row B) — a converted
   corpus's `(wat.rete.core/cond …)` is refused today. It is `classify_expr`'s own ledger row (Ax3)
   and it is in the calibration's OPEN table, so curing it moves the ledger AND needs the OPEN row
   re-keyed.
4. ⭐ **The other two quasiquote walkers** (§9.3) — `validate_quasiquote_template` (Ax3) and
   `is_quasiquote_form` (Ax1). One door was cured here; the rune on `walk_quasiquote` says the
   three must not disagree.
5. `rete/purity.rs::walk_rete_defn_callees` — still the **last** shape-B site and still the
   calibration anchor. Read 255.13 §4.3's note before touching the assertion.
6. Shape **E**, 125 sites, 78 in `check.rs` — untouched, and now **57%** of the ledger. The
   terminal cut needs A+E at zero.

## Artifacts

- `.floor/2026-09-23T01-54-39Z/` — the FIRST landable floor, **RED 2/6011**, 321.874 s, exit 100;
  both failures mine, `ARM.txt` holds them whole
- `.floor/2026-09-23T02-06-59Z/` — the landable floor, **6011 / 6011 green**, 322.846 s, exit 0
- `.census/2026-09-23T02-13-26Z.txt` — landable census, 2211 paths, `no STOP-8` vs the sixth draw's
- `.delta/2026-09-23T02-14-25Z/` — NEW 3 = baseline 3, RECOVERY 0
- `.floor/2026-09-23T02-38-37Z/` — the **converted** floor, **112 / 6011 red**, 459.531 s, exit 100;
  `ARM.txt` holds all 112 whole blocks untruncated
- `.floor/2026-09-23T02-54-37Z/` — the landable floor re-run on the **COMMITTED** tree,
  **6011 / 6011 green**, 323.896 s, exit 0
- `.census/2026-09-23T02-53-46Z.txt` — post-commit census, 2214 paths, `no STOP-8`, 213 = 213
- `tests/rete/probe_arc251_8d_then_item_identity.{wat,rs}` — the 6-row `:then` truth table
- `tests/rete/probe_arc251_8d_unquote_marker_identity.{wat,rs}` — the quasiquote-marker table
- `src/rete/purity.rs` `mod declaration_derived_identity_tests` — the 10 tightness rows
- `wat-scripts/scratch-pad/251-8d-ii-vii-is-primitive-spelling-sensitive.wat` — the 14-row
  `primitive?` spelling probe (question one)
- `tests/lint/keyword_heresy_ledger.rs` — the ledger at **220**, three new CURED rows, the shape-B
  anchor UNTOUCHED
