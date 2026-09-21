# AMEND — STONE 255.1: the membership list is NARROWER than the alias it replaced

**Orchestrator's verification row, `cf94c3a93` (not pushed).**

## ⛔ FLOOR RED — 22 of 5936. Clippy is clean (0, workspace).

```
     Summary [ 281.603s] 5936 tests run: 5914 passed (7 slow), 22 failed, 22 skipped
```

⭐ **Real progress is in this strike and it is not in doubt** — gap 1 CLOSED (0 declaration-name
`MalformedDecl`), gap 2 nearly closed (54 → 3), the identity door exists, `wat.type` has members, and
you disclosed the acceptance row you could not meet. **The fold is narrow.**

## CAUSE 1 — 18 of 22: `wat.type`'s membership list is smaller than the `format!` alias reached

```
UnresolvedReference :path ":wat::type::Tuple"
  "namespaced symbol ref — not a builtin, not a registered function (arc 251)"
  tests/resolve/probe_arc251_keyword_to_type_form.wat:20
```

⭐ **This is the cost of making the container real, and it is the stone working as designed.** The
old `canonicalize_type_kw` was a `format!` that forwarded **everything** to `wat.core`. A real
namespace has a **membership list** — and the list was built from a **narrower** source than the
alias reached. `[[feedback_impose_the_check_and_read_the_screams]]`: the screams are **legitimate
names**.

⚠ **Your non-vacuity row WORKED** — a non-member is refused. Seven real names simply ended up
non-members.

**Measured — of the 20 `wat.type/X` names the 8d codemod EMITS, 7 now fail:**

| still OK | ⛔ NOW FAILING |
|---|---|
| `i64` `String` `nil` `bool` `f64` `keyword` `u8` `Uuid` `Vector` `HashMap` `PersistentMap` `PersistentVector` `Value` | **`Tuple`** · **`Record`** · **`Struct`** · **`Error`** · **`Infer`** · **`char`** · `Bogus` |

⛔ **These are REGRESSIONS, not pre-existing gaps.** The floor was **5930/5930 green** at
`779b084e3`, and `char` specifically was measured working before this stone
(pre-stone scalar sweep: *"HOLDS: i64 u8 f64 bool char String"*).

**Where they land:** 9 `probe_arc251_keyword_to_type_form` · 6
`probe_arc255_the_type_position_has_its_own_authority` · 2 `probe_arc251_parametric_target` · 1
`wat_scripts_fixes_load` (`wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat`, 9 refs, all
`:wat::type::Tuple`) · 1 `check::tests::type_of_answers_every_is_type_name`.

⭐ **THE RULE FOR THE FIX — derive membership, do not hand-list it.** Every name the alias could
reach must be a member. ⛔ **A hand-written list is how this recurs**
(`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`) — if `register_builtin_leaf` is the door
that seeds membership, then **every** builtin must go through it, including the aggregates/opaques
(`Tuple`, `Record`, `Struct`, `Error`) and whatever registers `char` and `Infer`.
**Say which door you used and why it is total.**

## CAUSE 2 — 4 of 22: three WALLS tripped by the new code

| wall | offenders |
|---|---|
| `one_variant_separator` | **4 sites** spell `::` outside `crates/wat-reader/src/identifier.rs` |
| `one_param_spec` | **1 site** hand-rolls the `:-` param-spec marker outside `src/types.rs` |
| `no_loose_string_assert` | **11 sites**, incl. `src/types.rs:7292`, `:7293` |

⛔ **`one_param_spec` is the one to take seriously.** *"`:-` is wat's ONE parameterization operator"*
— a second recogniser is the exact defect this arc exists to remove, arriving by a new door. **Route
through `src/types.rs`; do not rune it.**

⚠ For `one_variant_separator` the precedent is in-file and you have used it correctly before
(`vocab.rs`, category `edn`). **Rune only a site that genuinely does not split an enum from a
variant, and cite the category.** For `no_loose_string_assert`, the rubric is the `.edn` golden —
**capture the whole value; never guess a `contains`**.

## STILL OPEN — the acceptance row you disclosed

> *"`wat.type/Vector` **annotates** … In **call** position `(wat.type/Vector 1 2)` is
> `UnresolvedReference`. `is_resolvable_call_head` does not consult TypeEnv membership. Identity on
> that predicate was tried and reverted (freeze risk). **Not smuggled back.**"*

⭐ **Disclosing it rather than half-landing it was right.** The ruling's row 2 is *"One position, not
two"*, and it is **not met**.

**Re-attempt it after CAUSE 1 is fixed** — a complete membership list may be exactly what
`is_resolvable_call_head` needed in order to consult TypeEnv safely. ⛔ **If it still risks the
freeze, STOP and report it as a finding for the builder** — do not force it, and do not quietly drop
the row. A `wat.type` that is a member in one position and absent in the other is the ruling's own
stated defect.

## The fold

⛔ **Fold into `cf94c3a93`.** Not a repair commit after — the stone must be green at its own landing.
Nothing is pushed; amending is safe.

## What I verified that is GOOD

| row | result |
|---|---|
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| no `.wat` converted | ✅ |
| gap 1 (declaration names) | ✅ **0 `MalformedDecl` on a name** — closed |
| gap 2 (`defrecord`/`defstruct`) | ✅ 54 → **3** |
| the delta re-run | ✅ done, and **honestly reported**: 104 → 101, with the caveat that *"the regression count is not the residue kind"* — the classification table is the real result |
| the freeze hazard | ✅ documented at the site (rust-scheme paths must not be clojure-round-tripped) |

## After the fold

State `cargo clippy --release --all-targets -p wat -- -D warnings` and the three wall tests. I re-run
the whole floor, workspace clippy and `census.sh --diff` uncontended, plus the 20-name `wat.type`
sweep and the 179-file delta. **Do not push. Do not start 8d-ii.**
