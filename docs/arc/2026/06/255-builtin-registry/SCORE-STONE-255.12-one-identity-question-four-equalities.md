# SCORE — STONE 255.12: one identity question, four `==`s

**Executed against `ff0e7f13f`** (`src/` identical to 255.11's landing commit `0817306f9` —
`git diff --stat 0817306f9 HEAD -- src/` is empty). Every number in this document was produced
this session. ⛔ **Not one `.wat` in the live tree was converted.**

## VERDICT — **LANDED**

**Three of the four sites cured, one REFUTED and reported. A fifth site the brief does not list
found and cured. Both instrument debts closed. ⛔ One floor went RED and is reported in full.**

⭐ **THE HEADLINE IS THE ADVERSARIAL ROW, AND IT CAUGHT THIS STONE'S OWN FIRST DRAFT.** The brief
demanded a divergent re-declaration *"that `type_denotation` might COLLAPSE"*. There is one:
`:wat::type::Infer` denotes to `:wat::core::Infer`, which **does not exist** — a field declared with
it is `UnknownNamedType` standing alone. Measured on two binaries built from one tree:

```
draft cure   (defrecord :my::R [n <- :wat::type::Infer])
             (defrecord :my::R [n <- :wat::core::Infer])   → rc 0  ⛔⛔ ACCEPTED
                                                              the second, ILLEGAL declaration
                                                              was swallowed as a benign no-op
pre-cure     the same file                                  → rc 1  DuplicateType
final cure   the same file                                  → rc 1  DuplicateType
positive control: the second declaration ALONE              → rc 1  UnknownNamedType :wat::core::Infer
```

The carve-out that fixes it **was already written down** — inside `check::format_type_path`, a
**renderer**. Every other consumer of `type_denotation` in the tree was collapsing `Infer`
silently, and it only became visible when an EQUALITY was pointed at the same door. The rule now
lives once, in `types::denoted_type_path`, and `format_type_path` calls it.

⭐ **AND THE SITE-2 LEAD IS REFUTED.** The brief's *"⭐ Measured lead (orchestrator, confirm it)"* —
that `ast_same_identity`'s `is_reference()` requirement is what makes `wat/holon/Ngram.wat` a
duplicate — is **wrong on the causal half**, measured three ways (§2).

Gates: floor **5986 / 5986 GREEN on the second run**, ⛔ **first run 5984/2 — both arms captured
verbatim in §7.4, one of them mine and fixed, one of them NOT mine and surfaced as a finding**;
clippy 0; census `no STOP-8`, **0 rc changes across all 2202 shared paths**; delta **4 → 3** with
**RECOVERY 0**.

---

## 1. THE FOUR SITES — confirmed, refuted, and the fifth

| # | site | brief says | **MEASURED HERE** |
|---|---|---|---|
| 1 | `types.rs` `register_validated` — `Some(e) if e == &def` | raw `==` | ✅ **CONFIRMED. CURED.** |
| 2 | `macros/registry.rs::ast_same_identity` | cross arm blocked by `is_reference()` | ⛔ **REFUTED. REPORTED, not cured.** |
| 3 | `edn/render.rs` `edn_to_typed_value_inner` — `match p.as_str()` | the 8d-ii STOP | ✅ **CONFIRMED. CURED. Reached from wat with NOTHING converted.** |
| 4 | `function/subsume.rs::value_matches_type_by_name` | raw Path compare | ✅ **CONFIRMED. CURED.** |
| **5** | `types.rs:4256` `register_stdlib_types_replacing` — `if existing != &def` | ⛔ **not in the brief** | ✅ **found by censusing the gate's own equivalence sites; CURED** |

### 1.1 How site 5 was found, and why a four-row table missed it

I did not grep for `==`. I enumerated the call sites of the **registration gate's own
`Existing::Equivalent` / `Existing::Divergent` decision** (`grep -rn "Existing::Equivalent"`), which
is the question all four sites are actually asking, and read each one. That yields seven sites:
five in `declare/preregister.rs` (presence-only, no comparison), `check/env.rs:318` (**already
correct** — it calls `schemes_same`, which already routes through `type_exprs_same`),
`macros/registry.rs:82`, and `types.rs:995`. `types.rs:4256` is the eighth: it is not a gate call
at all, it is the **door-replace** decision that runs *before* one, and it asks the identical
question with `!=`.

