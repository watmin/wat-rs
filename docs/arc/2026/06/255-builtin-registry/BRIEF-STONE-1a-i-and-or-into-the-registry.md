# BRIEF — STONE 1a-i: `and` and `or` into the registry

Phase 1a's first stone. CAMPAIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority.md`.
RULING: `docs/arc/2026/06/255-builtin-registry/RULING-the-registry-is-the-sole-authority.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5119, HEAD `b75aff916`.

## Why these two, and why first

The `solvere` cast found that **5 of 9 `OpClass::Form` rows cannot fold into the registry because
their targets are not in it** — and `:wat::core::and` / `:wat::core::or` are registered *only* in
`src/special_forms.rs`, a third registry that predates the intrinsic one. Registering them here is
the prerequisite the rete fold is blocked on.

⚠ **`src/special_forms.rs` has 30 rows, not 19** — an earlier count of mine matched only one of its
two spellings. 6 are already registered; 24 are not. **This stone takes 2**, the blocking pair. The
other 22 follow once this proves the path.

## Read in order

1. **`src/intrinsic/special/control_flow.rs`** — the `If` doc-only unit struct. That is the template.
2. **`src/intrinsic/special/fn_form.rs`** — the shape when a delegate is needed.
3. `src/special_forms.rs` — the third registry. Read its header; it calls itself a registry and
   waits on a `doc_string` an arc-141 stone never supplied.
4. `crates/wat-macros/src/wat_special_form_impl.rs` — the attribute, and what it does per role.

## The work

### 1 — two `#[wat_special_form]` rows

`src/intrinsic/special/` gains `and` and `or` as doc-only unit structs, in `If`'s shape, with real
grounded directives. They are lazy/short-circuiting: **`Preserving` on the operand-dependent axes if
you can ground it in one sentence each; `Unreviewed` if you cannot.**

⚠ **Check `@ExpandTime` against `macros/eval.rs`'s `is_expand_time_legal` residue FIRST.** Both are on
that list today. Declaring `Unreviewed` for a name on the residue **silently makes it illegal inside
macro bodies** — the trap that nearly shipped for `fn`.

### 2 — three role annotations, measured

```
role = eval   eval_and_tail / eval_or_tail   (src/runtime.rs)   returns Result<Value, EvalBreak>
role = tail   eval_and_tail / eval_or_tail   ← THE SAME FN; its eval arm calls the _tail fn too
role = check  infer_boolean_shortcircuit     (src/check.rs:2373) ← ONE fn shared by BOTH forms
```

★ `eval_and_tail`'s signature `(args, _list_span, env, sym) -> Result<Value, EvalBreak>` is
`TailHandler` **exactly**, and the eval-role macro already wraps a bare `Value` return.

### 3 — delete their rows from `src/special_forms.rs`

Once registered, their entry in the third registry is a duplicate authority. Remove `and` and `or`
only; leave the other 28 and the file's machinery intact.

⚠ Check `src/reflect/lookup.rs:197` — it consults `lookup_special_form`. Confirm what it does when a
name is absent there but present in the intrinsic registry, and **report what you find rather than
assuming the fallback is correct.** If reflection would regress, STOP and report.

## Blast radius

`src/intrinsic/special/` (two new modules + `mod.rs`) · `src/runtime.rs` and `src/check.rs`
(annotations only, no bodies) · `src/special_forms.rs` (two rows out) · `src/intrinsic/mod.rs`
(`REGISTRY_MEMBERSHIP_GAP_B` shrinks by 2) · whatever the compiler names. **No verb changes
behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — ONE FUNCTION, TWO ROLES.** `eval_and_tail` must carry both `role = eval` and
`role = tail`. If the macro rejects two `#[wat_special_form_impl]` attributes on one fn, **STOP and
report the exact error** — do not split the function, do not write a delegate to dodge it. Whether
the attribute stacks is a fact about the macro this stone needs to establish.

**⛔ STOP-2 — ONE FUNCTION, TWO FORMS.** `infer_boolean_shortcircuit` is the check impl for BOTH
`and` and `or` (`check.rs:2373`, a `|`-joined arm). It would carry two annotations naming different
FQDNs. Same rule: if the macro refuses, report it; do not duplicate the function.

**⛔ STOP-3 — `@ExpandTime` IS CHECKED, NEVER GUESSED.** Both names are on
`is_expand_time_legal`'s residue. Registering replaces that entry with the declared value.
`Unreviewed` would make `(and …)` illegal in macro bodies — a behaviour regression wearing
humility's clothes.

**⛔ STOP-4 — DO NOT TOUCH `RETE_OPS`.** The rete `Form` twins (`:wat::rete::core::and`/`or`) stay
exactly as they are. This stone *unblocks* their fold; it does not perform it.

**⛔ STOP-5 — DO NOT TOUCH `is_reserved_prefix` OR THE BLANKET-ACCEPT.** Still the last stone of that
thread. `grep -c "if is_reserved_prefix(head)" src/resolve/walk.rs` must remain **1**.

**⛔ STOP-6 — THE OTHER 28 ROWS STAY.** `def`, `defmacro`, `quote`, `quasiquote`, `use!` and the rest
are later stones. A stone that starts registering adjacent forms because they are in the same file is
a stone that grew.

**STOP-7 — the dead-arm gate will speak.** Once these carry handlers,
`registry_first_door_owns_every_handler_row_no_literal_arm_survives` may demand their
`dispatch_keyword_head_value` arms go, and the tail door may demand their `eval_tail` arms go. **Do
what the gates require and report which fired** — that is the ratchet working, not a break.

## Report

Per-file diff summary; both directive blocks verbatim **with the grounding sentence per axis**;
whether both names are on `is_expand_time_legal`'s residue and what you declared; **the STOP-1 and
STOP-2 outcomes — did the macro accept stacked attributes and two-FQDN annotations?**; what
`src/reflect/lookup.rs:197` does for a name absent from `special_forms.rs`; which gates fired and
what you did. Then: **what surprised you** — an axis you could not ground, a role whose impl was not
where the brief said, or a reflection path that regressed.
