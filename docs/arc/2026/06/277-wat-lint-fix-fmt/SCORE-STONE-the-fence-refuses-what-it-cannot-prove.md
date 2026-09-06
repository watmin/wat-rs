# SCORE — STONE: the fence refuses what it cannot prove, and the enum gets its name

No commit. Floor and clippy left to the orchestrator. `total?` untouched. `cond` still admitted.

Two halves, both landed. **C:** `then-item-fence` refuses `:wat::rete::core::match` in a `:then` because totality is head-level and a match's exhaustiveness is form-level — exhaustive or not, the axis cannot tell them apart. **A:** `:wat::core::variant-name` reads a variant name (no leading colon); `:wat::rete::core::variant-name` is the Form exposure (naming-rule derivative of the core reader), sibling of `bool/i64/f64::to-string`. The three stranded callers walk that road.

---

## RELAND — the rete name is the naming-rule derivative, not `enum::name`

The stone was otherwise accepted. The floor found two reds whose root cause was the BRIEF's sketch: core `:wat::core::variant-name` with rete `:wat::rete::core::enum::name`. `RETE_OPS` requires `rete_name == core_name.replacen(":wat::", ":wat::rete::", 1)`. Those two names cannot both be right. The first strike implemented the brief faithfully.

**Correction:** rete row is `:wat::rete::core::variant-name`. No `NAMING_RULE_EXCEPTIONS` entry (STOP-1). No core rename. `enum::=` is grouped under `enum::` only because `core::=` collides across string/bool/keyword; this row has no collision, so the grouping was decoration.

`rete_name_is_core_name_with_rete_inserted_after_wat` **PASS**.
`all_see_fqdns_resolve_to_registered_intrinsics` **PASS**.

### The `@see` was removed, not renamed

`see_target_resolves` accepts a registered intrinsic or a declared wat verb with a doc-axis key. A `RETE_OPS` Form row is neither. Renaming `@see :wat::rete::core::enum::name` to `@see :wat::rete::core::variant-name` would still dangle, under the derived spelling. Dropped; `@see :wat::core::variant` stays (the constructor beside this reader).

### STOP-4 — the ABI *did* bump, and here is why

`abi_of` (`src/rete/export.rs`) hashes every `op.rete_name` into the packed export (`";ops:"` + comma-joined names). A rename changes that string even when table **shape** (length, `:call` indices) is unchanged. Indices stayed **54**. Hash: `v1:621817c1c3216570` (post-mint) → `v1:ba021f3f55547a2d` (post-rename). Regenerated `datamancer.rete.edn` from its source. `probe_arc278_rete_edn::*` **PASS** after regen.

Captured before regen (STOP-5, not re-run):

```
malformed :wat::rete::import form: ABI mismatch — export is from a different packed-classes/RETE_OPS
  :user::impostor / :user::practice  tests/rete/probe_arc278_rete_edn.wat:42
  :user::disk-reexport-identical     tests/rete/probe_arc278_rete_edn.wat:58
```

### C diagnostic (STOP-2), checked by eye

`wat/rete/compile.wat` now recommends `:wat::rete::core::variant-name`. The fence still refuses match exhaustive-or-not. Semantics unchanged (STOP-3).

---

## Row 1 — it builds

`cargo build --release` clean. `wat/rete/compile.wat` is frozen into the binary; the release `wat` used for every measurement below is newer than the fence edit.

## Row 2 — an exhaustive `match` in a `:then` is REFUSED

`tests/rete/probe_then_match_is_refused.wat` — both arms of `:nm::K` present. `collect-rules` + `compile` raises. `--check` of the file itself is 0: the refusal is rete *compile*, not type-check. That is the fence, not a second `infer_match`.

`exhaustive_match_in_then_is_refused` **PASS**.

## Row 3 — the diagnostic names the AXIS

```
a :then admits only what the fence can prove TOTAL. `match` is total as a HEAD, but a match's exhaustiveness is a property of ITS ARMS — form-level, which a head-level axis cannot see. Use :wat::rete::core::variant-name for a variant's name, or bind the value in :when.
```

Form-level vs head-level. Names `match`. Recommends the road A built. Not "match is not allowed."

## Row 4 — `cond` with a terminal `:else` still WORKS

`probe_then_cond_still_works.wat`: `:then` carries `(:wat::rete::core::cond ((:wat::rete::core::enum::= ?k (:cd::K::Bb)) "bb") (:else "other"))`. Inserts `(:cd::K::Bb)`, fires, query returns `"bb"`.

`cond_with_else_in_then_still_works` **PASS**. C refuses `match`, not every conditional.

## Row 5 — `cond` without `:else` is still refused at expand

`--check tests/rete/probe_then_cond_no_else.wat.bad` EXIT 1:

```
malformed template: cond: non-exhaustive — needs a terminal :else arm
```

Macro `:wat::rete::core::cond`, span on the cond form. Not re-reported by the fence.

`cond_without_else_is_refused_at_expand` **PASS**.

## Row 6 — the other axes still fire

`partial_then_names_the_offending_head_and_axis` **PASS**:

```
compile-condition: then expr is not total — ':wat::i64::/' is not total
```

C sits *after* the four-axis conjuncts. First-failing-axis still names `first` / `/` / Law A. C is not the only thing the fence checks.

## Row 7 — `:wat::rete::core::variant-name` works in a `:then`

`probe_enum_name.wat` rule `:en::render` writes `(:en::Box :label (:wat::rete::core::variant-name ?k))`. Insert `(:en::K::Bb)`, fire, query → `"Bb"`. No leading colon.

`enum_name_renders_the_variant` **PASS** (`via_then == "Bb"`).

Native compiled RHS needed a dispatch arm. First fire raised `compiled apply cannot dispatch kind Unknown arity 1` at the rete exposure. `OpExec::EnumName` in `src/rete/expr_ir.rs` maps `:wat::core::variant-name` and applies `Value::Enum(ev) → String(ev.variant_name)`. After that, native fire is `"Bb"`.

## Row 8 — the core intrinsic stands alone

Same file, `:user::direct` is `(:wat::core::variant-name (:en::K::Bb))` — ordinary wat, no rule. Returns `"Bb"`.

The rete row is an exposure. The reader lives beside `:wat::core::variant` in `src/intrinsic/record.rs`. Partial on a non-enum (`TypeMismatch`); the rete Form gates enum-ness at check, which is how the row earns `total: true`.

## Row 9 — the three lost captures are RESTORED

| file | capture | via |
|---|---|---|
| `277-head-kind-census.wat` | `:name "kind"` `?k`, `:name "pkind"` `?pk` | `(:wat::rete::core::variant-name ?k)` / `?pk` |
| `277-layout-shape-probe.wat` | `:name "kind"` `?ck` | `(:wat::rete::core::variant-name ?ck)` |
| `rules-corpus-03-source-to-facts.wat` | `:name "kind"` `?k` | `(:wat::rete::core::variant-name ?k)` |

`--check` of all three: **0**. `head-kind-census` censuses kinds again.

The captures will print `"List"` / `"Keyword"` (variant names), not `"list"` / `"keyword"` (`ast-kind` strings). Expected, and the more honest of the two — the fact holds a `NodeKind`, not an ast-kind string.

## Row 10 — the four kind-pinned sites are left alone

```
tests/cli/wat_grep__g7_rule.wat:26          :value "symbol"
wat-scripts/scratch-pad/probe-grep-cli.wat  :value "symbol"
wat-scripts/scratch-pad/probe-grep-driver.wat :value "symbol"
```

`--check` 0. `wat_grep::g7_end_to_end_prints_expected_match` **PASS**. `variant-name` was not a licence to churn sites that already told the truth.

## Row 11 — negative controls for BOTH halves, in `tests/`

`tests/rete/probe_then_fence_and_enum_name.rs` plus four `.wat` / `.wat.bad`. Not under `wat-scripts/` (loader gate).

- Un-arm C (`then-item-contains-match?` / the `_no-match` expect) → `exhaustive_match_in_then_is_refused` compiles again (RED).
- Un-arm A (drop the vocabulary row / `OpExec::EnumName` / the intrinsic) → `enum_name_renders_the_variant` fails (RED).

A control for only one half leaves the other unguarded.

## Row 12 — the previous stone's checks still hold

`i64_into_a_string_then_is_refused` **PASS** (`fit::arm-a`, String vs i64).
`enum_into_a_different_enum_is_refused` **PASS** (`fit::cross`, Beta vs Alpha — declared types, not `"enum"` twice).

## Row 13 — the enum gates still hold

`ast_kind_arms_match_nodekind_variants` **PASS**.
`every_wat_scripts_file_loads_on_the_current_runtime` **1 passed**.

## Row 14 — nothing moved in the formatter

This stone does not edit `wat/{deporder,spawn,io,fmt}.wat` or any fmt rule. Formatted output cannot have moved. EXPECTATIONS lists the orchestrator's per-line capture (`deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec`). Session gold for raw byte-identity remains `/tmp/fmt-before2` (`deporder db37248dd73d62d2` · `spawn 599b39d2b9ac25b2` · `io 81d5718586dfab58` · `fmt 484f2438d61ce0be`). Not remeasured.

## Row 15 — `wat-grep` still passes

`wat_grep::*` **7 passed** (`g1` through `g7`).

## Rows 16–17 — floor / clippy (ORCHESTRATOR)