Left raw, it would have meant a merely RE-SPELLED stdlib declaration gets retracted and replaced —
the file's spelling silently winning — where `register_validated` now correctly calls it a no-op.
Curing it runs in the **restrictive** direction (strictly fewer retractions), so it carries none of
this stone's risk.

⚠ **What that census cannot see:** a site that decides equivalence **without** routing through
`resolve::register`. `ast_same_identity` was in the brief; a comparator that never names
`Existing` would not be in either list.

---

## 2. ⛔ SITE 2 — THE LEAD IS REFUTED. Three measurements.

**The brief's lead:** *"the converted body differs **only** by **3 `<-` + 2 `->` → 5 `:-`** — a
Symbol→Keyword flip — and `ast_same_identity`'s cross arm requires `id.is_reference()`, which `<-`
and `->` are not."*

**(a) Which difference actually causes it — by targeted revert of the conversion.**
The converted `Ngram.wat` differs from the original in **nine** ways, not five: five arrow flips
**and four head Keyword→Symbol flips** (`:wat::holon::Bundle`→`wat.holon/Bundle`,
`:wat::core::mapv`, `:wat::core::fn`, `:wat::holon::Sequential`, `:wat::seq::window`) **and** the
annotation types (`:wat::holon::Holons`→`wat.holon/Holons`). Restoring **exactly the two arrows
inside the quasiquoted template** — and nothing else — makes the converted file check **clean**:

```
converted Ngram                                                   → rc 1  DuplicateMacro
converted Ngram, body `fn` restored to `[window <- T] -> R`       → rc 0  ⭐ CLEAN
converted Ngram, only the `<-` restored                           → rc 1
converted Ngram, only the `->` restored                           → rc 1
```

So every head and annotation re-spelling is **already** handled by `canonical_identity`. Of the
brief's "3 `<-` + 2 `->`", **three are the macro's OWN parameter vector**, which is consumed at
parse (`MacroDef.params` is `Vec<String>`) and never reaches the comparator. **Only the two arrows
inside the template matter.**

**(b) A purpose-built pair, both dialects, no arrows** — `tests/macros/probe_arc255_12_macro_cross_spelling.wat`:
one macro declared twice, every head and annotation re-spelled. **rc 0 on the PRE-255.12 binary.**
The cross-spelling arm was never broken.

**(c) `is_reference()` could not have been the cause.** The arm compares `canonical_identity` of
both sides. `canonical_identity("<-")` is `"<-"`; `canonical_identity(":-")` is `":-"`. They are
different strings, so the arm returns false **before** `is_reference()` could matter. Pinned as a
unit test beside the function
(`src/macros/registry.rs::tests::the_annotation_markers_are_three_identities_not_one_name`).

### 2.1 ⛔ Why curing it would be wrong — and what the door cannot express

`<-` → `:-` and `->` → `:-` is a **grammar migration**, not a namespace re-spelling, and the codemod
folds **two** markers onto **one**. A blind structural walker has no position information, so
teaching it the equivalence means asserting `<-` ≡ `->` ≡ `:-` **anywhere in a template** — two
macros whose templates *emit different surface forms* would become one macro. There is no identity
door that expresses "these two tokens are the same marker **in this position**", and minting one
would be the new comparator the brief forbids.

And it would have to be done by relaxing the very guard that is load-bearing:
`tests/macros/probe_arc255_12_macro_binder_is_not_a_keyword.wat.bad` is two templates differing
only in `v` (a `let` binder) vs `:v` (a keyword). `is_reference()` is what keeps them two macros;
relaxing it makes that file freeze clean and silently discards the second template.

### 2.2 ⭐ What the Ngram delta row actually measures — a correction to the gate itself

`wat/holon/Ngram.wat` and `wat/source.wat` are `include_str!`'d into the binary
(`src/load/stdlib.rs:143`) **and** read off disk. The duplicate is the **baked** copy meeting the
**converted** copy — measured directly: copying the converted file to an unrelated path still
raises `DuplicateMacro`, so the collision is keyed on the NAME, not the path.

⚠ **Therefore the delta harness necessarily measures a MIXED-DIALECT tree** — a binary baked from
rust-scheme sources checking faithful-Clojure files. Under 8d-ii's real conversion the baked copy
converts too and both collisions dissolve without any comparator change. **Site 1's cure is still
correct and still needed** (it closes the mixed-dialect window that 8d-ii must pass through, and the
arc-054 shim shape is real independent of conversion); but the delta's last two files were never
purely a `src/` question, and the brief's *"the 2 remaining stdlib files should close"* is half
right: one closed, one is 8d-ii's. **Builder's call** on whether site 2 should be cured at all.

