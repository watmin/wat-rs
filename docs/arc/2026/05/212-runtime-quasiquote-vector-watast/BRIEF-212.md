# Arc 212 — BRIEF

**Slice scope:** Extend `walk_quasiquote` in `src/runtime.rs` to recurse into `WatAST::Vector` nodes (currently only `WatAST::List` is walked). The result: unquotes inside vector contexts (let-bindings, fn signatures, vector literals at template positions) get substituted instead of preserved verbatim.

**Closes:** t6_spawn_process_factory_with_capture_round_trips. Validates arc 211's panic-tooling (the EDN message led directly to the diagnosis).

## Root cause (diagnosed from arc 211b's panic-EDN)

T6 panic output (cross-process envelope, structured EDN per arc 211b):

```
#wat.kernel/ProcessPanics [#wat.kernel.ProcessDiedError/RuntimeError 
  ["<entry>:11:61: unknown function: :wat::core::unquote"]]
```

The child process saw the form with `:wat::core::unquote` still embedded — meaning the parent's quasiquote evaluator didn't substitute it. Tracing to `src/runtime.rs:9019` `walk_quasiquote`:

```rust
match form {
    WatAST::List(items, span) => { /* recurses with qq/unquote detection */ }
    // Leaves are preserved verbatim.
    other => Ok(other.clone()),
}
```

`WatAST::Vector(...)` falls into the `other` arm. T6's quasiquote template contains a let-binding `[main-form ...]` which is a `WatAST::Vector`. The walker skips it. The `~offset` inside stays unsubstituted.

## Scope

**File touches (exactly these):**

1. `src/runtime.rs:9019-9063` — `walk_quasiquote` function. Add a `WatAST::Vector(items, span)` arm that walks children identically to the plain-list children-walk path (lines 9056-9058) but preserves the Vector wrapper. Pattern:

```rust
WatAST::Vector(items, span) => {
    let walked: Result<Vec<_>, _> =
        items.iter().map(|c| walk_quasiquote(c, env, sym, depth)).collect();
    Ok(WatAST::Vector(walked?, span.clone()))
}
```

Inserted between the plain-list arm (end at line 9059) and the leaves arm (line 9061).

**NOT in scope:**
- `WatAST::StructPattern` — admits only Symbol children at parse time per `src/ast.rs:99` ("Empty `{}` and non-Symbol contents are rejected at PARSE time"). Can't contain unquotes. Skip.
- `unquote-splicing` inside Vector — out of scope; `walk_quasiquote`'s docstring notes splicing is "not yet handled here (it requires the outer-list-context scan; not yet surfaced as a real lab need)". If needed later, sibling arc.
- Other tests besides t6 — verify workspace count drops but don't scope-creep into other failures.

## Implementation protocol

1. Edit `src/runtime.rs` per Scope #1
2. Run t6 alone: `cargo test --release --test wat_arc170_program_contracts -- t6_spawn_process_factory_with_capture_round_trips` → expect PASS
3. Run full arc170 binary: `cargo test --release --test wat_arc170_program_contracts -- --no-fail-fast` → expect 24/24
4. Run workspace: `cargo test --release --workspace --no-fail-fast 2>&1 | tail -10` → expect ≤1 remaining failure (probe_lifeline; arc 213 territory)
5. Write SCORE-212.md inscribing: the fix; whether arc 211 panic-EDN was load-bearing for the diagnosis (YES); arc 211 closure unblocking status

## Success criteria

| # | Criterion | Verification |
|---|---|---|
| 1 | `walk_quasiquote` has `WatAST::Vector` arm | `grep -n "WatAST::Vector" src/runtime.rs` |
| 2 | t6 passes in isolation | cargo test --release --test wat_arc170_program_contracts -- t6_... |
| 3 | arc170 binary passes 24/24 | --no-fail-fast run |
| 4 | Workspace failure count: 2 → 1 | probe_lifeline remains (arc 213 territory) |
| 5 | SCORE inscribes arc-211-tooling validation | document the EDN's role in the diagnosis |

## Time prediction
10–15 min. Minimal fix; single-arm match extension.

## Tooling-proven-by-use validation (load-bearing for arc 211 closure)

This BRIEF documents the validation pathway:
- Arc 211b's panic-EDN format → produced `<entry>:11:61: unknown function: :wat::core::unquote` (precise file:line:col + symbol name)
- Without 211b: the same failure would have been `Box<dyn Any>` placeholder; we'd know t6 fails but not why
- With 211b: walked directly to `walk_quasiquote`'s scope-limit; ~10 minutes diagnosis

When this slice ships, SCORE-212 must inscribe this validation. That inscription IS the evidence arc 211 closure needs.

## Cross-references

- Arc 212 DESIGN.md
- Arc 211 DESIGN § "Tooling-proven-by-use closure condition"
- Arc 170 SCORE-SLICE-6 — the original "downstream stone" inscription
- `src/runtime.rs:9019` walk_quasiquote
- `src/ast.rs:81` WatAST::Vector
- `tests/wat_arc170_program_contracts.rs:398` t6 source
