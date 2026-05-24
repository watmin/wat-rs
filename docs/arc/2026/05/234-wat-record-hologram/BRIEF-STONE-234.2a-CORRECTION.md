# BRIEF — Arc 234 Stone 234.2a — forward-correction (TypeScheme heterogeneous struct_form)

**Status:** READY TO SPAWN (2026-05-24).

**Predecessor SCOREs:** `SCORE-STONE-234.2a.md` (the predecessor stone that authored the inconsistency; stays immutable), `SCORE-STONE-234.2b.md` (the consumer stone that surfaced it; sonnet authored earlier this session).

---

## What to do

Correct the TypeScheme for `:wat::Record::of` in `src/check.rs` so the struct_form parameter accepts heterogeneous Vec values. This is a forward-correction to Stone 234.2a's authoring (the umbrella DESIGN's intent + the runtime's behavior were both heterogeneous; only the type-checker contract was uniform-T).

The 234.2b probe 5 (`tests/probe_arc234_stone2b_defrecord_macro.rs::probe_5_multi_field_accessors_in_order`) is the load-bearing test that flips from FAIL to PASS when the correction ships.

ONE file changes: `src/check.rs`. Nothing else.

## Read these in order

1. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2a-CORRECTION.md`** — the sub-DESIGN with 8 locked decisions + 8 trap-doors + honest framing. THE LOAD-BEARING ARTIFACT.

2. **`docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2a-CORRECTION.md`** — the scorecard.

3. **`tests/probe_arc234_stone2b_defrecord_macro.rs`** — the load-bearing test (probe 5 currently FAIL with `TypeMismatch { callee: ":wat::core::vec", param: "#3", expected: ":i64", got: ":String" }`). Other 5 probes already PASS.

4. **`tests/probe_arc234_stone2a_record_primitives.rs`** — regression guard (6/6 PASS; must stay green).

5. **`src/check.rs` line 10885-10980** — `infer_arithmetic` precedent. This is the pattern to mirror: custom inference handler for a primitive that needs special-case type rules.

6. **`src/check.rs` line 16989-17001** — current `:wat::Record::of` TypeScheme registration (target of correction).

7. **`src/runtime.rs::eval_record_of`** (line ~14543) — runtime confirmation: accepts heterogeneous `Value::Vec` already.

## Implementation guidance

Add a custom inference handler `infer_record_of` to `src/check.rs` modeled after `infer_arithmetic`. The handler:

1. Checks arity = 3 (class keyword + struct_form vec + holon-form HolonAST)
2. Type-checks arg #1 against `:wat::core::keyword`
3. Type-checks arg #2 as a Vec-shaped expression (`:wat::core::vec` head OR Vector literal) WITHOUT enforcing element-type uniformity — each element can be any Value type
4. Type-checks arg #3 against `:wat::holon::HolonAST`
5. Returns `:wat::Record`

Investigate the primary inference dispatcher (likely `infer_list` or similar) to find where `:wat::core::+` routes to `infer_arithmetic` — mirror the same hook for `:wat::Record::of` → `infer_record_of`.

The existing `env.register(":wat::Record::of", TypeScheme {...})` at lines 16993-17001:
- If the dispatcher uses custom-handler-then-TypeScheme order, leave the registration in place as fallback
- If the dispatcher prefers TypeScheme over custom handlers, REMOVE the registration (it would intercept before the handler runs)

Investigate + choose; document the choice in SCORE.

For arg #2 vec-shape recognition: when the arg is a `:wat::core::vec` head call, iterate its sub-args and type-check each independently against any expected type (or against a fresh type var per element). Don't unify across elements.

For arg #2 Vector literal `[a b c]`: this parses to `(:wat::core::vec a b c)` per arc 109 slice 1f, so the head-call path handles it.

## Discipline reminders

- **`src/check.rs` ONLY** — STOP-5 fires on any other Rust change
- **NO modifications to `wat/Record.wat`** — sonnet's earlier 234.2b macro is correct as-shipped
- **NO modifications to `src/runtime.rs`** — `eval_record_of` is already correct
- **NO modifications to the 234.2a probe or the 234.2b probe** — both probes' contracts stay as-authored
- **NO modifications to SCORE-STONE-234.2a.md** — INSCRIPTION-immutable per `feedback_inscription_immutable`
- **NO touching holon-rs** — STOP-4
- **No HARD CUT discipline considerations** — pure substrate correction; no user-facing surface change

## What to commit

ONE new file + ONE modified file:
1. `src/check.rs` (MODIFIED — handler added + dispatch hook + optionally TypeScheme registration removed)
2. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` (NEW — your SCORE)

DO NOT COMMIT. Working tree should stay dirty with:
- Stone 234.2b sonnet's earlier work (`wat/Record.wat` NEW; `src/stdlib.rs` MODIFIED; `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` NEW)
- Your work (above)

Orchestrator atomic-commits both stones together when verification passes. The atomic commit message will name BOTH shipments + cite the honest framing.

## How you'll be scored

Per `EXPECTATIONS-STONE-234.2a-CORRECTION.md`. 11-row scorecard; binding command per row. Mode A target: 11/11 PASS.

The LOAD-BEARING row is row 2 — Stone 234.2b probe flipping from 5/6 to 6/6 PASS. The SUBSIDIARY load-bearing row is row 3 — Stone 234.2a probe staying at 6/6 PASS (regression check).

The orchestrator independently verifies LOAD-BEARING rows on return. Per FM 9.

The SCORE doc captures:
- 11-row scorecard with verbatim command outputs
- The dispatch-hook investigation finding (where does the dispatcher route primitives to custom handlers)
- Implementation surface (check.rs line counts; whether the existing TypeScheme registration stayed or was removed)
- Cascade depth (compile rounds + iterations)
- Time breakdown
- Trap-door audit (T1-T8) outcomes
- Honest deltas if any surface
- Rank-up evidence — predecessor pattern (`infer_arithmetic`) effectiveness

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2a-CORRECTION.md` — sub-DESIGN (load-bearing)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2a-CORRECTION.md` — paired EXPECTATIONS + scorecard
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — the load-bearing test (probe 5 flips)
- `tests/probe_arc234_stone2a_record_primitives.rs` — regression guard (stays green)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — sibling SCORE in atomic commit
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_inscription_immutable.md` — predecessor SCORE stays unchanged
