# SCORE — STONE 255.11: the wall audit

**Executed against `eb860cb96`.** Every number in this document was produced this session.
The diff is **4 `src/` files + 2 test files + 4 new fixtures**; ⛔ **not one `.wat` in the live
tree changed.**

## VERDICT — **LANDED**

**26 walls enumerated. 5 FAIL-OPEN on a name spelling. 4 cured; 1 reported (curing it would be
MORE permissive). 3 walls could not be probed and say so in their own rows.**

⛔⛔ **THE HEADLINE IS A LIVE CAPABILITY ESCAPE, AND IT IS THE NINETEENTH CORRECTION.**
The inherited FINDING says `:restricted-to` is *"INTACT, four ways"* and the brief says
*"⛔ Do not re-litigate."* I did not re-litigate its four probes — all four still hold. I ran a
**fifth position it never probed**, the one arc 198's own design ruling exists for
(*"a restriction governs MENTION, not head position"*), and the wall is **OPEN**:

```
;; ONE file pair, differing in ONE mention's spelling. Both RUN, not --check-only.
(:wat::core::quote (:my::kernel::restricted-fn 7))   → rc 3  DefRestrictedCallerNotAllowed
(:wat::core::quote (my.kernel/restricted-fn   7))   → rc 0  ⛔⛔ "ESCAPED, got 7000"
                                                              ↑ the restricted fn EXECUTED
```

The same hole covered **every Rust-side `#[restricted_to(…)]` substrate fence** — measured:
`wat.kernel/spawn-process`, `wat.kernel/close`, `wat.io.IOWriter/from-fd` all passed clean where
their keyword spellings were denied — because `freeze/env.rs`'s inventory drain writes them into
the **same** `binding_metadata` map this one walker reads. The brief listed those as UNPROBED;
they are not a different mechanism, they are the same wall.

⭐ **Two more FAIL-OPEN walls, both correcting a prior stone's disposition:**

- **the mutation walls** — 255.9 measured them BYPASSED-but-BACKSTOPPED and concluded *"I found no
  mutation head that escapes both walls."* ⛔ **Two do.** `(:wat::eval-ast! '(wat.config/set-redef!
  true))` returned **`ok:nil`** — evaluated, not refused.
- **macro hygiene Gate E** (`quasiquote_inner`) — 255.10's census classified it *"LOUD for the
  explicit spelling."* ⛔ **It is SILENT.** A name-capturing macro loaded rc 0.

Gates: floor **5968/5968, first run, GREEN**, clippy 0, census `no STOP-8` (212 = 212, **0 rc changes in either
direction across 2202 files**), delta **4 = baseline 4** with **RECOVERY 0**.

---

## 1. HOW I ENUMERATED THE WALLS

⛔ **The unit is a WALL, not a read-site.** I never counted `WatAST::Keyword` arms: `scan_for_setter`
is cured and still matches `Keyword`, and three of this stone's five FAIL-OPEN findings are not
head-dispatch sites at all. The question asked of every row is only:

> **when this wall does not recognise the form, does it REFUSE or does it PASS?**

**A wall, for this audit** = a site that (i) **refuses** a program for a capability / purity /
mutation / hygiene / integrity / position reason — not a type mismatch — and (ii) reaches that
refusal by **identifying something**. Four independent frames, unioned; each is stated with what it
cannot see.

| frame | probe | yield | what it CANNOT see |
|---|---|---|---|
| **F1 — gate-shaped names** | `fn (refuse\|reject\|forbid\|deny\|validate\|ensure\|guard\|enforce\|scan_for)…` under `src/` | **38** | a wall not named like one — and 4 of the 5 FAIL-OPEN findings are in this class of blindness (`walk_for_restricted_call`, `quasiquote_inner`, `check_quasiquote_for_literal_binders`, `is_quasiquote_form`) |
| **F2 — refusal error variants** | `[A-Z]\w*(Forbidden\|Refused\|NotAllowed\|Restricted\|Denied\|Disallowed\|Illegal\|Impure)` | **4** (`DefRedefForbidden`, `DefRestrictedCallerNotAllowed`, `EvalForbidsMutationForm`, `PeersDenied`) | the brief already warns variants undercount; F5 has no variant, and neither does `SetterInLoadedFile`, `ProgramBodyIntroducesName`, `ExpandOnlyOutsideMacro`, `OutOfScope`, `AnyBanned` |
| **F3 — the deny/allow LISTS themselves** | `is_mutation_form` · `is_mutation_head` · `is_expand_time_legal` · `starts_with(":wat::config::set-")` · `binding_metadata[":restricted-to"]` · the `#[restricted_to]` inventory · `ExpandTime::ExpandOnly` registry lookup | **7** | a wall with no list (containment, uid, thread-id, hash) |
| **F4 — the brief's own seed list** | 15 named functions | **15/15 EXIST** | see §6 — two of them are not walls and three more are spelling-independent |

