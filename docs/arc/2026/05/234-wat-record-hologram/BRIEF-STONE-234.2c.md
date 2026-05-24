# BRIEF — Arc 234 Stone 234.2c — runtime class-safety in per-field accessor bodies

**Status:** READY TO SPAWN (2026-05-24).

**Predecessor SCOREs:** `SCORE-STONE-234.2b.md` (the macro this extends), `SCORE-STONE-234.5.md` (the VSA integration sibling).

---

## What to do

Extend the `:wat::Record::def` macro at `wat/Record.wat` so each generated per-field accessor body grows a runtime class-equality guard. Wrong-class receiver → panic with informative message naming BOTH expected class and actual class.

ONE file changes: `wat/Record.wat`. No substrate (Rust) changes. No new probe (committed pre-spawn). No changes to constructor, predicate, or zero-field handling.

## Read these in order

1. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2c.md`** — sub-DESIGN with 10 locked decisions + 8 trap-doors. THE LOAD-BEARING ARTIFACT.

2. **`docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2c.md`** — 11-row scorecard.

3. **`tests/probe_arc234_stone2c_accessor_class_safety.rs`** — the load-bearing test (2/5 PASS initial; goal 5/5 PASS). Probes 2, 3, 4 currently silently return wrong field; your accessor extension makes them panic.

4. **`wat/Record.wat`** — the macro file you extend. Focus on the per-field accessor `:wat::core::let` body (mid-file).

5. **`src/runtime.rs::expect_panic`** (line ~13629) — the panic mechanism behind `Option/expect`; confirms runtime-computed message strings work as the msg arg.

6. **`src/runtime.rs::eval_option_expect`** (line ~13522) — the eval impl for Option/expect.

## Implementation guidance

Per D2 of sub-DESIGN, the accessor body grows from:

```
(:wat::core::defn :myapp::Voltage/magnitude [v <- :wat::Record] -> :wat::core::f64
  (:wat::Record/field-at v 0))
```

to:

```
(:wat::core::defn :myapp::Voltage/magnitude [v <- :wat::Record] -> :wat::core::f64
  (:wat::Record/field-at
    (:wat::core::Option/expect -> :wat::Record
      (:wat::core::if
        (:wat::core::=
          (:wat::core::type v)
          "myapp::Voltage")
        (:wat::core::Some v)
        :wat::core::None)
      (:wat::core::string::concat
        ":myapp::Voltage/magnitude: expected receiver of class :myapp::Voltage, got class :"
        (:wat::core::type v)))
    0))
```

The expand-time substitutions:
- `"myapp::Voltage"` (the class FQDN string) is built at expand time via `~(:wat::core::keyword/to-string fqdn)` — same pattern as 234.2b's predicate body
- The accessor name (`:myapp::Voltage/magnitude`) embeds in the message prefix as a literal string built via `~(:wat::core::string::concat ...)` at expand time
- The positional index `0` (or whatever `fi` resolves to) stays as the field-at second arg

Key syntax reminders:
- `:wat::core::if` requires `-> :T` annotation: `(:wat::core::if cond -> :T then else)`
- `:wat::core::None` is the FQDN variant constructor (per arc 109 slice 1h)
- `:wat::core::Some` takes one arg: `(:wat::core::Some v)`

The runtime concat in the message body (`(:wat::core::string::concat ... (:wat::core::type v))`) evaluates at error-time and produces the full panic message including the actual class FQDN.

## Per-field accessor loop location

The accessor splice lives mid-macro at the `~@(:wat::core::let [...] accessor-defns)` block (per 234.2b shape). Each iteration emits one accessor defn. Modify the emitted defn body shape; keep the iteration loop structure unchanged.

The inner `:wat::core::let` that builds the accessor has bindings: `idx`, `name-h`, `name-s`, `type-h`, `type-w`, `accessor-name`. Add a `class-str` binding (or compute inline) for the class FQDN string used in the equality check + message prefix.

## Discipline reminders

- **`wat/Record.wat` ONLY** — STOP-5 fires on any other file change (no Rust touches; no probe touches)
- **NO modifications to constructor body** — D7 (constructor unchanged)
- **NO modifications to predicate body** — D6 (predicate unchanged)
- **NO zero-field record changes** — D5 (zero-field emits zero accessors; nothing to wrap)
- **NO unchecked-accessor escape hatch** — D9 (HARD CUT — all accessors get the check)
- **NO touching holon-rs** — STOP-4

## What to commit

ONE modified file + ONE new file:
1. `wat/Record.wat` (MODIFIED — macro extension)
2. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2c.md` (NEW — your SCORE)

DO NOT COMMIT. The orchestrator commits after independent verification.

## How you'll be scored

Per `EXPECTATIONS-STONE-234.2c.md`. 11-row scorecard; binding command per row. Mode A target: 11/11 PASS.

LOAD-BEARING row: row 2 — the probe flipping 2/5 PASS → 5/5 PASS.

The probe uses `std::panic::catch_unwind` to catch the wat-side panic + downcasts to `wat::assertion::AssertionPayload` to read the message content. Probes 3 and 4 verify the message contains BOTH expected + actual class names.

Per FM 9: rows are claims; commands are proof.

The SCORE doc captures:
- 11-row scorecard with verbatim command outputs
- Macro line-count delta (added lines)
- Cascade depth (compile rounds + iteration cycles)
- Time breakdown
- Calibration delta (20-40 target; 60 STOP)
- Trap-door audit (T1-T8) outcomes
- Honest deltas if any surface

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2c.md` — sub-DESIGN (load-bearing)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2c.md` — paired EXPECTATIONS + scorecard
- `tests/probe_arc234_stone2c_accessor_class_safety.rs` — the FM 2-bis probe (2/5 PASS verified)
- `wat/Record.wat` — the macro file (target of extension)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — predecessor (the macro this extends)
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_any_defect_catastrophic.md` — the doctrine driving this stone
