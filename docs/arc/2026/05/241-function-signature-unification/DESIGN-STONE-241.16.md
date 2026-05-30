# DESIGN — Stone 241.16 — `:wat::core::define` EVAL-TIME RESIDUE COMPLETION (Enemy 3 of 4)

**Status:** STRIKE-READY (2026-05-29 very late). Enemy 3 in the define-family death campaign. Completes Stone 241.11's partial HARD CUT (startup-check rejects; eval-time scaffolding survived). **LAST scheme-style retirement before broader clojure conversion arcs (172-181).** After this: Stone 241.17 INSCRIPTION closes arc 241.

## User direction (load-bearing)

User direction 2026-05-29 very late: *"our scheme conversions are nearly done - our clojure form await - take it."*

The "scheme → clojure" conversion theme: `define` (Scheme name) retires in favor of `defn` (Clojure name). The def-prefix family choice itself is clojure-aligned. Stone 241.16 completes this conversion by killing the eval-time `define` scaffolding that Stone 241.11 left behind for defense-in-depth.

After arc 241 closes (Stone 241.17), the broader clojure conversion continues with arcs 172 (comma-to-apostrophe), 173/174 (clojure macros), 175/176/177 (enum/struct/defmacro syntax clojure), 181 (match syntax clojure).

## What Stone 241.11 left behind

Stone 241.11 HARD-CUT `:wat::core::define` at startup-check (the `check.rs` rejection arm). But eval-time scaffolding survived deliberately:

> *"Stone 241.11 — `:wat::core::define` is HARD CUT at startup (check time). At eval time (eval_in_frozen), `:wat::core::define` is still a mutation"* (`src/freeze.rs:1638-1639`)

The defense-in-depth rationale: even if someone constructs an AST programmatically (bypassing parser/startup-check) with `:wat::core::define` head, the eval-time mechanism would refuse it. This required `:wat::core::define` to remain in `is_mutation_head` etc.

**Per `feedback_hard_cut_admits_no_bypasses`: HARD CUT is TOTAL.** Defense-in-depth via keeping retired form recognized is the same shape as the zombies Stone 241.15 just buried. Stone 241.16 completes the segregation.

## Substrate residue surface

