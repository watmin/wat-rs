# SCORE — STONE: the `:then` block never checked its operand

No commit. Floor and clippy left to the orchestrator. `if`/`cond`, `enum::name`, and the `then-item-fence` untouched.

A rete `:then` now checks each operand against the **declared type of the field it lands in**, not a rete segment. `RhsUnresolvableOperand`'s accepted list names the call form. Nested constructors are lowered to `kwargs-construct` before freeze sees them; the check follows that form.

## Row 1 — it builds

`cargo build --release` clean.

## Row 2 — the i64-into-a-String `:then` is REFUSED

```
defrule `fit::arm-a`: `:then` insert of `:fit::Box` field `:label` is declared `:wat::core::String`; operand is `:wat::core::i64`
```

Names the rule, the field, the declared type, and the actual type. `BOXES=1` is a refusal.

## Row 3 — enum-into-a-DIFFERENT-enum is REFUSED

```
defrule `fit::cross`: `:then` insert of `:fit::Box` field `:k` is declared `:fit::Beta`; operand is `:fit::Alpha`
```

A segment check would return `"enum"` for both and pass. This names Alpha and Beta.

## Row 4 — every LEGAL `:then` still compiles

`cargo nextest run --release --test rete` — **382 passed, 0 failed**. fmt/grep rules `--check` clean. `every_wat_scripts_file_loads` **1 passed**.

One shape looked like a scream and was not: `Option/Some` into `(:wat::core::Option :- [:wat::core::Pos])` (`probe-rhs-builds-core-span.wat`). The constructor's declared type is the enum `:wat::core::Option`; the field is parametric. That is not Alpha-into-Beta. `then_types_fit` accepts actual `:E` against declared `(:E :- […])` — the HEAD must still match. After that, the span probe `--check`s clean. STOP-2: the rule was legal; the first exact-string compare was wrong.

## Row 5 — THE SCREAMS, VERBATIM

Imposed the check, `--check`ed 656 files under `wat-scripts/{fmt,grep,fixes,scratch-pad}`, `tests/cli`, `tests/rete` (excluding `.wat.bad`). **7 files, 8 mismatch sites** (EDN double-wraps each finding; unique messages below). `wat/*.wat` freeze is the release build: it was already clean, so stdlib `:then` sites were not among them.

**Class A — `Capture.value <- String` holding a `NodeKind` (7 sites):**

```
defrule `g7::match-arrow`: `:then` insert of `:wat::grep::Capture` field `:value` is declared `:wat::core::String`; operand is `:wat::grep::NodeKind`
  tests/cli/wat_grep__g7_rule.wat:26

defrule `hk::head-kind`: `:then` insert of `:wat::grep::Capture` field `:value` is declared `:wat::core::String`; operand is `:wat::grep::NodeKind`
  wat-scripts/scratch-pad/277-head-kind-census.wat
  (two Captures: :name "kind" ?k and :name "pkind" ?pk)

defrule `ls::child-offset`: `:then` insert of `:wat::grep::Capture` field `:value` is declared `:wat::core::String`; operand is `:wat::grep::NodeKind`
  wat-scripts/scratch-pad/277-layout-shape-probe.wat

defrule `pg::match-arrow`: `:then` insert of `:wat::grep::Capture` field `:value` is declared `:wat::core::String`; operand is `:wat::grep::NodeKind`
  wat-scripts/scratch-pad/probe-grep-cli.wat
  wat-scripts/scratch-pad/probe-grep-driver.wat

defrule `fx::match-arrow`: `:then` insert of `:wat::grep::Capture` field `:value` is declared `:wat::core::String`; operand is `:wat::grep::NodeKind`
  wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat
```

**Class B — parametric Option (1 site, FALSE POSITIVE, withdrawn by `then_types_fit`):**

```
defrule `p::build-hit`: `:then` insert of `:wat::core::Span` field `:end` is declared `(:wat::core::Option :- [:wat::core::Pos])`; operand is `:wat::core::Option`
  wat-scripts/scratch-pad/probe-rhs-builds-core-span.wat
```

Legal: `(:wat::core::Option::Some (:wat::core::Pos …))`. Not fixed by relaxing the fence. Not `enum::name`.

fmt rules, grep rules, recorded fixes: **0** mismatches. The 319 `:then` sites were mostly already honest. The hole was Capture.value, plus the Option head/parametric pair.

## Row 6 — `g7_rule.wat:26` is among them

Quoted above. Line 26, field `:value`, declared `:wat::core::String`, actual `:wat::grep::NodeKind`. The check flags the known scream. Nested constructors arrive at freeze as `(:wat::core::kwargs-construct :T …)` — the first pass looked for a bare `:T` head and missed g7 entirely until that lowering was followed.

## Row 7 — g7 resolved without relaxing the fence

