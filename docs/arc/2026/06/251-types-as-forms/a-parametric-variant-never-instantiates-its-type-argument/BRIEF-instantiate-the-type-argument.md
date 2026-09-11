# BRIEF — `{:keys}` destructuring never instantiates a type argument

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C /home/john/work/holon/wat-rs`
for git. Never use worktrees. Do not touch `~/work/holon/` (the frozen root).

Read `NOTE-the-measurement.md` (this brief's sibling) first — it carries how the defect was found
and what it is NOT.

## ⚠ Tree state

HEAD is a long, UNPUSHED chain of local WIP commits landing the dot-notation flip
(`Enum.Variant`, formerly `Enum::Variant`). `cargo build --release` is clean and the corpus loads.

```
floor   5334 tests run   5326 passed   8 failed   22 skipped
```

**A red start is expected.** Those 8 are tracked elsewhere and are NOT yours. Your job is two
defects that are currently INVISIBLE to the floor — no test covers them.

## The defect, measured

```
parametric RECORD,  {:keys} destructure   ⛔  "body produces :X; signature declares :wat::core::i64"
parametric STRUCT,  {:keys} destructure   ⛔  same
parametric VARIANT, {:keys} destructure   ⛔  same
parametric VARIANT, field ACCESSOR        ⛔  same
────
parametric RECORD,  field ACCESSOR         ✓  works
parametric ENUM via MATCH                  ✓  works
NON-parametric anything                    ✓  works
```

A signature may name a parametric aggregate with an argument — `(:u::Cell :- [:wat::core::i64])` —
and the argument is **accepted and then ignored**. Reads return the *uninstantiated* parameter
(`:X`), not `i64`.

Four-line reproduction:

```wat
(:wat::core::defrecord :u::Cell :- [X] [x <- :X])
(:wat::core::defn :u::f [c <- (:u::Cell :- [:wat::core::i64])] -> :wat::core::i64
  (:wat::core::let [{:keys [x]} c] x))
```

## ① THE BROAD SITE — `src/check.rs` ~:12531, the keys-destructure path

```rust
let type_name = match &rhs_ty {
    TypeExpr::Path(n) => n.clone(),
    TypeExpr::Parametric { head, .. } => crate::types::parametric_head_fqdn(head),  // ARGS DROPPED
    other => { /* "an aggregate type" refusal */ }
};
let (agg_name, agg_fields): (String, Vec<(String, TypeExpr)>) =
    match env.types().get(&type_name) {
        Some(TypeDef::Aggregate(a)) => (a.name.clone(), a.fields.clone()),   // DECLARED types, RAW
        Some(TypeDef::Enum(e))      => /* the singleton's one variant's fields, RAW */
    };
```

The `..` discards the type arguments; the `TypeDef` is then looked up by head and its **declared**
field types are used verbatim. `type_params -> args` is never substituted. Keep the args and
substitute them across the fields before they are bound.

This one arm serves **record, struct AND variant** — all three rows above fail through it.

## ② THE NARROW SITE — the VARIANT field accessor's return type

A record's `:T/field` accessor instantiates correctly; a variant singleton's does not. That is a
separate omission on the enum path, not the same code.

## ★ PRIOR ART — this exact class was fixed once, for constructors

`src/declare/register.rs:834`, `parametric_decl_type`, with a doc that states the identical failure:

> *"Without this, `register_struct_methods` / `register_enum_methods` synthesized constructors with
> bare-path return types — fine for monomorphic decls but broken for parametric ones, since the type
> checker saw the body produce `:Foo` and rejected it against a `(:Foo :- [i64])` signature.
> Surfaced by arc 070's `(WalkStep :- [A])`."*

Same sentence, same cause, three positions — and the remedy reached only one. **Read that function
and mirror its intent.** You are not inventing a fix; you are finishing an existing one.

⚠ `parametric_decl_type` builds a type FROM params. You need the inverse at the use site:
substitute the SUPPLIED args INTO the declared field types. `rename` / `apply_subst` in
`src/check.rs` and `src/types.rs` are the existing substitution helpers — use one of them rather
than hand-rolling a walk.

## Acceptance — the discriminating grid

Every row must hold. The rows that already pass are the control: a fix that breaks one of them is
worse than the defect.

```
parametric RECORD  {:keys}    -> i64      (currently :X)
parametric STRUCT  {:keys}    -> i64      (currently :X)
parametric VARIANT {:keys}    -> i64      (currently :T)
parametric VARIANT accessor   -> i64      (currently :T)
parametric RECORD  accessor   -> i64      (already passes — MUST STILL PASS)
parametric ENUM via match     -> i64      (already passes — MUST STILL PASS)
NON-parametric, every shape               (already passes — MUST STILL PASS)
Demo.Has -> Demo slot ACCEPTED · Demo -> Demo.Has slot REFUSED   (subtyping unchanged)
```

Add probes under `tests/types/` covering all of them; no test covers this today, which is why it
survived.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** A fix requires changing SUBTYPING. It does not: `Demo.Has <: Demo` is implemented
correctly and verified three-for-three. If your change touches `assignable`'s subtype logic, stop.

**STOP-2.** A fix requires making same-head parametric arguments COVARIANT. That is arc 278 Stone 2
(a channel's send/recv types are exact) and is out of scope — a separate, open question. If you find
yourself needing it, report rather than proceed.

**STOP-3.** The floor's failure count rises above 8, or any of the "already passes" control rows
breaks. Report the verbatim failing block.

**STOP-4.** You find a THIRD site with the same shape. Report it with a four-line repro; do not fix
it silently — the census is the orchestrator's.

## What to run

`cargo build --release`, your new probes, and scoped
`cargo nextest run --release -E 'binary_id(wat::types)'`. **Do not run `scripts/floor.sh` and do not
run clippy** — the orchestrator measures centrally, once, on a quiescent tree. Run every command in
the FOREGROUND and block on it. Do not commit. Do not push.

## Report

Per site: what was wrong, what you changed, and the grid above with real exit codes. Every STOP
finding. Anything you were less than certain about.