**Where the brief's seed list is wrong** — §6. The short version: it is a **name grep**, and a name
grep counted **three `#[test]` functions** (`form_match.rs`'s `rejects_unknown_head` /
`rejects_non_list` / `rejects_empty_list`) and **one error CONSTRUCTOR**
(`refuse_export_without_arm`, `-> EvalBreak`, which decides nothing) as walls — while missing the
one wall that turned out to be a live capability escape.

### 1.2 The classifier — pipeline order, reused from 255.9/255.10 and re-verified

```
2. collect_entry_file            ← raw symbols
3. resolve_loads                 ← raw symbols        (255.9's cures)
3b. extract_rete_defn_names      ← raw symbols        (255.10)
4. register_defmacros → expand_all ← raw symbols      ⛔ THIS STONE (hygiene Gate E)
5. register_types                ← raw symbols        (255.10)
6. register_defines
7. normalize_symbol_refs   ←←← SYMBOL → KEYWORD, CODE POSITIONS ONLY
8. check_program                                      ⛔ THIS STONE (the capability wall)
9. freeze → run                                       ⛔ THIS STONE (the two mutation walls)
```

⭐⭐ **The rule that found this stone's headline, and the reason a stage-8 wall was NOT safe.**
255.9 and 255.10 both wrote *"anything at or after step 7 in a CODE position cannot see a symbol
head"* — and both then classified stage-8/9 walls as UNREACHABLE on that basis. **The second half of
their own sentence is the hole:** `resolve/boundary.rs` never normalizes a **DATA** position
(`quote` / `forms` / `holon::literal` / a quasiquote template / a `matches?` or `match` arm pattern /
`make-rule`'s quoted `:when`/`:then`), and the codemod's rewrite is **position-independent**
(255.9 §1.1). So a stage-8 wall is safe **only for the code half of its input**. Every one of this
stone's three cures sits on the data half.

---

## 2. ⭐ THE AUDIT TABLE

Disposition key: **FAIL-OPEN** = the unrecognised branch passes the form on (`_ => recurse`,
`Ok(())`, `None`). **FAIL-CLOSED** = the unrecognised branch refuses, or the wall has no name-keyed
branch at all. **UNPROBED** = said so, never assumed.

### 2.1 The walls that dispatch on a NAME — the whole exposed population

| # | wall | what it refuses | unrecognised branch | | probe · LAYER | ⛔ positive control | verdict |
|---|---|---|---|---|---|---|---|
| **1** | **`check.rs::walk_for_restricted_call`** — the capability whitelist. Serves ALL THREE stores: wat `{:restricted-to […]}`, `defstruct` ctor/field whitelists (`declare/register.rs:896`), and the Rust `#[restricted_to(…)]` inventory (`freeze/env.rs:592`) | a MENTION of a restricted binding from a caller outside its whitelist | was `if let WatAST::Keyword(..)` → else fall through to `children()` recursion | ⛔ **FAIL-OPEN** | **RUN + `--check`, stage 8.** One file pair, one mention's spelling: kw rc 1 / sym rc 0; the symbol form **ran the restricted fn** through `eval-ast!` ("ESCAPED, got 7000"). Substrate fences `wat.kernel/spawn-process` · `wat.kernel/close` · `wat.io.IOWriter/from-fd`: kw rc 1 / sym rc 0 | permitted caller `:my::kernel::ok-caller`, SAME mention, BOTH spellings → rc 0, prints `"7"` | ⭐ **CURED** §3 (a) |
| **2** | **`runtime.rs::refuse_mutation_forms_in`** — `eval-ast!` / `eval-edn!` / `eval-file!` | the 10 mutation heads inside constrained eval | `Some(WatAST::Keyword(head,_))` → else recurse | ⛔ **FAIL-OPEN** | **RUN, stage 9 (eval).** `(:wat::eval-ast! '(wat.config/set-redef! true))` → **`ok:nil`** vs `err:eval refused mutation form: :wat::config::set-redef!` | `(:wat::eval-ast! '(wat.i64/+ 20 22))` → `ok:42`, both spellings | ⭐ **CURED** §3 (b) |
| **3** | **`freeze.rs::refuse_mutation_forms`** — `eval_in_frozen` (the Rust embedding API; `eval-digest!` / `eval-signed!`) | the 12 mutation heads (superset: adds `def`, `defsurface`) | same shape | ⛔ **FAIL-OPEN** | **UNIT, stage 9.** `eval_in_frozen((wat.config/set-redef! true))` reached eval pre-cure; refuses with the SAME canonical head post-cure. (No wat verb selects this path from a CLI program — probed at the layer it lives on) | `eval_in_frozen((wat.i64/+ 20 22))` → `42` | ⭐ **CURED** §3 (b) |
| **4** | **`macros/expand.rs::quasiquote_inner` + `check_quasiquote_for_literal_binders`** — hygiene Gate E (arc 249 stone 249.2b-ii, macro name capture) | a quasiquote template that introduces a literal binder | `quasiquote_inner` → `None` → the template is walked as ordinary code, binder scan never runs | ⛔ **FAIL-OPEN** | **`--check`, stage 4 (expand).** kw rc 1 `ProgramBodyIntroducesName{binder:"y"}` / sym rc 0, SILENT | symbol-spelled nested qq with **no** literal binder → rc 0, computes `6` (CLI) / `true` (fixture) | ⭐ **CURED** §3 (c) |
| **5** | **`macros/expand.rs::is_quasiquote_form`** — the defmacro body ROUTER | nothing. It **routes**: `parse_defmacro_form` runs `validate_macro_definition` (hygiene + the F5 purity gate) only when this returns **false** | keyword-only | ⚠ **FAIL-OPEN, but the OPEN direction is the SAFE one** | **read + `--check`.** A symbol-spelled qq body is routed to the program-body path, so it gets MORE validation, not less | — | ⛔ **REPORTED, NOT CURED** §4 (a) — curing it SKIPS a validation |
| 6 | `macros/eval.rs::validate_pure_total` + `is_expand_time_legal` — the F5 default-deny macro purity gate | an impure head in a macro body | allow-list miss → `RefusedInMacro` | **FAIL-CLOSED** (cured 255.10) | **`--check`, stage 4.** `(wat.kernel/println …)` in a macro body → `MalformedDefmacro` in BOTH spellings | pure macro body → rc 0, prints `"3"`, BOTH spellings | ✅ **INTACT — re-probed** |
| 7 | `macros/eval.rs::refuse_expand_only_in_program` — the mirror wall | an `ExpandTime::ExpandOnly` head in ordinary program code | `head_fqdn` → `None` → recurse | FAIL-OPEN **by shape**, spelling-agnostic **in fact** (`head_fqdn` reads Keyword + reference Symbol) | **`--check`, stage 4.** `(wat.core/macro-error "boom")` in a `defn` → `ExpandOnlyOutsideMacro`, BOTH spellings | `mirror_wall_control` (macro-error inside a defmacro still legal) — on the floor | ✅ **INTACT** |
| 8 | `load/loader.rs::scan_for_setter` / `reject_setters_in_loaded` | a `:wat::config::set-*!` in a LOADED (non-entry) file | `canonical_identity_of` → `None` | **FAIL-CLOSED** (cured 255.9) | **`--check`, stage 3.** `SetterInLoadedFile`, **same message**, BOTH spellings | a clean loaded file loads and runs → `"1"` | ✅ **INTACT** |
| 9 | `load/loader.rs::match_load_form` | — (recognition, not refusal; a miss made the load VANISH) | `canonical_identity_of` | **FAIL-CLOSED** (cured 255.9) | inherited; **not re-derived** | the 2-file load chain in row 8's control | ✅ **INTACT (inherited)** |
| 10 | `config.rs::collect_entry_file_inner` / `setter_head_of` — entry-file setter discipline | a setter after a non-setter (`SetterAfterNonSetter`) | `setter_head_of` already reads Keyword **and** Symbol through `canonical_identity` | **FAIL-CLOSED w.r.t. spelling** | **`--check`, stage 2.** The head read is symmetric. ⚠ **The REFUSAL ARM IS UNPROBED** — a setter after a `defn` did **not** fire in EITHER spelling (it is handled by `register_runtime_defs_form`; see the STOP-3 note in `intrinsic/special/config_set_redef.rs`) and I could not construct a position that reaches it | leading setter → rc 0, BOTH spellings | ✅ head read symmetric · ⚠ **refusal arm UNPROBED** |
| 11 | `check.rs` redef wall — `infer_def` → `DefRedefForbidden` | a `def` rebinding a top-level name without `set-redef!` | keyed on the resolved BINDING NAME in `CheckEnv`, stage 8, post-normalize | **FAIL-CLOSED** | **`--check`, stage 8.** Two `def`s of one name → `DefRedefForbidden`, BOTH spellings | every other rc-0 probe in this document defines names once and passes | ✅ **INTACT** |
| 12 | `check.rs::restricted_to_entries` / `restricted_to_entry_spelling`; `types/defstruct.rs:103/236` — the whitelist ENTRY reader | a `:restricted-to` entry that is neither keyword nor symbol | **hard error** (`MalformedForm`), never a silent `filter_map` drop | ⭐ **FAIL-CLOSED by design** (251.8d-i) | **inherited (4 probes), not re-derived** — the brief forbids re-litigating it, and my row 1 does not touch it | inherited | ✅ **INTACT (inherited)** |
| 13 | `load/loader.rs::parse_verify_algo` / `parse_payload_interface` — the `:wat::verify::*` marker slots | an unrecognised verify marker | `MalformedLoadForm` | **FAIL-CLOSED (LOUD)** | **`--check`, stage 3.** `MalformedLoadForm` in BOTH spellings | ⚠ **NO POSITIVE CONTROL** — I could not get a verified load to ACCEPT in either spelling; see §5.4. 255.9 measured **0 corpus users** of any verified-load form | ⚠ **refusal arm symmetric · ACCEPT arm UNPROBED** |
| 14 | `rete/purity.rs::refuse_core_structural_on_multi` | `cond`/`match`/`fn` as a rete primitive on a multi-axis classify | `if let Some(WatAST::Keyword(k,s))` → else `Ok(())`; the body then calls `resolve_core_name(k)` on the raw text | ⛔ **FAIL-OPEN by shape** | ⛔ **UNPROBED.** I did not build a rete rule that reaches this arm, and I am not classifying it from the shape alone | — | ⛔ **UNPROBED — REPORTED** §5.1 |
| 15 | `check.rs::validate_def_positions_in_forms` / `validate_def_position_with_wrapper` | a `def` in a non-top-level position | keyword-only head, `_ =>` recurse **marking children `NonTopLevel`** — the unrecognised branch is the MORE restrictive one | FAIL-OPEN by shape, **restrictive in direction** | stage 8, post-normalize. ⚠ **The refusal arm is UNPROBED** — `:wat::core::def`'s arm is RETIRED from this match (arc 170 Gap I-B); the position error now comes from runtime `DeclarationInExpressionPosition`, which my mutation-wall probe shows fires in BOTH spellings | — | ⚠ **UNPROBED, low exposure** |

### 2.2 The walls that do NOT dispatch on a name — enumerated, classified, and why they are out of the spelling class

Every one of these was read. None has a name-keyed branch, so **no spelling can reach it**; each is
listed so the audit is a census and not a sample.

| # | wall | decides on | unrecognised branch | |
|---|---|---|---|---|
| 16 | `load/loader.rs::ScopedLoader::resolve_within_scope` | `canonical.starts_with(canonical_root)` after `fs::canonicalize` | none — a path is contained or `OutOfScope` | **FAIL-CLOSED**. Its input is a `StringLit` the codemod never rewrites. ⚠ **UNPROBED by me at the wat layer** (no CLI verb selects `ScopedLoader`); its three escape tests are on the floor |
| 17 | `hash::verify_source_hash` / `verify_ast_signature` | a byte comparison of a digest / an Ed25519 signature | mismatch → `EvalVerificationFailed`, **no code runs** | **FAIL-CLOSED**, spelling-independent |
| 18 | `capability/policy.rs::CommsPolicy::admits` | `peer.uid == my_euid` ∧ pid lineage / exact pid | `false` | **FAIL-CLOSED**, no name |
| 19 | `kernel/listener.rs` allowed-pids (`deny` / `PeersDenied`) | pid set membership | not-a-member → denied | **FAIL-CLOSED**, no name |
| 20 | `rust_deps/custodia.rs::ensure_owner` | `ThreadId` equality | `!=` → refuse | **FAIL-CLOSED**, no name |
| 21 | `distribution/mcp.rs::reject_stale_ticket` | `presented == Some(session.ticket)` | **absent ticket refuses** (`None != Some(t)`) | **FAIL-CLOSED** |
| 22 | `distribution/mod.rs::validate_user_grep_signature` | presence + arity + return type of `:user::grep` | absent → error | **FAIL-CLOSED**; the key is a post-freeze symbol-table name |
| 23 | `check.rs::validate_aggregate_containment` · `validate_named_type_annotations` · `freeze.rs::validate_holon_record_capacity` | iterate **every** member of `TypeEnv` | no branch to fall through — the loop visits all | **FAIL-CLOSED**; operate on resolved types, not AST spellings |
| 24 | `types.rs::validate_union_members` | an explicit allow-list of `TypeExpr` shapes | `Fn` / `Var` → `InvalidUnionMember` | ⭐ **FAIL-CLOSED** — the one gate-shaped fn in F1 whose unrecognised branch REFUSES |
| 25 | `types.rs::reject_any` (`AnyBanned`) | `TypeExpr::Path == ":Any"` / `Parametric.head == "Any"` | falls through | Probed: `:Any`, bare `Any`, `wat.core/Any` → **all rc 1**. FAIL-CLOSED in practice (an unrecognised spelling is refused earlier, by `UnknownNamedType`) |
| 26 | `rete/kernel/stratify.rs::refuse_non_terminating` | a rule graph built from runtime `Value`s | ⭐ `NotAnalysable { rules }` — **skips are COUNTED so a skip cannot read as a proof** | **FAIL-CLOSED**; reads `Value`, post-eval, no spelling |

---

## 3. THE CURES — each through the identity door, each with the probe that fails without it

**Diff: 4 `src/` files, 2 test files, 4 new fixtures.** ⛔ **ZERO `.wat` in the live tree.**

**(a) `check.rs::walk_for_restricted_call`.** The `WatAST::Keyword` arm gains a
`WatAST::Symbol(id,_) if id.is_reference()` sibling through `edn::render::canonical_identity`, and
the resulting key is looked up under **both joins** (`types::other_join_spelling`) before the
whitelist test — because `wat.io.IOWriter/from-fd` renders `:wat::io::IOWriter/from-fd` under one
join and `:wat::io::IOWriter::from-fd` under the other, and which is right depends on whether
`IOWriter` is a TYPE, a question this walker has no `TypeEnv` to ask. A restriction fires if
**either** rendering carries one — the restrictive direction. This is 255.10 (e)'s precedent, and the
named recurring class from the injected `CLAUDE.md` (*"a string comparison with one side normalized
and the other not"*) — **fifth instance in arcs 278/255.**

**Why this door.** `canonical_identity` is the exact inverse of the `ns_to_wat_path` the codemod's
`keyword/to-symbol` used to produce the symbol, and it is the door 255.1/255.2/255.6/255.7/255.9/255.10
already routed seven subsystems through. **Rejected:** a second `WatAST::Symbol` arm per lookup (the
shape that cost 255.4 thirty-two reds), and `declare::parse::head_fqdn` — which would work but is the
*declaration-name* door carrying STOP-3's *"do not use the result as a lookup key"*, and this **is** a
lookup key.

**Restraint.** `is_reference()` only: a bare symbol is a binder or a local callable and keeps its
old pass-through, so no `let`-bound name and no macro parameter becomes a restricted mention.

**(b) `freeze.rs::refuse_mutation_forms` + `runtime.rs::refuse_mutation_forms_in`.** Same door, same
`is_reference()` restraint. ⭐ **The refusal reports the CANONICAL identity**, so the diagnostic is
**byte-identical in either spelling** — `is_mutation_form` / `is_mutation_head` (two deliberately
different deny-lists) are untouched, still keyed on `":wat::…"` strings.

**(c) `macros/expand.rs::quasiquote_inner` and `check_quasiquote_for_literal_binders`.** Same door.
⚠ **Both were needed** — curing `quasiquote_inner` alone left the wall silent, because the binder
scan's own `let`/`fn` head test was keyword-only too, so `(wat.core/let [y 1] y)` inside a
now-recognised template still read as an ordinary form. Measured: after the first cure the symbol
probe was **still rc 0**; after the second it is rc 1 with a **byte-identical message**.
⛔ **Its sibling `is_quasiquote_form` is deliberately NOT changed** — §4 (a).

### 3.1 ⭐ THE GUARDS HAVE FAILED ONCE — by targeted revert of the cures under them

`[[A GUARD IS NOT A GUARD UNTIL IT HAS FAILED ONCE]]`. `git stash push -- src/` (tests and fixtures
kept), rebuild, run, pop:

```
PRE-CURE (src stashed, tests present):
  test …restricted_mention_in_data_position_is_denied_in_the_symbol_spelling ... FAILED
  test …restricted_mention_in_data_position_still_admits_a_permitted_caller  ... ok      ← the control
  test …hygiene_bound_fires_on_a_symbol_spelled_quasiquote_template          ... FAILED
  test …a_symbol_spelled_quasiquote_without_a_literal_binder_still_expands   ... ok      ← the control
POST-CURE: all four ok.
```

⭐ **Both positive controls pass in BOTH states.** That is what makes the two REDs mean "the wall was
open", not "the fixture was broken".

⚠ **The mutation-wall test lives in `src/freeze.rs`, so the stash removed it too** — it cannot fail
that way. Its failing-once proof is the **two binaries** (`wat-baseline` from the stashed tree,
`wat-cured`), on the same file:

```
wat-baseline  mut-sym.wat  → "set-redef!  -> ok:nil"                                        rc 0
wat-cured     mut-sym.wat  → "set-redef!  -> err:eval refused mutation form: …set-redef!"   rc 0
wat-cured     mut-kw.wat   → BYTE-IDENTICAL to mut-sym.wat's three lines
```

### 3.2 What the five new rows pin

| file | row | what has to break for it to go red |
|---|---|---|
| `tests/kernel/wat_arc198_def_restricted.rs` | `restricted_mention_in_data_position_is_denied_in_the_symbol_spelling` | the walker's spelling read — and nothing else (typed-error match on callee / enclosing fn / prefixes, no string rendering) |
| same | `…_still_admits_a_permitted_caller` | ⛔ the control: a walker that refuses everything |
| `tests/macros/probe_arc249_macro_engine.rs` | `hygiene_bound_fires_on_a_symbol_spelled_quasiquote_template` | `quasiquote_inner` **or** the binder scan's head read |
| same | `a_symbol_spelled_quasiquote_without_a_literal_binder_still_expands` | ⛔ the control |
| `src/freeze.rs` | `a_mutation_head_is_refused_in_either_spelling_and_nothing_else_is` | the wall's head read; row 3 is the control (`wat.i64/+` still evaluates to 42) |

All assertions are exact `assert_eq!` on the canonical head / typed error fields — never `contains`
(`no_loose_string_assert` is on the floor, and 255.9 went red on exactly that).

---

## 4. WALLS REPORTED RATHER THAN CURED — each with its reason

**(a) ⛔ `macros/expand.rs::is_quasiquote_form` — curing it is the MORE PERMISSIVE direction.**
This is a **router**, not a wall: `parse_defmacro_form` runs `validate_macro_definition` — hygiene
Gate E **and** the F5 default-deny purity gate — only when it returns **false**. Teaching it the
symbol spelling would make a symbol-spelled quasiquote macro body **skip both**. The brief's line is
explicit and this is the case it names. Left keyword-only, with the reason written at the site.
⚠ **The residual asymmetry, stated:** a symbol-spelled top-level quasiquote body now receives the
hygiene walk (via my `quasiquote_inner` cure through `check_program_body_hygiene`) that the keyword
spelling does **not** receive (it routes past `validate_macro_definition` entirely). That is a
divergence in the RESTRICTIVE direction; the honest reading is that the KEYWORD path is the weaker
one, and closing *that* gap is a widening of what gets validated — **the builder's call.**

**(b) ⚠ `rete/purity.rs::refuse_core_structural_on_multi` — FAIL-OPEN by shape, UNPROBED.** I did not
build a rete rule that reaches its arm, so I do not know whether a symbol head arrives. **Reported as
unprobed, not as clean, and not cured** — §5.1.

**(c) 255.9's and 255.10's open lists are UNCHANGED and I did not narrow them**: the `:wat::verify::*`
markers and their `runtime.rs:14272/14343` siblings (row 13); the explicit
`:wat::core::quasiquote`/`unquote` spelling as an 8d-iii blocker (103 lines, 34 files);
`runtime.rs:12674/14723/19970`'s session/`run()` paths (**still unmeasured — I built no session-level
probe either**); `TypeEnv::register_validated`'s `wat.type/X` denotation hole.

**(d) The `:restricted-to` ENTRY reader (row 12) was NOT re-litigated.** The brief forbids it and it
is correct. My row 1 is about the **mention**, a different half of the same mechanism.

---

## 5. ⛔ WHAT MY GREEN CANNOT SEE

1. ⛔ **`refuse_core_structural_on_multi` is UNPROBED and FAIL-OPEN by shape.** Named, not guessed.
2. ⛔ **`ScopedLoader`'s containment wall is UNPROBED at the wat layer.** No CLI verb selects it; I
   read the code and relied on three Rust unit tests I did not write. The TOCTOU window its own
   comment names is also unmeasured.
3. ⛔ **Row 10's refusal arm and row 15's refusal arm never FIRED, in either spelling.** I measured
   that their head reads are symmetric; I did **not** measure that they refuse. A wall I could not
   make fire is a wall I have not tested.
4. ⛔ **Row 13 has NO POSITIVE CONTROL.** I could not get `digest-load!` to ACCEPT a correct digest in
   either spelling (my 4-argument form was malformed and I did not chase the right one). All I
   measured is that it REFUSES identically in both — which is exactly the failure mode the brief's
   "a wall that refuses everything looks intact" warning describes. **Declared, not papered over.**
5. ⛔ **THE DELTA AND THE CENSUS ARE BOTH BLIND TO ALL THREE DEFECTS THIS STONE CURES, AND I CAN PROVE
   IT: both are unchanged to the digit.** Delta 4→4, recovery 0→0, zero original-side rc changes;
   census 212→212, **zero rc changes in either direction across 2202 files**. No file in the 179
   sample mentions a `:restricted-to` binding in a data position, hands a mutation form to
   `eval-ast!`, or nests an explicitly-spelled quasiquote. A gate that cannot move is not evidence
   that the diff is safe — it is evidence that the gate does not look here.
6. ⛔ **The capability escape I proved runs through `eval-ast!`; I did not enumerate the other exits.**
   A quoted symbol mention also survives into `make-rule` clauses, `match` arm patterns and
   `holon::literal`. My cure fires on the MENTION, so it covers them by construction — but that is an
   argument about where the walker recurses, not four more measurements.
7. ⛔ **"FAIL-CLOSED by construction" in §2.2 is a READING, not a probe** for rows 16–24 and 26. I read
   each one and named its deciding expression. None was executed by me.
8. ⛔ **The UNREACHABLE half of 255.9's/255.10's census is still an argument, and this stone is the
   evidence that arguing it is dangerous.** Both stones marked stage-8/9 sites UNREACHABLE from
   pipeline order; the capability wall is a stage-8 site and it was **LIVE**, because the argument
   silently assumed a code position. ⭐ **There is still no gate on pass ordering, and now there is a
   worked example of the reasoning failing.** Recommended for the third time.
9. **My cures widen three default-deny walls' INPUT.** For a keyword head nothing changes. For a
   namespaced symbol head they are strictly more restrictive than the status quo, which consulted
   **nothing**. ⚠ **No probe here constructs a name whose two join renderings disagree in
   restriction status**; I could not build one that is also reachable. Unmeasured, and declared so.
10. **`--check` is not `run`.** 156 of the 179 converted files check clean; only the probes in §2/§3
    and the corpus census actually RAN. 255.10's disclosure stands.

---

## 6. ⭐ WHAT THE BRIEF GOT WRONG — the nineteenth correction, and four more

⭐⭐ **(1) THE INHERITED "KNOWN STATE" ROW IS WRONG, AND IT IS THE ROW THE BRIEF SAYS NOT TO TOUCH.**
The FINDING's *"✅ `:restricted-to` — INTACT, 4 probes incl. a positive control … ⛔ Do not
re-litigate"* is true of the four positions it probed — the whitelist ENTRY (both spellings), the
CALL HEAD (backstopped by normalize), the converted fixture, and the permitted caller. **It is false
as a statement about the wall**, because the wall's own design ruling is *"a restriction governs
MENTION, not head position"*, and the mention position it does not cover — **data** — is exactly the
one `normalize_symbol_refs` never reaches. Measured live, both `--check` and RUN, with the restricted
fn executing. ⛔ **The brief's `❓ unprobed` row for the Rust-side `#[restricted_to(…)]` sites is also
wrong in kind:** they are not *"a different mechanism"*, they are the same `binding_metadata` map and
the same walker, and they were open with it.

⭐ **(2) THE SEED LIST IS A NAME GREP AND IT COUNTS FOUR NON-WALLS.** All 15 named functions exist —
but `form_match.rs`'s `rejects_unknown_head` / `rejects_non_list` / `rejects_empty_list` are
**`#[test]` functions inside `mod tests`**, and `refuse_export_without_arm` is an **error
constructor** (`-> EvalBreak`) that makes no decision and has no unrecognised branch. The brief's
*"7 of 8 sampled dispatch on `WatAST::Keyword`"* cannot be true of any of them. Three more it names
(`refuse_non_terminating`, `validate_aggregate_containment`, `validate_holon_record_capacity`) do not
touch an AST at all — they read `Value` or `TypeEnv`. **And the grep missed the wall that mattered:**
`walk_for_restricted_call` is not named `refuse_*` or `validate_*`.

⭐ **(3) 255.9's MUTATION-WALL DISPOSITION IS WRONG.** *"I found no mutation head that escapes both
walls"* — **two do.** `:wat::config::set-redef!` and `:wat::config::set-eval-redef!` return
**`ok:nil`** in the symbol spelling. 255.9's three probes (`load-file!`, `core::defmacro`,
`config::set-global-seed!`) happened to pick three that ARE backstopped. The full ten-head matrix is
in §2; the two that escape are the two whose eval arms are reachable `Ok(Value::Unit)` returns rather
than `DeclarationInExpressionPosition` raisers. ⚠ **The blast radius today is nil** — the eval arm is
documented to ignore its argument entirely, the flag having been committed at freeze time — but the
**wall** was open, and "the escape is currently harmless" is not a disposition.

