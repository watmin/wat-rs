# BRIEF — the variant DECOMPOSER pairs the composer

Step ③a of `DESIGN-the-codemod-asks-and-the-flip-is-a-PAIR.md`. **Read that design first, including
its ⛔ opening correction** — it is the reason this stone exists. Small, one concern, and
**behaviour-preserving by construction**.

## Why

`compose_variant` (landed, `crates/wat-reader/src/identifier.rs`) writes a variant's name. Nothing
pairs it. Decomposition goes through the grammar's GENERAL accessors:

```rust
identifier::leaf(name)  →  name.rsplit("::").next()             identifier.rs:284
identifier::path(name)  →  name[..name.rfind("::")]             identifier.rs:290
```

and `TypeEnv::variant_parent_enum` (`src/types.rs:948`) splits with exactly those:

```rust
let fq     = parametric_head_fqdn(name);
let parent = wat_reader::identifier::path(&fq);
let leaf   = wat_reader::identifier::leaf(&fq);
```

⛔ **So the separator is written in one place and read in another, and the reader is a
general-purpose accessor that does not know it is about variants.** Move the separator and the
registry stops recognising the names it just composed — measured, that is exactly what happens.

⚠ `path` and `leaf` **cannot** simply flip: they split EVERY namespaced name in the substrate. The
pair must be variant-specific.

## The work

Mint `decompose_variant` in `crates/wat-reader/src/identifier.rs`, **immediately beside
`compose_variant`**, as its explicit inverse. Point `TypeEnv::variant_parent_enum` at it.

```rust
/// The INVERSE of `compose_variant`. …
pub fn decompose_variant(name: &str) -> Option<(&str, &str)>   // (enum_path, variant_name)
```

`None` when the name has no separator — mirroring `path`'s `""` return, which
`variant_parent_enum` already treats as "not a variant" via its `parent.is_empty()` guard.

The doc must say, in this order:

1. that it is `compose_variant`'s inverse, and that **the two must always agree** — the separator is
   one decision, and it is spelled in exactly these two bodies;
2. that it exists because `identifier::path`/`leaf` are GENERAL splitters used for every namespaced
   name, so they cannot carry a variant-specific separator;
3. that the one-name-grammar lint bans a hand-rolled `rfind` outside this file — which is why this
   lives here rather than in `types.rs`.

## ⛔ THE SEPARATOR DOES NOT CHANGE

`decompose_variant` splits on `"::"`, exactly as `path`/`leaf` do today. This stone is
**behaviour-preserving**: `variant_parent_enum` must answer identically for every input before and
after. The flip is step ③b and it lands with the codemod.

## Read in order

1. `crates/wat-reader/src/identifier.rs` — `compose_variant` (your sibling), and `leaf`/`path` at
   :284/:290 (the shape to invert).
2. `src/types.rs:948` — `variant_parent_enum`, the one caller. Read its doc block first; it explains
   `parametric_head_fqdn`, the `parent.is_empty()` guard, and why the returned string is the enum's
   OWN stored `.name` rather than a re-assembled guess. **Do not disturb any of that.**
3. `src/reflect/verbs.rs` — `eval_variant_parent_of`, which reaches `variant_parent_enum` from wat.
   It is how you can exercise the change end to end.

## Blast radius

`crates/wat-reader/src/identifier.rs` (the new fn) · `src/types.rs` (`variant_parent_enum`'s two
split lines). **Nothing else. No separator change. No new verb.**

## Acceptance

```
cargo build --release
cargo nextest run --release -E 'test(probe_arc296)'                     unchanged
cargo nextest run --release -E 'test(enum) or test(variant)'            unchanged
then the five-row discrimination, unchanged, via a /tmp .wat:
    :usr::p::E::Alpha              -> its parent
    :usr::p::E::LooksLikeOne       -> NONE      (a defrecord of the identical shape)
    :wat::core::Option::Some       -> :wat.core/Option
    :wat::cache::Cache::GetRequest -> NONE
    :usr::NotEvenAThing            -> NONE
```

★ **Nothing should change observably.** A moved expectation means `decompose_variant` and
`path`/`leaf` disagree on some input — report it verbatim; that IS the finding.

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If you change the separator anywhere — STOP. Step ③b, not this stone.

**STOP-2.** If you change `identifier::path` or `identifier::leaf` — STOP. They split every
namespaced name in the substrate; that is precisely why the variant pair must be its own.

**STOP-3.** If any of the five discrimination rows moves — STOP with the verbatim result. Row 2 and
row 4 are lookalike `defrecord`s and must stay `NONE`.

**STOP-4.** If `variant_parent_enum` needs more than its two split lines changed — STOP and say
what. Its `parametric_head_fqdn` normalization, its `is_empty` guard, and its "return the enum's own
stored name" contract are all load-bearing and stay.

**STOP-5.** If a SECOND caller of `path`/`leaf` turns out to be doing variant decomposition — STOP
and name it. That would mean the pair has a third member and my census found two.

## Tier

You edit and report. Run the acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh` or `cargo clippy`** — the orchestrator runs those centrally, once, on a quiescent
tree. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.
