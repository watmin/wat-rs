# Arc 212 stone δ-comm-positions — SHARPEN `validate_comm_positions` with position-awareness

**Your ONE concern this spawn:** sharpen ONE walker so it can migrate to `node.children()` recursion without producing false positives. Verify ONE named test passes. Nothing else.

This is a SHARPENING stone, NOT a mechanical migration. The walker's RULE gains a new permitted slot. Read the design below carefully before editing.

---

## The walker

**Function:** `validate_comm_positions`
**File:** `/home/watmin/work/holon/wat-rs/src/check.rs`
**Line:** ~2162 (the inscribed comment block at line 2137 explains the temporary List-only fallback)

**Current state:** the walker is List-only with an inscribed TEMPORARY classification (post-arc-212-α reframing). The rule recognizes THREE permitted slots for comm calls (`send`, `recv`, `process-send`, `process-recv`, `Process/readln`, `Process/println`):
1. Match scrutinee position
2. Result/expect value position
3. Option/expect value position

The TEMPORARY List-only behavior exists because naive `children()` migration produces false positives on the FOURTH permitted pattern (bound-name-later-matched-or-expected):

```scheme
(:wat::core::let
  [recv-result (:wat::kernel::recv rx)                   ;; comm in binding-RHS
   _val (:wat::core::match recv-result -> ...)]          ;; bound name later matched
  ...)
```

The walker's rule must learn this fourth slot.

---

## The sharpened rule

**Fourth permitted slot:** a comm call (`send` / `recv` / `process-send` / `process-recv` / `Process/readln` / `Process/println`) appearing in a `:wat::core::let` binding-RHS position is PERMITTED if and only if the binding-name appears later in the same let as:
- a `:wat::core::match` scrutinee (position 1 of `(:wat::core::match <name> ...)`)
- a `:wat::core::Result/expect` value (position 2 of `(:wat::core::Result/expect -> :T <name> "msg")`)
- a `:wat::core::Option/expect` value (same shape)

"Later in the same let" = appears in any later binding-RHS OR in any body form of THIS let.

If the bound name is NOT consumed in the let, the comm-in-binding-RHS is STILL ILLEGAL.

---

## Implementation approach (pre-walk for consumed-names)

When the walker descends into a `:wat::core::let` form:

1. **Pre-walk the let** (all binding-RHSes after each binding's position, plus all body forms) to collect a set `consumed_names: HashSet<String>` of names that appear as:
   - The first symbol of `(:wat::core::match <symbol> ...)`
   - The value argument of `(:wat::core::Result/expect ...)` or `(:wat::core::Option/expect ...)`

2. **Walk each binding-RHS** with a modified `CommCtx`:
   - If the binding's name is in `consumed_names` → push a permitted slot context for THIS binding's RHS (similar to how match-scrutinee position permits)
   - Otherwise → use the existing Forbidden context

3. **Walk the body forms** with the existing context flow (already permits match/expect)

The walker now migrates safely to `node.children()` for generic recursion outside the let-form handler; the let-form handler does its own scope-aware traversal.

---

## Migration target

```rust
fn validate_comm_positions(
    node: &WatAST,
    ctx: CommCtx,
    errors: &mut Vec<CheckError>,
) {
    // Let-form scope-aware handler — implements the fourth permitted slot
    // (bound-name-later-matched-or-expected). Per arc 212 stone
    // δ-comm-positions sharpening: recognize that comm-in-binding-RHS is
    // valid when the binding-name is later consumed in the same let.
    if let WatAST::List(items, _) = node {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if head == ":wat::core::let" {
                // 1. Pre-walk body + later bindings to collect consumed names
                let consumed = collect_consumed_names_in_let(items);
                // 2. Walk bindings with consumed-aware context
                // 3. Walk body with normal context flow
                // ... (sonnet implements this section)
                return;  // do NOT fall through to generic recursion
            }
        }
    }

    // Walker-specific List-head logic for comm-call positions (existing
    // three-slot detection preserved verbatim from pre-arc-212).
    if let WatAST::List(items, _) = node {
        // ... existing head-keyword detection + ctx-based error emission ...
    }

    // Arc 212 — generic recursion via children() covers List, Vector,
    // and StructPattern uniformly. Let-form handler above intercepts the
    // scope-aware case; this handles everything else.
    for child in node.children() {
        validate_comm_positions(child, ctx.recurse_into_child(), errors);
    }
}

fn collect_consumed_names_in_let(items: &[WatAST]) -> HashSet<String> {
    // Walk the let's body + later binding-RHSes; collect symbol names
    // that appear as match-scrutinee or expect-value.
    // ... sonnet implements ...
}
```

You may rename `collect_consumed_names_in_let` or `recurse_into_child` if cleaner names emerge. The signatures + types above are illustrative.

---

## The wat-test proof gate

ONE canonical test exercises the fourth permitted slot:

| Test | Verifies |
|---|---|
| `cargo test --release --test arc112_slice2b_process_send_recv` | Includes `arc112_slice2b_schemes_wire_through_typechecker` which uses the bound-name-later-matched pattern with `recv` |

**This test MUST pass post-migration.**

Additionally run the broader comm-discipline tests to ensure no regression:

| Test | Verifies |
|---|---|
| `cargo test --release --test arc112_scheme_probe` | Sibling arc 112 walker coverage |
| `cargo test --release --test wat_arc208_process_io_result` | Process I/O Result handling (touches the same diagnostics) |

If ANY of these fail and the failure is NOT a "previously-silent bug now caught" case (Mode B honest delta), STOP-trigger 1 fires.

---

## STOP triggers — VERBATIM

Non-negotiable. If any fires, STOP IMMEDIATELY.

1. **Any named test FAILS post-migration AND the failure mode is "previously-passing code now flagged as illegal."** STOP. Revert your edit. Inscribe in SCORE: which test, which file:line in the test, what got flagged. Do not investigate WHY beyond reading the test fixture. Do NOT modify the test. Return.

   **Sub-rule (Mode B):** if the test fails because the sharpened rule now catches a recv/send/etc. in a comm-position that was previously slipping past silently AND the test's intent is to verify rejection of that pattern, that's substrate-teaching not a sharpening failure. Inscribe in SCORE; orchestrator decides.

2. **cargo build FAILS.** STOP. Inscribe the compile error. Return.

3. **You see a failing test OUTSIDE the three named.** STOP. Workspace failure count is NOT your concern. Do not investigate.

4. **You feel the urge to migrate another walker while you're here.** STOP. ONE walker per stone.

5. **You feel the urge to "improve" CommCtx semantics beyond adding the fourth permitted slot.** STOP. The sharpening is bounded: add the fourth slot, migrate the recursion, preserve everything else.

6. **The implementation approach (pre-walk for consumed-names) reveals an architectural problem that can't be resolved within this stone's scope.** STOP. Inscribe what surfaced. Return. Orchestrator decides next move.

Honest STOP + clean report = Mode A's sibling.

---

## What the SCORE file contains

`docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-DELTA-COMM-POSITIONS.md`:

1. Header: `# Arc 212 stone δ-comm-positions — SCORE: sharpen validate_comm_positions`
2. Summary: the fourth permitted slot now recognized; walker migrated to children() for generic recursion; let-form handler does scope-aware pre-walk
3. Implementation: brief description of `collect_consumed_names_in_let` (or equivalent) + how the binding-RHS context flows
4. Verification: three lines (one per named test) showing pass/fail
5. Build line: cargo build clean
6. Honest-delta note if Mode B
7. Mode classification

---

## Constraints

- Edit ONLY `src/check.rs`
- Touch ONLY `validate_comm_positions` + any new helper fns this stone introduces
- Zero other code edits anywhere
- Zero git operations (orchestrator commits)
- Zero test-file edits
- Run ONLY the three named tests + cargo build
- No `cargo test --workspace`

---

## Time prediction

20-40 min Mode A. This is a sharpening stone (substantive design work), not a mechanical migration. Plan: read walker (~5 min) → implement helper (~10 min) → wire let-form handler (~10 min) → run tests + iterate if needed (~10 min).

---

## Mode classification

- **Mode A:** sharpening implemented; all three named tests green; cargo build clean; SCORE written
- **Mode B (acceptable):** test failure traceable to substrate-teaching (previously-silent comm pattern now caught); REVERTED + inscribed honest delta with file:line evidence
- **Mode C:** STOP rule broken (touched another walker, "improved" CommCtx semantics beyond fourth-slot, modified a test, scope-crept)

The substrate teaches; you sharpen; you migrate; nothing else.
