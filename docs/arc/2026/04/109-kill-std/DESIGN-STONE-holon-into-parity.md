# DESIGN — STONE: holon into parity — the doctrine, not the domain, was the outlier

> **Builder, 2026-09-01:** *"the holon stuff is the oldest in wat.... if its out of parity.... we may
> need to force into parity...."* → then, on the measurement: ***"the entire registry endeavor is
> forcing consistency across the code base... holon is not special here."***
>
> ⊘ **This RE-OPENS `[[NOTE-REFUTED-the-holon-outcome-cut]]`.** That refutation was correct **under
> the doctrine as written**. The builder has ruled the doctrine wrong. The cut returns.

## ⛔ WHY THE DOCTRINE GOES — its premise was never true

`src/holon/mod.rs` (Stone HOME-8, **2026-08-26**) draws its line as:

> *"a function taking `env: &Environment` and/or `sym: &SymbolTable` is **binding** … It lives in
> `runtime.rs` today"* … *"nothing here reaches back into **`runtime.rs`'s evaluator** (`eval_inner`,
> **`Environment`, `SymbolTable`**) — that boundary is the whole point."*

**`Environment` and `SymbolTable` are not `runtime.rs`'s.** They live in `src/value/environment.rs`
and `src/value/symbol_table.rs` — and they were **already there on the very commit that wrote that
sentence** (`d43f75887`), which also carries `runtime.rs:758-770`'s `pub use crate::value::{…}`
re-export.

★★★ **The doctrine was authored against a facade artifact.** The same mechanism that made
`check.rs:56` import `SymbolTable` from `runtime`, that inflated every home's measured cycle count by
66–83% (`[[NOTE-the-crate-boundary-is-the-real-cut-and-eight-homes-are-cyclic]]`), and that STOP-1 has
fenced in every brief since. Here it did not merely mislead an import — **it authored an
architectural rule.**

⚠ And the rule is already broken in its own home: `src/holon/sigma.rs` (4 signatures) and
`hologram.rs` (2) take `sym: &SymbolTable`. Both were relocated **wholesale** (the Layout calls them
*"formerly top-level `src/sigma.rs`"*), so the signature test was applied to the algebra lift and not
to the modules that arrived intact.

## ★ THE MEASUREMENT THAT SETTLES "SPECIAL"

```
binding-signature params, per impl home
  edn 61 · collection 55 · numeric 40 · record 31 · reflect 31 · kernel 14 · function 6 · declare 2
  holon 8   ← the home whose doctrine forbids them

eval_inner call sites, per impl home
  collection 44 · reflect 20 · numeric 15 · record 15 · edn 0
  of the twelve blocked functions: ONE calls it (run_ast_arg_for_eval_coincident); eleven call nothing
```

Every home built during this campaign holds binding functions freely. **Eleven of the twelve are
blocked by the false clause alone.** Holon is not stricter because its domain demands it; it is
stricter because one stone wrote a rule from a mistaken premise six days ago.

## THE ONE CONTRACT DECISION — pinned

**The two-layer doctrine is replaced by the rule every impl home already observes:**

> **An impl home must not reference `crate::intrinsic`. It may use `crate::value` types and call the
> evaluator, exactly as `collection`, `edn`, `numeric`, `record` and `reflect` do.**

That rule is *derivable* — an impl referencing its own edge is a cycle — where the old one rested on
a misattribution. ★ `codec.rs`'s **stricter** bar (no `WatAST`/`Value`/`RuntimeError`/`Span` either)
is a separate, genuinely earned claim about a wire format and **stays**; this stone does not touch it.

## What ships

1. **`src/holon/mod.rs`'s doctrine is rewritten** — the algebra/binding split struck, the universal
   rule stated, the reason recorded (a facade artifact, with the commit that proves it), and
   `codec.rs`'s stricter bar explicitly preserved. ⚠ `sigma.rs`/`hologram.rs` stop being violations
   as a consequence, not as a fix.
2. **The twelve move**, enumerated on the current file:

