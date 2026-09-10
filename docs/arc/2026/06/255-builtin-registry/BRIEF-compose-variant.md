# BRIEF — ③b-ii ⑤: `:wat::runtime::compose-variant`

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C /home/john/work/holon/wat-rs`.

Read `DESIGN-the-composition-door-gets-a-wat-surface.md` first.

## ⚠ THE TREE IS MID-LANDING AND DOES NOT LOAD

HEAD is `2b4d709cf`; `085f3ed3e` beneath it is an unpushed WIP save point carrying the dot flip:
the corpus is rewritten to `Enum.Variant`, both door bodies read/write `.`, and three of four
newly-found composition classes are fixed. **`cargo build --release` succeeds, but running any wat
program currently fails with ~818 type-check errors** — 456 of them `arm head ... is not
namespaced`. Those are the fourth class, and closing it is this stone. A red at the start is the
expected state, not a surprise; you are finishing a landing, not starting on green.

## The verb

```
(:wat::runtime::compose-variant <enum-keyword> <variant-keyword>) -> :wat::core::keyword
```

The exact mirror of `:wat::runtime::variant-parent-of`, which is your template in both files:

```
src/reflect/verbs.rs:1585-1640   the doc contract + #[wat_intrinsic] + eval fn
src/check.rs:2866-2915           the checker arm (arity, the literal-or-computed door, the ret type)
```

Read both in full before writing. The eval body is
`wat_reader::identifier::compose_variant(enum_path, variant_leaf)` wrapped as a keyword `Value`,
and nothing else — the separator lives in ONE file and this verb is how wat reaches it.

### Three things the template teaches, each of which cost this arc something

**① The literal-or-computed door.** `variant-parent-of` carries a ⛔ comment saying it deliberately
is NOT `is-type?`'s literal-only gate. That is scar tissue: this arc shipped `variant-parent-of`
literal-only, it passed every gate, and it was **unusable by its only caller** — found only by
writing that caller. `service.wat` passes a COMPUTED enum keyword. Both arguments must accept a
literal keyword OR an inferred `:wat::core::keyword`.

**② The doc contract is enforced.** `#[wat_intrinsic]` parses the `///` block and `compile_error!`s
on any `DocError`. Axes: `@Category Reflection`, `@Purity Pure`, `@Determinism Deterministic`,
`@Totality Total`, `@ExpandTime Legal`. `purity_mandated_examples` then makes a **RUNNABLE**
`@example` mandatory — `Pure ∧ Deterministic` earns no `@example-norun`.

**③ Give it at least two examples: one LITERAL and one COMPUTED**
(`(:wat::keyword::from-string …)`). The computed row is the one `service.wat` depends on, and an
example set that only exercises literals is how ① shipped broken.

⚠ **It composes; it does not validate.** Total — no type-env lookup, no `Option`. A name composed
for an enum that does not exist simply fails where it is used.

## Then: `wat/service.wat`

Thirteen `string::interpolate` templates spell the variant separator inside a string literal:

```wat
admin-init-kw (:wat::keyword::from-string
                (:wat::string::interpolate "{b}::Admin::Init" :b fqdn-base))
```

becomes

```wat
admin-init-kw (:wat::runtime::compose-variant
                (:wat::keyword::from-string (:wat::string::interpolate "{b}::Admin" :b fqdn-base))
                :Init)
```

`"{b}::Admin"` stays `::` — that is the enum's own namespace path, not a variant boundary. One site,
`status-started-str`, wants a String: wrap the composed keyword in `:wat::keyword::to-string`.

**When you are done, `grep -nE '\{b\}::[A-Z][A-Za-z0-9]*::' wat/service.wat` must return nothing** —
the file no longer knows what the variant separator is.

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** After both halves land, the corpus still does not load, OR loads with a NEW error shape
not present in the ~818 you started from. Report the shape and a verbatim instance — a new shape is
a fifth class, and the census is the orchestrator's to close.

**STOP-2.** A `service.wat` site cannot be expressed through the verb — its enum path is not
separable from its variant leaf. Report it: that would mean the verb's shape is wrong, not that the
call site should keep interpolating.

**STOP-3.** You need to weaken `rename-keyword-exact`, or to hand-edit a `.wat` file other than
`wat/service.wat`. Both are refused: the string-literal protection exists because arc 296 N RELAND 8
corrupted `Option`'s unit variant, and the corpus migration is already done.

**STOP-4.** The doc gates refuse your axes. Do NOT weaken the axes to satisfy them — report what
fired. Three gates corrected this arc's axes once already and they were right each time.

## Blast radius

`src/reflect/verbs.rs`, `src/check.rs` (the one new arm), whatever registry/membership row the
template needs, and `wat/service.wat`. Nothing else.

## What to run

`cargo build --release`, then `./target/release/wat --check` on a small file to prove the corpus
loads, plus `cargo nextest run --release -E 'binary_id(wat::reflection)'`. **Do not run
`scripts/floor.sh` and do not run clippy** — the orchestrator measures centrally, once. Foreground
everything. Do not commit. Do not contact any peer.

## Report

The verb's doc block verbatim; the before/after of two `service.wat` sites (one keyword, the
`status-started-str` one); the `grep` above returning empty; and the error count after your change
versus the 818 you started from.
