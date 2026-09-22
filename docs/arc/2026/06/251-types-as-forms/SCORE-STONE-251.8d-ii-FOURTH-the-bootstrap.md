# SCORE — STONE 251.8d-ii (FOURTH DRAW): the bootstrap — **STOP**

Branch `main`, drawn against `214ad3a33`. **The conversion is NOT committed.** `wat/` restored
(`git checkout -- wat/` + rebuild, the fourth time it has been proven). **Not pushed.** 8d-iii not
started. Parent: `BRIEF-STONE-251.8d-ii-FOURTH-the-bootstrap.md`.

Every number below was produced this session. `cargo build --release` was run after **every** `wat/`
or `src/` edit and each row names which binary produced it.

## VERDICT — **STOP.** The one blocking site is CURED and the cure LANDS. The conversion does not.

⭐ **The blocker is gone and the number is exact: 3 747 → 416 failures, and the 416 are the SAME 416
the third draw already measured** — `comm` on the two sorted failure sets is **0 added, 0 removed**.
So this draw did what it was drawn to do (the Gate E site, the self-application proof, idempotence)
and the remaining wall is a *different*, already-named one that no part of this stone touches.

⛔ **The floor is the gate and the floor is RED (416 / 5 989).** `wat/` is restored. What lands is
the **cure alone**: `src/macros/expand.rs` + three fixtures + three test rows, on which the floor is
**5 989 / 5 989 GREEN**, delta **3 / RECOVERY 0**, census **no STOP-8**, clippy **0**.

---

## 1. ⭐ THE ISOLATION — and the brief's mechanism story is WRONG in its load-bearing half

Seven single-factor files, one macro shape, `target/release/wat --check`, pre-cure binary at
`214ad3a33`. Each differs from `pA` in exactly one thing.

| probe | macro body | fn head | marker | type | rc |
|---|---|---|---|---|---|
| **pA** | `(:wat::core::let …)` wrapping the qq | `wat.core/fn` | `:-` | `wat.gen/Coord` | **1 — FIRES** |
| pB | `let`-wrapped | `:wat::core::fn` | `<-` | `:wat::gen::Coord` | 0 |
| pC | **the qq itself** | `wat.core/fn` | `:-` | `wat.gen/Coord` | **0** |
| pD | `let`-wrapped | `wat.core/fn` | `:-` | **`:wat::gen::Coord`** | **0** |
| **pE** | `let`-wrapped | **`:wat::core::fn`** | `:-` | `wat.gen/Coord` | **1 — FIRES** |
| pF | `let`-wrapped | `wat.core/fn` | **`<-`** | `wat.gen/Coord` | 1 |
| pG | `let`-wrapped | `wat.core/fn` | `:-` / `:-` | `wat.gen/Coord` | 1 |

pA, verbatim:

```
#wat.macro/ProgramBodyIntroducesName {:message "macro :user::mk program body refused — quasiquote
template introduces literal name `wat.gen/Coord` in binder position; …" … :binder "wat.gen/Coord"}
```

### ⭐ What `wat/gen.wat:705` has that a five-line equivalent does not — TWO factors, jointly necessary

**(i) REACHABILITY — and this is why the brief's repros returned rc 0.**
`src/macros/parse.rs:269`:

```rust
if !super::expand::is_quasiquote_form(&body_item) {
    super::expand::validate_macro_definition(&body_item, &list_span, &name)?;
}
```

Gate E runs **only for a defmacro body that is NOT itself a quasiquote**. `wat/gen.wat`'s `record`
macro body is a `(:wat::core::let [cv (fresh-symbol "coord") …] `(…))` — a program body, so Gate E
runs and finds the nested template through `quasiquote_inner`. The brief's minimal repro
``` `(wat.core/fn [~t :- m/Coord] :- wat.core/i64 1) ``` is a **bare quasiquote**, which
`is_quasiquote_form` routes **past `validate_macro_definition` entirely**. Gate E was never invoked.
**pC is that repro inside a defmacro and it is still rc 0; pA is the same template under a `let` and
it fires.** This is not an obscure property of the template — it is 255.11's own §4(a), which says
`is_quasiquote_form` is deliberately kept keyword-only *because* teaching it the symbol spelling
would make a quasiquote body **skip** this validation. The brief told me to read that stone and
presented the non-reproduction as a mystery about the template's contents; it is a documented
routing fact.