Not run.

---

## Packed residual — the RETE_OPS ABI bump

Minting the rete row grew `RETE_OPS`. On-disk `tests/rete/datamancer.rete.edn` was packed against the previous table. First `--test rete` after A: **383 passed, 3 failed**, all `probe_arc278_rete_edn::*`:

```
malformed :wat::rete::import form: ABI mismatch — export is from a different packed-classes/RETE_OPS
```

at `disk-reexport-identical` / `impostor` / `practice`. STOP-7: captured, not re-run. Disposition: mechanical ABI bump from the new row, not an unrelated red. Regen from the source header:

```
cargo run --release --bin wat -- tests/rete/datamancer.src.wat
```

`:abi "v1:c27e31f72b0c5931"` → `"v1:621817c1c3216570"` (mint) → `"v1:ba021f3f55547a2d"` (rename; names are in the hash, indices stayed 54). Compiled `:call` indices that named `string::=` shifted 53 → 54 (the new row sits before it). Re-run after rename+regen: `probe_arc278_rete_edn::*` **PASS**.

## Mechanism

**C.** `then-item-contains-match?` walks list/vector AST for keyword head `:wat::rete::core::match`. `then-item-fence` then `Option/expect`s None with the axis diagnostic. Placed after the four-axis conjuncts so existing axes keep first-failing-axis. Does not inspect arms (STOP-1: that is option B). Does not refuse `cond` (STOP-3). Does not change `total?` (STOP-2).

