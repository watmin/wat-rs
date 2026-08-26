# ⛔ NOTE — the registry asserts properties NOTHING VERIFIES, and the numerics rehome found six tables asserting them separately

Filed 2026-08-26, at the builder's direction: *"we need to add a NOTE-<slug>.md somewhere to deeply
scrutinize the registry's tagged values for all the things... this is one of many that needs
corrective work."*

**Not scoped, not queued.** A class, with the instances measured this session so a later stone starts
from evidence rather than from this paragraph.

## The class, in one sentence

**A verb's properties — pure, deterministic, total, its type scheme, its arity, whether it is a rete
primitive — are asserted in hand-maintained tables keyed by literal string, and nothing derives them
from the thing they describe or checks them against it.**

Two consequences, both measured below: the same property is restated in several places and they
drift; and an assertion can simply be FALSE and nothing notices, because no instrument compares the
claim to the behaviour.

## INSTANCE 1 — a property that is simply false

`HashMap/keys`, `HashMap/values`, `PersistentMap/keys`, `PersistentMap/values` were classified
**Pure ∧ Deterministic** in `src/rete/purity.rs`. Measured, three consecutive processes:

```
HashMap/keys        [:c :a :b :d :e]   · [:c :d :a :e :b]   · [:e :b :c :d :a]
PersistentMap/keys  [:i :d :a :f :j …] · [:f :e :g :a :j …] · [:e :h :d :a :g …]
```

Not deterministic, and not marginally — a different order every run, for both containers. The
substrate says so itself in `src/value/pmap.rs`: *"iteration order is deliberately NOT part of the
contract."* The purity table never got the message.

**Corrected 2026-08-26**, with the non-vacuity stated: ZERO `.wat` rules use these inside a fence
today (39 call sites, none in a `where`/`then`/accumulator), so the floor did not move. The fix is
prophylaxis — it closes the lie before the persistent-backend swap makes it load-bearing.

## INSTANCE 2 — one property, six tables, found one test at a time

Arc 255's numerics rehome (`b2d10158f` → `11b85591e`) renamed one family and broke **six separate
hand-maintained tables**, each discovered only when a test went red:

| # | site | asserts |
|---|---|---|
| 1 | runtime dispatch (keyword-head) | where the verb goes |
| 2 | `src/macros/eval.rs` `is_pure_total` | pure ∧ total, for the macro F5 gate |
| 3 | `src/rete/purity.rs` `pure_det` | pure ∧ deterministic |
| 4 | `src/rete/purity.rs` `total` | total — **missed by two stones that updated #3** |
| 5 | `src/check.rs` builtin schemes | the type |
| 6 | `src/runtime.rs` `dispatch_substrate_impl` | where `apply` sends it |

#3 and #4 sit in the SAME FUNCTION and drifted from each other. #6 was skipped on a reasoned argument
("the registry door is consulted first") that was true for a keyword head and false for `apply` of a
bound keyword.

## INSTANCE 3 — the gap that mirroring would have preserved

`is_pure_total` was missing six per-type verbs (`i64::=`, `i64::not=`, `i64::to-bigint`,
`i64::to-rational`, `f64::=`, `f64::not=`) under **both** spellings. Nothing had ever noticed,
because a false REFUSAL only surfaces if some macro body happens to call the verb. Default-deny hides
its own gaps: the list cannot tell "deliberately excluded" from "never added".

## INSTANCE 4 — the property with no home at all

`#[wat_intrinsic]`'s doc preamble declares `@Purity` (277 uses), `@Determinism` (269), `@Category`
(274), `@added` (196). **There is no `@Total`.** Totality is expressible NOWHERE at the registration
site, which is exactly why it lives in a hand-curated list with per-op prose reasoning (`f64::*` is
not total — overflows to ±Inf; `f64::>` is — its output is a bool).

So purity and determinism are ALREADY registry properties that the hand-lists merely restate;
totality is the one that genuinely has nowhere else to live. **A corrective stone starts there:** mint
`@Total`, move the curated reasoning to the registration site, and let the consumers read it.

## INSTANCE 5 — the checker accepts a type nobody registered

```
a field typed  :wat::core::NotARealType   --check EXIT=0
```

