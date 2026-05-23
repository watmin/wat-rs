# BRIEF — Arc 224 Stone 224.5 — Group A L1 fixes (substrate naming honesty)

## What we're doing

Closing arc 224's own Group A scope: 4 substrate-internal naming lies that the intueri audit surfaced and recommended for an in-arc sonnet stone.

Group B was atomize/materialize substrate-wide rename — that became arc 225 (already SHIPPED at `189b033`).

Group A is mechanical doc + small rename fixes inside `src/runtime.rs` + `src/check.rs`. No semantic change. Low cascade risk.

## Audit pre-verification (drift accounting)

The audit was 2026-05-22 morning; substrate has drifted since. Verified state on HEAD `7a5e4b5`:

| Item | Audit said | Reality on HEAD | Disposition |
|---|---|---|---|
| L1-runtime-2 | `runtime.rs:1105-6` + 5 expected-strings at 18160/18252/18320/18406/18821 | `runtime.rs:1105-6` confirmed; expected-strings drifted to 19025/19117/19185/19271/19686 | NEEDS FIX |
| L1-runtime-3 | `runtime.rs:13605-10` — `holon_item_to_value` error op | `runtime.rs:13547` says: *"Arc 225 Stone 225.1 — renamed from `holon_item_to_value`. `op: &str`"* | **ABSORBED BY ARC 225 — DO NOT RE-DO** |
| L1-check-A | `check.rs:3675-99` — `type_contains_sender_kind` doc + rename | Function at `check.rs:3700`; doc at adjacent lines | NEEDS FIX |
| L1-check-B | `check.rs:143` — `ScopeDeadlock` variant doc | Confirmed at `check.rs:143` (QueueSender/QueuePair refs in doc) | NEEDS FIX |
| L1-check-C | `check.rs:15624` — `symbol_ty` closure → `keyword_ty` rename | Closure drifted to `check.rs:15834`; 4 citations at 15843/15852/15861/15910 | NEEDS FIX |

**4 L1 fixes remaining, not 5.**

## Baseline (pre-flight)

`cargo test --release --lib -p wat`: **827 passed, 0 failed, 1 ignored** on HEAD `7a5e4b5`.

This is the post-stone target.

## The 4 fixes (verbatim)

### Fix 1 — L1-runtime-2 (the type_name lie)

**File:** `src/runtime.rs`

**Lie:** `Value::type_name()` for `wat__kernel__Sender` / `wat__kernel__Receiver` returns `rust::crossbeam_channel::Sender` / `Receiver` — the internal transport name. The wat-level type checker enforces the tier distinction structurally; `type_name` SHOULD return the user-visible wat-level kind.

**Truth:** The wat-visible kind is `wat::kernel::Sender` / `wat::kernel::Receiver`. The transport name is implementation detail that leaks through `type_name()` user-facing error messages.

**Change at `runtime.rs:1105-6`:**

```rust
// BEFORE:
Value::wat__kernel__Sender(_) => "rust::crossbeam_channel::Sender",
Value::wat__kernel__Receiver(_) => "rust::crossbeam_channel::Receiver",

// AFTER:
Value::wat__kernel__Sender(_) => "wat::kernel::Sender",
Value::wat__kernel__Receiver(_) => "wat::kernel::Receiver",
```