⭐ **(4) 255.10's CENSUS ROW FOR THE QUASIQUOTE DISCRIMINANTS IS WRONG.** It reads *"UNREACHABLE for
the sugar; LOUD for the explicit spelling"* and left them alone. The sugar half is right. The
explicit half is **SILENT**: a name-capturing macro loads rc 0, and Gate E never speaks.

⚠ **(5) The brief's delta baseline of 4 is EXACT** (measured 4, recovery 0, on the committed
179-path list, sha256 `33ede…1fbe3` used as-is). Its floor figure of 5963 is now **5968** — five
rows added here. No correction, recorded for the next hand.

---

## 7. THE GATES — evidence, not claims

### 7.1 Delta — **4 = baseline 4**, ⭐ **RECOVERY 0**

`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`, 179 paths, sha256
`33ede76cbb9b8809ecebc7656db3d4733769df54c2641b7e6d798773f1cbc3e5` — **used as committed, never
rebuilt**. All 179 present (`missing=0`). ⚠ **Originals AND converted both checked from copies**
under scratch (the asymmetry that manufactured a phantom regression in the 255.9 weigh).

| binary | orig clean | conv clean | **NEW (clean→broken)** | ⭐ **RECOVERY (broken→clean)** |
|---|---|---|---|---|
| **pre-cure** (`eb860cb96` src) | 160/179 | 156/179 | **4** | **0** |
| **cured** | 160/179 | 156/179 | ⭐ **4** | ⭐ **0** |

