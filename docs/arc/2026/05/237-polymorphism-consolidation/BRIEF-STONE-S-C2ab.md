# BRIEF — Stone S-C.2ab — field names → RecordDef + re-route name-access off holon_form

**Status:** READY TO SPAWN. `model: "sonnet"`.
**Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (`pwd` first; reject `.claude/worktrees/`; `git -C` if needed).
**Sub-DESIGN:** `DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md` § "DESIGN CORRECTION 2" + its "PARITY
invariant + precise site map" subsection. Read those first — they ARE the design.

## What to do (one coherent, baseline-preserving change)

Make field **names** a class property (`RecordDef`) and re-route the name→index resolution
that currently walks `holon_form` onto it. This is the dependency the base record (S-C.2c)
needs — and combining "add the names" with "consume them" means the existing keyword-access +
assoc tests **exercise** the macro→`field_names` composition (the FM-2-bis proof). Holonic
behaviour is **unchanged** (same answers, new source — parity guarantees struct ≡ holon).

1. **`RecordDef` gains `field_names: Vec<String>`** (`src/types.rs:191`). Field names ARE a
   class property (Ruby: class defines attrs, instance holds values); `struct_form` stays
   positional `Arc<Vec<Value>>`.
2. **`recordtype` becomes 3-arg** `(:wat::core::recordtype :Name :Parent [field-name-strings])`
   — HARD CUT (no 2-arg fallback; a 0-field record passes `[]`). `parse_recordtype`
   (`types.rs:2344`) parses the 3rd arg (a vector of string field names, declaration order)
   into `RecordDef.field_names`.
3. **The `:wat::Record::def` macro emits the names** (`wat/Record.wat`). It already extracts
   field names from the `fields` AST (the holon-walk at ~120-153, for accessors); reuse that
   to emit `(:wat::core::recordtype ~fqdn :wat::Record [~@field-name-strings])`.
4. **Re-route the 3 name→index sites from `holon_form` to `RecordDef.field_names` + positional
   `struct_form`** (look up the name's index in `field_names`, then `struct_form[index]`):
   - `keyword_accessor_record` (`runtime.rs:6440` — the `field_binds = match holon_form` walk).
   - the name-pairing helper (`runtime.rs:16684`).
   - `eval_record_assoc`'s name lookup (`runtime.rs:16825`).
   These need the record's `class_fqdn` → look up the `RecordDef` in the TypeEnv (via `sym`/
   `env` — the same TypeEnv `subtype?`/`conforms?` use) → `field_names`. **Holonic assoc must
   still rebuild BOTH forms** (parity — `runtime.rs:16912` struct + `16917-43` holon stay);
   only the name→index *source* changes, not the rebuild.

## Ripple (HARD-CUT the 2-arg recordtype callers)

The 3-arg change breaks every `recordtype` caller. Update them:
- `wat/Record.wat` macro → emits 3-arg (item 3).
- `tests/probe_arc237_sB1_recordtype.rs` — its `(recordtype …)` calls → 3-arg.
- `tests/probe_arc237_sA1_assignable.rs` — its transitive test calls bare
  `(:wat::core::recordtype :my::Special :my::Circle)` → `(… :my::Circle [])` (0 fields).
- Grep `recordtype` across `tests/` + `wat/` for any other caller — FM-2: don't trust this list.

## Discipline + the error-pivot law

- `src/` + `wat/Record.wat` + the named test files ONLY. No holon-rs (STOP-5). No base variant
  (that's S-C.2c). No macro split (S-C.3).
- **If you hit an error whose message does not make the fix obvious, STOP and surface it
  verbatim — do NOT guess.** A confusing error is a substrate defect we fix, not an obstacle
  to work around (`feedback_nonintuitive_error_is_pivot`). Our diagnostics are teaching-grade;
  trust them, and flag any that aren't.

## STOP triggers (REJECTION)

1. Lib baseline drops below **827/0** for a reason other than a recordtype-arity test update
   you can mechanically fix (the change is baseline-preserving — holonic answers unchanged).
2. A records-thread probe regresses for a reason other than the recordtype 3-arg arity (which
   you update): S-A 10/10, S-B.1 6/6, S-B.2 5/5, S-A1 6/6.
3. keyword-access or assoc gives a DIFFERENT answer than before (it must not — parity).
4. You touch holon-rs, add the base variant, or split the macros.
5. A non-obvious error (→ pivot, per above).
6. 60 min (STOP-3); 90 (STOP-4).

## Regression suite (the FM-2-bis proof — the composition is exercised here)

```
cargo build --release -p wat
cargo test --release --lib -p wat                                    # >= 827, 0 failed
cargo test --release --test probe_arc234_stone3c_keyword_accessor    # keyword-access via NEW source — same answers
cargo test --release --test probe_arc234_stone3b_record_assoc        # assoc via NEW source — same answers + parity
cargo test --release --test probe_arc237_sA1_assignable              # 6/6 (after its recordtype 3-arg update)
cargo test --release --test probe_arc237_sB1_recordtype              # 6/6 (after 3-arg update)
cargo test --release --test probe_arc237_sB2_defrecord_recordtype    # 5/5
cargo test --release --test probe_arc227_stone2_defrecord            # defrecord surface
```
If keyword-access/assoc don't already cover a **multi-field** record (name-order matters),
ADD one case — that's the contract that proves the macro emits names in the right order.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-C2ab.md` (NEW). Mirror
SCORE-STONE-S-C1: scorecard + the RecordDef/recordtype/macro changes + the 3 re-routed sites +
the recordtype-arity caller updates + honest deltas + `git status --short`. DO NOT commit.

## Calibration

Bounded substrate change: RecordDef field + parse_recordtype + macro emission + 3 one-helper
re-routes + ~3 recordtype-caller arity updates. Baseline-preserving (holonic answers identical;
parity). **Target band: 40–70 min Mode A; 90 STOP-3; 120 STOP-4.** Mirror SCORE-STONE-S-C1.
