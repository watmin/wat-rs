# BRIEF — seq/collection checker↔runtime container parity (close 3 false-reject drifts)

**Single-hop executor (sonnet). Do NOT spawn sub-agents. Do NOT run git. Do NOT run `cargo wat`
(orchestrator-only). You MAY `cargo build` / `cargo test`.** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)
Three collection ops type-check a NARROWER container set than the runtime actually handles, so the checker
false-rejects valid programs. Extend each checker arm so its accepted container set EQUALS the runtime's — the
runtime is already complete and correct; this is checker-side only. The corrected megafile doctrine applies:
these are NECESSARY changes to `check.rs` / `collection/infer.rs`, sized to correctness (both sides of the
feature must agree). Contract: `DESIGN-STONE-seq-container-drift.md`. The RED probe
(`tests/probe_seq_container_parity.rs`, 7 tests) is the target — green it without weakening it.

## Read in order (the rooms)
1. `tests/probe_seq_container_parity.rs` — the 7 RED tests (the contract). Each defn type-errors at HEAD.
2. **`infer_positional_accessor`** (`src/check.rs:9991`) — `first`/`second`/`third`. The existing `Vector<T>`
   arm (`:10035`) and `List<T>` arm (`:10049`) BOTH return `Option<inner>`. **Add two arms beside them:**
   - `TypeExpr::Parametric { head, args: targs } if head == "wat::core::PersistentVector"` → mirror the
     `Vector` arm EXACTLY (return `Option<inner>`; empty-args → `fresh.fresh()`).
   - `TypeExpr::Path(p) if p == ":wat::WatAST"` → return `Option<:wat::WatAST>`
     (`TypeExpr::Parametric { head: "wat::core::Option", args: vec![TypeExpr::Path(":wat::WatAST".into())] }`).
     (Runtime returns `Option<wat__WatAST>` for a List form — `runtime.rs:10987`.)
   - Update the `_ =>` error message (`:10062`, currently `"tuple, Vec<T>, or List<T>"`) to name all five
     containers.
3. **`rest`** checker arm (`src/check.rs:5301`) — currently `Vector → Vector`, `List → List` (identity).
   **Add two arms** preserving container identity (match the runtime, `collection/eval.rs` rest):
   - `PersistentVector<T> → PersistentVector<T>` (mirror the Vector arm).
   - `:wat::WatAST → :wat::WatAST` (`TypeExpr::Path(":wat::WatAST")`).
   - Update its error message (currently `"Vec<T> or List<T>"`) to name all four.
4. **`infer_conj`** (`src/collection/infer.rs:129`) — currently `Vector`/`PersistentVector`/`HashSet`. **Add a
   `List<T> → List<T>` arm** (mirror the Vector arm; runtime dispatches `Value::wat__core__List` →
   `list_conj_inner` at `runtime.rs:12410`). Update the error message (`:179`) to include `List<T>`.

## Implementation sketch (positional-accessor PV arm — copy the Vector arm verbatim, swap the head)
```rust
TypeExpr::Parametric { head, args: targs } if head == "wat::core::PersistentVector" => {
    if let Some(inner) = targs.first() {
        let result_ty = TypeExpr::Parametric { head: "wat::core::Option".into(), args: vec![apply_subst(inner, subst)] };
        return if local_errors.is_empty() { CheckResult::ok(result_ty) } else { CheckResult::partial_with(result_ty, local_errors) };
    } else {
        return if local_errors.is_empty() { CheckResult::ok(fresh.fresh()) } else { CheckResult::partial_with(fresh.fresh(), local_errors) };
    }
}
```

## Blast radius
`src/check.rs` (positional-accessor arms + rest arms) and `src/collection/infer.rs` (conj arm) ONLY. NO runtime
change (`runtime.rs`/`collection/eval.rs` already handle every container — do not touch them). No new types, no
new ops, no signature changes to existing ops.

## STOP triggers (halt + report; do not improvise)
1. If greening any test seems to require a RUNTIME change — STOP. The runtime is complete; a needed runtime
   change means the diagnosis is wrong. Report what you found.
2. If the `WatAST` element/return type is NOT `TypeExpr::Path(":wat::WatAST")` in these functions' context —
   STOP and report the actual representation; do not guess a different type.
3. If greening needs any file beyond `src/check.rs` + `src/collection/infer.rs` — STOP and report.

## Done = green
`cargo test --release -p wat --test probe_seq_container_parity` → 7/7. AND no regressions:
`cargo build --release` clean; `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"`
→ 941 passed / 36 failed (unchanged floor); the rete differentials still pass
(`--test probe_arc278_8b_accumulate_native_differential`, `--test probe_arc278_6b_ii_a_where_oracle`).

## Report back
The exact diffs (the 4 arms + 3 error-message updates), the probe count verbatim (7/7), the lib floor, and any
STOP. Your final message is all I see — report what the disk shows, not what you intended.