| Location | What | Action |
|---|---|---|
| `src/runtime.rs:2588-2609` | `register_defines` walker function (misleadingly named; doesn't process define forms post-Stone-241.11) | RENAME to honest (e.g., `register_top_level_defs`) OR keep name and remove stale comment |
| `src/runtime.rs:3547-3551` | `is_define_form` predicate function + caller at 3551 | DELETED |
| `src/runtime.rs:4399+` | `parse_define_form` function + ~30 error-construction sites | DELETED entirely |
| `src/runtime.rs:27427` | `is_mutation_head` function — `:wat::core::define` arm | DELETED (arm removed) |
| `src/freeze.rs:1312, 1324` | `is_mutation_form` function — `:wat::core::define` arm | DELETED (arm removed) |
| `src/freeze.rs:1355, 1361` | `is_declaration_form` function — `:wat::core::define` arm | DELETED (arm removed) |
| `src/check.rs:2884` | sandbox-scope-check looking for `:wat::core::define` head in inner forms | DELETED (branch unreachable post-startup-check) |
| `src/check.rs:3141-3142` | check arm `":wat::core::define" => {` (pre-Stone-241.11 path?) | DELETED |
| `src/check.rs:7049` | another `":wat::core::define" => {` arm (likely Stone 241.11's HARD-CUT arm — KEEP) | INVESTIGATE; if it's the HARD-CUT arm, KEEP; else DELETE |
| `src/check.rs:2414` | error format string `":wat::core::define (body)"` | MIGRATE to `:wat::core::defn` reference |
| `src/check.rs:913`, `src/runtime.rs:2513` | sandbox-scope error messages mentioning `(:wat::core::define ...)` | MIGRATE to `:wat::core::defn` |
| `src/special_forms.rs:175` | registry entry | DELETED |
| `src/special_forms.rs:331` | reference in `registry_covers_audited_forms` test | DELETED (removed from list) |
| Comments at runtime.rs:23, 1413, 1428, 2101 | function/module documentation mentioning define | UPDATE to defn references where current-tense; preserve historical "Stone 241.11 retired" comments |

## Test fixture migration

**Bypass-rejection test fixtures (`src/freeze.rs:1651, 1807, 1985`):**

These tests construct AST programmatically with `:wat::core::define` head to verify "even if startup is bypassed, the runtime refuses." Post-Stone-241.16: `:wat::core::define` is no longer in `is_mutation_head`, so these tests would PASS the bypass without rejection.

**Migration: rewrite tests to use a different known mutation head** (e.g., `:wat::core::defmacro` or `:wat::core::defstruct`). The MECHANISM under test (eval-time refuses mutation forms not declared at startup) preserves; the specific head used is incidental.

Alternative: DELETE these tests entirely if the underlying defense-in-depth mechanism is also being retired (sonnet's judgment).

**`tests/wat_arc144_special_forms.rs:210-211`:**

```rust
assert_special_form(":wat::core::define", ":wat::core::define");
let (_, sig, _) = three_probes(":wat::core::define");
```

These assert `:wat::core::define` IS a special form (per registry). Post-stone: NOT a special form. MIGRATE to assert HARD CUT OR DELETE.

**`tests/wat_arc144_uniform_reflection.rs:103, 121`:**

```rust
// carries the `:wat::core::define` head keyword (the load-bearing claim
...
line.contains(":wat::core::define"),
```

STALE post-Stone-241.11. Reflection now emits `:wat::core::defn`. UPDATE the assertion.

**`tests/probe_let_splice_define.rs`, `tests/probe_do_splice_define.rs`:**

File names suggest these are old (pre-Stone-241.11). MIGRATE or DELETE per per-file judgment.

**Comment-only references** (`tests/wat_arc167_vector_ast.rs:20`, `tests/wat_arc220_char.rs:266`, `tests/probe_closure_body_prelude_lift.rs:109/242/244`, `tests/probe_declaration_form_lift.rs:121/131/267`):

Historical references in comments per `feedback_inscription_immutable` — KEEP.

## What this stone delivers

### S1 — DELETE `parse_define_form` entirely

`src/runtime.rs:4399+` — full function deletion. ~30 error-construction sites die with it.

### S2 — DELETE `is_define_form` + caller

`src/runtime.rs:3547-3551` — function + the one caller at line 3551.

### S3 — DELETE `:wat::core::define` arms from form predicates

- `freeze.rs:1312` `is_mutation_form` — remove define arm
- `freeze.rs:1355` `is_declaration_form` — remove define arm
- `runtime.rs:27427` `is_mutation_head` — remove define arm

### S4 — DELETE check.rs processing arms

- `check.rs:2884` sandbox-scope inner-form scan (the branch is unreachable post-startup-check)
- `check.rs:3141-3142` `":wat::core::define" =>` arm (if not the Stone 241.11 HARD-CUT arm)

### S5 — DELETE special_forms.rs entries

- Line 175: registry entry
- Line 331: spot-check test reference

### S6 — Migrate error message strings

- `check.rs:913` sandbox-scope error hint: `(:wat::core::define {} ...)` → `(:wat::core::defn {} ...)`
- `runtime.rs:2513` same migration
- `check.rs:2414` error format string: `":wat::core::define (body)"` → `":wat::core::defn"` (or similar)

### S7 — Migrate test fixtures

- `freeze.rs:1651, 1807, 1985` bypass-rejection tests: rewrite to use another known mutation head (e.g., `:wat::core::defstruct`) OR delete if defense-in-depth mechanism retiring
- `tests/wat_arc144_special_forms.rs:210-211`: migrate to assert HARD CUT OR delete
- `tests/wat_arc144_uniform_reflection.rs:103, 121`: update assertion to reflect defn-emission
- `tests/probe_let_splice_define.rs`, `tests/probe_do_splice_define.rs`: per-file judgment

### S8 — Reflection emitter audit

Per Stone 241.12/13/14/15 trap-door precedent:
```
grep -n "Keyword.*::define\b" src/
```

Any AST-construction emitting `:wat::core::define` keyword migrates or dies.

### S9 — Probe verification

`tests/probe_arc241_stone16_define_eval_residue.rs` (NEW). FM 2-bis disconfirming.

### S10 — SCORE doc

Per `feedback_score_present_check_before_closure`. `SCORE-STONE-241.16.md` at strike-end.

## Locked decisions

### D1 — HARD CUT TOTAL per `feedback_hard_cut_admits_no_bypasses`

No "defense-in-depth via keeping define in is_mutation_head" framing. Stone 241.11 left this residue deliberately; Stone 241.16 completes the cut. The bypass-rejection mechanism preserves (refuses unknown mutation forms); the specific head `:wat::core::define` is no longer in the recognized set.

### D2 — `parse_define_form` DELETED entirely

Not "tombstone." DELETED. ~30 sites of error-construction die with it. Per Stone 241.13 (`src/dispatch.rs` 445-line file deletion) precedent.

### D3 — Bypass-rejection tests migrate, not preserved-with-define

`freeze.rs:1651/1807/1985` tests migrate to use another known mutation head (recommend `:wat::core::defstruct`). The MECHANISM preserved; the specific-head fixture changes.

### D4 — `register_defines` function name preserved OR renamed (sonnet judgment)

The name `register_defines` is stale (doesn't process define forms post-Stone-241.11; processes defalias + def). Sonnet judges whether to rename to `register_top_level_defs` (honest) or preserve for caller cascade avoidance. Either acceptable.

### D5 — Vigilia NOT required (no namespaced home)

### D6 — INTERSTITIAL orchestrator-exclusive

### D7 — SCORE-write at end

### D8 — Stone 241.17 scope is INSCRIPTION only

Sonnet does NOT touch INSCRIPTION work; that's Stone 241.17 orchestrator-direct paperwork. Sonnet ships substrate work + SCORE only.

## Trap-door audit

### T1 — `parse_define_form` deletion cascades through ~30 error-construction sites

Many sites construct `RuntimeError::MalformedForm { head: ":wat::core::define".into(), ... }` style errors INSIDE parse_define_form. When the function dies, the errors die with it. But if any caller PATTERN MATCHES on these specific errors, those callers break.

Resolution: grep for `MalformedForm.*":wat::core::define"` callers; verify none exist outside parse_define_form's body.

### T2 — `register_defines` name preservation

Renaming `register_defines` → `register_top_level_defs` has caller-cascade cost. ~5-10 call sites likely. Sonnet judges whether the honesty improvement is worth the cascade. Recommend renaming only if cascade is < 10 sites.

### T3 — `check.rs:7049` arm context unclear

Need to verify whether this is the Stone 241.11 HARD-CUT-rejection arm (KEEP) or pre-Stone-241.11 processing arm (DELETE). The earlier grep showed:
```
src/check.rs:7049:            ":wat::core::define" => {
```
Sonnet reads surrounding context to judge.

### T4 — Test fixture migration trap-door

`freeze.rs:1651/1807/1985` tests test bypass-rejection mechanism. If sonnet migrates the head to `:wat::core::defstruct` but defstruct is REGISTERED at startup (no bypass to test), the test breaks. Need a mutation head that is REGISTERED but the specific instance hasn't been (the bypass scenario).

Resolution: re-read the test bodies to understand the bypass mechanism precisely; pick a head that fits.

### T5 — Reflection emitter trap-door (Stone 241.12/13/14/15 class)

Per the precedent — if any AST-construction site emits `:wat::core::define` keyword, the prologue re-freeze or runtime walker breaks. Audit.

### T6 — Sonnet "defense-in-depth preservation" temptation

Per D1 + `feedback_hard_cut_admits_no_bypasses`. STOP if surfaces. The HARD CUT is total.

## STOP triggers — REJECTION

1. Compile errors not traced to define eval-time deletion cascade
2. Lib < 890 (post-Stone-241.15 baseline) — note: test migrations may shift count; document
3. **180 min elapsed** (this stone is BIG — comparable to Stone 241.13's 445-line deletion scope)
4. holon-rs touched (STOP-5)
5. `:wat::core::define` use classified as "defense-in-depth preservation" without deletion → D1 + `feedback_hard_cut_admits_no_bypasses` violation
6. `parse_define_form` PRESERVED (D2 violation — DELETED is the action)
7. Files outside permitted scope (`src/runtime.rs` / `src/check.rs` / `src/freeze.rs` / `src/special_forms.rs` / `src/closure_extract.rs` if reflection emitters touched / test files in S7 inventory / `tests/probe_arc241_stone16_*` / SCORE doc)
8. Stone 241.16 probe < N/N
9. Stone 241.x or arc 237/238/242 probes regress (except test files in S7 which may migrate)
10. Clippy > 935 (looser gate; substrate refactor; arc 109 sweeps to zero)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → D6 + `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.16.md NOT authored at end → D7 + `feedback_score_present_check_before_closure` violation
14. Stone 241.17 scope touched (INSCRIPTION paperwork) → D8 violation

## FM 2-bis evidence

`tests/probe_arc241_stone16_define_eval_residue.rs` (NEW; written + verified disconfirms at HEAD before BRIEF spawns).

## Calibration

**Target band: 90-180 min Mode A.**

Stone 241.16 scope decomposition:
- `parse_define_form` deletion + cascade (~30 error-construction sites) — **~30-45 min**
- Form-predicate arm deletions (`is_mutation_form` + `is_declaration_form` + `is_mutation_head`) — **~10 min**
- check.rs processing arm deletions — **~10-15 min**
- special_forms.rs entry deletions (2) — **~5 min**
- Error message string migration (3 sites) — **~5-10 min**
- Test fixture migration (3 bypass-rejection tests + wat_arc144 reflection + special_forms test) — **~20-30 min**
- Per-file judgment on probe_let_splice_define + probe_do_splice_define — **~10-15 min**
- Reflection emitter audit — **~5 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Within-band: 90-180 min. Recent stones (241.13/14/15) all substantially under-band; this one is BIGGER (parse_define_form is comparable to src/dispatch.rs deletion scope) so closer to predicted band.

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.15.md (zombie purge pattern; bulk-sed-friendly cascade), SCORE-STONE-241.13.md (substrate scaffolding deletion of comparable scope; 445-line file deletion + plumbing cascade), SCORE-STONE-241.11.md (the original define HARD CUT this completes).

## What this unblocks

**Stone 241.17 — INSCRIPTION closes arc 241.** Explicit acknowledgment of:
- Stone 241.6 → 241.10 orphaned commitment closed by Stone 241.14 (25 days late)
- `feedback_defer_by_naming` doctrine memory inscribed
- The def-family death campaign complete (5 stones: 241.12 alias + 241.13 dispatch + 241.14 def-restricted + 241.15 zombie purge + 241.16 define residue)
- 12-entry RETIREMENT_TABLE as historical record
- Scheme → Clojure conversion at the define layer complete

**Arc 237.8b** reopens after Stone 241.17 per `feedback_no_regression_until_arc_done`.

**Broader Clojure conversion arcs** queued post-arc-241:
- Arc 172 — comma-to-apostrophe-dispatch (Clojure quoting style)
- Arcs 173/174 — clojure macros + features
- Arcs 175/176/177 — enum/struct/defmacro syntax Clojure
- Arc 181 — match syntax Clojure

**The substrate's identity** clarifies: wat-rs is a typed Lisp on Rust with Clojure-aligned conventions (defn not define; PascalCase containers; lowercase scalars; metadata-map binding annotations; one-canonical-path-per-task). Scheme-style legacy is buried in the graveyard (RETIREMENT_TABLE). The substrate is what it WANTED to be from arc 241's start: Clojure-flavored homoiconic Lisp on Rust.