**(ii) THE DEFECT — a TYPE in a converted param vector is a `Symbol`.**
`src/macros/expand.rs`, the `fn` arm of `check_quasiquote_for_literal_binders`, stepped the params
vector **one item at a time** and refused every `WatAST::Symbol` whose text was not `->`, `<-` or
`&`. In the keyword dialect a type is a `Keyword` (`:wat::gen::Coord`) and this scan never looked at
it. In the faithful-Clojure dialect a type is a `Symbol` (`wat.gen/Coord`) — so the **annotation**
was read as a binder **name**.

### ⛔⛔ IS 255.11 THE CAUSE? **NO — and the twenty-first correction is the brief's own hypothesis.**

The brief: *"255.11 cured Gate E by teaching the binder scan symbol-spelled `let`/`fn` heads. Before
that, a converted `(wat.core/fn …)` inside a template was not recognised as a `fn` form at all, so
its param vector was never walked. Now it is — and the walk treats the post-`:-` type as a name."*

**pE refutes the load-bearing half.** With the **KEYWORD** `fn` head — the spelling
`check_quasiquote_for_literal_binders` has read since arc 249 stone 249.2b-ii, years before 255.11 —
and a **symbol-spelled type**, the gate fires with the identical error and the identical binder. The
defect lives in the arm's own *"every Symbol in a params vector is a binder"* assumption, which was
**latent** in the keyword dialect and which 255.11 never touched.

What 255.11 *did* is make the defect **reachable at this particular site**, because `gen.wat`'s
converted template head is the Symbol `wat.core/fn`. So:

- **255.11 is a necessary enabler for the spelling, not the cause of the class.**
- ⛔ **Reverting or loosening 255.11 would not have fixed it.** It would have re-hidden the same
  latent defect behind a spelling — and re-opened the capture hole 255.11 closed. pE is the proof
  that "loosen Gate E" was never on the table.