```
8896   4  enum PairedVectors        9241  22  run_ast_arg_for_eval_coincident
8901  75  pair_values_to_vectors    9270  23  coincident_of_two_values
9139  38  cosine_outcome_from_values 9295 66  eval_form_digest_coincident_shared
9180  23  presence_q_from_values    9363  70  eval_form_signed_coincident_shared
9206  27  coincident_q_from_values  9442   6  enum FallbackVerdict
9485  41  classify_fallback_outcome 9529  14  dot_outcome_from_values
                                                              409 lines
```

Placement by role, **verified against the bodies**: the outcome constructors to `outcome.rs`; the
coincident family plausibly its own `coincident.rs`; `pair_values_to_vectors` where its callers sit.

## ⛔ TWO ITEMS INSIDE THE SPAN ARE NOT HOLON

`no_field_names` (**8981**) and `builtin_enum_variant_names` (**9087**) sit between the moving items
and are consumed by **10 and 7 other homes** respectively — `io`, `services`, `intrinsic/mod`,
`stream`, `rete/purity`, `host`, `edn`, `declare` among them. Generic `Value`/`EnumValue`
constructors. The re-cast named both EXCLUDED; they are the ninth and tenth intruders this campaign
has found inside a proposed range.

## ★ THE PREDICTION — falsifiable

```
runtime.rs        24,580 -> ~24,180   (-400)
src/holon/        gains 12 items; its mod.rs doctrine rewritten
sigma.rs / hologram.rs   UNCHANGED — they stop violating a rule that no longer exists
codec.rs's stricter bar  UNCHANGED, and still stated
no_field_names / builtin_enum_variant_names   still in runtime.rs
behaviour         every holon verb identical
```

## Out of scope = REJECTED (not deferred)

- **`codec.rs`'s stricter purity bar.** Earned on its own evidence (a wire format), untouched.
- **The two excluded generics.** Ten and seven consumers; not holon's.
- **Registering the twelve as `#[wat_intrinsic]` shims.** The old doctrine named that as their
  future; with the doctrine corrected it is an arc-255 homing question, independent of this move.
- **The remaining re-cast modules** — kernel family · died-error cluster · `option`/`result` ·
  purity classifier.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **correct the doctrine, then move the twelve** | YES | YES | YES | YES | ✅ **ADMITTED** |
| move the twelve, leave the doctrine standing | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| correct the doctrine, move nothing | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| keep the split, fix only its wording | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| strike `codec.rs`'s bar too, for uniformity | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **move-and-leave-the-doctrine Honest? NO** — it ships twelve documented violations and leaves the
  written rule contradicting the code, which is the wall's-paperwork failure exactly.
- **correct-and-move-nothing Honest? NO** — a corrected rule with nothing consuming it is
  unfalsifiable. `[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`
- **fix-only-the-wording Honest? NO** — the measurement says the split has no basis: every sibling
  home holds binding functions, and eleven of the twelve call no evaluator at all. Preserving the
  split with better prose keeps holon unlike every sibling for a reason that does not survive
  contact with the tree.
- **strike-`codec`-too Honest? NO** — its bar is a real claim about a wire format, argued on its own
  evidence, not inherited from the facade mistake. Uniformity is not the goal; **consistency where
  the reason is the same** is.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ the doctrine states the universal rule | `src/holon/mod.rs` | no `Environment`/`SymbolTable` clause; `crate::intrinsic` rule stated |
| ★ `codec.rs`'s stricter bar survives | `src/holon/mod.rs` Layout | still stated, still stricter |
| the twelve moved | `grep -c` each in `src/runtime.rs` | 0 |
| ★ the two generics did NOT | `grep -c "fn no_field_names\|fn builtin_enum_variant_names" src/runtime.rs` | **2** |
| the impl does not know its edge | `grep -c "crate::intrinsic" src/holon/*.rs` | 0 |
| no facade imports | each touched file's `use` block | `crate::value::` direct |
| behaviour unchanged | every holon verb | identical |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
