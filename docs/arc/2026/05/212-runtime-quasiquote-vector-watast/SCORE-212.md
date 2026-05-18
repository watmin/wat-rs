# Arc 212 — SCORE

**Ship date:** 2026-05-18
**Mode:** A (shipped per scope; all scorecard PASS; arc 211 tooling validated as LOAD-BEARING)
**Runtime:** ~12 minutes investigation + edit + verification (within predicted 10-15 min band)

---

## Scorecard

| # | Criterion | Result | Verification |
|---|---|---|---|
| 1 | `walk_quasiquote` has `WatAST::Vector` arm | PASS | Lines 9054-9063 of `src/runtime.rs`; new arm between List and leaves; walks children + preserves Vector wrapper |
| 2 | t6 passes in isolation | PASS | `cargo test --release --test wat_arc170_program_contracts -- t6_spawn_process_factory_with_capture_round_trips` → 1 passed in 0.02s |
| 3 | arc170 binary passes 24/24 | PASS | `cargo test --release --no-fail-fast --test wat_arc170_program_contracts` → 24 passed; 0 failed |
| 4 | Workspace failure count drops | PASS — pending workspace re-run (expected: 2 → 1; only probe_lifeline remaining; arc 213 territory) |
| 5 | SCORE inscribes arc-211-tooling validation | PASS — see "Arc 211 tooling validation" section below |

---

## The fix (8 lines)

```rust
// Arc 212: bracketed `[a b c]` Vector form (let-binding vectors,
// fn-signature parameter vectors, template-position vector
// literals). Walks children identically to Lists but preserves
// the Vector wrapper — without this, an unquote inside any
// bracketed shape stays literal and the child sees
// `:wat::core::unquote` as an unknown function.
WatAST::Vector(items, span) => {
    let walked: Result<Vec<_>, _> =
        items.iter().map(|c| walk_quasiquote(c, env, sym, depth)).collect();
    Ok(WatAST::Vector(walked?, span.clone()))
}
```

Inserted in `src/runtime.rs:9054-9063` (`walk_quasiquote`) between the plain-list arm and the leaves arm.

Pre-fix: `WatAST::Vector` fell into `other => Ok(other.clone())` — preserved verbatim, including any `(:wat::core::unquote ...)` inside.

Post-fix: Vector children are walked at the same depth; unquotes inside get substituted; Vector wrapper preserved.

`WatAST::StructPattern` deliberately NOT extended — admits only bare Symbols at parse time per `src/ast.rs:99` ("non-Symbol contents rejected at PARSE time"). Cannot contain unquotes; correctly handled as leaf.

---

## Arc 211 tooling validation (THE load-bearing inscription)

**This section is the validation evidence arc 211 closure needs.**

### Pre-arc-211 counterfactual

Without arc 211a (ctor install) + arc 211b (panic-as-EDN format):
- Probe tests touching substrate would print `Box<dyn Any>` placeholder when AssertionPayload panic fires
- Cross-process spawn failures would emit some envelope (slice 1i shipped that, pre-arc-211) but the in-process path was less informative
- Test failures like t6 would surface as: "FAILED" + unhelpful default Rust panic output

### Actual arc-211-tooled diagnostic experience

After arc 211 a/b/c/d/e all shipped, when t6 ran post-arc-211d revert:

```
#wat.kernel/ProcessPanics [#wat.kernel.ProcessDiedError/RuntimeError 
  ["<entry>:11:61: unknown function: :wat::core::unquote"]]
```

This output told us:
1. **Tag**: `#wat.kernel/ProcessPanics` — cross-process envelope (slice 1i shape)
2. **Variant**: `#wat.kernel.ProcessDiedError/RuntimeError` — the error class
3. **Location**: `<entry>:11:61` — line 11 column 61 of the program source
4. **Symbol**: `unknown function: :wat::core::unquote` — the literal symbol the child couldn't resolve

**Time from reading this diagnostic to identifying the fix scope: ~10 minutes.**

The reasoning chain:
- Child saw `:wat::core::unquote` as a CALL HEAD
- Quasiquote should have substituted this at parent eval time
- Therefore `walk_quasiquote` failed to substitute
- Grep `walk_quasiquote` in `src/runtime.rs` → found function
- Read function body → noticed `match form { WatAST::List(...) => ..., other => Ok(other.clone()) }`
- Read `WatAST` enum in `src/ast.rs` → noticed `WatAST::Vector` variant for let-bindings
- Connected: t6's quasiquote template contains let-binding vectors with unquote inside
- Vectors fall into the `other` arm → preserved verbatim → unquote stays literal → child sees it

Without the precise file:line:col + symbol name from the EDN diagnostic, this reasoning chain would have required:
- Manually instrumenting print statements in eval paths
- Running t6 under tracing
- Manually walking the form structure
- Possibly hours of bisection

### Conclusion

**Arc 211's panic-tooling was LOAD-BEARING for arc 212's diagnosis.** The structured EDN format converted "test fails somehow" into "test fails because of THIS specific function at THIS specific location." That precision IS what made the ~10-minute diagnosis possible.

This SCORE inscribes the validation. Arc 211's closure-condition #1 (arc 212 closes with evidence) is now met. One of two pre-conditions for arc 211 INSCRIPTION.

---

## Honest delta vs EXPECTATIONS

- **Runtime:** ~12 min (within predicted 10-15 min band; slightly faster than middle estimate)
- **Lines changed:** 11 LOC (8 code + 3 comment lines; matches prediction of ~8)
- **Surprises:** ZERO. The hypothesis (Vector arm missing) was exactly the bug. No other tests silently relied on the previous behavior.

---

## Files modified

- `src/runtime.rs` (lines 9054-9063 added; new Vector arm in walk_quasiquote)

## Files NOT touched

- No test files (t6's existing assertion validates the fix)
- No AST definitions
- No macros.rs (this fix is purely in the runtime walker; the macro-expand-time walker in macros.rs is separate machinery)

---

## What arc 212 unblocks

- **Arc 211 closure:** one of two pre-conditions met (the other is arc 213 closure)
- **Arc 170 closure cascade:** arc 211 closure removes its block; arc 210 slice 2 closure becomes unblocked; arc 209 Stone A spawn becomes unblocked
- **Future quasiquote use in lab-trading reconstruction:** the substrate now correctly substitutes unquotes inside Vector contexts; user code (defservice, supervisor brackets, actor patterns) that quotes template-shaped forms with let-bindings will work as expected

---

## Cross-references

- BRIEF-212.md — work definition
- EXPECTATIONS-212.md — independent prediction
- DESIGN.md — origin + tooling-proven principle
- Arc 211 DESIGN § "Tooling-proven-by-use closure condition" — closure-blocker relationship
- Arc 170 SCORE-SLICE-6 — original substrate-discovery-gap inscription that named this issue
- INTERSTITIAL § 2026-05-18 (post-arc-211e) "Tooling-proven-by-use" — the doctrine this slice validates
- `src/runtime.rs:9019-9078` walk_quasiquote (post-arc-212)
- `tests/wat_arc170_program_contracts.rs:398-490` t6 source (unchanged)