---

## 3. THE CURES — each through an EXISTING door, each with the probe that fails without it

⛔ **No new comparator was minted.** `type_denotation` / `type_exprs_same` /
`parametric_head_fqdn` are 255.8's doors; `denoted_type_path` is `check::format_type_path`'s
already-written rule **moved**, not invented (its body is byte-for-byte what `format_type_path`
held, and `format_type_path` now calls it).

### 3.1 Site 1 — `register_validated`

`Some(e) if e == &def` → `Some(e) if type_defs_same(e, &def)`.

⭐ **`type_defs_same` is `==` OVER DENOTATION-NORMALIZED DEFS, not a hand-written field walk.**
This is the one comparator in the arc that runs in the **permissive** direction, so the danger is
not that it is too strict but that a hand-rolled walk silently omits a field — `nature`, `purity`,
`restrictions`, `type_params`, a variant's name, `max_request_bytes` — and calls two genuinely
divergent declarations equivalent. Normalizing only the `TypeExpr` leaves and then deferring to the
derived `PartialEq` makes that class **unrepresentable**: every field the comparator does not
explicitly denote is still compared byte-for-byte.

The normalizer reaches every `TypeExpr` in every `TypeDef` variant, including the `ArgSpec` nested
inside `SurfaceMember::Method`.

**Probe that fails without it:** `tests/types/probe_arc255_12_redeclare_cross_spelling.wat` —
record, parametric-field record, enum, typealias, each declared twice in the two spellings.
**pre-cure rc 1 `DuplicateType :p255_12::Rec`; post-cure rc 0.**

### 3.2 Site 5 — `register_stdlib_types_replacing`

`if existing != &def` → `if !type_defs_same(existing, &def)`. Same door. Restrictive direction.
⚠ **Declared unprobed as a behaviour change:** I have no fixture that reaches the stdlib-mode
door's replace path with a re-spelled declaration, so this cure is argued from the code, not
measured. Saying so rather than claiming it.

### 3.3 Site 3 — the EDN coerce table

`match p.as_str()` → `match crate::types::denoted_type_path(p).as_str()`, and the parametric head
table likewise through `parametric_head_fqdn` + `denoted_type_path`. The two `_` arms deliberately
keep the RAW `p`: `is_type_var_path` is a shape test on the written form, and `TypeEnv::get` is
already denotation-aware.

⭐ **REACHED FROM WAT, WITH NOTHING CONVERTED** — `:wat::edn::validate` is a thin wrapper over this
exact walker (`runtime.rs::eval_edn_validate`). ⚠ **`--check` cannot see it**; these are RUN, not
checked.

```
                                                     PRE-CURE                         POST-CURE
(:wat::edn::validate 42 :wat::core::i64)             Validation.Valid                 Valid
(:wat::edn::validate 42 :wat::type::i64)             ⛔ Invalid {:expected            ⭐ Valid
                                                       ":wat::core::i64"
                                                       :got "Integer"}
(:wat::edn::validate 42 wat.type/i64)                ⛔ Invalid (same)                ⭐ Valid
(:wat::edn::validate "not-an-integer" :wat::type::i64)  Invalid {… :got "String"}     ⭐ Invalid  ← non-vacuity
```

⛔ **Read the pre-cure diagnostic.** It says *expected `:wat::core::i64`, got `Integer`* — and an
Integer **is** an i64. The message denotes on the way out (`format_type`) while the comparison did
not, so it reported a comparison that was never made. A self-contradicting refusal.

### 3.4 Site 4 — defclause runtime dispatch

`p.as_str() == val_type` → `… || denoted_type_path(p) == denoted_type_path(val_type)`.

```
                                            PRE-CURE                                   POST-CURE
(defclause :p::pick ([x <- :wat::core::i64] …))  (:p::pick 42) → 42                    42
(defclause :p::pick ([x <- :wat::type::i64] …))  (:p::pick 42) → ⛔ rc 1                ⭐ 42
    NoMatchingClause: "clause 0 skipped (arg 0: expected :wat::core::i64, got :wat::core::i64)"
```

⛔ **Two byte-identical strings, and the clause was skipped** — same denote-on-render/compare-on-raw
split as site 3, one tier over.

⭐ **This cure makes the runtime matcher agree with its own documented mirror.** `declared_type_subsumes`
(`subsume.rs:48`) — which the file's own doc calls *"the exact mirror of `value_matches_type_by_name`"* —
compares through `check::format_type`, which **already denotes**. The two halves of one stated
equivalence disagreed; they no longer do.

