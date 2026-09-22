# SCORE — STONE 251.8d-ii (FIFTH DRAW): the purity head

Branch `main`, drawn against `8cce51d54`. Parent: `BRIEF-STONE-251.8d-ii-FIFTH-the-purity-head.md`.
**Not pushed.** Every number below was produced this session, on a binary I built; `cargo build
--release` was run after **every** `src/` edit and each row names which binary produced it.

## VERDICT — **LANDED (the cure). STOP (the conversion).**

⭐ **The door is cured, and the probe that fails without it is a one-line behaviour flip on an
UNCONVERTED tree** — no `.wat` touched, no conversion needed to see it:

```
(:wat::rete::pure? '(:wat::core::< 1 2))   ->  true    true
(:wat::rete::pure? '(wat.core/< 1 2))      -> FALSE →  true      ⭐ the cure
```

⭐ **THE LEDGER: 232 → 229, and the count is the QUIET half of it.** `intrinsic_meta` **3 [Bx3] → 0**.
`effectful_by_prefix` **8 [Bx8] → 8 [Ax8]** — the same count, a better class, and ⛔ **the ratchet
could not see that**, because it froze counts and not shapes. It does now (§5.3), and the new arm has
failed once by construction. **Crate-wide shape B: 14 → 3.**

⭐ **AND THE 416 MOVED: the converted floor is 416 → 283.** `comm` on the two sorted failing-test
sets: **133 fixed, 0 new** — a strict subset. `grep -c "is not proven pure"` over the whole converted
log is **0**: the 131-strong class the fourth draw named as *"the single highest-value target for the
fifth draw"* is **gone, not reduced**.

⛔ **283 is still RED, so the conversion does NOT land.** `wat/` restored (`git checkout -- wat/` +
rebuild — the fifth time this recovery has been proven). What lands is `src/rete/purity.rs` +
`tests/lint/keyword_heresy_ledger.rs` + two new test files, on which the floor is **6002 / 6002
GREEN**.

⛔ **THREE CORRECTIONS TO THE BRIEF, and the first is the twenty-third in the series** — the brief's
own address table quotes a line whose `_` arm is **provably dead code** (§8).

---

## 1. ⭐ THE ADDRESS — the orchestrator verified the FILE; the LINE it quotes is dead code

255.13's lint named `src/rete/purity.rs` by data flow and the brief carried that forward correctly.
Its three-row address table is right about two rows and wrong about the one it starred.

| the brief says | measured here |
|---|---|
| `purity.rs:1232` — `classify_expr`'s head extraction — ⛔ *"a SYMBOL head becomes the literal string `"<structural form>"`"* | ⛔ **FALSE, and the arm is UNREACHABLE.** Line 1232 is inside the **RetePrimitive structural-guard refusal arm**, whose match guard (line 1225, at `HEAD`) is `matches!(items.first(), Some(WatAST::Keyword(k, _)) if …)`. The arm only matches when `items.first()` IS a `Keyword`, so the `_ => "<structural form>"` fallback on the next line is dead. **No symbol head can reach it.** |
| `purity.rs:260` — `intrinsic_meta(head: &str)`, keyword-keyed | ✅ confirmed |
| `purity.rs:2116` — `effectful_by_prefix(head: &str)`, prefix tests | ✅ confirmed |

⭐ **The real read is 113 lines further down** — `classify_expr`'s **general-list arm**
(`HEAD:1345-1359`), which is exactly what 255.13's lint printed without being told:

```rust
let head = match head_node {
    None => return Ok(()),
    Some(WatAST::Keyword(k, _)) => k.as_str(),
    Some(WatAST::Symbol(id, _)) => id.as_str(),   // ← `wat.core/<`, RAW
    Some(other) => return Err(AxisViolation::at(other.span().clone(), "<non-keyword/symbol head>", axes[0])),
};
```

The distinction is not pedantry: the brief's version says the gate **loses** the name (it becomes a
placeholder string), and the fix for that would be to teach the structural guard a second spelling.
The code says the gate **keeps** the name and **mis-keys** it — 255.13's shape **B**, *dual-raw* —
and the fix for that is a door on the read. ⛔ **They are different cures.** The structural guard is
deliberately keyword-only (§9.4) and I did not touch it.

### 1.1 ⚠ AND THE WAT SURFACE ALMOST CANNOT REACH THIS DOOR AT ALL — which decides the fixtures

Before writing one line of cure I tried to reproduce the failure from a user program. **I could not**,
four ways, and the reason is load-bearing for every row below:

```
(wat.core/defn user/my-sort [coll :- …] :- …
  (wat.core/sort$native (wat.core/fn [a :- … b :- …] :- wat.core/bool (wat.core/< a b)) coll))
   → "[1 2 3]"   rc 0      ⛔ WORKS pre-cure
```

`resolve::normalize` (`normalize_symbol_refs` + `normalize_stored_function_bodies`, freeze steps 7
and 7.7) rewrites a namespaced `WatAST::Symbol` call head to its `Keyword` **before** anything
classifies it. Measured on the pre-cure binary: a symbol head inside a `(:wat::rete::where …)` is
normalized too (an unresolvable one raises `UnresolvedReference` at load, naming `:not::a::real::op`
— the *normalized* path — so the rewrite provably ran there).

⭐ **The one wat-surface position a namespaced Symbol survives to the purity walk is a QUOTE
BOUNDARY**, which `normalize_form` deliberately does not descend — i.e. exactly the argument of
`:wat::rete::pure?` / `deterministic?` / `total?` / `primitive?`. That is why every wat row in
§3 quotes its subject: an unquoted row would prove `normalize` works, not that the door does.