Surfaced 2026-08-25 by a rider proving a checker non-vacuous. A pre-written gate is banked against
it: `tests/types/probe_diag_typealias_leniency.rs:16`.

## INSTANCE 6 — the blanket-accept, and why it is the endgame not a stone

`src/resolve/walk.rs:268` — `if is_reserved_prefix(head) { return true }` — accepts ANY `:wat::` name,
real, retired, or invented. Its comment defers to the type checker; the type checker does not do it.

**Measured 2026-08-26**, imposing default-deny as a throwaway probe:

```
Summary  5059 tests run: 2520 passed, 2539 FAILED, 19 skipped
```

Half the floor. This cannot close until the registry answers for far more than its current 207 names
— which is what the homes campaign is for. Its gate is already written and disarmed:
`tests/wat_lang/probe_undefined_builtin_resolves.rs`, *"unlock when we circle back to arc 255"*.

## ★ INSTANCE 4-bis — THE TWO DESIGNS NEVER MET, and 255.3 cannot land without one field

Builder, 2026-08-26: *"our current arc 255 is the one who will address `@Total`, right?"* — and the
answer is **yes, it already has the slice, but the slice cannot close as written.**

`255/DESIGN.md` commits to exactly this collapse. All three of its `total` mentions are the
FUNCTION name, not the property:

```
:380   macros::is_pure_total (eval.rs:344, the macro-expand allow-list) — DELETES; queries …
:464   macros::is_pure_total (eval.rs:344) DELETES → queries `:expand-time-legal`
:482   255.3 — consumers collapse (rete/purity + macros::is_pure_total delete; is_effectful_op …)
```

**But the LOCKED RECORD MODEL's Layer-1 baseline (2026-06-21) is:**

```
name · arity · kind · pure · deterministic · expand_time_legal · defined_in · layer
```

`pure` ✓ · `deterministic` ✓ · **`total` ✗.**

The macro half is covered — `is_pure_total` deletes and queries `:expand-time-legal`. The rete half
is not, because its fence is FOUR axes (arc 278 Stone 6a):

```
(and (pure? f) (deterministic? f) (total? f) (primitive? f))
```

Three of the four have a home in the baseline. The fourth has nowhere to move to, so **255.3 would
delete a consumer whose property the destination cannot hold.**

**The mechanism is a date.** 255's model was locked 2026-06-21. The four-axis fence is arc 278's, and
it invented the Total axis *afterwards* — in a hand-list, because that was the only place available.
Neither arc is wrong; they simply never met, and nothing in either document points at the other. That
is this NOTE's class one level up: not a property nothing verifies, but a property no design **owns**.

**Consequence for scoping:** 255.3 needs one field added to a model explicitly marked LOCKED, sitting
beside `pure` and `deterministic` where it obviously belongs. That is a deliberate act on a locked
model and the builder's ruling, not a rider's. Once the field exists, `@Total` at the registration
site is mechanical — `@Purity` has 290 uses and `@Determinism` 282 across 256 registered intrinsics,
so the shape is proven — and rete's curated per-op totality reasoning (`f64::*` is not total: it
overflows to ±Inf; `f64::>` is: its output is a bool) moves to the site that declares the verb, where
it can be read next to the code it describes.

## The shape a corrective stone would take

Not "add the missing entries" — that is what this arc has been doing all week, six times, and the
seventh table will drift too. The shape is:

1. **The registration site is the single source of truth.** It already carries purity, determinism,
   category, arity. Mint `@Total` so it carries the last one.
2. **Consumers DERIVE.** `src/check.rs`'s `register_builtins` already does this for one case — it
   walks `registry().all_entries()` and aliases new spellings onto old schemes rather than restating
   36 names, and that derivation **caught a real defect on its first run** (the variadic `max-of`
   divergence, at check time, loudly). That is the proof the shape works.
3. **The hand-lists shrink to what the registry cannot answer** — special forms, wat-defined verbs —
   and shrink further as each home lands. A ratchet, not a rewrite.
4. **A property must be checkable, not merely declared.** `keys`/`values` were declared deterministic
   for years. The claim was never compared to the behaviour, and the comparison took one loop.
