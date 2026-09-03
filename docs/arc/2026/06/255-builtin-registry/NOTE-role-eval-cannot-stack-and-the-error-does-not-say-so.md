# NOTE — `role = eval` cannot stack on one fn, and the compiler error does not say so

> Found by the 1a-ε rider, self-caught in its own first draft, and **verified independently by the
> orchestrator**. It will bite again: 15 rows remain to register and shared arms are the norm.

## The asymmetry

`#[wat_special_form_impl]` stacks fine for some roles and not others, and nothing says which:

```
role = check     ✅ stacks — emits SOURCE TEXT only.  Proven: src/check.rs:15553 carries two
                   (`and` + `or`) on infer_boolean_shortcircuit, with a comment saying so.
role = declare   ✅ stacks — source text only. Proven: config.rs + declare/register.rs each carry
                   two (set-redef! + set-eval-redef!) after Stone 1a-ε.
role = eval      ⛔ CANNOT stack — emits a shim.
role = tail      ⛔ CANNOT stack — emits a shim.
```

## The measurement

Adding a second `role = eval` for a different FQDN to one fn:

```
error[E0428]: the name `__wat_special_form_eval_eval_config_set_redef` is defined multiple times
```

`crates/wat-macros/src/wat_special_form_impl.rs` names the generated shim **from the annotated fn's
identifier alone** — `__wat_special_form_eval_<fn_ident>` — never from the FQDN. Two FQDNs on one fn
therefore emit the same symbol twice.

★ **The rung is right** — a compile error, not a silent wrong answer, which is where you want a
mistake of this class to land. The defect is not the refusal; **it is that the refusal does not say
what happened.**

## Why the message is the problem

A reader who stacks two eval roles — mirroring the `role = check` precedent that is documented in the
tree and works — gets a mangled internal symbol name and no mention of `role`, `eval`, stacking, or
FQDNs. Nothing points at the attribute that caused it. **They must reverse-engineer the macro's
codegen to learn that this role is not stackable while its sibling is.**

⚠ And the precedent actively invites the mistake: `check.rs:15553`'s own comment reads *"same fn,
different fqdn, exactly the way `role = eval` / `role = tail` stack two annotations on
`eval_and_tail`"* — which is TRUE for two different ROLES on one fn, and false for two FQDNs at the
same role. The one existing signpost is worded in a way that mis-generalises.

## The two honest fixes, either of which closes it

1. **Name the shim from the FQDN**, not the fn identifier — then stacking works for every role and the
   asymmetry disappears.
2. **Emit a `compile_error!` that names the act**: *"`role = eval` cannot be stacked: `<fn>` already
   registers an eval shim for `<other fqdn>`; give each FQDN its own eval fn."*

★ (1) removes the class; (2) only reports it. `[[extirpare]]`'s ladder says prefer (1) — but the
shim's name is also what the registry keys on at fold time, so (1) needs measuring before it is
picked.

## Worked example — what the rider did instead, correctly

`:wat::config::set-redef!` and `:wat::config::set-eval-redef!` share one eval arm in `runtime.rs` and
one check fn in `check.rs`. The rider gave them **two identically-bodied eval delegates** and stacked
only the `check` and `declare` roles. That is the correct shape today and cost three extra lines —
but it was arrived at by hitting the error and reading the codegen, not by anything the tree told it.
