# BRIEF — STONE: holon into parity

Correct `src/holon/`'s two-layer doctrine, then move the twelve binding functions it was wrongly
excluding. DESIGN: `docs/arc/2026/04/109-kill-std/DESIGN-STONE-holon-into-parity.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` for a fast read. **You may not
spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not
commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at 5114.

## Read in order

1. The DESIGN above — why the doctrine goes, and what replaces it.
2. **`src/holon/mod.rs:8–52`** — the doctrine you are rewriting. Read it whole before editing a word
   of it; it is a deliberate ruling (Stone HOME-8), and the stone's claim is narrow: its **premise**
   was false, not its author's care.
3. **`src/record/`** — the home that shipped one stone ago, same shape. Its `use` blocks are the
   standard.
4. `src/value/environment.rs:148` · `src/value/symbol_table.rs:32` — where `Environment` and
   `SymbolTable` actually live. **This is the whole argument.** Read STOP-2 before writing a `use`.

## The work

### 1 — rewrite the doctrine in `src/holon/mod.rs`

Strike the algebra/binding split. State the rule every impl home already observes:

> an impl home must not reference `crate::intrinsic` (its own edge — that is a cycle); it may use
> `crate::value` types and call the evaluator, as `collection`, `edn`, `numeric`, `record` and
> `reflect` do.

**Record why the old rule went** — it named `Environment`/`SymbolTable` as `runtime.rs`'s when both
live in `src/value/`, and did so on a commit (`d43f75887`) whose own `runtime.rs:758-770` re-exports
them. A future reader must not re-derive the old split from the same mistake.

⛔ **`codec.rs`'s stricter bar STAYS and stays stated.** It forbids `WatAST`/`Value`/`RuntimeError`/
`Span` in its signatures — a real claim about a wire format, argued on its own evidence, not
inherited from the facade error. Do not touch it, do not weaken it, do not fold it into the general
rule.

⚠ `sigma.rs` and `hologram.rs` currently violate the old split (6 signatures taking `sym`). After
the rewrite they simply do not. **That is a consequence, not a task** — change neither file.

### 2 — move the twelve

```
8896   4  enum PairedVectors         9241  22  run_ast_arg_for_eval_coincident
8901  75  pair_values_to_vectors     9270  23  coincident_of_two_values
9139  38  cosine_outcome_from_values 9295  66  eval_form_digest_coincident_shared
9180  23  presence_q_from_values     9363  70  eval_form_signed_coincident_shared
9206  27  coincident_q_from_values   9442   6  enum FallbackVerdict
9485  41  classify_fallback_outcome  9529  14  dot_outcome_from_values
```

Place by role, **verified against the bodies**: the outcome constructors belong with
`src/holon/outcome.rs`'s existing `CosineOutcome`/`DotOutcome`/`DegenerateSide` constructors; the
coincident family may want its own `coincident.rs`; `pair_values_to_vectors` goes where its callers
sit. **Report which callers decided each placement.**

### 3 — re-point the call sites

The compiler names them. Leave a short retirement comment at each cut, in the shape the previous
stones used.

## Blast radius

`src/holon/mod.rs` (the doctrine + any new `mod`) · `src/holon/outcome.rs` and possibly one new file ·
`src/runtime.rs` (12 items out) · whatever the compiler names. No `.wat` corpus change. No
registrations. **No holon verb changes behaviour.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `no_field_names` (8981) and `builtin_enum_variant_names` (9087) MUST NOT MOVE.** They
sit *between* the moving items and are consumed by **10 and 7 other homes** — `io`, `services`,
`intrinsic/mod`, `stream`, `rete/purity`, `host`, `edn`, `declare` among them. Generic
`Value`/`EnumValue` constructors, not holon's. `grep -c "fn no_field_names\|fn builtin_enum_variant_names"
src/runtime.rs` must still be **2**. Ninth and tenth intruders found inside a proposed range in this
campaign.

**⛔ STOP-2 — IMPORT FROM THE CANONICAL HOME, NEVER THROUGH `runtime`'s FACADE.** This stone exists
*because* that facade authored a false rule. `use crate::runtime::SymbolTable` compiles and is a lie.
Import from `crate::value::`, `crate::ast`, `crate::span`. Report each touched file's `use` block —
and note that `eval_inner` **is** a genuine `crate::runtime` resident and is now permitted here.

**⛔ STOP-3 — `codec.rs`'s stricter bar is not yours.** If the rewrite seems to want to simplify it
away, STOP. Uniformity is not the goal; consistency where the reason is the same is.

**STOP-4 — `src/holon/` must not reference `crate::intrinsic`.** The new rule's whole content.
⚠ Watch your own prose: the acceptance check is a textual grep, and two riders on prior stones wrote
the forbidden substring into a *doc comment* explaining the rule. Describe the boundary without
embedding it.

**STOP-5 — verbatim.** No signature tidying. Visibility changes forced by the boundary are expected,
on both sides; report each.

**STOP-6 — run the orphaned-doc-block scan** over the whole of `runtime.rs`. ⚠ A prior rider found a
doc block that mixed `///` with a plain `//` rationale above an `#[allow]` and whose extraction
script silently truncated it — scan for contiguous `//` too, not only `///`.

## Report

Per-file diff summary; **the rewritten doctrine verbatim** (it is the stone's centre — the
orchestrator must read exactly what now stands); where each of the twelve landed and which callers
decided it; each touched file's `use` block; confirmation the two generics and `codec.rs`'s bar are
untouched; before/after `wc -l src/runtime.rs`; the doc-block scan result; and what surprised you.
