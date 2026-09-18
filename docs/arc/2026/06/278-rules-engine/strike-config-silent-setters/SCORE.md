# SCORE — Ω4 silent config setters

Cure + gates landed. **STOP-1 fired. Floor RED. Did not re-run.**

## Scorecard

| # | result |
|---|---|
| 1 ★ Ω4a cured | **HOLD.** Typo `setmax-fire-rounds!` → `UnknownSetter`, span on the typo, CLI exit 3. Never 10000. |
| 2 ★ Ω4b cured | **HOLD.** Valid setter after a body form → `SetterAfterNonSetter`, span on the setter, CLI exit 3. |
| 3 ★ variant REACHABLE | **HOLD.** `SetterAfterNonSetter` is constructed from `collect_entry_file_inner` remainder scan; the Ω4b gate drives it. |
| 4 ★ accessors still legal | **HOLD.** Control prints `4096`, rc=0; `(:wat::config::dim-count)` in `:user::main` / `:user::dim` still works. |
| 5 one name grammar | **HOLD.** Remainder scan uses `wat_reader::identifier::leaf`. No `rsplit("::")`. `one_name_grammar` was not among the 9 failed. |
| 6 floor | **FAIL. STOP-1.** `Summary [ 450.861s] 5436 tests run: 5427 passed (1 slow), 9 failed, 21 skipped`. Captured `.floor/2026-09-05T11-23-05Z/`. **Did not re-run.** |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0, taken before the floor. |
| 8 blast radius | **HOLD on src.** `src/config.rs` only under `src/`. Zero lines in `resolve/` or `check.rs`. Gates in `tests/program/probe_arc278_config_silent_setters*`. The red is one existing fixture, named below. |

## What shipped

After the setter section `break`s, the remainder is scanned for `:wat::config::…!`. `ends_with('!')` is the discriminator. A `set-` leaf is `SetterAfterNonSetter`; any other bang leaf is `UnknownSetter`. Both carry the form's span. Accessors are untouched. The in-loop `SetterAfterNonSetter` guard (dead: assign-then-break) was removed; the post-loop scan is the one construction site.

Ω4a uses `UnknownSetter` (the typo leaf is `setmax-fire-rounds!`, not `set-*`). Ω4b uses `SetterAfterNonSetter`.

## STOP-1 — the red

**File:** `tests/wat_lang/wat_arc157_def.wat:25`
**Form:** `(:wat::config::set-eval-redef! true)` after top-level `def` / `let` / `if` / `defn`.
**Comment in situ:** *"Test 19: set-eval-redef! recognized at top-level (no error)"* — that "no error" was Ω4b.

**Exact arm (all nine):** `wat_arc157_def.rs:35` `expected startup success for tests/wat_lang/wat_arc157_def.wat` and `freeze.rs:1165` `call_beside_value: fixture … failed to freeze`, both with:

```
#wat.config/SetterAfterNonSetter {:message "config setter follows non-setter; entry-file discipline requires all :wat::config::set-eval-redef! setters before any load! or program body" :location #wat.core/Span {:file "tests/wat_lang/wat_arc157_def.wat" :line 25 :col 1 …} :setter-head ":wat::config::set-eval-redef!"}
```

**The nine:** `def_basic_float_literal`, `def_position_illegal_inside_define_body`, `def_position_illegal_inside_if`, `def_position_legal_direct_top_level`, `def_position_legal_let_splice_with_closure`, `def_runtime_let_splice_closure_capture`, `def_runtime_pi_in_let_addition`, `def_runtime_pi_resolves_to_value`, `def_set_eval_redef_form_recognized`.

wat-scripts / docs load-gates were green. No `wat/` stdlib form tripped this. This is the one existing entry file on the floor that carried a `:wat::config::…!` in the remainder. Not patched. BRIEF: a finding, not a nuisance.

## Not landed (named, cut)

`cernere` C1 (close the `:wat::` vocabulary). `RequiredFieldMissing` still declared-and-never-constructed; empty entry files still commit defaults.