**Update the doc comment at `runtime.rs:1100-4`** to reflect the honest name (the comment about "report the same type_name" stays — both tiers DO report the same user-visible kind; that's the structural-tier-distinction point — but the example name in the comment if any should be wat-kernel form).

**Update 5 expected-string call sites** to match the new honest name. Current `expected:` strings at these lines:

- `runtime.rs:19025` — `expected: "rust::crossbeam_channel::Sender"` → `"wat::kernel::Sender"`
- `runtime.rs:19117` — `expected: "rust::crossbeam_channel::Receiver"` → `"wat::kernel::Receiver"`
- `runtime.rs:19185` — `expected: "rust::crossbeam_channel::Receiver"` → `"wat::kernel::Receiver"`
- `runtime.rs:19271` — `expected: "rust::crossbeam_channel::Sender | rust::crossbeam_channel::Receiver"` → `"wat::kernel::Sender | wat::kernel::Receiver"`
- `runtime.rs:19686` — `expected: "rust::crossbeam_channel::Receiver"` → `"wat::kernel::Receiver"`

**DO NOT change** the type-equality checks in `check.rs` (lines 3716, 4549, 4563, 10259-60, 10266) — those check against the canonical TYPE ALIAS resolution chain (`wat::kernel::Sender` → `rust::crossbeam_channel::Sender`), and the head name `rust::crossbeam_channel::Sender` IS correct after `expand_alias` unwrapping. The check.rs sites operate on resolved TYPE expressions, not on user-facing value type_name strings. They are NOT the lie; only the user-visible runtime `type_name()` was lying.

**DO NOT change** the type registrations in `src/types.rs:968+975` — those register `rust::crossbeam_channel::Sender/Receiver` as the canonical head name for the type alias chain. Those are correct.

### Fix 2 — L1-check-A (type_contains_sender_kind doc + rename)

**File:** `src/check.rs`

**Lie:** Function name `type_contains_sender_kind` is a YES/NO-shape name but the function returns `Option<&'static str>` (the kind on hit). Rust convention for Option-returning search is `find_*` or `*_kind_in_*`. The doc at adjacent lines uses retired vocabulary (`QueueSender`, `QueuePair`).

**Truth:** The function answers "what sender-bearing kind, if any, does this type contain?" — that's the `sender_kind_in_type` shape.

**Changes:**

1. **Rename the function** at `check.rs:3700` from `type_contains_sender_kind` → `sender_kind_in_type`.
2. **Update all callers** (lines 3529, 3732, 3743, 3746, 3750, 3758, 3774, 9789 — exhaustive sweep via grep `type_contains_sender_kind`).
3. **Update doc references** at `check.rs:4534` + `check.rs:9701`.
4. **Rewrite the doc comment** at `check.rs:3675-3699` block:
   - Drop `QueueSender` / `QueuePair` retired vocabulary
   - Use canonical: `wat::kernel::Sender`, channel pair (via `pair()`), `HandlePool`
   - The "Why QueueSender" justification block needs the canonical analogue: "Why Sender (and not also bare `rust::crossbeam_channel::Sender` in isolation)"
   - The QueuePair → Sender alias-unwrapping explanation now references `wat::kernel::Sender` as the post-resolution head

### Fix 3 — L1-check-B (ScopeDeadlock variant doc)

**File:** `src/check.rs:143`

**Lie:** The `ScopeDeadlock` variant doc text mentions `QueueSender`, `QueuePair`, `HandlePool` as the detection set. `QueueSender` / `QueuePair` are retired names (arc 109 K.kernel-channel rename).

**Truth:** Canonical names are `wat::kernel::Sender`, channel `pair()`, `HandlePool`.

**Change:** Rewrite the doc text at `check.rs:143` (within the `ScopeDeadlock` variant doc block — likely spans a few lines above/below 143) to use canonical vocabulary.

This may overlap structurally with Fix 2's doc rewrite if both reference the same vocabulary set. Coordinate consistency.

### Fix 4 — L1-check-C (symbol_ty → keyword_ty closure rename)

**File:** `src/check.rs`

**Lie:** Closure `symbol_ty` at `check.rs:15834` constructs `TypeExpr::Path(":wat::core::keyword".into())` — the type IS keyword, not symbol. The closure name lies about its return value.

**Truth:** It returns the keyword type. Name it accordingly.

**Changes:**

1. **Rename closure** at `check.rs:15834`: `symbol_ty` → `keyword_ty`.
2. **Update 4 citation sites** at `check.rs:15843`, `15852`, `15861`, `15910` (each calls `symbol_ty()` as part of `params: vec![...]` construction).
3. **Update adjacent comment** if any references the old name.

Run a grep for `symbol_ty` after the rename to confirm 0 references remain (other than the rename itself).

## Out of scope (affirmative scope-bounding)

These are NOT to be touched by this stone:

- **L1-runtime-3** — absorbed by arc 225 Stone 225.1 v3; the function `holon_item_to_value` was renamed + `op: &str` threaded as part of that work (see `runtime.rs:13547` comment).
- **L2 stale-vocabulary mumbles** — NOT enumerated specifically by the audit (only categorized). Out of arc 224's scope for now; can be addressed by a future general doc-refresh stone when one opens. NOT "deferred" — explicitly out of arc 224's scope per `feedback_no_known_defect_left_unfixed`'s acceptable-language pattern.
- **Type-equality checks in check.rs against `rust::crossbeam_channel::Sender`** — those check against alias-resolved heads, which IS the canonical name post-`expand_alias`. NOT lies. See Fix 1 explanation.
- **Type registrations in types.rs** — those register the alias-chain head correctly.
- **holon-rs** — NOT touched.
- **wat-edn** — NOT touched (no overlap with these substrate naming concerns).

## Verification flow (must run; cite results in SCORE)

```
cargo build --release -p wat                    # must compile clean
cargo test --release --lib -p wat --no-fail-fast   # must show 827 passed (baseline match)
cargo clippy --release --lib -p wat -- -D warnings  # must show 0 substrate warnings (clippy mountain pre-existing is OK; new warnings rejected)
```

Then targeted greps to verify renames are exhaustive:

```
grep -n "type_contains_sender_kind" src/                 # expect 0 hits (rename complete)
grep -n "symbol_ty" src/check.rs                          # expect 0 hits (rename complete)
grep -n "rust::crossbeam_channel::Sender" src/runtime.rs  # expect 0 hits (5 expected-strings + 2 type_name + doc-comment all updated)
grep -n "rust::crossbeam_channel::Receiver" src/runtime.rs # expect 0 hits
grep -nE "QueueSender|QueuePair" src/check.rs              # expect 0 hits (doc-only retired vocab)
```

`src/check.rs` outside the fix scope MAY still mention `rust::crossbeam_channel::Sender` in type-equality contexts — those are correct (alias-resolution heads) and OUT OF SCOPE.

## STOP triggers (REJECTION criteria — not permission-to-defer)

- **STOP-1:** unexpected compile errors after the renames (anything beyond expected ripple)
- **STOP-2:** any test from baseline (827 passing) goes red post-stone
- **STOP-3:** 150 min elapsed (upper-bound runtime)
- **STOP-4:** holon-rs touched accidentally — REJECTION
- **STOP-5:** clippy `-D warnings` on `src/` adds any NEW warning beyond pre-existing mountain
- **STOP-6:** stale-vocab L2s touched (out of scope)
- **STOP-7:** L1-runtime-3 re-done (already absorbed by arc 225)

If any STOP fires: STOP, do not workaround, report in SCORE as honest delta.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator will set explicitly)
- Bash + cargo + Edit work for sonnet — trust the tools naturally
- HARD CUT — no aliases. No `_old_name` / `_legacy_name` deprecation shims. Renames are clean.
- Per `feedback_inscription_immutable`: do NOT edit past SCORE / INSCRIPTION docs; this is forward work on adjacent files
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.

## Cross-references

- `docs/arc/2026/05/224-substrate-naming-honesty-audit/AGGREGATE-FINDINGS.md` — Group A definition (lines 23-35)
- `docs/arc/2026/05/224-substrate-naming-honesty-audit/FINDINGS-INTUERI-RUNTIME.md` — L1-runtime-2 + L1-runtime-3 source
- `docs/arc/2026/05/224-substrate-naming-honesty-audit/FINDINGS-INTUERI-CHECK.md` — L1-check-A + L1-check-B + L1-check-C source
- `docs/arc/2026/05/225-bridge-naming-family/SCORE-STONE-225.1.md` — L1-runtime-3 absorption evidence
- `feedback_no_known_defect_left_unfixed` — out-of-scope language pattern
- `feedback_inscription_immutable` — historical docs immutable; sonnet writes the new SCORE in NEW file