**Gate ✅ 4 ≤ 4. RECOVERY 0 — nothing went broken→clean, so no checker was disarmed.**
`join`ing the two runs shows **0 rc changes on the ORIGINAL side across all 179**, and 0 on the
converted side: the cures are invisible to this sample in both spellings. ⛔ **§5.5 says what that
means and what it does not.**

⭐ **The RECOVERY column, institutionalised — I did not do it.** 255.9 recommended it as a standing
gate; 255.10 printed it and recommended it again; this is the third stone to print it by hand and
the third to leave it un-institutionalised. **Saying so, per the brief's ask.** It is one `awk` clause
(`$2!=0 && $3==0`) in the same join and it belongs in `scripts/`.

### 7.2 Census — `no STOP-8`, and **zero changes in either direction**

⚠ `[[DIFF LIKE AGAINST LIKE]]` — comparability CHECKED BEFORE USE, not assumed from recency:
`.census/2026-09-22T06-32-56Z.txt` was produced by 255.10's cured binary, and
`git diff 465728902..eb860cb96 -- src/` is **EMPTY**, so it was produced by a binary whose `src/` is
identical to this stone's parent. (The newest artifact is not always the right one — the
`03-34-24Z` census has 343 failures because it was captured under the 8d-ii converted-stdlib tree.)