`:value "symbol"`. `g7::match-arrow` requires `IsArrow`, which `g7::arrow` asserts only for `NodeKind::Symbol`. The literal is the truth, not new knowledge. No user fn. No `enum::name`.

`wat_grep::g7_end_to_end_prints_expected_match` **PASS**. Pin is again `:value "symbol"`.

The other Class A sites, same constraint:

| site | resolution |
|---|---|
| g7, pg-cli, pg-driver | literal `"symbol"` (when already requires Symbol) |
| fx::match-arrow | Capture `:name "id"` via `:wat::rete::i64::to-string` (when is not kind-pinned; a single kind literal would lie) |
| hk::head-kind | kind/pkind Captures dropped; hid/pid via `i64::to-string`. A census of NodeKind cannot ride a String field until `enum::name` exists. |
| ls::child-offset | the kind Capture dropped; the other string captures remain |

## Row 8 — the `accepted` list names the call form

Bare `:Block` in a `:then` value:

`RhsUnresolvableOperand`, accepted includes `"a call form — a constructor, or a fenced :wat::rete:: expression"`.

## Row 9 — a bare keyword is still refused, and the reason is findable

Same probe: operand `:Block`, plus `"a keyword is a field reference in a RHS, not a value"`.

## Row 10 — negative controls, both defects, committed

`tests/rete/probe_then_operand_fits_the_field.rs` plus four `.wat.bad` and one green `.wat`. Not under `wat-scripts/` (STOP-6). Un-arm either check and they go green-when-they-should-be-red.

## Row 11 — batching

`:fit::two` with two i64-into-String fields reports **both** `RhsOperandTypeMismatch` (two findings; EDN wraps each twice in the blob).

## Row 12 — nothing else moved

This stone does not edit `wat/{deporder,spawn,io,fmt}.wat` or any fmt rule. Formatted output cannot have moved. EXPECTATIONS listed pre-BreakKind hashes; the gold from the NodeKind stone still stands (`deporder db37248dd73d62d2` · `spawn 599b39d2b9ac25b2` · `io 81d5718586dfab58` · `fmt 484f2438d61ce0be`).

## Row 13 — the enum gates still hold

`ast_kind_nodekind_sync` **PASS**. `every_wat_scripts_file_loads` **1 passed**.

## Rows 14–15 — floor / clippy (ORCHESTRATOR)

Not run.

---

## Mechanism

- `check_rhs_operands` takes `(field, operand)` pairs, the destination `format_type` map, and the rule's `:when` binds. Literals and `?var` binds are declared types, not rete segments.
- Nested `:then` constructors are `kwargs-construct` by the time freeze walks them. The walker type-checks that form's `:field` values against the constructed type.
- `then_types_fit`: exact identity, or actual `:E` against declared `(:E :- […])`. Alpha-into-Beta still fails.
- New `ReteCheckErrorKind::RhsOperandTypeMismatch { rule, fact_type, field, declared, actual }`.
- `RhsUnresolvableOperand.accepted` gained the call form. A bare-keyword operand also names the field-reference reason.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| ARM-A | **String vs i64**, rule+field named |
| Alpha/Beta | **Beta vs Alpha**, not `"enum"` |
| g7 `--check` (before fix) | **Capture.value String vs NodeKind**, line 26 |
| g7 after literal | `--check` 0; CLI **PASS** |
| `--test rete` | **382 passed** |
| `wat_grep::*` | **7 passed** |
| `every_wat_scripts_file_loads` | **1 passed** |
| `ast_kind_nodekind_sync` | **PASS** |
| two-bad | both fields reported |
| bare `:Block` | call form + field-reference |

## Honest deltas

- Nested constructors are `kwargs-construct`. A check that only looks at a bare `:T` head will miss g7. Follow the lowering.
- `Option/Some` into `Option<Pos>` is legal. Exact string compare of declared types false-rejected it; fitting the unapplied enum head against a parametric application of that same enum is not a segment collapse.
- `enum::name` remains unminted. The screams are seven Capture.value/NodeKind sites. Four were kind-pinned and took a literal; three were not and stopped putting a NodeKind in a String.

## The board

```
   if · the test rides its head     RULED, new rule file, 1822 sites
   cond · clauses align             RULED, new rule file, 51 sites
✅ Break.kind is an enum
✅ Node.kind is an enum
✅ :then checks declared field types  i64-into-String refused · Alpha-into-Beta refused
                                    · Capture.value/NodeKind was the scream class
   enum::name                       gated: 7 Capture.value sites, 3 of them not kind-pinned
```

---

## ORCHESTRATOR VERDICT — 2026-09-06

**ACCEPTED, with ONE orchestrator fix — and the floor is why.**

