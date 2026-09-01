# BRIEF — give the refusal the span the author wrote, and the gate will keep it

`refuse_export_without_arm` is a user-reachable refusal whose two call sites stamp
`rust_caller_span!()`, so the user is pointed at a `src/rete/kernel/fire/*.rs` line they never
wrote. The real wat span is one frame up at every real entry, already in hand and already used.
Thread it. Read `DESIGN.md` beside this file first — its ★ says why threading is the *guard* and not
only the fix, and its "out of scope" cuts three shapes including a lint widening that I measured
before rejecting.

## Read in order

1. `src/rete/kernel/fire/rules.rs:629-660` — `fire_rules_on_session` (no span param) and the
   `refuse_export_without_arm` call at `:658`.
2. `src/rete/kernel/fire/rules.rs:599-612` — `eval_fire_rules_native`, which **holds `list_span` and
   already uses it** for its ArityMismatch. This is the span that should reach the refusal.
3. `src/rete/kernel/fire/mod.rs:1028-1042` — `fire_once_session` and its call at `:1036`.
4. `src/rete/kernel/fire/mod.rs:1326-1334` — `eval_fire_once_native`, same shape.
5. `src/rete/kernel/fire/delta.rs:812-841` — `eval_fire_rules_explain`, the third real caller, also
   holding `list_span`.
6. `tests/lint/span_substitution_justified.rs:22-38` — the lint's predicate and the exclusion it
   states as a principle. **You are making its proxy true, not evading it.**

## Sketch

```rust
pub(crate) fn fire_rules_on_session(
    session: &Value,
    span: &Span,                       // ← the param that both fixes and gates
    sym: &SymbolTable,
    support: Option<&mut ExplainSupport>,
) -> Result<Value, EvalBreak> {
    …
    return Err(refuse_export_without_arm(OP, span));
}
```

Same for `fire_once_session`. The three real callers pass their `list_span`; the eight test callers
pass a synthetic span, which is correct — tests are leaves.

## Blast radius — nine call sites for one fn

`fire/rules.rs`, `fire/mod.rs`, `fire/delta.rs`, and five test modules
(`kernel/tests/{strat,fanout,gather_probe,cascade,accum}_cost.rs`). **Enumerated, not estimated:**
`fire_rules_on_session` has 9 callers (3 real + 8 test... the arithmetic is 1 self + 1 delta + 8
test; count them yourself and report the number you find). `fire_once_session` has 1.

## Traps named in advance — each with its step

1. **★ THE MUTATION IS THE POINT OF THIS STRIKE.** After threading, put `rust_caller_span!()` back
   in one of the two bodies. **The EXISTING lint must now flag it.** **Step:** run
   `cargo nextest run --release -E 'binary_id(wat::lint)'` under that mutation and confirm
   `span_substitution_justified` reddens, naming the site. If it stays green, the threading did not
   bring the fn into the gate's view and the ★ decision is unmet — report that, do not adjust the
   lint.
2. **Nine call sites, eight of them tests.** **Step:** `grep -rn 'fire_rules_on_session(' src/ tests/`
   and update all of them. A synthetic span in a test call is legitimate and needs **no rune** —
   tests are the leaves the lint's exclusion exists for.
3. **Do not widen `span_substitution_justified`.** DESIGN cuts it with a measurement (534 tree-wide,
   71 in rete, mostly leaves) and a recorded failure mode about call-graph audits. **Step:** instead
   record the blind spot AT the lint — that its "no span param" test is a *proxy* for "no choice
   exists", with those numbers — so the next reader knows what it does and does not cover.
4. **`support: Option<&mut ExplainSupport>` is already a param.** Adding a fourth may trip clippy's
   arity ceiling, which has bitten this tree before (`check_operand_field_ref` hit 8 and was
   bundled). **Step:** if it fires, bundle rather than `#[allow]` — the ceiling is doing its job, and
   `ClauseCtx` in `validate/typing.rs` is the worked example of the bundling shape.
5. **New test code trips `wat::lint`.** **Step:** run that binary before reporting. It has caught a
   red for three consecutive riders.
6. **E1 is driven and ready but is NOT this strike.** **Step:** if you find yourself in
   `validate/typing.rs`, stop — that is the next strike and its evidence is already in DESIGN.

## STOP triggers

- **STOP-1** — if trap 1's mutation leaves the lint green, STOP and report. That is the whole
  structural claim of this strike and I would rather know it is false.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if a caller turns out NOT to have a real span available, STOP and name it. DESIGN
  asserts all three do; if a fourth exists, that assertion is wrong and I want to hear it.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-variant-diagnostic/` — the strike immediately before this,
same arc, same "the refusal fires correctly and says the wrong thing" subject.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Sixteen riders before you each returned a prescription of
mine that did not survive contact. The last found that my sketch, followed literally, would have
emitted a self-contradicting diagnostic — the exact class its own strike existed to delete. If a
step here is wrong, unnecessary, or impossible, say it plainly.
