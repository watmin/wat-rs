# BRIEF — `:wat::runtime::variant-parent-of`

**Read `DESIGN-the-substrate-can-be-ASKED-what-a-variant-is.md` first.** Step ① of three. One verb.
It is what makes the corpus codemod exact instead of heuristic.

## The work

```
(:wat::runtime::variant-parent-of :Ns::Enum::Variant)
    -> (:wat::core::Option :- [:wat::core::keyword])
```

`Some` carrying the parent enum's canonical name iff the argument names a registered enum VARIANT;
`None` otherwise. The predicate already exists and is already load-bearing:

```rust
TypeEnv::variant_parent_enum(&self, name: &str) -> Option<&str>     src/types.rs:929
```

⛔ **Do not reimplement it, and do not also mint `is-variant?`.** `None` ⟺ not a variant; a second
verb would be a second door onto one question.

## Read in order

1. `src/reflect/verbs.rs:1545` — `eval_is_type`, **the template**: same file, same
   `pub(crate)`-predicate-plus-reflect-verb shape, same `(type_kw_ast, sym, span)` signature, same
   "arg must be a literal keyword" door. Mirror it.
2. `src/types.rs:929` — `variant_parent_enum` and its doc. It normalizes through
   `parametric_head_fqdn` before splitting; read that note before deciding what you pass it.
3. `src/reflect/verbs.rs:1256` — `unit_variant` / `tagged_variant`, the local helpers that BUILD an
   `Option` value in Rust. Use them; do not hand-roll an `EnumValue`.
4. `src/reflect/verbs.rs:495` and `src/reflect/lookup.rs:312` — `body-of` and `lookup-define`, the
   two Option-returning reflect verbs. **Their `@example` idiom is the one you want** (below).
5. `src/check.rs:2837` — `is-type?`'s literal check arm, which enforces the literal-keyword rule.

## The axes — is-type?'s, and they transfer

```
@Purity Pure · @Determinism Deterministic · @Totality Partial · @ExpandTime Legal
@Category Reflection
```

`Partial` for the same reason as the template: a non-keyword argument raises. Do not copy an
`Unreviewed` from anywhere.

## ★★ THE EXAMPLES — the idiom is already in the tree; do not invent a rendering

`Pure ∧ Deterministic` means `purity_mandated_examples` REQUIRES a RUNNABLE `@example`. Do **not**
try to predict how an `Option`-wrapped keyword renders. Both sibling Option-returning verbs
sidestep that by MATCHING, and the expected side is then a plain value:

```
/// @example (:wat::core::match (:wat::runtime::lookup-define :wat::core::if)
///            [:wat::core::Option::Some {:value _} true]
///            [:wat::core::Option::None {} false]) #=> true
```

**Two examples are required, and the second is the whole reason this verb exists:**

```
a VARIANT          :wat::core::Option::Some          → the Some arm, yielding the parent
a LOOKALIKE        :wat::cache::Cache::GetRequest    → the None arm
```

`:wat::cache::Cache::GetRequest` is a `defrecord` inside a `:messages` block (`wat/cache.wat:171`),
identical in SHAPE to a variant. If the negative example does not come out `None`, the verb does not
do its job and that is STOP-2.

⚠ Both examples will themselves be written in the `::` spelling. That is correct — the flip is step
③ and it will migrate them like everything else.

## ⚠ THE GATE THAT WILL FIRE IF YOU SKIP IT

`is-type?` carries BOTH a literal check arm AND a `CheckEnv` scheme — proven by its ABSENCE from
`FROZEN_CHECKER_DEBT_LEDGER` (`src/intrinsic/mod.rs`), which names every registered row
`check_env.get` cannot answer for. **Mirror both.** If `checker_skip_debt_is_named_and_frozen` fires
at central weigh, the scheme is missing — report it rather than adding the name to the ledger.

## Blast radius

`src/reflect/verbs.rs` (the verb) · `src/check.rs` (its scheme, and a literal arm if the template
needs one). **No change to `src/types.rs` — the predicate is already there and already correct.**

## Acceptance

```
cargo build --release
cargo nextest run --release -E 'test(probe_arc296_is_type)'                     unchanged
cargo nextest run --release -E 'test(purity_mandated_examples)'                 PASS
cargo nextest run --release -E 'test(checker_skip_debt)'                        PASS
then, in a /tmp .wat that defines an enum AND a defrecord of the lookalike shape:
    (variant-parent-of <the variant>)   → Some, carrying the parent enum
    (variant-parent-of <the defrecord>) → None
    (variant-parent-of :usr::NotEvenAThing) → None
```

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If you find yourself reimplementing the variant lookup instead of calling
`variant_parent_enum` — STOP. The authority exists; a second copy is the defect this arc exists to end.

**STOP-2.** If the `Cache::GetRequest` case does NOT answer `None` — STOP with the verbatim result.
That case is why the verb is being minted, and a wrong answer there means the codemod would rename
a `defrecord`.

**STOP-3.** If you cannot write a RUNNABLE `@example` — STOP. `-norun` is not available here: the
axes are `Pure ∧ Deterministic` and the gate mandates a runnable one. If it truly cannot be written,
one of the axes is wrong and THAT is the finding.

**STOP-4.** If you also mint `is-variant?`, or expose `is_variant_type` — STOP. Subsumed by `None`.

**STOP-5.** If `variant_parent_enum` needs its argument in a shape the caller cannot supply (leading
colon, parametric head) — STOP and report the exact mismatch. Do not paper it over with a strip or a
re-add at the call site; that is how the fifteen hand-rolled composers happened.

## Tier

You edit and report. Run the acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh` or `cargo clippy`** — the orchestrator runs those centrally, once, on a quiescent
tree. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