The other position is the **converted stdlib**, whose `defclause` clause bodies produced the fourth
draw's 468 log occurrences at `wat/core.wat:1555`. That one costs a 23-minute conversion to observe,
and it is §7.

---

## 2. THE CURE — one door, one arm, and the KEYWORD side is byte-identical

`src/rete/purity.rs`, `classify_expr`'s general-list arm:

```rust
let head: std::borrow::Cow<'_, str> = match head_node {
    None => return Ok(()),
    Some(WatAST::Keyword(k, _)) => std::borrow::Cow::Borrowed(k.as_str()),
    Some(WatAST::Symbol(id, _)) => {
        std::borrow::Cow::Owned(crate::edn::render::canonical_identity(id.as_str()))
    }
    Some(other) => return Err(AxisViolation::at(other.span().clone(), "<non-keyword/symbol head>", axes[0])),
};
```

**`edn::render::canonical_identity` is the door the ledger's own failure text prescribes** for *"a
bare name string"*, and it is one of the eight members of the door set 255.13 derives by fixpoint
from its two seeds — not a comparator I minted.

⭐ **Only the Symbol arm takes it, and that is a decision, not an omission.** 255.13 §2.2's DUAL-ARM
RULE: *the `Keyword` payload IS the internal identity, keyword-spelled by construction.* Leaving that
arm byte-identical means **nothing on the keyword side can move** — including the one case where
`canonical_identity` would have re-spelled a keyword: a leading-colon head with **no** `::` and a
`/` (`:Client/rep`) becomes `:Client::rep`, which `accessor_meta` — whose whole predicate is
`head.contains('/')` — would then miss. Routing the keyword arm through the door would have been a
silent, unfixtured behaviour change in a door I was not sent to touch.