⭐ **And the gate's own header already documented the correct rule while the code did something
else** (`src/macros/expand.rs`, Gate E's block comment, unchanged since arc 249):

> `(:wat::core::fn [name <- :T ...] ...)` — param names at positions **0, 3, 6, …** (argspec triples)

Positions 0/3/6 is exactly right. The code stepped by 1. `[[a_comment_can_ship_a_gap_as_a_law]]`.

---

## 2. THE CURE — one arm, one door, no loosening

`src/macros/expand.rs`, `check_quasiquote_for_literal_binders`, the `:wat::core::fn` arm:

```rust
if crate::types::is_param_annotation_arrow(&param_items[i]) {
    // The next item is the TYPE this annotation introduces, never a name.
    i += 2;
    continue;
}
if let WatAST::Symbol(ident, _) = &param_items[i] {
    let s = ident.as_str();
    // Exclude the `->` return arrow and the `&` rest marker.
    if s != "->" && s != "&" { return Err(/* ProgramBodyIntroducesName */); }
}
i += 1;
```

`types::is_param_annotation_arrow` is the **existing one door** — `node.is_bare_symbol("<-") ||
is_binder_marker(node)`, documented at `src/types.rs:5370` as *"the same predicate
`argspec::parse_triple` uses — one door for 'the next sibling is a type'"*. `<-` leaves the string
exclusion because the arrow arm now consumes it; `->` and `&` stay, since neither introduces a name
and neither is followed by a type.

⛔ **This does not loosen what Gate E inspects.** It removes exactly one category — the item in a
**type position** — and every Symbol in a **name position** is still refused. The rows in §3 are the
proof, not the argument.

### The probe that fails without it

`git stash push -- src/macros/expand.rs` (explicit path; `wat/` untouched) → `cargo build --release`
(22.13 s) → `cargo nextest run --release -E 'binary(macros) and (test(fn_binder) or
test(type_annotation))'`:

```
     Summary [   0.037s] 3 tests run: 2 passed, 1 failed, 122 skipped
        FAIL [   0.035s] (1/3) wat::macros probe_arc249_macro_engine::a_symbol_spelled_type_annotation_is_not_a_binder
    thread '…::a_symbol_spelled_type_annotation_is_not_a_binder' (292055) panicked at
    /home/john/work/holon/wat-rs/tests/macros/probe_arc249_macro_engine.rs:22:44:
    startup: #wat.macro/ProgramBodyIntroducesName {:message "macro :my::typed-id program body refused
    — quasiquote template introduces literal name `wat.core/i64` in binder position; this could
    capture caller-site names (hygiene bound gate E, arc 249 stone 249.2b-ii); use ~-unquote to splice
    the name from a macro parameter" :location … :macro-name ":my::typed-id" :binder "wat.core/i64"}
```

`:binder "wat.core/i64"` — **the type, named as a binder.** `git stash pop` + rebuild (22.28 s)
restores it; `binary(macros)` is then **121 / 121**.

---

## 3. ⭐ GATE E'S NON-VACUITY FIXTURE — a genuine violation still refused, BOTH spellings

Three fixtures, three rows, in `tests/macros/probe_arc249_macro_engine.rs` (block **E3**):

| file | row | what it pins |
|---|---|---|
| `probe_arc251_8d_hygiene_fn_binder_keyword.wat.bad` | `hygiene_bound_still_fires_on_a_keyword_spelled_fn_binder` | a template `fn` introducing the literal binder `y`, keyword spelling → `ProgramBodyIntroducesName{macro_name: ":my::capturing-param", binder: "y"}` |
| `probe_arc251_8d_hygiene_fn_binder_symbol.wat.bad` | `hygiene_bound_still_fires_on_a_symbol_spelled_fn_binder` | the SAME violation, faithful-Clojure spelling → the SAME typed error, the SAME binder |
| `probe_arc251_8d_hygiene_fn_type_is_not_a_binder.wat` | `a_symbol_spelled_type_annotation_is_not_a_binder` | ⛔ the positive control: `gen.wat`'s `record` shape in miniature — `let`-bodied macro, `~`-spliced hygienic binder, symbol-spelled type — **registers, expands and computes `true`** |

The two `.bad` files differ **only** in the fn head, the annotation arrows and the type. Both
assertions are exact matches on the typed error's `macro_name` and `binder` fields — never
`contains` (`no_loose_string_assert` is on the floor).

⭐ **The two non-vacuity rows pass BOTH with and without the cure** (§2's stashed run: `2 passed`).
That is what makes them a test of the *wall* rather than a test of the *cure*.

Hand-probed on the same binary, every one refused with its located binder:

| probe | template | binder named |
|---|---|---|
| pH | `` `(:wat::core::fn [y <- :wat::core::i64] -> :wat::core::i64 y) `` | `y` |
| pI | `` `(wat.core/fn [y :- wat.core/i64] :- wat.core/i64 y) `` | `y` |
| pJ | `` `(wat.core/fn [~cv :- wat.gen/Coord y :- wat.core/i64] :- ~T 1) `` | `y` ⭐ **a literal binder AFTER a skipped type is still caught** |
| pK | `` `(wat.core/let [tmp 1] tmp) `` | `tmp` |
| pL | `` `(:wat::core::let [tmp 1] tmp) `` | `tmp` |
| pM | `` `(wat.core/fn [& rest :- wat.core/i64] :- ~T 1) `` | `rest` ⭐ the `&`-rest binder |

---

## 4. ⭐ SELF-APPLICATION — **OWED THREE DRAWS. PAID.**

With the converted stdlib compiled in (`cargo build --release` exit 0, 23.01 s), the converted
codemod ran against copies of **four real unconverted files**:

```
"[to-faithful-clojure] …/self/conv/census-one-param-spec.wat"
"[to-faithful-clojure] …/self/conv/probe_arc272_rs1_state_must_be_record.wat"
"[to-faithful-clojure] …/self/conv/probe_arc278_northstar_cold_and_windy.wat"
"[to-faithful-clojure] …/self/conv/probe_arc278_sift_rules_arena.wat"
SELF-APP RC=0        real 0m38.401s
```

⛔ **"It starts" is not "it works" — so here is the work.** It converted:

| file | `(:wat::` forms in | forms out | changed lines |
|---|---|---|---|
| `census-one-param-spec.wat` | 301 | 24 (string literals / comments) | 630 |
| `probe_arc272_rs1_state_must_be_record.wat` | 15 | **0** | 46 |
| `probe_arc278_northstar_cold_and_windy.wat` | 16 | **0** | 40 |
| `probe_arc278_sift_rules_arena.wat` | 196 | **0** | 528 |

```diff
-(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
+(wat.core/defrecord weather/Temperature [celsius  :- wat.type/i64  location :- wat.type/String])
-(:wat::rete::defrule :weather::cold-and-windy
+(wat.rete/defrule weather/cold-and-windy
-     (:wat::rete::i64::< ?c 20))
+     (wat.rete.i64/< ?c 20))
```

⭐ **And it is CORRECT, not merely non-empty.** After restoring `wat/` and rebuilding, the
**pre-conversion** codemod was run on fresh copies of the same four inputs (rc 0) and `cmp`'d:

```
census-one-param-spec.wat                     BYTE-IDENTICAL
probe_arc272_rs1_state_must_be_record.wat     BYTE-IDENTICAL
probe_arc278_northstar_cold_and_windy.wat     BYTE-IDENTICAL
probe_arc278_sift_rules_arena.wat             BYTE-IDENTICAL
```

⭐ **This is the first draw where self-application passes with NOTHING uncommitted in `src/` beyond
the Gate E cure.** The third draw got here only under two reverted `type_denotation` probes; 255.12
landed those.

## 5. IDEMPOTENCE — 0 changes

Second pass over the converted live `wat/`, same 64-path list, same binary: **rc 0, 598 s,
64/64 files processed, `md5sum -c` against the post-pass-1 image → 0 FAILED.**

---

## 6. ⛔ THE FLOOR — A RED IS A RED. Captured, not re-run.

**FLOOR-CONVERTED** — converted `wat/` + the Gate E cure, i.e. exactly the state this stone would
have landed. `.floor/2026-09-22T09-53-33Z/`:

```
     Summary [ 443.028s] 5989 tests run: 5573 passed (13 slow), 416 failed, 22 skipped
```

exit **100**. Not re-run. The untruncated ANSI-stripped log and the whole-block ARM capture are on
disk (`clean.log` 3.1 MB, `ARM.txt` 3.2 MB, `raw.log` 3.5 MB).

⭐ **The set is IDENTICAL to the third draw's FLOOR-B.** Sorted failing-test names, `comm` both ways
against `.floor/2026-09-22T03-57-43Z/`:

```
only in FLOOR-B : 0
only in mine    : 0
```

416 = 416, same names. So the 5 286-failure `wat.gen/Coord` block is **entirely gone** and nothing
new appeared. My three new rows are **not** among the 416.

By binary: `wat::rete` 195 · `wat` 78 · `wat::services` 43 · `wat::lint` 31 · `wat::cli` 29 ·
`wat::kernel` 21 · `wat::program`/`macros`/`function`/`comms` 3 each · `wat_lang`/`resolve` 2 each ·
`types`/`process`/`diagnostics` 1 each.

### ⭐ A MECHANISM THE THIRD DRAW DID NOT NAME — the purity registry is keyed on the KEYWORD spelling

Classifying all 416 whole blocks by their stderr:

```
  131  purity-spelling  — `wat.core/<` is not proven pure
   74  other
   62  MalformedForm (other)
   60  CheckErrors (other)
   55  TypeMismatch
   33  UnresolvedReferences
    1  lint greps wat/*.wat as TEXT
```

**131 of 416 — the largest single class — is one string comparison.** Whole block, verbatim:

```
        FAIL [   1.073s] ( 475/5989) wat::rete probe_arc278_6b_ii_a_where_oracle::where_blocks_when_predicate_false
  stdout ───

    running 1 test
    test probe_arc278_6b_ii_a_where_oracle::where_blocks_when_predicate_false ... FAILED

    failures:

    failures:
        probe_arc278_6b_ii_a_where_oracle::where_blocks_when_predicate_false

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 519 filtered out; finished in 1.06s
    
  stderr ───

    thread 'probe_arc278_6b_ii_a_where_oracle::where_blocks_when_predicate_false' (3944886) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_6b_ii_a_where_oracle.rs:72:5:
    where (> -5 0) false → 0 Gates; got Err(#wat.runtime/MalformedForm {:message "malformed :wat::core::sort$native form: comparator `:wat::core::Fn` is not pure: `wat.core/<` is not proven pure (sort$native refuses an impure or nondeterministic comparator BEFORE any comparison runs, so no effect from a bad comparator is ever observable)" :location #wat.core/Span {:file "wat/core.wat" :line 1555 :col 5 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 1558 :col 12}}} :causes [] :head ":wat::core::sort$native" :reason "comparator `:wat::core::Fn` is not pure: `wat.core/<` is not proven pure (sort$native refuses an impure or nondeterministic comparator BEFORE any comparison runs, so no effect from a bad comparator is ever observable)"})
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Arm: `sort$native`'s comparator-purity refusal. `wat/core.wat:1555` now passes the comparator as the
**Symbol** `wat.core/<`; the purity prover's table is keyed on the **keyword** spelling, so it proves
nothing and the refusal fires **before any comparison runs**. The **468** occurrences of the phrase
in the log name exactly **one** callee — `wat.core/<` — which is why the class is so sharp. ⭐ **This
is the third instance of the recurring class the injected CLAUDE.md names: a string comparison with
one side normalized and the other not** — the siblings being 255.8's `is_subtype` and 255.12's
`edn_to_typed_value_inner` / `value_matches_type_by_name`. **It is the single highest-value target
for the fifth draw and it is not the type system.**

Second distinct arm — a lint that reads `wat/*.wat` as TEXT (1 of 416, unchanged from FLOOR-A):

```
        FAIL [   0.012s] (   3/5989) wat::lint ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants
  stdout ───

    running 1 test
    test ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants ... FAILED

    failures:

    failures:
        ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 359 filtered out; finished in 0.00s
    
  stderr ───

    thread 'ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants' (3932999) panicked at /home/john/work/holon/wat-rs/tests/lint/ast_kind_nodekind_sync.rs:43:9:
    defenum :wat::grep::NodeKind not found in wat/grep.wat
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Third distinct arm — rete inference over **unconverted corpus** files against a **converted** stdlib
(`wat::lint rete_compile_gate::every_wat_scripts_rete_rule_compiles_shard_02`, 13.03 s): five
`wat-scripts/` files whose rules no longer compile, e.g.

```
      wat-scripts/scratch-pad/probe-inline-constraint-bypasses-law-a.wat
          startup: #wat.check/CheckErrors {… #wat.check/TypeMismatch {:message ":wat::rete::i64::>:
          parameter #1 expects :wat::core::i64; got :wat::core::keyword" … :callee ":wat::rete::i64::>"
          :param "#1" :expected ":wat::core::i64" :got ":wat::core::keyword" …}}
```

All 416 whole blocks are at `.floor/2026-09-22T09-53-33Z/ARM.txt`, untruncated. Quoting 416 blocks
in this document is not possible; three arms are quoted whole and every block is classified above.

⚠ `rete::kernel::tests::harvest_cost::harvest_wrap_split` did **not** fire in either floor.

---

## 7. ⭐ FLOOR-C — the state that DOES land: the cure alone, on the unconverted stdlib

`git checkout -- wat/` (`git status --porcelain` shows zero `wat/` paths) → `cargo build --release`
**22.72 s, exit 0** → `scripts/floor.sh`, `.floor/2026-09-22T10-17-52Z/`:

```
     Summary [ 320.150s] 5989 tests run: 5989 passed (9 slow), 22 skipped
```

exit **0**. 5 989 = the orchestrator's unconverted baseline **5 986** (`.floor/2026-09-22T08-26-51Z`)
**+ my three new rows**. ⭐ **A healthy floor takes 320 s; the converted one took 443 s and the
3 747-failure one took 24 s. A fast floor is a symptom.**

---

## 8. EVERY GATE, WITH ITS EVIDENCE

| gate | converted state | landable state (cure only) |
|---|---|---|
| conversion 64/64 | ✅ rc 0, **1 405 s**, `md5sum -c` = **64 FAILED** (all 64 changed), `git status` = exactly those 64 `wat/` paths, nothing else | n/a |
| `cargo build --release` | ✅ **exit 0, 23.01 s**, `Compiling wat` shown | ✅ **exit 0, 22.72 s** |
| binary starts | ✅ (control fixture `--check` rc 0) | ✅ |
| ⭐ **floor** | ❌ **RED — 416 / 5 989**, 443.03 s, exit 100 | ✅ **GREEN — 5 989 / 5 989**, 320.15 s, exit 0 |
| ⭐ self-application converts | ✅ rc 0, 38.4 s, 4 files, **byte-identical** to the pre-conversion reference | (reference side) |
| ⭐ idempotence | ✅ rc 0, 598 s, **0 / 64 changed** | n/a |
| census `no STOP-8` | ❌ **129 STOP-8**, rc 8 (wat-scripts 85 · tests 40 · docs 3 · wat-tests 1); 213 → **342** failures, **0 recoveries**; `wat/` standalone **37 → 37** | ✅ **`census-diff: no STOP-8`, rc 0**, 213 = 213 |
| ⭐ `delta.sh` NEW ≤ 3 | ✅ **NEW 3** | ✅ **NEW 3** (`probe-c1-clean-surface`, `probe-kwargs-peer`, `wat/holon/Ngram.wat`) |
| ⛔ `delta.sh` RECOVERY | ❌ **9**, **exit 9** — explained below | ✅ **0**, **exit 0** |
| clippy `-D warnings --all-targets --workspace` | not run (state reverted) | ✅ **0 warnings, rc 0**, 12.8 s |
| `wat/` converted, nothing else | ✅, then reverted | ✅ `wat/` clean |

Delta list: `delta-sample-179.txt`, sha256 `33ede76cbb9b8809ecebc7656db3d4733769df54c2641b7e6d798773f1cbc3e5`,
paths=179 missing=0, both runs.

### ⛔ The 9 RECOVERYs, explained file by file — the denominator moved, no wall was disarmed

```
tests/rete/probe_enum_name.wat                            wat-scripts/fmt/rules/defn-args.wat
tests/services/probe_arc278_sift_logs.wat                 wat-scripts/grep/core-numerics-ops.wat
wat-scripts/fixes/rename-four-families-to-their-homes.wat wat-scripts/probes/arc-278/s2s-process-probe.wat
wat-scripts/fixes/rename-string-verbs-to-their-home.wat   wat-scripts/scratch-pad/277-all-atom-pair-runs.wat
                                                          wat-scripts/scratch-pad/277-is-colon-dash-always-a-type-app.wat
```

These **nine are a subset of the twelve** the THIRD draw listed under *"the post-conversion 16 is a
number whose DENOMINATOR moved"* — files whose **unconverted original** stops checking once the
stdlib it resolves against is converted. `delta.sh` compares orig-vs-converted **against whatever
stdlib is embedded**: with a converted stdlib the ORIG side fails and the CONV side passes, which is
precisely `RECOVERY`. It is an artifact of measuring the delta from inside the converted state, not a
checker that stopped refusing.

⭐ **Proven, not asserted:** the identical delta on the landable state (unconverted stdlib, cure
applied) returns **RECOVERY 0, exit 0** with the same list and the same NEW count. The RECOVERY
column is therefore reporting the reference, not a disarmed wall — and it is also the reason
`delta.sh` should not be read as a gate while a conversion is embedded.

---

## 9. ⛔ WHAT MY GREEN CANNOT SEE

1. ⛔ **The conversion did not land and every converted-state number describes a reverted tree.**
   There is no committed conversion to defend. What is committed is a `src/` cure and three fixtures.
2. ⛔ **The 416 are not diagnosed, only classified.** I named the 131-strong purity-spelling class and
   quoted its arm; I did **not** patch it, and I did **not** verify that curing it clears the other
   285. The third draw's "323 tests bought by two denotation sites" is the warning: the classes
   interact.
3. ⛔ **Self-application is four files, not 2 209.** Byte-identity on four real files is a strong
   witness and not a census.
4. ⛔ **Idempotence is proven for `wat/` only** (64 files), on the second pass of one run.
5. ⛔ **Gate E's `let` arm is untouched and unmeasured under conversion.** Converted `let` binder
   vectors carry no type annotations today (`wat/core.wat`'s 129 ` :- ` occurrences are all type
   BINDERS in declarations, which this arm never reaches), so no probe of mine distinguishes
   "correct" from "not exercised".
6. ⛔ **The cure's skip is unconditional on what follows an arrow.** A malformed template
   `[<- x]` now skips `x` where it previously refused it. That is consistent with the arm's own
   documented pass-through policy for malformed `fn` forms, but it is a behaviour change I did not
   fixture, because I could not construct a *capturing* version of it.
7. ⛔ **A spliced binder is still invisible.** `[~@params]` in a template is a List, not a Symbol, and
   Gate E cannot see what it expands to. That hole pre-dates this stone and I did not narrow it.
8. ⛔ **`is_quasiquote_form` remains keyword-only**, so a **bare-quasiquote** macro body still skips
   Gate E *and* the F5 purity gate entirely, in **both** spellings (pC). 255.11 reported this for the
   builder; my isolation is a second, independent witness that it is reachable and silent. **I did
   not close it** — closing it widens what gets validated and is the builder's call.
9. ⛔ **255.8's wrong-join hole, the `load-file!` keyword-only head, and the lint family that greps
   `wat/*.wat` as TEXT are all untouched** — the third draw's list stands verbatim.
10. ⛔ **The 5 292 `wat.gen/Coord` lines I cite in the inherited floor are the ORCHESTRATOR's run**
    (`.floor/2026-09-22T09-17-21Z`), read off disk by me. I did not reproduce that state.

---

## 10. ⛔ WHAT THE BRIEF GOT WRONG — the twenty-first correction, and three more

1. ⭐⭐ **The mechanism hypothesis is wrong in its load-bearing half.** "255.11 … Now it is [walked] —
   and the walk treats the post-`:-` type as a name" attributes the defect to 255.11. **pE refutes
   it**: the keyword-spelled `fn` head, read by this gate since arc 249, fires identically on a
   symbol type. 255.11 enabled the *spelling*; the defect is the arm's own cadence, and it was
   latent from the day it was written. The brief was right to flag the story UNPROVEN.
2. ⛔ **The non-reproduction was not a mystery about the template.** A bare-quasiquote macro body is
   routed past `validate_macro_definition` by `is_quasiquote_form` — documented in 255.11 §4(a), the
   stone the brief told me to read. Any five-line repro of that shape returns rc 0 whatever its
   contents (pC). The discriminating factor is the **enclosing `let`**, which the brief listed as one
   candidate among six without saying it is a *routing* condition rather than a template property.
3. **The floor denominator.** The brief's gate reads *"floor GREEN (5968+, not 2 239)"*. The
   unconverted baseline on disk is **5 986** (`.floor/2026-09-22T08-26-51Z`), not 5 968; with my
   three rows it is **5 989**. And the "2 239" is the *passed* count of the converted run, whose
   failure count is 3 747 — the two are not comparable numbers.
4. **"5 286 of 3 747 failures name ONE site"** cannot be read as written (5 286 > 3 747). The
   measurable fact on disk is **5 292 log LINES** mentioning `wat.gen/Coord` across **3 747** failing
   tests. The attribution is correct; the arithmetic is not.

---

## 11. WHAT THE FIFTH DRAW NEEDS (not started here)

1. ⭐ **The purity registry's spelling** — `wat.core/<` is not proven pure. 131 of the 416, one
   callee, `sort$native`'s comparator gate, `wat/core.wat:1555`. Same class as 255.8 / 255.12; find
   the table and ask it through a denotation door. **Weigh what else that comparison gates before
   touching it** — 255.12's two sites bought 323 tests and nobody has weighed their blast radius
   either.
2. The remaining ~285: `MalformedForm` 62, `CheckErrors` 60, `TypeMismatch` 55, `UnresolvedReferences`
   33, other 74. Classify by mechanism, not by binary.
3. The 129 census STOP-8 — unconverted corpus files against a converted stdlib. This is 8d-iii's
   body of work and it cannot be cleared from inside 8d-ii.
4. `is_quasiquote_form`'s router gap (§9.8) — the builder's call.

## Artifacts

`.floor/2026-09-22T09-53-33Z/` (converted, 416 red) · `.floor/2026-09-22T10-17-52Z/` (cure only,
5 989 green) · `.census/2026-09-22T10-03-49Z.txt` (converted) · `.census/2026-09-22T10-25-28Z.txt`
(landable) · `.delta/2026-09-22T10-05-05Z/` (converted, RECOVERY 9) ·
`.delta/2026-09-22T10-24-40Z/` (landable, RECOVERY 0) · fixtures
`tests/macros/probe_arc251_8d_hygiene_fn_binder_{keyword,symbol}.wat.bad` and
`tests/macros/probe_arc251_8d_hygiene_fn_type_is_not_a_binder.wat`.