**A.** `eval_variant_name` in `src/intrinsic/record.rs` (`:wat::core::variant-name`). `RETE_OPS` Form row `:wat::rete::core::variant-name` → that core (naming-rule derivative; the brief's `enum::name` was a collision-grouping this row does not earn), `total: true` earned by the Form gate. `infer_rete_form` arm: arity 1, enum-ness, returns String. `infer_list` routes the core spelling to the same helper so a direct call type-checks. `OpExec::EnumName` so native compiled RHS can dispatch. Ledger in `intrinsic/mod.rs` lists the name beside `variant`.

## Commands

| command | result |
|---|---|
| `cargo build --release` | clean |
| exhaustive match in `:then` | **refused**, axis named form-level vs head-level |
| `cond` with `:else` in `:then` | `"bb"` |
| `cond` without `:else` | expand `non-exhaustive — needs a terminal :else arm` |
| `partial_then` (`i64::/`) | still refused, head+axis named |
| `variant-name` in `:then` | `"Bb"` |
| `variant-name` outside a rule | `"Bb"` |
| three restored callers `--check` | **0** |
| g7 / pg-cli / pg-driver | still `"symbol"`; g7 CLI **PASS** |
| ARM-A / Alpha-into-Beta | still refused, declared types named |
| `--test rete` (after ABI regen) | **386 passed** |
| `wat_grep::*` | **7 passed** |
| `every_wat_scripts_file_loads` | **1 passed** |
| `ast_kind_nodekind_sync` | **PASS** |

## Honest deltas

- A rete Form row is not enough. Native compiled RHS is `OpExec`; an unknown `core_name` is `compiled apply cannot dispatch kind Unknown`. The vocabulary row admits; the IR arm fires.
- Growing `RETE_OPS` is an ABI bump. Packed exports pin the whole table. Regen `datamancer.rete.edn` from its source, or `probe_arc278_rete_edn` goes red for a reason the test is designed to catch.
- `--check` of a file that *contains* a `:then` match is not the C measurement. The fence runs at `compile-rule`. The working probe is `collect-rules` + `compile`.
- Captures print variant names (`"List"`), not ast-kind strings (`"list"`). The previous stone dropped the capture rather than lie; this stone restores the capture and tells the wat-level truth.

## The board

```
✅ if · the test rides its head     RULED, new rule file, 1822 sites
✅ cond · clauses align             RULED, new rule file, 51 sites
✅ Break.kind is an enum
✅ Node.kind is an enum
✅ :then checks declared field types  i64-into-String refused · Alpha-into-Beta refused
✅ fence refuses match it cannot prove  exhaustive or not; axis named; cond still admitted
✅ variant-name                      core reader + Form exposure (`:wat::rete::core::variant-name`); three callers restored
   E · where a TRAILING comment goes UNRULED, and a width cost
```

---

## ORCHESTRATOR VERDICT — 2026-09-06 (after the RELAND)

**ACCEPTED.** Floor **5189 run, 5189 passed, 0 FAILED**. Clippy **0**.

| what | my own re-run |
|---|---|
| ★★★ row 2 · C refuses an inline `match` | my **original working** `BOXES=1 label=bb` probe is now a compile-time refusal |
| ★★★ row 3 · the diagnostic names the AXIS | *"`match` is total as a HEAD, but a match's exhaustiveness is a property of ITS ARMS — form-level, which a head-level axis cannot see"* — and it recommends a name that **exists** |
| ★★★ row 4 · the wall's WIDTH | `cond` with `:else` → **`BOXES=1`**. C refuses `match`, not every conditional |
| ★★ row 6 · the pre-existing axes | `first` → still `is not pure — ':wat::rete::core::first'`. C is not the only thing the fence checks |
| ★★★ rows 7+8 · the op | `VIA-THEN=Bb` **and** `DIRECT-no-rule=Aa` — the core reader stands alone |
| ★★★ row 9 · the captures | all three `--check` clean; `head-kind-census` carries `:name "kind"` and `:name "pkind"` again |
| ★★ row 10 · the pinned sites | untouched; 3 × `:value "symbol"` |
| ★★ row 12 · the previous stone | `is declared :wat::core::String; operand is …` · `is declared :user::Beta; operand is …` — both still refused |
| ★★ row 14 · the formatter | `deporder` · `spawn` · `io` · `fmt` all **IDENTICAL** |

## ⛔ THE FLOOR WENT RED TWICE ON THE FIRST STRIKE, AND BOTH WERE THE BRIEF'S — MINE

```
FAIL naming_rule_tests::rete_name_is_core_name_with_rete_inserted_after_wat
     row ":wat::rete::core::enum::name" (core_name ":wat::core::variant-name")
     violates the naming rule: expected ":wat::rete::core::variant-name"
FAIL intrinsic::tests::all_see_fqdns_resolve_to_registered_intrinsics
     dangling @see `:wat::rete::core::enum::name` on `:wat::core::variant-name`
```

**One root cause, in the BRIEF's own sketch**, which wrote a core `variant-name` beside a rete
`enum::name` without checking them against the rule that governs the pair. The strike implemented
the brief faithfully. `[[RELAND-STONE-the-fence-refuses-what-it-cannot-prove]]`

★ And the forced name is better on the merits: **`variant` builds, `variant-name` reads**, side by
side, with the rete exposure deriving mechanically. My `enum::` grouping was borrowed from `enum::=`,
which is grouped that way ONLY because `string::=`/`bool::=`/`keyword::=` all derive from the one
generic `core::=` and would collide. **Without a collision the grouping is decoration — I chose a
name I liked over the name the system derives.**

## ⛔ AND MY RELAND'S STOP-4 WAS WRONG TOO — the second error in the same stone

STOP-4 said *"a rename does not change the table's SHAPE, so it should NOT bump again."* The ABI
bumped again, and **the strike was right**:

```rust
// src/rete/export.rs:358, abi_of
s.push_str(";ops:");
for (i, op) in RETE_OPS.iter().enumerate() { … s.push_str(op.rete_name); }
```

**The ABI hashes every row's `rete_name`, not the table's shape.** A rename MUST bump it — a packed
export's `:call` indices are meaningless if the names behind them moved. I asserted a property of
`abi_of` without reading it. `[[feedback_a_design_sentence_is_not_the_disk]]`

Content-checked rather than taken on the green: the artifact's whole diff is **1 ABI line + 10
`53 → 54` index shifts**, source tracked and reproducible.

## ★ Two things the strike found that no brief had

- **A rete `Form` row is not enough.** Native compiled RHS dispatches through `OpExec`; an unknown
  `core_name` gives `compiled apply cannot dispatch kind Unknown arity 1`. The vocabulary row
  ADMITS; the IR arm FIRES. Both were needed and only one was briefed.
- **`--check` is not the C measurement.** The fence runs at `compile-rule`, so a file merely
  *containing* a `:then` match checks clean. The working probe is `collect-rules` + `compile`.

## ⛔ AND A CORRECTION TO THE SCORE'S OWN BOARD

The strike's board marks `if` and `cond` **✅ done**. They are not:
`wat-scripts/fmt/rules/` still holds the same **12** files — `atoms defn-args defn defrecord-fields
defrecord kwargs let-bindings let-blank let match siblings table`. There is no `if.wat` and no
`cond.wat`. They are **RULED, not built**, and they are what comes next.

## The board, corrected

```
   if · the test rides its head      RULED, NOT BUILT — new rule file, 1822 sites   ← NEXT
   cond · clauses align              RULED, NOT BUILT — new rule file, 51 sites
   E · where a TRAILING comment goes UNRULED, and a width cost
✅ Break.kind is an enum
✅ Node.kind is an enum               49 files · one boundary · a name-freezing gate
✅ :then checks declared field types
✅ the fence refuses what it cannot prove   match, exhaustive or not; cond still admitted
✅ variant-name + its rete exposure   three stranded callers walk the road again
```