⛔ **One door covers every table below it.** `head_ok` consults, in order: `constructor_meta`,
`accessor_meta`, `sym.has_function` → `classify_fn`, `rete_vocabulary_admitted` / `rete_op_for`,
the `ClassifyCtx::Runtime` environment lookup, then `is_effectful_op` / `intrinsic_meta`. All seven
now see an identity. **No second spelling test was added to any table** (the brief's §1 constraint).

### 2.1 Why a re-spelling cannot forge a green

A purity gate is DEFAULT-DENY, so this cure runs in the **permissive** direction and that is the
whole risk. What bounds it: `canonical_identity` is a total, injective-on-namespaced-names
re-spelling of a `(namespace, name)` pair. It maps `wat.core/<` onto `:wat::core::<` **and nothing
else**; it cannot invent table membership. An impure head keeps its impure identity
(`wat.kernel/println` → `:wat::kernel::println`, which `effectful_by_prefix` still refuses); an
unknown head keeps its unknown identity (`not.a.real/op` → `:not::a::real::op`, in no table); and a
**bare** symbol with no namespace is returned byte-identical, which is what keeps `head_ok`'s
environment-lookup door (`env.lookup(head)` for a local binding like `keyfn`) working.

### 2.2 The probe that fails without it

`src/rete/purity.rs` reverted to `Cow::Borrowed(id.as_str())` — the exact pre-cure read — and nothing
else; `cargo build --release`; then both suites:

```
Summary [5.191s] 9 tests run: 2 passed, 7 failed, 6015 skipped

the purity head's identity door answered wrongly on 3 row(s):
  :user::lt-sym-pure? → false (want true) — THE CURE: wat.core/< is the same verb as :wat::core::<
  :user::uuid-sym-pure? → false (want true) — THE CURE, a second verb
  :user::rete-sym-primitive? → false (want true) — THE CURE, through the rete vocabulary
```

and, at the struct level, the located reasons the wat surface cannot show:

```
a_proven_pure_head_is_admitted_in_both_spellings        left: Some("wat.core/<")        right: None
a_second_effectful_namespace_is_refused_in_both…        left: Some("wat.io/read-file")  right: Some(":wat::io::read-file")
an_impure_head_is_refused_in_both_spellings_with…       left: Some("wat.kernel/println") right: Some(":wat::kernel::println")
an_unknown_head_is_refused_in_both_spellings            left: Some("not.a.real/op")     right: Some(":not::a::real::op")
law_a_still_refuses_a_core_spelled_head_in_both…        left: Some("wat.core/<")        right: Some(":wat::core::<")
the_door_re_spells_a_name_it_does_not_hand_out_a…       left: Some("wat.uuid/v4")       right: None
```

Restored (`cp` back from the pre-revert copy) + rebuilt; **9 / 9 green.**

---

## 3. ⛔ THE NON-VACUITY ROWS — the stone, and they are asserted in BOTH tiers

The brief asks for three families in one test. There are **four**, plus a positive control, and they
live in one `#[test]` (`tests/rete/probe_arc251_8d_purity_head_identity.rs`), asserted as a whole
table so a failure prints every wrong row rather than the first. The co-located fixture is
`tests/rete/probe_arc251_8d_purity_head_identity.wat` (17 entry points, all quoting their subject).

| row | PRE-CURE | POST-CURE | what it pins |
|---|---|---|---|
| `lt-kw-pure?` `'(:wat::core::< 1 2)` | true | true | ⛔ control — the keyword side may not move |
| ⭐ `lt-sym-pure?` `'(wat.core/< 1 2)` | **false** | **true** | **THE CURE** |
| `uuid-kw-pure?` | true | true | control |
| ⭐ `uuid-sym-pure?` `'(wat.uuid/v4)` | **false** | **true** | THE CURE, a second verb |
| `println-kw-pure?` | false | false | `effectful_by_prefix` `:wat::kernel::` |
| ⭐ **`println-sym-pure?` `'(wat.kernel/println "x")`** | **false** | **false** | ⛔ **THE ADVERSARIAL ROW — a genuinely impure comparator stays refused** |
| `io-kw-pure?` / ⭐ `io-sym-pure?` | false / false | false / false | a second impure prefix, so the row is not one namespace wide |
| `unknown-kw-pure?` / ⭐ `unknown-sym-pure?` | false / false | false / false | ⛔ **unknown stays unknown** — re-spelling mints no membership |
| ⭐ `bare-sym-pure?` `'(nope 1)` | false | false | a namespace-less symbol names nothing |
| `uuid-kw-det?` / ⭐ `uuid-sym-det?` | false / false | false / false | ⭐ **THE AXES DO NOT COLLAPSE** — `v4` is pure ∧ NON-deterministic in both spellings |
| `lt-kw-primitive?` / ⭐ `lt-sym-primitive?` | false / false | false / false | ⭐ **LAW A IS NOT LOOSENED** — a core-spelled op is not a rete primitive, either spelling |
| `rete-kw-primitive?` | true | true | the positive control that keeps law A's rows from passing on a gate that refuses everything |
| ⭐ `rete-sym-primitive?` `'(wat.rete.i64/< 1 2)` | **false** | **true** | THE CURE, through the rete vocabulary |

⭐ **Exactly three of seventeen rows flip, and all three flip toward `true`.** The ten refusal rows
are green on **both** binaries — which is what makes them a test of the **wall** and not a test of
the cure (255.12's own discipline).

### 3.1 ⭐ THE LOCATED HALF — the adversarial row's *reason*, not just its verdict

`:wat::rete::pure?` answers a **bool**; the brief's row 2 is about the **located reason**. That lives
on `AxisViolation` (`span` + `head` + `axis`), which is `pub(crate)` and invisible to `tests/`, so
the rows are eight unit tests beside the door (`src/rete/purity.rs`, `mod purity_head_identity_tests`,
excluded from the ledger as `#[cfg(test)]`). Every assertion is on the **struct fields** — never a
rendered string — and each one also asserts the violation is located at the **head node's own span**,
so a future edit that replaced the head NODE (rather than the head STRING) could not hide behind a
correct `head`.

```
an_impure_head_is_refused_in_both_spellings_with_the_same_located_reason
    THE WALL  (green on both binaries): both spellings produce Some(_)
    THE CURE  (red pre-cure):  Some(":wat::kernel::println") == Some(":wat::kernel::println")
                               pre-cure it was Some("wat.kernel/println") vs Some(":wat::kernel::println")
```

⛔ **State the direction plainly: pre-cure the symbol spelling WAS refused — but as an unknown name
falling off the end of the table, not by the effectful-namespace guard that exists for it.** The cure
does not add a refusal; it makes the two spellings of one effectful verb carry **one** reason. Every
adversarial unit test now asserts the wall (`is_some()`) FIRST and the identity SECOND, so the
equality can never be satisfied by two spellings that are both silently admitted.

### 3.2 ⭐ THE ADVERSARIAL SEARCH — what I tried to make the cure wrongly admit, and what I found

The brief: *"If you cannot construct a case that the cure wrongly admits, say so."* I constructed
four candidate attacks. **Three are refuted. The fourth found a real asymmetry, and it runs in the
STRICT direction, not the permissive one.**

1. **Re-spell an impure verb into a pure row.** Refuted by construction: `canonical_identity` maps
   `ns/name` → `:ns::name` and touches nothing else, so `wat.kernel/println` can only land on
   `:wat::kernel::println`. Measured, both tiers.
2. **Re-spell an unknown head into membership.** Refuted: `not.a.real/op` → `:not::a::real::op`,
   still in no table; the refusal now *names* the identity instead of the written text.
3. **Collapse the axes** — get `Pure` admission to drag `Deterministic` with it. Refuted:
   `wat.uuid/v4` is admitted on `Pure` and refused on `Deterministic`, identically to
   `:wat::uuid::v4`. (This is the row that would have caught a cure that returned `Ok(())` for any
   head it could canonicalize.)
4. **Smuggle a core-spelled op into a rete `where`.** Refuted: `primitive?` on `'(wat.core/< 1 2)`
   is **false** post-cure. Law A's deny is `head_ok`'s fallthrough and re-spelling does not reach it.

⚠ **And the thing the search DID find** — `classify_expr`'s quote/quasiquote/`holon::literal` **data
guard** is a keyword-only `matches!` on the AST node, and my door sits **downstream** of it:

```
(:wat::rete::pure? '(:wat::core::quote (:wat::kernel::println "x")))  ->  true   ← data, not walked
(:wat::rete::pure? '(wat.core/quote    (:wat::kernel::println "x")))  ->  false  ← walked as a CALL
```

The symbol spelling is treated as a call, so its "arguments" are classified as code and the impure
inner form is found. ⛔ **That is an asymmetry, and it is STRICTER, not looser** — it cannot forge a
green. I did **not** cure it: teaching a *data* guard a second spelling makes it swallow more as
data, which is the false-green direction, and 255.11 §4(a) already ruled that class the builder's
call. **Reported, not cured** (§9.4).

---

## 4. ⭐ THE CENSUS THE BRIEF ORDERED — every other caller of both helpers

⛔ **The brief's warning is exactly right and it fired: one of the two helpers is NOT "fixed for
free."** Measured with `grep -rn '<fn>(' src/ tests/ crates/ --include=*.rs`, then each hit read.

### `intrinsic_meta(head: &str)` — 5 real call sites, all in `src/rete/purity.rs`

| site | what it is handed | verdict |
|---|---|---|
| `:992` / `:1001` / `:1010` — `head_ok`'s Pure / Deterministic / Total arms | `head_ok`'s `head` param | ⭐ **CURED by this stone** — `head_ok`'s only spelling-bearing caller is `classify_expr`'s general arm |
| `:1457` — `classify_fn`'s `FunctionBody::Native` arm | `fqdn`, a `SymbolTable` registration key | canonical **by construction** |
| `:1485` — `classify_native_fn(path, …)` | `path`; callers are `freeze.rs:813` (`&label`) and `collection/transform.rs:331` (`&comparator_label`), both `func.name` = the registration name | canonical **by construction** |
| `:2798` — the purity-COMPLETENESS gate | `dispatch_verbs(runtime.rs source)` — keyword literals scraped from Rust source | canonical, and `#[cfg(test)]` |
| `intrinsic/mod.rs:3630` | ⚠ **not a call** — a `&str` of the fn's *signature line*, used by a source-reading lint | n/a |

⭐ **And the ledger agrees without being told: `intrinsic_meta` 3 [Bx3] → 0.**

### `effectful_by_prefix(head: &str)` — 2 call sites, and the second one keeps the row open

| site | what it is handed | verdict |
|---|---|---|
| `purity.rs:2136` — inside `is_effectful_op` | `is_effectful_op`'s `head` param | ⛔ **partially cured** — see below |
| `intrinsic/mod.rs:2951` | `entry.name`, a registry row's name | canonical by construction |

`is_effectful_op` itself has **two** callers:

| site | what it is handed | verdict |
|---|---|---|
| `purity.rs:989` — `head_ok`'s Pure arm | `head_ok`'s `head` | ⭐ **CURED by this stone** |
| ⛔ `runtime.rs:13199` — `step_list`'s effectful refusal | `&head_kw`, built one screen above from `WatAST::Keyword(k, _) => k.clone()`, with the `WatAST::Symbol` arm returning `NoStepRule` **before** this line | ⛔ **KEYWORD-ONLY. Still open.** |

⭐ **So `effectful_by_prefix` stays in the ledger at 8 sites — but as shape A, not shape B.** It is
no longer fed a value that joins both payloads raw; its remaining non-door feed is a read that can
only ever be a keyword, because the step engine refuses a symbol head before the purity test runs
(the code's own comment: *"Bare-symbol heads … need a higher-order step rule that hasn't shipped
yet. Phase 3 territory."*). ⛔ **I did not cure it.** Making `step_list` see symbol heads is a change
to the step/normalizer contract, it is the thing that `Phase 3` note defers, and a `canonical_identity`
wrapped around a value that is already a keyword would be theatre that moved a ledger number without
moving a behaviour.

### The third shape-B site in the same file, named and NOT cured

`rete/purity.rs::walk_rete_defn_callees` (the rete-defn **cycle detector**) holds the identical
one-line dual-raw read and remains **Bx1**. It is a different decision — it feeds
`resolve_core_name` and a quote-family guard, not a purity table — and curing it widens what a cycle
detector recurses into, unfixtured, in the permissive direction. **Reported, not cured.** With
`rete/kernel/arm.rs`'s two (255.13 §4.3), it is one of the **three** shape-B sites left in the crate.

---

## 5. ⭐ THE LEDGER — 232 → 229, and the shape mix is the real number

`cargo nextest run --release -E 'test(keyword_heresy)'` — **4 / 4 green** at the new freeze
(1.14 s).

### 5.1 What moved

| row | before | after |
|---|---|---|
| `src/rete/purity.rs` `intrinsic_meta` | 3 `[Bx3]` | ⭐ **gone (0)** |
| `src/rete/purity.rs` `effectful_by_prefix` | 8 `[Bx8]` | ⭐ **8 `[Ax8]`** — same count, better class |
| `src/rete/purity.rs` `classify_expr` | 3 `[Ax3]` | 3 `[Ax3]` — the quote/quasiquote data guard, §9.4 |
| `src/rete/purity.rs` `walk_rete_defn_callees` | 1 `[Bx1]` | 1 `[Bx1]` — §4 |
| `src/rete/purity.rs` `is_declaration_derived_construction` | 2 `[Ax2]` | 2 `[Ax2]` |
| **LEDGER_TOTAL** | **232** | ⭐ **229** |

**Shape mix, crate-wide:** A 93 → **101**, B 14 → **3**, C 0, E 125 → 125. (A rises by 8 because
`effectful_by_prefix`'s eight sites reclassified **into** A out of B; the ledger's dangerous class
fell by 11 of its 14.) ⭐ **Shape B in `rete/purity.rs`: 12 → 1.** The brief predicted *"B should fall
by ~12"* — in COUNT it fell by 3, and the missing 9 are the reclassification the count cannot show.

### 5.2 The verbatim ratchet output that told me the number

```
⭐ THE LEDGER SHRANK — 232 → 229. This is the good direction, and the
ratchet is deliberate: the frozen census below has to be tightened by hand so the number
can never drift back up silently. Update `LEDGER_TOTAL` and the rows named here.

CURED:
  (none)
GONE ENTIRELY:
  − src/rete/purity.rs  fn intrinsic_meta  was 3 [Bx3], now 0
```

### 5.3 ⛔⛔ AND THE RATCHET COULD NOT SEE THE BETTER HALF OF ITS OWN NEWS

The freeze compared **counts only**. `effectful_by_prefix` went `Bx8 → Ax8` and the gate said
**nothing** — and the same silence would have covered `Ax8 → Bx8`, a site **silently acquiring** the
strictly-worse dual-raw class, which is the one thing this ledger exists to count. 255.13 §7 states
the principle it was missing: *"each row's shape tag IS its reason for being in the ledger."* A
freeze that does not check the reason freezes a number.

⭐ **Fixed in the same commit**, as a third arm beside `grew` / `shrank`, and **measured over every
row**: with the shape check armed and the frozen mixes left as 255.13 wrote them, exactly **one** of
**110** compared rows drifts — mine, in the good direction.

⭐ **The new arm has failed once, by construction.** Frozen mix set back to `Bx8`:

```
⇄⇄ A LEDGER ROW CHANGED SHAPE WITHOUT CHANGING COUNT.
  ⇄ src/rete/purity.rs  fn effectful_by_prefix  8 site(s) [Bx8] → [Ax8]  (count unchanged)
```

### 5.4 The calibration moved with it, and was RE-ANCHORED rather than deleted

`the_discriminator_separates_the_cured_from_the_open` held `intrinsic_meta` in its **OPEN** table
with the reason *"THE 251.8d-ii FOURTH DRAW'S DEFECT"*, plus a standing assertion that its mix
contains `B`. Both had to move, and how they moved matters:

- `intrinsic_meta` is now a **CURED** row (`":wat::core::+"`, *"251.8d-ii FIFTH — `classify_expr`'s
  Symbol arm takes `canonical_identity`"*), so a revert of this stone goes red as
  **"THE DISCRIMINATOR CONVICTED A SITE THIS ARC ALREADY CURED"**.
- `effectful_by_prefix` stays **OPEN**, with its reason rewritten to name the new mechanism
  (*"now shape A: its remaining raw feed is `runtime.rs::step_list`"*).
- ⛔ The shape-`B` calibration was **re-anchored, not deleted** — onto
  `rete/kernel/arm.rs::compile_acc_fold` (still `Bx1`, 255.13 §4.3's own find). Deleting it would
  have retired the instrument's only proof that pass C still carries provenance across the call
  graph, at exactly the moment the cure made that proof matter most.

---

## 6. EVERY GATE ON THE LANDABLE STATE, WITH ITS EVIDENCE

| gate | landable state (cure + ledger + tests, `wat/` untouched) |
|---|---|
| `cargo build --release` after every `src/` edit | ✅ exit 0, 22.1–22.2 s each, `Compiling wat` shown |
| ⭐ **`scripts/floor.sh`** | ✅ **GREEN — 6002 / 6002**, 22 skipped, **326.715 s**, exit 0 · `.floor/2026-09-22T22-35-33Z/` |
| floor denominator | 5993 (255.13's landing) **+ 9 of mine** = 6002 ✓ |
| ⚠ floor duration | **326.7 s** — in the healthy band (the two prior greens on disk: 328.5 s and 320.1 s). ⚠ *A fast floor is a symptom*: the critical path is `every_wat_scripts_file_loads_on_the_current_runtime`, 300 s+ on its own, and it ran |
| ⚠ `harvest_wrap_split` | **PASS** [0.140 s] (1563/6002) — named either way, per the standing bookkeeping exception |
| clippy `-D warnings --all-targets --workspace` | ✅ **0 warnings, rc 0** — and **not cached**: `touch src/rete/purity.rs` first, **14.21 s** elapsed |
| census | ✅ **`census-diff: no STOP-8`, rc 0** · 2209 paths · **213 failing = 213 failing** · ⭐ the two census files are **byte-identical** (`diff` of the sorted files is empty) |
| ⭐ `scripts/replay/delta.sh` | ✅ **NEW 3** = baseline 3 (`probe-c1-clean-surface`, `probe-kwargs-peer`, `wat/holon/Ngram.wat`), ⭐ **RECOVERY 0**, exit 0 · ORIG-CLEAN 160/179, CONV-CLEAN 157/179 · list-sha `33ede76c…f1cbc3e5`, paths=179 missing=0 |
| ⭐ ledger | ✅ **4 / 4 green at 229** (`-E 'test(keyword_heresy)'`, 1.14 s) |
| `wat/` clean | ✅ `git status --porcelain` names no `wat/` path |

---

## 7. ⭐ THE CONVERSION — re-measured, and the 416 is the number

⛔ **Operational discipline, stated because the arc has paid for its absence twice this week:** the
64 paths came from `git ls-files | grep -E '^wat/.*\.wat$'` (⚠ **not** `git ls-files 'wat/**/*.wat'`,
which returns 31 of 64), were passed as one explicit EDN vector, and **no `cargo` command and no
`git add` ran while the codemod was writing `wat/`.**

### 7.1 The numbers

| | fourth draw | **this draw** |
|---|---|---|
| conversion | 64/64, rc 0, 1405 s | ✅ **64/64, rc 0, ~1330 s** (22 min); `git status` = exactly those 64 `wat/` paths, nothing else |
| `cargo build --release` on the converted tree | exit 0, 23.01 s | ✅ **exit 0, 22.84 s** |
| binary starts | ✅ | ✅ (the 17-row probe still answers correctly on the converted binary) |
| ⭐ **floor** | ❌ **416 / 5989**, 443.03 s | ❌ ⭐ **283 / 6002**, **453.01 s**, exit **100** · `.floor/2026-09-22T23-07-17Z/` |

⭐ **416 → 283. `comm` on the two sorted failing-test-name sets:**

```
only in the FOURTH draw's 416 (i.e. FIXED here) : 133
only in mine                (i.e. NEW)          :   0
```

⛔ **A strict subset. 133 tests fixed, nothing broken, and none of my 9 new rows is among the 283.**
The brief said *"DO NOT promise 416 − 131"* — correctly, and the answer is **−133**: the 131-strong
purity-spelling class plus two more that the same cure carried with it.

⭐ **The class is GONE, not reduced. `grep -c "is not proven pure"` over the whole converted
`clean.log` returns 0** (the fourth draw's log had 234 such lines across its 416, and 468 occurrences
in the run before that). The single string comparison the fourth draw named as *"the largest single
class … one callee … the single highest-value target for the fifth draw"* is closed.

### 7.2 The 283 that remain — CLASSIFIED, NOT DIAGNOSED

⛔ **Stating that plainly, as the fourth draw did: I classified every one of the 283 whole blocks by
the error its stderr carries. I did not diagnose one of them, and the classes may interact.**

| n | class |
|---|---|
| 123 | `#wat.check/CheckErrors` (other — arity, unknown name, …) |
| 59 | `#wat.check/TypeMismatch` |
| 54 | a bare assertion / panic with no typed wat error |
| 33 | `#wat.resolve/UnresolvedReferences` |
| 6 | `#wat.runtime/MalformedForm` |
| 3 | `StartupError` (other) |
| 1 | the lint that greps `wat/*.wat` as TEXT |
| 4 | unclassified (the `probe_arc278_8custom_native_differential` + journal-backend differentials) |
| **283** | |

By binary: `wat::rete` 105 · `wat` 77 · `wat::lint` 31 · `wat::services` 28 · `wat::cli` 21 ·
`wat::kernel` 4 · `wat::program`/`macros`/`function`/`comms` 3 each · `wat::wat_lang` 2 ·
`wat::types`/`process`/`diagnostics` 1 each.

⚠ `rete::kernel::tests::harvest_cost::harvest_wrap_split` — **PASS [0.175 s] (1566/6002)** on the
converted run and **PASS [0.140 s] (1563/6002)** on the landable one. Named either way, per the
standing bookkeeping exception.

### 7.3 Three arms, whole, verbatim

⛔ Quoting 283 blocks is not possible; every block is on disk untruncated at
`.floor/2026-09-22T23-07-17Z/ARM.txt`, and every one is classified above. Three arms whole:

**(a) the largest class — a converted `:wat::rete::acc::` head loses its arguments**

```
        FAIL [   0.836s] (1523/6002) wat rete::kernel::tests::accum_cost::accum_materialize_split
  stdout ───

    running 1 test
    test rete::kernel::tests::accum_cost::accum_materialize_split ... FAILED

    failures:

    failures:
        rete::kernel::tests::accum_cost::accum_materialize_split

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1341 filtered out; finished in 0.83s
    
  stderr ───

    thread 'rete::kernel::tests::accum_cost::accum_materialize_split' (825783) panicked at src/rete/kernel/tests/accum_cost.rs:1059:10:
    accum-axis world should freeze: #wat.check/CheckErrors {:message "4 type-check errors" :location nil :causes [] :errors [#wat.check/ArityMismatch {:message ":wat::rete::acc::count: expected 1 argument(s); got 0" :location #wat.core/Span {:file "<entry>" :line 11 :col 9 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 11 :col 31}}} :causes [] :callee ":wat::rete::acc::count" :expected 1 :got 0} #wat.check/ArityMismatch {:message ":wat::rete::acc::sum: expected 2 argument(s); got 1" :location #wat.core/Span {:file "<entry>" :line 16 :col 9 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 16 :col 29}}} :causes [] :callee ":wat::rete::acc::sum" :expected 2 :got 1} #wat.check/ArityMismatch {:message ":wat::rete::acc::min: expected 2 argument(s); got 1" :location #wat.core/Span {:file "<entry>" :line 21 :col 9 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 21 :col 29}}} :causes [] :callee ":wat::rete::acc::min" :expected 2 :got 1} #wat.check/ArityMismatch {:message ":wat::rete::acc::max: expected 2 argument(s); got 1" :location #wat.core/Span {:file "<entry>" :line 26 :col 9 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 26 :col 29}}} :causes [] :callee ":wat::rete::acc::max" :expected 2 :got 1}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

⭐ **An `ArityMismatch` of exactly ONE argument, on four sibling verbs.** That is the signature of an
argument being consumed as something else by the lowering, not of a name failing to resolve — worth
the sixth draw's first look, and it is `rete/kernel/arm.rs`'s neighbourhood, where 255.13 §4.3 found
the two remaining shape-B sites. **Named as a lead, not diagnosed.**

**(b) `TypeMismatch` — a rete-var binding read as a keyword (the fourth draw's third arm, unchanged)**

```
        FAIL [   0.899s] ( 633/6002) wat::rete probe_arc278_export::import_refuses_a_driver_tower_past_the_depth_bound
  stdout ───

    running 1 test
    test probe_arc278_export::import_refuses_a_driver_tower_past_the_depth_bound ... FAILED

    failures:

    failures:
        probe_arc278_export::import_refuses_a_driver_tower_past_the_depth_bound

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 520 filtered out; finished in 0.88s
    
  stderr ───

    thread 'probe_arc278_export::import_refuses_a_driver_tower_past_the_depth_bound' (818219) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:890:41:
    freeze: #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::rete::i64::+: parameter #1 expects :wat::core::i64; got :wat::core::keyword" :location #wat.core/Span {:file "tests/rete/probe_arc278_export.wat" :line 24 :col 52 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 24 :col 54}}} :causes [] :callee ":wat::rete::i64::+" :param "#1" :expected ":wat::core::i64" :got ":wat::core::keyword" :remedies []}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

⚠ **The fixture is an UNCONVERTED `tests/rete/*.wat` resolving against a CONVERTED stdlib.** Same
shape the fourth draw named for its `wat-scripts` shard: this class is the corpus/stdlib straddle,
which is 8d-iii's body of work and cannot be cleared from inside 8d-ii.

**(c) `UnresolvedReferences` — a surface-method head the converted stdlib no longer registers**

```
        FAIL [   0.937s] (3868/6002) wat::program probe_arc259_program_init_fn::thread_init_populates_user_program
  stdout ───

    running 1 test
    test probe_arc259_program_init_fn::thread_init_populates_user_program ... FAILED

    failures:

    failures:
        probe_arc259_program_init_fn::thread_init_populates_user_program

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.92s
    
  stderr ───

    thread 'probe_arc259_program_init_fn::thread_init_populates_user_program' (863670) panicked at src/freeze.rs:1176:9:
    call_beside_value: fixture beside "/home/john/work/holon/wat-rs/tests/program/probe_arc259_program_init_fn.rs" failed to freeze: #wat.resolve/UnresolvedReferences {:message "2 unresolved references" :location nil :causes [] :unresolved [#wat.resolve/UnresolvedReference {:path ":wat::spawn::thread/init" :context "call head — not a builtin, not a registered function" :span #wat.core/Span {:file "tests/program/probe_arc259_program_init_fn.wat" :line 11 :col 14 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 11 :col 38}}}} #wat.resolve/UnresolvedReference {:path ":wat::spawn::thread/init" :context "call head — not a builtin, not a registered function" :span #wat.core/Span {:file "tests/program/probe_arc259_program_init_fn.wat" :line 41 :col 14 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 41 :col 38}}}}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

⭐ `:wat::spawn::thread/init` — a **`Type/method` surface join**, the one shape `canonical_identity`
and `reconstruct_call_path` disagree about (`::` vs `/` in the leaf). Second lead for the sixth draw,
and it is the same *"a string comparison with one side normalized and the other not"* family.

### 7.4 ⛔ THE CONVERSION IS RESTORED — the fifth proof of the recovery

```
git checkout -- wat/    →  git status --porcelain names no wat/ path
cargo build --release   →  exit 0, 22.99 s
```

Re-verified on the restored binary: the 17-row wat probe answers exactly as §3's POST-CURE column,
and `-E 'test(purity_head_identity) or test(keyword_heresy)'` is **13 / 13 green**.

---

## 8. ⛔ WHAT THE BRIEF GOT WRONG — three, and the first is the twenty-third

1. ⭐⭐ **THE ADDRESS TABLE'S STARRED ROW QUOTES DEAD CODE.** *"`src/rete/purity.rs:1232` —
   `classify_expr`'s head extraction — `Some(WatAST::Keyword(k, s)) => …, _ => "<structural form>"`
   — **a SYMBOL head becomes the literal string `"<structural form>"`**."* At `HEAD`, line 1232 is
   inside the **RetePrimitive structural-guard refusal arm**, and that arm's match guard (line 1225)
   is `matches!(items.first(), Some(WatAST::Keyword(k, _)) if …)`. **The arm only fires when the head
   IS a Keyword, so the `_` fallback is unreachable and no symbol head has ever produced
   `"<structural form>"`.** The real read is the **general-list arm** 113 lines later
   (`HEAD:1345-1359`) — `Symbol(id, _) => id.as_str()`, the raw written text — which is precisely
   what 255.13's lint printed. ⛔ **The two stories imply different cures**: "the name is lost"
   argues for teaching the structural guard a second spelling (the false-green direction 255.11
   §4(a) warns about); "the name is mis-keyed" argues for a door on the read. This is the
   twenty-third correction, and it is the same *kind* as the twenty-second — a line cited from a
   stack trace's neighbourhood rather than from the data flow.
2. **"B should fall by ~12."** It falls by **3** in the ledger's COUNT (232 → 229) and by **11 of
   14** in the SHAPE MIX, because `effectful_by_prefix`'s eight sites reclassify `Bx8 → Ax8` at an
   unchanged count. The brief's instinct was right and its instrument could not have shown it —
   the ratchet froze counts only, which is why this stone also fixes the ratchet (§5.3).
3. ⚠ **"`intrinsic_meta` and `effectful_by_prefix` … if you normalize at extraction, they are fixed
   for free."** Half true, and the brief's own next sentence is why: `intrinsic_meta` **is** fixed
   for free (3 → 0); `effectful_by_prefix` is **not**, because it is reached through
   `is_effectful_op`, whose second caller (`runtime.rs::step_list`) hands it a keyword-only head
   (§4). The census the brief ordered is exactly what found it.

⚠ **And one thing the brief could not have known, which changed how every row is written:** the wat
surface reaches this door through a **quote boundary only** — `resolve::normalize` rewrites a
namespaced symbol call head everywhere else, so the obvious `sort`-shaped repro returns rc 0 on the
PRE-cure binary (§1.1). A fixture written the obvious way would have been vacuous in both directions.

---

## 9. ⛔ WHAT MY GREEN CANNOT SEE

1. ⛔ **The conversion did not land**, so every converted-state number in §7 describes a tree that
   no longer exists. What is committed is a `src/` cure, a ledger re-freeze and two test files.
1a. ⛔ **The 283 are CLASSIFIED, NOT DIAGNOSED.** I bucketed every block by the typed error its
   stderr carries and quoted three arms whole; I diagnosed none of them, and the third draw's
   warning that *"the classes interact"* is the reason −133 is a measurement and not a model. The
   `ArityMismatch`-by-one and the `:wat::spawn::thread/init` surface-join are **leads I named**, not
   causes I proved.
1b. ⚠ **`133 fixed, 0 new` compares two runs with different denominators** (5989 then, 6002 now —
   255.13's 4 rows plus my 9). The `comm` is over failing-test NAMES, so the added rows can only
   have appeared as `new`, and none did; but the two runs are not the same suite.
2. ⛔ **The wat surface can reach this door through exactly ONE position — a quote boundary.**
   Every `.wat` row I wrote is a `pure?`/`deterministic?`/`primitive?` over quoted data. The
   *other* position (a stored `defclause` body in a converted stdlib) is only observable under a
   conversion, i.e. only in §7. A user program cannot reach the door at all, because `normalize`
   rewrites the head first (§1.1). **My green therefore does not prove the door is reachable in
   any third way, and it does not prove the two known ways are the only two.**
3. ⛔ **`effectful_by_prefix` is still 8 ledger sites.** Shape A, fed by `runtime.rs::step_list`.
   Better class, not zero. Nothing in this stone touches the step engine.
4. ⛔ **`classify_expr`'s quote / quasiquote / `holon::literal` DATA guard is still keyword-only,
   and the two spellings therefore disagree** — measured, §3.2: `'(wat.core/quote <impure>)` is
   walked as a call and refused, where `'(:wat::core::quote <impure>)` is admitted as data. The
   asymmetry is **strict**, so it cannot forge a green, but it is real and it is in the function I
   edited. Curing it widens what a gate swallows as data, which is the false-green direction, and
   255.11 §4(a) already put that class in the builder's hands.
5. ⛔ **The three structural-guard arms (`cond`/`match`/`fn` under `RetePrimitive`) are likewise
   keyword-only**, so a symbol-spelled one falls through to the general arm. Post-cure it is denied
   there by law A (`lt-sym-primitive?` is the row) rather than by the named structural refusal, so
   the VERDICT agrees and the REASON differs. I did not unify them.
6. ⛔ **`walk_rete_defn_callees` keeps the same dual-raw read**, one screen from the one I cured
   (§4). Named, not cured.
7. ⛔ **`canonical_identity` on a keyword is untested by me, because I deliberately did not call it
   there.** My argument that routing the Keyword arm through the door would break `accessor_meta`
   on a single-segment type (`:Client/rep` → `:Client::rep`) is read from the two functions'
   source; I have **no fixture** that declares a record in a single-segment namespace and calls its
   accessor inside a classified expression. It is a reading, not a probe — the exact thing 255.12
   §6.4 flagged about itself.
8. ⛔ **The adversarial search is four constructed attacks, not a proof of non-existence.** I could
   not construct a case the cure wrongly admits; that is a statement about my search.
9. ⛔ **The ledger's own blind spots are unchanged** (255.13 §2.3): it cannot see reachability, a
   keyword-only READ feeding a covered decision, the 68 `D-unresolved` sites, macro-generated code,
   or anything outside `src/`. My 229 inherits every one of them.
10. ⛔ **`delta.sh`'s RECOVERY column is only meaningful against an UNCONVERTED embedded stdlib** —
    the fourth draw proved that (9 RECOVERYs that were a moved denominator, not a disarmed wall).
    My delta row is the landable one.

---

## 10. WHAT THE SIXTH DRAW NEEDS (not started here)

1. ⭐ **The 283** (§7.2). Two leads, both named and neither proved: (a) the `ArityMismatch` of
   **exactly one argument** on four sibling `:wat::rete::acc::` verbs — the signature of an argument
   consumed by the lowering, in `rete/kernel/arm.rs`'s neighbourhood, which is where 255.13 §4.3 put
   the two remaining shape-B sites; (b) `:wat::spawn::thread/init` unresolved — a **`Type/method`
   surface join**, the one shape where `canonical_identity` (`::` in the leaf) and
   `reconstruct_call_path` (`/` in the leaf) disagree. ⭐ **That is the same recurring class again —
   a string comparison with one side normalized and the other not.**
2. `effectful_by_prefix`'s 8 shape-A sites: `runtime.rs::step_list` reads a keyword-only head and
   its Symbol arm is an explicit *"Phase 3 territory"* TODO. That is a step-engine stone, not a
   purity stone.
3. `rete/purity.rs::walk_rete_defn_callees` and `rete/kernel/arm.rs`'s two — **the last three
   shape-B sites in the crate**. Each is the same one-line dual-raw read; each feeds a different
   decision, so each needs its own non-vacuity rows.
4. ⛔ **The spelling asymmetry in `classify_expr`'s DATA guards** (§3.2/§9.4). It is strict today,
   so it forges no green, but the two spellings of `quote` mean different things inside a purity
   walk and that is the builder's call, not a rider's.
5. Shape **E**, 125 sites, 78 in `check.rs` — out of scope here and still the majority of the
   ledger. ⭐ **The terminal cut needs A+E at zero, not just B.**

## Artifacts

- `.floor/2026-09-22T22-35-33Z/` — the landable floor, **6002 / 6002 green**, 326.715 s
- `.floor/2026-09-22T23-07-17Z/` — the **converted** floor, **283 / 6002 red**, 453.012 s, exit 100;
  `ARM.txt` holds all 283 whole blocks untruncated
- `.census/2026-09-22T22-41-53Z.txt` — landable census, byte-identical to 255.13's
  `.census/2026-09-22T20-51-56Z.txt`
- `.delta/2026-09-22T22-42-46Z/` — NEW 3, RECOVERY 0
- `tests/rete/probe_arc251_8d_purity_head_identity.{wat,rs}` — the 17-row wat truth table
- `src/rete/purity.rs` `mod purity_head_identity_tests` — the 8 located-reason rows
- `tests/lint/keyword_heresy_ledger.rs` — the ledger at **229**, with the shape-mix ratchet
