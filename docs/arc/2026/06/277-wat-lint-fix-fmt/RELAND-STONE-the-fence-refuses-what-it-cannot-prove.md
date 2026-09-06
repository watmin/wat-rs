# RELAND — STONE: the fence refuses what it cannot prove

**The stone is otherwise ACCEPTED.** Every row I could check holds — C refuses my own working
inline-match probe with a diagnostic that names the axis, `cond` with `:else` still fires, `first` is
still refused by the pre-existing axes, `enum::name` renders `"Bb"` both through a `:then` and
directly, all three captures are restored and `head-kind-census` censuses kinds again, and the four
pinned sites were left alone.

## ⛔ TWO REDS ON THE FLOOR, AND THE DEFECT IS THE BRIEF'S — MINE

```
FAIL wat rete::vocabulary::naming_rule_tests::rete_name_is_core_name_with_rete_inserted_after_wat

  row ":wat::rete::core::enum::name" (core_name ":wat::core::variant-name")
  violates the naming rule: expected ":wat::rete::core::variant-name"
    left:  ":wat::rete::core::enum::name"
    right: ":wat::rete::core::variant-name"

FAIL wat intrinsic::tests::all_see_fqdns_resolve_to_registered_intrinsics

  dangling @see `:wat::rete::core::enum::name` on `:wat::core::variant-name`
```

**One root cause, and it is in the BRIEF's sketch**, which wrote both names without checking them
against the rule that governs them:

```
core:  (:wat::core::variant-name <enum value>)
rete:  :wat::rete::core::enum::name
```

`RETE_OPS`' naming rule is `rete_name == core_name.replacen(":wat::", ":wat::rete::", 1)`. Those two
names cannot both be right. The strike implemented the brief faithfully.

## AND AN EXCEPTION IS NOT AVAILABLE — checked, not assumed

`NAMING_RULE_EXCEPTIONS` exists, and the same test guards it:

> *"is listed as a naming-rule exception but its rete_name now EQUALS the literal rule's output — it
> no longer needs the exception"*

The six existing exceptions earn their standing because the naive rule genuinely **collides**
(`string::=` / `bool::=` / `keyword::=` all derive from the one generic `core::=`, and three
same-named rows would leave only the last-registered `TypeScheme` reachable). Here there is no
collision: `:wat::rete::core::variant-name` is unique. An exception would be refused by the
`assert_ne!`.

## THE CORRECTION — one name, fully determined

**The rete row becomes `:wat::rete::core::variant-name`.** No exception, no core rename.

★ And it is better on the merits, not merely legal: `variant-name` sits beside `:wat::core::variant`
— **`variant` builds, `variant-name` reads** — and the rete exposure derives mechanically. My
`enum::` grouping was borrowed from `enum::=`, which is grouped that way **only** because of the
collision above. Without a collision the grouping is decoration, and I wrote a name I liked rather
than the name the system derives.

## SITES — 15, across 9 files

```
src/rete/vocabulary.rs      the row's rete_name
src/intrinsic/record.rs     the @see on :wat::core::variant-name
src/check.rs                the infer_rete_form arm
wat/rete/compile.wat        ⚠ the C DIAGNOSTIC recommends the op BY NAME — it must not
                              recommend a name that does not exist
tests/rete/probe_enum_name.wat
tests/rete/probe_then_fence_and_enum_name.rs
wat-scripts/scratch-pad/277-head-kind-census.wat
wat-scripts/scratch-pad/277-layout-shape-probe.wat
wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat
```

## STOP TRIGGERS

- **STOP-1 — do NOT add a `NAMING_RULE_EXCEPTIONS` entry.** The test refuses an exception that is not
  genuinely needed, and this one is not. If the rename cannot be made to satisfy the literal rule,
  STOP and report.
- **STOP-2 — the C diagnostic in `wat/rete/compile.wat` names the op.** A rename that leaves the
  fence recommending `enum::name` sends every future reader to a name that does not exist. It is
  prose, so no test catches it — check it by eye.
- **STOP-3 — this is a RENAME. No semantics change.** `variant-name` still returns the bare variant
  name, the fence still refuses `match` exhaustive-or-not, `cond` stays admitted, `total?` stays
  untouched.
- **STOP-4 — the RETE_OPS ABI is already bumped** and `tests/rete/datamancer.rete.edn` regenerated
  (`v1:c27e31f7…` → `v1:621817c1…`, one `53 → 54` index shift, 11 lines, source tracked). A rename
  does not change the table's SHAPE, so it should NOT bump again. **If it does, say why.**
- **STOP-5 — if any other test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## THE FLOOR IS MINE

Both reds were mine to find and both were found by the floor, not by a targeted run: `--test rete`
was green on this tree.