```
baseline:   .census/2026-09-22T06-32-56Z.txt   files=2202   nonzero 212
this stone: .census/2026-09-22T07-20-15Z.txt   files=2202   nonzero 212
$ ./scripts/replay/census.sh --diff .census/2026-09-22T06-32-56Z.txt .census/2026-09-22T07-20-15Z.txt
census-diff: no STOP-8                                                            (rc 0)
$ join <(sort BASE) <(sort CURR) | awk '$2!=$3' | wc -l
0
```

### 7.3 Clippy

```
$ cargo clippy --all-targets --workspace -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.03s
clippy exit=0
```

### 7.4 Floor

#### ONE run, on a FROZEN tree — `.floor/2026-09-22T07-21-10Z/`

```
     Summary [ 323.300s] 5968 tests run: 5968 passed (10 slow), 22 skipped
[floor] doctests exit=0
[floor] exit=0. Log kept at .floor/2026-09-22T07-21-10Z/ regardless — a green run is evidence too.
```

**0 `FAIL` lines in the whole captured log** (`grep -c FAIL` = 0); **no `ARM.txt` was written** (the
script writes one only on a red). 5963 → **5968**: the five rows this stone adds.

⭐ **The tree was frozen before the run and nothing was touched during it** — 255.10 made that
process error twice in one stone and wrote the lesson down; this is the first floor since. Nothing
was re-run to see whether it would go green, because nothing went red. The Summary line is what I
read, never a piped exit code (`./scripts/floor.sh | tail` returns `tail`'s — the trap 255.9 walked
into).