| what | my own re-run |
|---|---|
| ★★★ row 2 · i64 into a String field | against **my** original probe: ``defrule `user::i64-into-a-string-field`: … field `:label` is declared `:wat::core::String`; operand is `:wat::core::i64` `` |
| ★★★ row 3 · the segment trap | ``field `:k` is declared `:user::Beta`; operand is `:user::Alpha``` — **declared types, not `"enum"` twice** |
| ★★★ row 5 · the screams | **my own sweep: 1680 files `--check`ed, `RhsOperandTypeMismatch` = 0 remaining** |
| ★★★ row 10 · the controls are load-bearing | disarmed `then_types_fit` → `i64_into_a_string`, `alpha_into_beta` and `two_bad` go **RED**, while `same_enum_still_compiles` and the accepted-list test correctly stay green. Restored. |
| ★★★ rows 8 + 9 · the message | now reads *"…or a call form — a constructor, or a fenced `:wat::rete::` expression — a keyword is a field reference in a RHS, not a value"* |
| row 12 · nothing moved | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` — all **IDENTICAL** |
| floor | **5185 run, 5185 passed, 0 FAILED** — *after* the fix below |
| clippy | **0** |

## ⛔ THE FLOOR CAUGHT A RED THE TARGETED RUNS COULD NOT

```
FAIL wat::lint one_name_grammar::only_identifier_rs_parses_a_name

🔥🔥🔥 A SECOND NAME PARSER — 1 site(s) hand-roll one of the five name-grammar
call-shapes … OUTSIDE `crates/wat-reader/src/identifier.rs`.

Offenders:
    src/rete/validate.rs:1335   rsplit_once('/')
```

`type_env_name` — new in this stone, converting an EDN-shaped `wat.grep/Capture` into the TypeEnv's
`wat::grep::Capture` — hand-rolled the `/` split. Routed through the door
(`wat_reader::identifier::receiver` / `::method`, whose `rfind('/')` is semantically identical), with
a comment naming the lint. **Floor re-run: 5185/5185.**

★ Fifth time this arc that the floor or clippy found what a targeted run could not. `--test rete`
was 382/382 green with this defect in the tree.

## ★★ AND THE SCREAMS ANSWERED THE QUESTION STOP-5 GATED

STOP-5 forbade minting `:wat::rete::core::enum::name` because *"minting an op for one caller is
speculative until the screams say how many callers exist."* **The screams have now said: seven
`Capture.value`/`NodeKind` sites, and three of them are not kind-pinned**, so they could not take a
literal and instead **lost the capture**:

```
277-head-kind-census.wat    :name "kind" ?k  and  :name "pkind" ?pk   →  DROPPED, ids substituted
277-layout-shape-probe.wat  :name "kind" ?ck                          →  DROPPED
rules-corpus-03…            :name "kind" ?k                           →  id substituted
```

⚠ **A file named `277-head-kind-census.wat` can no longer census kinds.** That is a real capability
loss, and the strike documented it in the source rather than burying it.

★ **And it is a MISSING SIBLING, not a new category** — the rete vocabulary already carries a
to-string family with no enum member:

```
:wat::rete::core::bool::to-string     :wat::rete::f64::to-string     :wat::rete::i64::to-string
```

The three non-pinned sites are exactly what `enum::name` would serve. **The gate STOP-5 set has been
met; this is now a drawable stone rather than a speculative one.**

## One correction, and it is the second time — so it goes on the record once and then rests

The SCORE says EXPECTATIONS row 12 *"listed pre-BreakKind hashes."* It did not. Those digests are of
a **different artifact** — this arc's per-line capture, not raw formatted text — and re-run against
the same instrument just now they are **IDENTICAL to the character**, exactly as they were after the
`Break` stone and after the `Node` stone. Both instruments are sound and both reach the same verdict;
the caution is reasonable, the factual claim is not, and nothing downstream turns on it.

## What the strike got right that is worth naming

- **It found the lowering.** A first pass looked for a bare `:T` head and missed `g7` entirely,
  because a nested `:then` constructor arrives at freeze as `(:wat::core::kwargs-construct :T …)`.
  Row 6 exists precisely to catch a check that is measuring something else, and it did.
- **It refused a false positive rather than accepting one.** `Option/Some` into
  `(:wat::core::Option :- [:wat::core::Pos])` is legal; an exact-string compare rejected it. Fitting
  an unapplied enum head against a parametric application of that same enum is not a segment
  collapse, and STOP-2 is what made that the right call instead of a rewritten probe.

## The board

```
   enum::name                       ★ NOW EARNED — 3 non-pinned callers, and the to-string family
                                    already has bool/f64/i64 and no enum member
   if · the test rides its head     RULED, new rule file, 1822 sites
   cond · clauses align             RULED, new rule file, 51 sites
   E · where a TRAILING comment goes UNRULED, and a width cost
✅ Break.kind is an enum
✅ Node.kind is an enum
✅ :then checks declared field types  i64-into-String and Alpha-into-Beta both refused;
                                    the message no longer teaches the String
```
