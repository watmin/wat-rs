# BRIEF — Stone 238.1 — complete `values_equal` (the `=` verb) for data types

**Status:** READY TO SPAWN. `model: "sonnet"`.
**Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (`pwd` first; reject any `.claude/worktrees/` path; `git -C` if needed).
**DESIGN:** `docs/arc/2026/05/238-core-equality-completeness/DESIGN.md` — read it; the audit table + the
data-vs-opaque doctrine ARE the spec.

## What to do (one additive change to ONE function)

`:wat::core::=` (and its inverse `not=`) route through `values_equal` (`src/runtime.rs:9322`).
That function has arms for Vec/List/Tuple/Option/Result/Enum/Struct/Vector/HolonAST/scalars but is
MISSING records, maps, sets, Instant, Duration. So `(= rec rec)` / `(= map map)` / `(= set set)`
ERROR today. Add the missing arms, all BEFORE the final `_ => None`. Additive — you are not
touching any existing arm, so existing behavior cannot change (baseline-preserving by construction).

Add these arms (mirror the shapes already in the function):

1. **Records — ONE or-patterned arm** (handles all 4 flavor combos):
   ```rust
   (Value::wat__holon__Record { class_fqdn: ca, struct_form: sa, .. }
        | Value::wat__Record { class_fqdn: ca, struct_form: sa },
    Value::wat__holon__Record { class_fqdn: cb, struct_form: sb, .. }
        | Value::wat__Record { class_fqdn: cb, struct_form: sb }) => {
       if ca != cb { return Some(false); }
       if sa.len() != sb.len() { return Some(false); }
       for (x, y) in sa.iter().zip(sb.iter()) {
           match values_equal(x, y) { Some(true) => continue, Some(false) => return Some(false), None => return None }
       }
       Some(true)
   }
   ```
   This is exactly the `Struct` arm (runtime.rs:9541) with `class_fqdn` in place of `type_name`.
   Type-strict (same class + same field values); cross-flavor ⟹ cross-class ⟹ `Some(false)` (the
   or-pattern means base-vs-holonic is caught here and decided by class, NOT routed to `None`).

2. **HashMap:** `(Value::wat__std__HashMap(a), Value::wat__std__HashMap(b)) => Some(a == b)`
   — storage is `Arc<HashMap<Value,Value>>` (arc 216.5c); `Value: PartialEq` exists (216.5a);
   `==` is order-independent + structural + total.

3. **HashSet:** `(Value::wat__std__HashSet(a), Value::wat__std__HashSet(b)) => Some(a == b)`
   — `Arc<HashSet<Value>>` (216.5b); order-independent.

4. **Instant:** `(Value::Instant(a), Value::Instant(b)) => Some(a == b)` — mirror the
   `values_compare` Instant arm (runtime.rs:9609); `chrono::DateTime<Utc>: Eq`.

5. **Duration:** `(Value::Duration(a), Value::Duration(b)) => Some(a == b)` — mirror
   `values_compare` (9610); i64 nanos.

6. **WatAST — ONLY IF `WatAST: PartialEq`:** `(Value::wat__WatAST(a), Value::wat__WatAST(b)) => Some(a == b)`
   — symmetry with the `holon__HolonAST` arm (9540). **First `grep` / check that `WatAST` derives or
   impls `PartialEq`. If it does NOT, STOP and surface it — do NOT fabricate an impl.** (This one is
   optional/conditional; the other five are required.)

## Method

Add the arms → `cargo build --release -p wat` → `cargo test --release --test probe_arc238_eq_completeness`
(the RED probe on disk; make it 8/8) → lib baseline. If a compiler error is non-obvious, STOP and
surface it verbatim (`feedback_nonintuitive_error_is_pivot`).

## Co-located unit tests (Rust layer — for the arms the external probe can't easily reach)

The external probe covers records/maps/sets (constructible at the wat surface). Add `#[cfg(test)]`
unit tests in `runtime.rs` for **Instant** + **Duration** equality (construct two equal values
directly in Rust; `=`/`values_equal` returns `Some(true)`; two different → `Some(false)`), since
constructing those at the wat surface needs time verbs. If you added the WatAST arm, a unit test for
it too.

## STOP triggers (REJECTION)

1. You modify an EXISTING `values_equal` arm (only ADD new arms before `_ => None`).
2. You add an arm for an OPAQUE type (fn, clauses, Sender/Receiver/Handle/HandlePool, Engram/
   EngramLibrary/Hologram/OnlineSubspace/Reckoner, io readers/writers, RustOpaque) — those stay
   erroring (out of scope; value-equality of a handle/fn is meaningless).
3. You fabricate a `WatAST: PartialEq` impl (if it's missing, STOP + surface).
4. You touch `values_compare`, `eval_eq`, or anything outside `values_equal` + its co-located tests.
5. You touch holon-rs (STOP-5).
6. Lib baseline drops below 828/0 for any reason other than your new co-located unit tests ADDING
   to the count (828 → 830ish).
7. A non-obvious error (→ pivot, surface verbatim).
8. 45 min (STOP-3); 60 (STOP-4). This is a small additive change.

## Regression suite

```
cargo build --release -p wat                                    # 0 errors
cargo test --release --lib -p wat                               # >= 828, 0 failed (+ your Instant/Duration unit tests)
cargo test --release --test probe_arc238_eq_completeness        # 8/8 (was RED)
cargo test --release --test probe_arc237_sC2c_base_record       # 6/6 (record variant regression)
cargo test --release --test probe_arc227_stone2_defrecord       # 35/35
```

## SCORE doc

`docs/arc/2026/05/238-core-equality-completeness/SCORE-STONE-238.1.md` (NEW). Scorecard table + each
arm added + WatAST disposition (added / absent-and-skipped) + honest deltas + `git status --short`.
DO NOT commit (orchestrator commits).

## Calibration

~5-6 arms in ONE function, each mirroring an existing arm; additive; baseline-preserving. Smaller
than S-C.2c. **Target band: 20-40 min Mode A; 45 STOP-3; 60 STOP-4.**