⚠ **DISCLOSED — what the floor's tree did and did NOT contain.** All four new fixtures and both
modified test files were on disk before the run started. **This SCORE was not** — it is written
after the gate, as every SCORE is. Several lint tests read `docs/` (`no_new_broken_doc_link`,
`no_inlined_wat_in_tests`, `no_inlined_edn`, …), so the whole lint binary was re-run with this file
present rather than argued about: `cargo nextest run --release -E 'binary(lint)'` →
`Summary [241.807s] 360 tests run: 360 passed, 0 skipped`.

### 7.5 ⛔ NOT ONE `.wat` CONVERTED

```
$ git status --porcelain -- '*.wat'
?? tests/kernel/wat_arc255_11_restricted_symbol_mention_in_data_denied.wat
?? tests/kernel/wat_arc255_11_restricted_symbol_mention_permitted_ok.wat
?? tests/macros/probe_arc255_11_hygiene_symbol_control.wat
```
Four NEW fixtures (the fourth is `…_hygiene_symbol_quasiquote.wat.bad`), written by hand as this
stone's non-vacuity rows. **No existing `.wat` in the tree was modified** — every conversion in this
SCORE ran on copies under `/home/john/.claude/jobs/edaabf97/tmp/`.

---

## 8. REPRODUCTION

```bash
cargo build --release
cargo test --release --lib a_mutation_head
cargo test --release --test kernel -- restricted_mention
cargo test --release --test macros -- symbol_spelled hygiene_bound_fires
cargo clippy --all-targets --workspace -- -D warnings
./scripts/replay/census.sh && ./scripts/replay/census.sh --diff <BASE>.txt <CURR>.txt
./scripts/floor.sh
# the capability escape, on the pre-cure binary:
#   git stash push -- src/ && cargo build --release   → wat-baseline
#   wat esc-sym.wat  →  rc 0  "ESCAPED, got 7000"
# delta: copy the 179 committed paths TWICE, codemod ONE copy, check BOTH from copies
printf '[…179 abs paths…]\n' | ./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat
```

Scratch (not committed): `/home/john/.claude/jobs/edaabf97/tmp/` — `wat-baseline` / `wat-cured`,
`delta/{orig,conv,pre.tsv,post.tsv}`, `w/` (every probe file in §2).
Committed: the four `src/` files, the two test files, the four fixtures, and this SCORE.
