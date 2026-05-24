# SCORE — Arc 234 Stone 234.2c — runtime class-safety in per-field accessor bodies

**Status:** SHIPPED (2026-05-24).
**Implementor:** sonnet (claude-sonnet-4-6).
**Mode:** A (inline, one-shot).

---

## 11-row scorecard

| # | Row | Command | Output | Result |
|---|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | `warning: 'wat' (lib) generated 107 warnings ... Finished 'release' profile [optimized] target(s) in 17.70s` | PASS — 0 errors |
| 2 | **Probe FLIPS 5/5 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 3 | Stone 234.2b regression guard | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 4 | Stone 234.5 regression guard | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 5 | Stone 234.2a regression guard | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s` | PASS |
| 6 | Stone 234.1.5 regression guard | `cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` | PASS |
| 7 | Stone 234.1 regression guard | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | PASS |
| 8 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.19s` | PASS — 827 ≥ 827 |
| 9 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `54` | PASS — 54 ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | (empty) | PASS |

**Final result: 11/11 PASS.**

---

## Macro line-count delta

- File before: 213 lines (`wat/Record.wat`)
- File after: 232 lines (`wat/Record.wat`)
- Diff stat: `22 insertions(+), 2 deletions(-)` — net +20 lines
- Per-accessor body grew from 3 lines to ~15 lines (added `msg-prefix` binding + 12-line class-safety guard inside quasiquote)

### Per-field accessor body shape after 234.2c

For `:myapp::Voltage/magnitude` at index 0:

```
(:wat::core::defn :myapp::Voltage/magnitude [v <- :wat::Record] -> :wat::core::f64
  (:wat::Record/field-at
    (:wat::core::Option/expect -> :wat::Record
      (:wat::core::if
        (:wat::core::=
          (:wat::core::type v)
          "myapp::Voltage")
        -> :wat::core::Option<wat::Record>
        (:wat::core::Some v)
        :wat::core::None)
      (:wat::core::string::concat
        ":myapp::Voltage/magnitude: expected receiver of class :myapp::Voltage, got class :"
        (:wat::core::type v)))
    0))
```

The two expand-time substitutions:
- `"myapp::Voltage"` — produced by `~(:wat::core::unquote fqdn-str)` inside the quasiquote
- `":myapp::Voltage/magnitude: expected receiver of class :myapp::Voltage, got class :"` — produced by `~(:wat::core::unquote msg-prefix)` where `msg-prefix` is bound as `(:wat::core::string::concat ":" fqdn-str "/" name-s ": expected receiver of class :" fqdn-str ", got class :")` at expand time

The field index literal `0` (or `fi`) stays as the `field-at` second arg.

---

## Cascade depth

- **Round 1 attempt:** compiled clean; probe ran; 0/5 passed — type annotation `:wat::core::Option<:wat::Record>` failed with "illegal leading ':' on inner argument" (per WAT-CHEATSHEET.md § 2 — no colon inside `<>`).
- **Fix:** changed to `:wat::core::Option<wat::Record>` (bare symbol inside the angle brackets).
- **Round 2:** compiled clean; 5/5 passed immediately.

Total compile rounds: 2. Total iteration cycles: 1 fix.

---

## Trap-door audit

| Trap | Description | Outcome |
|---|---|---|
| T1 | `Option/expect` msg arg accepts runtime expressions | CONFIRMED — `(:wat::core::string::concat msg-prefix (:wat::core::type v))` evals at error-time correctly |
| T2 | `Option/expect` signature `(Option<T>, String) -> T` works for `Option<:wat::Record>` | CONFIRMED — unwraps to `:wat::Record`, then `field-at` fires on matched v |
| T3 | `(:if ... (Some v) :None)` produces `Option<T>` | CONFIRMED — type annotation `-> :wat::core::Option<wat::Record>` makes both branches unify |
| T4 | `:wat::core::None` is the correct FQDN form | CONFIRMED — used as keyword literal, not a function call; correct first-try |
| T5 | Multi-field expansion preserves the pattern | CONFIRMED — probe 4 (`:myapp::Triple/b` on Other) verified each accessor independently checks class |
| T6 | Zero-field records emit zero accessors (no impact) | CONFIRMED — 234.2b probe 6 (`Tag []`) passes in regression guard (6/6 PASS) |
| T7 | Predicate-gated pattern works | CONFIRMED — probe 5 returns -1.0 (fallback) without panic when predicate is false |
| T8 | Panic message format — colon handling | CONFIRMED — `msg-prefix` ends in `", got class :"` and runtime concat appends actual FQDN without leading colon; message reads `:myapp::Voltage/magnitude: expected receiver of class :myapp::Voltage, got class :myapp::Point` |

---

## Honest deltas

**Delta 1 — Type annotation syntax error first try (T3/T4 trap-door).** Used `:wat::core::Option<:wat::Record>` with a leading colon on the inner type argument. WAT-CHEATSHEET.md § 2 doctrine: inside `<>`, type arguments are bare Rust symbols; the colon prefix lives at the outermost position only. Fixed to `:wat::core::Option<wat::Record>`. This is the rule from `feedback_wat_keyword_whitespace.md` — noted in the BRIEF as a known risk. One compile round to surface and fix.

**Delta 2 — `:wat::core::None` FQDN form.** Used correctly first-try as a bare keyword literal (not a function call). Confirmed by grep of existing `.wat` files + check.rs. No trap here in practice.

**Delta 3 — `msg-prefix` computed as a new let binding** (rather than inlined in the quasiquote). This is cleaner than inline and keeps the quasiquote body readable. The BRIEF described it as either "add a `class-str` binding (or compute inline)" — binding approach chosen; meets the discipline of "one outer let* per function" per `feedback_simple_forms_per_func`.

---

## Time breakdown

- Read artifacts: ~5 min
- Understand pattern + plan change: ~5 min
- Write edit: ~3 min
- Compile round 1 + diagnose type annotation error: ~2 min
- Fix + compile round 2: ~2 min
- Run full scorecard: ~3 min
- Write SCORE: ~5 min

**Total: ~25 min. Target band: 20-40 min. PASS.**

---

## Rank-up evidence

234.2b's macro patterns were reused directly:
- `fqdn-str` already bound in outer let — referenced inside inner let body
- `(:wat::core::quasiquote ... (:wat::core::unquote ...) ...)` pattern — extended without breaking
- `(:wat::core::string::concat ...)` at expand time for msg-prefix — same pattern as predicate-name build in 234.2b
- `(:wat::core::keyword/to-string fqdn)` already in scope as `fqdn-str` — no duplication needed

The 234.2b shape reuse was effective. Mechanical work as predicted.

---

## Working-tree state on delivery

```
 M wat/Record.wat
?? docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2c.md
```

Only these two files. No Rust files touched. No probe files touched. No holon-rs files touched.

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.2c.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2c.md` — sub-DESIGN
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2c.md` — scorecard source
- `tests/probe_arc234_stone2c_accessor_class_safety.rs` — FM 2-bis probe (5/5 PASS)
- `wat/Record.wat` — the modified macro file
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — predecessor SCORE