---

## 4. ⛔ THE NON-VACUITY ROWS — the stone

Thirteen rows, in `tests/types/probe_arc255_12_one_identity_question.rs`,
`tests/macros/probe_arc255_12_macro_identity.rs` and `src/macros/registry.rs`'s unit tests.
Every refusal is asserted **structurally** — on the `TypeErrorKind`/`MacroErrorKind` variant AND its
`name` field — never `contains()` over a rendered Debug.

| row | pre-cure | post-cure | what it pins |
|---|---|---|---|
| byte-identical re-declaration (arc 054's own fixture) | 0 | 0 | the permitted no-op survives |
| cross-spelling record / enum / alias / **parametric head + arg** | **1** | ⭐ **0** | THE CURE |
| divergent field TYPE (`i64` vs `String`) | 1 | ⭐ **1** | denotation does not collapse it |
| divergent **NATURE** (`defrecord` vs `defstruct`, identical fields) | 1 | ⭐ **1** | a field the comparator never denotes |
| divergent field NAME | 1 | ⭐ **1** | a `String` inside `fields` |
| divergent field ORDER (same names, same types) | 1 | ⭐ **1** | `Vec` `==` is order-sensitive |
| divergent enum VARIANT name | 1 | ⭐ **1** | |
| ⭐⭐ **divergence `type_denotation` COLLAPSES** (`wat.type/Infer` vs `wat.core/Infer`) | 1 | ⭐ **1** | **the adversarial row** |
| macro: cross-dialect identical | **0** | 0 | ⛔ already worked — the refutation |
| macro: divergent template (`+1` vs `+2`) | 1 | 1 | |
| macro: arrow-flipped template (Ngram's shape) | 1 | 1 | what the door cannot express |
| macro: binder `v` vs keyword `:v` | 1 | 1 | why `is_reference()` must stay |
| site 3/4 wrong-type refusals + a two-clause discriminator | Invalid / — | ⭐ Invalid / correct clause | the matcher is not a wildcard |

### 4.1 ⭐ THE GUARDS HAVE FAILED ONCE — by targeted revert, twice

**(a) Revert `src/` to the parent, keep the tests** (`git checkout HEAD -- src/` + rebuild):

```
types:  2 FAILED, 7 passed  →  a_re_declaration_in_the_other_spelling_is_a_no_op
                               the_runtime_tables_read_the_denotation
macros: 4 passed            →  site 2 is untouched by this stone, as reported
```

Exactly the **two cure rows** go red; **all seven refusal rows stay green**. `src/` restored and
re-verified green.

**(b) Revert ONLY the `Infer` carve-out** inside `denoted_type_path` (leave every cure in place):

```
types:  1 FAILED, 8 passed  →  divergent_declaration_that_denotation_collapses_is_still_refused
```

⭐ **Exactly one row, and it is the adversarial one.** Nothing else in the file can see the
carve-out — which is precisely the argument for the row's existence.

⚠ **Honest limit:** the four macro rows are green on **both** binaries. They are **PINS, not
guards** — they record what the door does today so a future attempt to "cure" site 2 has to face
them. Saying so rather than counting them as proven guards.

---

## 5. ⭐ THE TWO INSTRUMENT DEBTS — both closed

### 5.A `scripts/replay/delta.sh` — RECOVERY as a standing column

Printed by hand for three consecutive stones (255.9, 255.10, 255.11), recommended by each.
Now a script: copies the committed list twice, codemods the `conv/` side only, `--check`s **both
sides from copies** (the symmetry whose absence manufactured a phantom regression in the 255.9
weigh), and prints all four columns every run. **`RECOVERY != 0` exits 9 and is a STOP.**
`.delta/` is gitignored, as `.floor/` and `.census/` are.

⭐ **It caught a defect in itself on the first run, and that is worth recording.** The first draft
used `while read -r p; do "$WAT" --check …; done < "$LIST"` — and **`wat` reads stdin**, so the
first file that blocked on a read ate the rest of the list. It measured **10 of 179**, found NEW 4
and RECOVERY 0, and printed a delta that looked exactly right. The script now runs the checks
through `xargs` with `/dev/null` on stdin **and refuses to report a partial delta** (`measured $got
of $n` → exit 1). This is the whole argument for institutionalising the column: the hand-printed
version had no such guard.

⚠ **What this instrument CANNOT see:** it measures a converted COPY against a binary baked from
UNCONVERTED sources — see §2.2. NEW and RECOVERY are both honest about that tree; neither is a
prediction about a fully-converted one.

### 5.B `src/freeze/pass_order.rs` — the pass-ordering gate

Recommended three times; absent until now. The premise *"anything at or after step 7
(`normalize_symbol_refs`) cannot see a symbol head"* was enforced by **nothing** — the step numbers
live in `//` comments in `freeze.rs` and `freeze/env.rs`.

Each pass announces itself through `pass_order::record(…)`. **Outside the crate's own test build
that function is empty with no state behind it** — nothing allocated, nothing locked, every shipped
binary untouched. Under `cfg(test)` the names accumulate in a thread-local and a unit test asserts
the sequence against `EXPECTED_ORDER`. Fourteen call sites, one module, ~90 lines including the doc.

⭐ **IT HAS FAILED ONCE.** I moved `resolve_references` ahead of `normalize_symbol_refs` in
`freeze/env.rs` — a real pipeline re-order — and the gate went red with the moved pass named:

```
⛔ the startup pipeline's pass order CHANGED.
   expected: [… "6-register-defines", "7-normalize-symbol-refs", "7-normalize-stored-function-bodies", "7-resolve-references", …]
   got:      [… "6-register-defines", "7-resolve-references", "7-normalize-symbol-refs", "7-normalize-stored-function-bodies", "7-resolve-references", …]
```

⛔ **The mutated program still FROZE CLEAN.** Nothing else would have noticed. Mutation reverted;
gate green.

⭐ **It already corrected the documented pipeline.** `freeze.rs`'s header lists ONE step-7
`normalize_stored_function_bodies`. The code calls it **twice** — once at step 7 and again after the
step-7.7 extend-type pre-registration. `EXPECTED_ORDER` is derived from what the passes *announce*,
so it carries `7.7-normalize-stored-function-bodies`. The prose did not.

⛔ **WHAT THIS GATE CANNOT SEE, stated in its own module doc.** It pins the ORDER. It does **not**
pin what actually went wrong in 255.9 and 255.10, which was the *second* half of their sentence:
`normalize_symbol_refs` rewrites **CODE positions only**, so a DATA position still carries raw
symbols at steps 8 and 9. Both stones marked a live capability wall "unreachable" while the order
was exactly what they believed. **Gating that needs a different instrument** — a post-normalize walk
of the residue reporting which positions still hold a namespaced `WatAST::Symbol`, pinned as a
census with a per-site disposition, in the shape of `tests/lint/`'s walkers. That is a stone of its
own and this module does not pretend to it.

⚠ **Second limit:** `record` is active only under the crate's own `cfg(test)`, so the gate observes
the **lib-test** build. An integration test's freeze is not traced.

---

## 6. ⛔ WHAT MY GREEN CANNOT SEE

1. **Delta 4→3 and census 212→212 are nearly silent, and that is not reassurance.** Exactly ONE of
   358 checks changed (`wat/source.wat`, converted side, 1→0) and **zero** of 2202 census paths
   moved in either direction. No file in either sample declares a type twice in two spellings,
   validates a `wat.type`-spelled target at runtime, or dispatches a `wat.type`-spelled defclause.
   **The evidence for this stone is the hand-written probes and the two-binary pairs, not the
   headline numbers.** (255.11's line, and it applies here unchanged.)
2. **Site 5 is argued, not measured** (§3.2).
3. **`type_defs_same` is only as sound as `type_denotation`.** The `Infer` row proves the door has
   at least one name it collapses wrongly. I found that one by construction. ⛔ **I did not
   enumerate the `:wat::type::` namespace to prove `Infer` is the ONLY such name** — that would be a
   census of every name X for which `:wat::type::X` and `:wat::core::X` are not the same thing, and
   I did not run it. If a second such marker exists, this stone widened the gate for it.
4. **The `Value::Aggregate` arm of `value_matches_type_by_name` was left raw** (`bare_p ==
   a.class.as_ref()`). A record class cannot live under `:wat::type::` (the reserved-prefix gate),
   so I judged it out of reach — **a reading, not a probe.**
5. **The parametric coerce table's `_` fall-through was left on the raw `head`.** It routes through
   `parametric_head_fqdn` + the denotation-aware `TypeEnv::get`, so I believe it is covered — again
   a reading; no probe reaches a `wat.type`-spelled parametric USER type.
6. **The two remaining arc-170 delta files are not sites 3/4 and I did not diagnose them.** They
   fail with eight checker `TypeMismatch`es about `(:wat::kernel::Address :- [:S :R])` vs concrete
   instantiations — a type-parameter substitution question, a different class entirely. Naming that
   they are NOT this stone's, not what they are.
7. **The pass-order gate's two blind spots** (§5.B).
8. **No probe drives site 3 through its PRODUCTION caller.** `eval_edn_validate` is the
   request-sanitization wall's wrapper; I reached the walker through it, but not through a live
   `defservice` op's frame decode.

---

## 7. THE GATES — evidence, not claims

### 7.1 Delta — **4 → 3**, ⭐ **RECOVERY 0**

`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`, sha256
`33ede76cbb9b8809ecebc7656db3d4733769df54c2641b7e6d798773f1cbc3e5` — **used as committed**, all 179
present (`missing=0`). Both sides checked **from copies**, by `scripts/replay/delta.sh`.

| binary | ORIG-CLEAN | CONV-CLEAN | **NEW** | ⭐ **RECOVERY** |
|---|---|---|---|---|
| pre-cure (`ff0e7f13f` src) | 160/179 | 156/179 | **4** | **0** |
| cured (final tree) | 160/179 | **157/179** | ⭐ **3** | ⭐ **0** |

**Gate ✅ 3 < 4. RECOVERY 0 — nothing went broken→clean, so no checker was disarmed.**
Diffing the two runs line-for-line: **0 rc changes on the ORIGINAL side across all 179**, and
**exactly one** on the converted side — `wat/source.wat` 1 → 0. One change in 358 checks.

The three remaining: `wat/holon/Ngram.wat` (§2 — REPORTED, 8d-ii's) and the two arc-170 probes
(§6.6 — a different class).

### 7.2 Census — `no STOP-8`, **0 rc changes**

⚠ `[[DIFF LIKE AGAINST LIKE]]` — comparability checked, not assumed. Baseline
`.census/2026-09-22T07-20-15Z.txt` is 255.11's own, and `git diff --stat 0817306f9 HEAD -- src/` is
**empty**, so it came from a binary whose `src/` is identical to this stone's parent.

```
baseline:   .census/2026-09-22T07-20-15Z.txt   files=2202   nonzero 212
this stone: .census/2026-09-22T08-33-12Z.txt   files=2208   nonzero 213
$ ./scripts/replay/census.sh --diff <BASE> <CURR>
census-diff: no STOP-8                                                            (rc 0)
$ join <(sort BASE) <(sort CURR) | awk '$2!=$3' | wc -l
0
```

⚠ **2202 → 2208 disclosed, and accounted for exactly.** `comm` on the path sets: **nothing left**,
six arrived — three are 255.11's fixtures (tracked by its landing commit, after its census was
taken) and three are mine (`probe_arc255_12_macro_cross_spelling.wat`,
`probe_arc255_12_coerce_and_dispatch.wat`, `probe_arc255_12_redeclare_cross_spelling.wat`), all
rc 0. The 212 → 213 is 255.11's `…_denied.wat`, a negative fixture. **Zero rc changes on all 2202
shared paths, in either direction.**

### 7.3 Clippy

```
$ cargo clippy --all-targets --workspace -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.69s
clippy exit=0
```

### 7.4 ⛔⛔ FLOOR — THE FIRST RUN WENT RED. BOTH ARMS, VERBATIM.

#### Run 1 — `.floor/2026-09-22T08-13-55Z/` — **RED**

```
     Summary [ 324.806s] 5986 tests run: 5984 passed (9 slow), 2 failed, 22 skipped
```

⛔ **Not re-run to see whether it would go green.** `scripts/floor.sh` had already captured the
untruncated ANSI-stripped log and written `ARM.txt`. Both blocks are reproduced below **whole**.

##### ARM 1 — `wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert` — ⭐ **MINE**

```
        FAIL [   0.096s] ( 148/5986) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
  stdout ───

    running 1 test
    test no_loose_string_assert::tests_carry_no_loose_string_assert ... FAILED

    failures:

    failures:
        no_loose_string_assert::tests_carry_no_loose_string_assert

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 359 filtered out; finished in 0.08s

  stderr ───

    thread 'no_loose_string_assert::tests_carry_no_loose_string_assert' (1969689) panicked at /home/john/work/holon/wat-rs/tests/lint/no_loose_string_assert.rs:135:5:


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

    tests/macros/probe_arc255_12_macro_identity.rs:57

    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**Arm:** the offender list, one site — **this stone's own new test file**, asserting
`err.contains("DuplicateMacro")` over a rendered `Debug`.
**Cure, not an exemption rune:** both new test files now assert **structurally** — on the
`MacroErrorKind::DuplicateMacro(name)` / `TypeErrorKind::DuplicateType { name }` variant **and its
name field**, reached by destructuring `StartupError`. That is strictly stronger than the lint
demanded: a `contains()` would have passed on a duplicate of some *other* name. The lint was right
and it improved the stone.

##### ARM 2 — `wat rete::kernel::tests::harvest_cost::harvest_wrap_split` — ⛔ **NOT MINE, AND A RED IS A RED**

```
        FAIL [   0.279s] (1561/5986) wat rete::kernel::tests::harvest_cost::harvest_wrap_split
  stdout ───

    running 1 test
    test rete::kernel::tests::harvest_cost::harvest_wrap_split ... FAILED

    failures:

    failures:
        rete::kernel::tests::harvest_cost::harvest_wrap_split

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1333 filtered out; finished in 0.27s

  stderr ───

    thread 'rete::kernel::tests::harvest_cost::harvest_wrap_split' (1985079) panicked at src/rete/kernel/tests/harvest_cost.rs:337:5:
    combined harvest (9.73 ms) is not accounted for by scan (5.70 ms) + wrap (14.73 ms) — the apportionment this test reports no longer adds up, so one of the three closures is measuring something other than what its name says
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**The exact arm:** `src/rete/kernel/tests/harvest_cost.rs:337`, the **APPORTIONMENT** assert, and
within it the **LOWER** bound — `h >= (s + w) * 0.5`. Recorded: `h = 9.73`, `s = 5.70`, `w = 14.73`,
so `(s + w) * 0.5 = 10.215` and `h` is **4.75 % below** it. The upper bound (`h <= (s+w)*2.0`) did
not fire, nor did the liveness assert, nor the class-filter non-vacuity `assert_eq!(collected.len(),
N)` above it — so the filter matched all 40 000 facts and the loops ran.

**What the numbers say about the instrument, as fact, not disposition.** `w`'s closure wraps a
**pre-collected** vector; `h`'s closure does the collect **and** the same wrap. `h`'s work is a
strict superset of `w`'s, so `h < w` is arithmetically impossible if each closure measures what its
name says — and the run recorded `w = 14.73 > h = 9.73`. **The test's own message is therefore
correct on its own terms**, and the unsound reading is `w`'s, not `h`'s. `s`, `w` and `h` are each a
`min` over `RUNS = 3` samples of `ns_per_iter(1, …)` — a **single-iteration wall clock**, taken
while nextest was running the rest of a 5 986-test floor in parallel. ⛔ **That is a hypothesis
about the mechanism, and I did not test it, because testing it means re-running.**

**Why it is not my diff — by naming the call graph, not by asserting unrelatedness.** The timed
closures call exactly four things: `PVec::iter`, a `matches_class` **closure defined inside the test
body** (`a.class.as_ref() == CLASS`, no registry, no `TypeExpr`), `PMap::from_pairs`, and
`Value::clone`. My diff touches `types.rs`, `check.rs`, `edn/render.rs`, `function/subsume.rs`,
`macros/registry.rs`, `freeze.rs`, `freeze/env.rs`. **None of them is on that path**, and
`value_matches_type_by_name` — the one cure that could plausibly sit in a hot loop — is not called:
the test rolls its own class test.

**The record, as evidence and nothing more.** Nineteen kept floor logs on disk cover this test; all
nineteen PASS, in 0.120 s – 0.264 s. This run is the slowest of the twenty at 0.279 s. ⛔ **"It has
always passed" is a fact about the record, not a disposition.** `[[A WALL-CLOCK RATIO IS NOT A
GATE]]` describes this shape exactly: a bound calibrated on one sampling regime, enforced over a
weaker one. **Surfaced to the orchestrator as a finding.**

#### Run 2 — `.floor/2026-09-22T08-26-51Z/` — **GREEN**

Re-run only **after** both arms were captured and **after a real code change** (the structural
assertions). Not a re-run to see whether it would go green.

```
     Summary [ 325.151s] 5986 tests run: 5986 passed (9 slow), 22 skipped
[floor] doctests exit=0
[floor] exit=0. Log kept at .floor/2026-09-22T08-26-51Z/ regardless — a green run is evidence too.
```

`grep -c FAIL` over the whole captured log = **0**; **no `ARM.txt` was written** (the script writes
one only on a red). `harvest_wrap_split` PASSed at 0.210 s.

⛔ **Run 2 does not erase run 1.** ARM 2 stands as a finding.

**5968 → 5986 = +18**, accounted for exactly: 9 (`tests/types`) + 4 (`tests/macros`) + 3
(`src/macros/registry.rs` units) + 2 (`src/freeze/pass_order.rs` units).

⚠ **What the floor's tree did and did NOT contain.** Every fixture, both test files, both new
modules and the `.gitignore` edit were on disk before run 2 started, and nothing was touched during
it. **This SCORE was not** — it is written after the gate. Several lint tests read `docs/`, so the
whole lint binary was re-run with this file present rather than argued about (result in §7.6).

### 7.5 ⛔ NOT ONE `.wat` CONVERTED

```
$ git status --porcelain -- '*.wat'
A  tests/macros/probe_arc255_12_macro_cross_spelling.wat
A  tests/types/probe_arc255_12_coerce_and_dispatch.wat
A  tests/types/probe_arc255_12_redeclare_cross_spelling.wat
```

Three NEW fixtures plus eight `.wat.bad` negatives, all hand-written. **No existing `.wat` in the
tree was modified.** Every conversion in this SCORE ran on copies under
`/home/john/.claude/jobs/edaabf97/tmp/`.

### 7.6 Lint binary re-run with this SCORE on disk

```
$ cargo nextest run --release -E 'binary(lint)'
     Summary [ 242.816s] 360 tests run: 360 passed, 0 skipped                     (rc 0)
```

---

## 8. ⭐ WHAT THE BRIEF GOT WRONG — the twentieth correction, and three more

1. ⭐⭐ **Site 2's mechanism (the twentieth).** *"`ast_same_identity`'s cross arm requires
   `id.is_reference()`, which `<-` and `->` are not"* — true and **not the cause**; the arm compares
   `canonical_identity`, and `"<-"` ≠ `":-"` before `is_reference()` is consulted. §2.
2. **Site 2's arrow count.** *"3 `<-` + 2 `->`"* is the file-level count; **three of the five are
   the macro's own parameter vector**, consumed at parse. Only the **two inside the template**
   matter — proven by restoring exactly those two.
3. **The brief's four-site table is incomplete** — `register_stdlib_types_replacing` asks the same
   question with the same `==`. §1.1.
4. **"The 2 remaining stdlib files should close"** — one closed; the other is 8d-ii's, because the
   collision is a **baked-vs-on-disk dialect mismatch**, not an `==`. §2.2.
5. ⚠ **Also worth the builder's eye:** `edn::render::type_denotation`'s own doc comment says
   *"Non-members are left as identity so the caller can refuse them as 'not a member of
   wat.type'"*. It is a blind prefix rewrite over **every** tail; `Infer` is the case where that
   matters, and `check::format_type_path` had quietly worked around it for one consumer only.

---

## 9. REPRODUCTION

```bash
cargo build --release
cargo test --release --test types  -- arc255_12
cargo test --release --test macros -- arc255_12
cargo test --release --lib macros::registry::tests
cargo test --release --lib pass_order
cargo clippy --all-targets --workspace -- -D warnings
./scripts/replay/census.sh && ./scripts/replay/census.sh --diff <BASE>.txt <CURR>.txt
./scripts/replay/delta.sh                      # the new instrument; prints RECOVERY, exits 9 on non-zero
./scripts/floor.sh
# the pre-cure binary, for every two-binary pair in this SCORE:
#   git checkout HEAD -- src/ && cargo build --release   → wat-baseline   (restore immediately)
# the adversarial row's own guard:
#   delete the INFER_TYPE_PATH early-return in types::denoted_type_path → exactly 1 row goes red
# the pass-order gate's own guard:
#   move resolve_references ahead of normalize_symbol_refs in freeze/env.rs → the gate goes red,
#   and the program still freezes clean
```

Scratch (not committed): `/home/john/.claude/jobs/edaabf97/tmp/` — `wat-baseline` / `wat-cured` /
`wat-nocarveout`, `d12-pre/` `d12-post/` `d12-final/` (each with `results.tsv`, `orig/`, `conv/`),
`w/` (every probe file in §3 and §4), `ng/` (the Ngram arrow bisect).
Committed: 5 modified `src/` files, 1 new `src/` module, 2 test files, 11 fixtures,
`scripts/replay/delta.sh`, a `.gitignore` line, and this SCORE.
