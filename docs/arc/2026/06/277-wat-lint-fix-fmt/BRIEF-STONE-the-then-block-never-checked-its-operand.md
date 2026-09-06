# BRIEF — STONE: the `:then` block never checked its operand

Make a rete `:then` check each operand against the **declared type of the field it lands in**, and
make `RhsUnresolvableOperand`'s message stop under-reporting what a RHS accepts. Read
`[[DESIGN-STONE-the-then-block-never-checked-its-operand]]` first — it carries both probes, the
segment trap that makes the obvious fix wrong, and the one scream known in advance.

**This stone is RUST.** The two enum stones before it were wat; this is the substrate underneath them.

## READ IN ORDER

1. **`src/rete/validate.rs:1132-1168`** — `rhs_operand_can_never_resolve` + `check_rhs_operands`.
   **This is the site.** The check is purely syntactic: it asks whether an operand can ever resolve,
   never whether it fits.
2. **`src/rete/validate.rs:1051-1086`** — `resolve_operand_type`. **Read it to see why it is NOT the
   answer**: it returns a rete SEGMENT (`"i64"` / `"string"` / `"enum"`), and two different enums
   both segment to `"enum"`.
3. **`src/rete/validate.rs:829`** — `lookup_field_types`. The destination's declared types. It exists.
4. **`src/rete/validate.rs:984`** — `collect_rule_bind_types`. The rule-wide `?var → type` map,
   already built for `:when` at `:524` and `:548`.
5. **`src/rete/validate.rs:1404` and `:1416`** — the two `check_rhs_operands` call sites, kwargs and
   positional. `field_names`, `kv_pairs` and `types` are already in scope; `binds` is not.
6. **`src/rete/validate.rs:215-222`** — the `RhsUnresolvableOperand` Display arm and its `accepted`
   vector. Defect two.
7. **`tests/cli/wat_grep__g7_rule.wat:26`** — the known scream.

## SKETCH

```rust
// check_rhs_operands gains the destination's field types and the rule's binds, and asks
// the question it never asked. DECLARED types, not segments — two enums both segment to
// "enum", and the hole admits exactly that today.
//
//   for (field, operand) in ...:
//       let declared = field_types[field];
//       let actual   = <the operand's declared type: literal, or binds[?var]>;
//       if actual != declared  ->  a NEW ReteCheckErrorKind naming
//                                  rule, fact_type, field, declared, actual
//
// and the message that taught the String:
//   accepted: vec![
//       "a ?var bound by this rule's :when",
//       "an integer / float / boolean / string literal",
//       "a call form — a constructor, or a fenced :wat::rete:: expression",   // NEW
//   ]
```

## BLAST RADIUS

```
src/rete/validate.rs    the check, a new error kind, the accepted list
tests/rete/…            the negative controls — both defects, both directions
<what the screams name>  UNKNOWN by construction: 319 :then sites exist
```

## STOP TRIGGERS

- **STOP-1 — REPORT EVERY SCREAM VERBATIM before fixing any of them.** File, rule, field, declared
  type, actual type. 319 `:then` sites exist and nobody knows how many lie. **The list is this
  stone's first deliverable**; a summary or a count is not the list.
- **STOP-2 — if the check rejects a rule that was LEGAL, the check is wrong, not the rule.** Stop and
  report the shape it mis-rejects. A wave of false rejections is a failure, not progress.
- **STOP-3 — do NOT compare rete SEGMENTS.** `resolve_operand_type` is right there and returns
  `"enum"` for every enum. The Alpha/Beta probe in the DESIGN passes a segment check and is exactly
  the case that surfaced this bug.
- **STOP-4 — do NOT relax the `then-item-fence`.** That a `:then` admits only `:wat::rete::` ops is a
  separate ruling. A user conversion fn in a `:then` is REFUSED and that was measured before this
  brief: `':user::k-name' is not a rete primitive`.
- **STOP-5 — do NOT mint a `:wat::rete::core::enum::name` vocabulary row.** It is the general answer
  to g7's conversion and it is OUT OF SCOPE: minting an op for one caller is speculative until
  STOP-1's list says how many callers exist. **If g7 cannot be resolved without it, STOP and report
  that** — do not build it quietly.
- **STOP-6 — the negative controls go in `tests/`, not `wat-scripts/scratch-pad/`, and the reason is
  structural rather than stylistic.** These probes are rules the fix must REJECT, so the moment it
  lands they stop type-checking — and `every_wat_scripts_file_loads` parses every `.wat` under
  `wat-scripts/`. **Putting them there turns the loader gate red on success.** They belong where a
  refusal is the expected outcome, and a probe that is not in the floor cannot fail when someone
  removes the check.
- **STOP-7 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## ⚠ TRAPS

- **The known scream is the PREVIOUS stone's own defect.** `g7_rule.wat:26` stores a `NodeKind` into
  `Capture.value <- :wat::core::String`. It was reported honestly as a delta and nothing could catch
  it. **If your check does not flag that site, it is measuring something else.**
- **g7's obvious resolution is refused** (STOP-4) and **no rete op renders an enum as a string** —
  the vocabulary holds `core::enum::=` and `core::enum::not=`, nothing more. The narrow resolution
  the DESIGN chooses is to capture the literal `"symbol"`: `g7::match-arrow` requires `IsArrow`,
  which `g7::arrow` asserts **only** for `NodeKind::Symbol`, so the literal is the truth rather than
  new hard-coded knowledge.
- **Batch, do not bail.** `validate_rete_rules` returns every finding; a rule with two bad operands
  must report both.
- **`wat/*.wat` is FROZEN into the release binary** — `cargo build --release` between an edit and any
  measurement.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — `cargo build`, the two probes, `-E 'test(rete)'`, `wat_grep` — and report the
numbers, and above all report STOP-1's list.
