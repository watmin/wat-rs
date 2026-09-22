# BRIEF — STONE 255.11: the wall audit

**Drawn 2026-09-22 against `main` @ `854c2b984`.** Floor 5963/5963, clippy 0, census `no STOP-8`.
Delta baseline **4**, on the committed list
`docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`. ⛔ **USE THAT FILE.**

## ⭐⭐ THE RULE THIS STONE EXISTS TO ENFORCE

> ⛔ **A DEFAULT-DENY GATE THAT DISPATCHES ON A SPELLING IS DEFAULT-ALLOW FOR EVERY OTHER SPELLING.**
> A wall whose **unrecognised branch** is `_ => recurse`, `_ => Ok(None)` or `_ => true` **FAILS OPEN.**

**This is not a theory.** 255.10 found the **F5 expand-time purity gate** (arc 249, default-deny)
bypassed by the symbol spelling. Measured on two binaries built from one tree:

```
PRE-CURE   (:wat::kernel::println …) in a macro body → rc=1  "default-deny F5 gate, arc 249"
PRE-CURE   (wat.kernel/println   …) in a macro body → rc=0  ⛔⛔ CLEAN — the macro was DEFINED
POST-CURE  both spellings                            → rc=1, BYTE-IDENTICAL refusal
```

⛔ **It was LIVE, not latent** — an ordinary keyword-spelled file with ONE symbol-spelled call
hand-written into a macro body. **8d-iii makes the bypass the DEFAULT spelling.** ⭐ **That is why
this audit precedes the flip.**

## ⭐ The unit is a WALL, not a read-site

⛔ **THIS IS NOT A SWEEP.** 255.10 measured **665** keyword read-sites and the brief that counted
them was told so twice. **A wall is a bounded thing and there are a few dozen.** Seeds, measured here:

**~30 gate-shaped functions** (`fn refuse_* | validate_* | reject_* | scan_for_* | forbid_* | deny`),
of which **7 of 8 sampled dispatch on `WatAST::Keyword`**:
`refuse_mutation_forms_in` · `refuse_mutation_forms` · `scan_for_setter` · `reject_setters_in_loaded`
· `validate_pure_total` · `validate_macro_definition` · `validate_def_positions_in_forms` ·
`refuse_core_structural_on_multi` · `refuse_expand_only_in_program` · `refuse_export_without_arm` ·
`refuse_non_terminating` · `validate_aggregate_containment` · `validate_quasiquote_template` ·
`validate_holon_record_capacity` · `validate_named_type_annotations` · the `validate_rete_*` family.

**4 named refusal variants:** `DefRedefForbidden` · `DefRestrictedCallerNotAllowed` ·
`EvalForbidsMutationForm` · `SetterInLoadedFile`. ⚠ **The F5 gate has NO dedicated variant** — it
reports through `MalformedDefmacro`. **Variants undercount. Do not enumerate by variant alone.**

⚠ **THE SEED LIST IS MINE AND IT IS A LEAD, NOT A CENSUS.** Correct it.

## ⛔⛔ A `WatAST::Keyword` MATCH IS NOT THE DEFECT

`scan_for_setter` was **CURED in 255.9 and still matches `Keyword`.** ⛔ **Counting Keyword arms will
give you false positives and a false sense of coverage.** The question is **only** this:

> **When this wall does not recognise the form, does it REFUSE or does it PASS?**

## The method — per wall, three answers and a probe

1. **What does it refuse, and what is its unrecognised branch?** → **FAIL-OPEN** or **FAIL-CLOSED**.
2. ⭐ **PROBE IT AT ITS OWN LAYER.** ⛔ **A runtime wall cannot be probed with `--check`** — the
   orchestrator's own mutation-wall probe died at type-check and reached nothing, which is why this
   brief claims no measurement there. An eval-time wall needs the program **RUN**. 255.10 disclosed
   **154 of 156** clean converted files were **never run**.
3. ⛔ **POSITIVE CONTROL, MANDATORY.** A wall that refuses everything is **not** intact. Every row
   needs a legitimate case that **still passes**, in both spellings.

## Cure or report — the line

- ⭐ **CURE a FAIL-OPEN wall** through the identity door. Closing a bypass makes the system **more
  restrictive** — the safe direction — but ⛔ **it can break working code, so every cure carries the
  floor and the delta as its positive control at scale.**
- ⛔ **REPORT, DO NOT CURE, anything that would make a wall MORE PERMISSIVE.** That is the
  red→false-green direction and it is **the builder's call**, as 255.9 said of the mutation walls.
- A wall you cannot probe is **REPORTED AS UNPROBED**, never assumed either way.

## Known state, inherited — do not re-derive

| | |
|---|---|
| ✅ `:restricted-to` | **INTACT, 4 probes incl. a positive control** — symbol call head, bare-symbol entry, fully-converted fixture all DENIED; permitted caller still ALLOWED. `restricted_to_entries` takes `Keyword` **and** `Symbol` deliberately. ⛔ **Do not re-litigate.** |
| ✅ F5 purity gate | **CURED 255.10**, byte-identical refusal both spellings |
| ✅ load head / setter head | **CURED 255.9** |
| ⚠ the two mutation walls | `freeze.rs:1894`, `runtime.rs:14442` — **keyword-only BY SHAPE**, 255.9 measured **BYPASSED-but-BACKSTOPPED** (*the refusal survives, the DIAGNOSTIC does not*). ⭐ **START HERE.** |
| ❓ unprobed | Rust-side `#[restricted_to(…)]` (`io.rs`, `rete/kernel/arm.rs`) · field/ctor restrictions (`declare/register.rs:901/908`, `types/defstruct.rs:103/236`) · **runtime vs check-time everywhere** |

## The gate

- ⭐ **THE AUDIT TABLE IS THE DELIVERABLE** — every wall, its unrecognised branch, FAIL-OPEN or
  FAIL-CLOSED, how it was probed, and the positive control. ⛔ **A wall listed without a probe must
  say so in its own row.**
- Each cure names the probe that **fails without it**.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` 0; census `no STOP-8`.
- Delta **≤ 4**, ⭐ **and REPORT THE RECOVERY (fail→clean) COLUMN** — ⛔ **non-zero is a STOP until
  explained.** (Recommended as a standing gate twice now and still not institutionalised; if you
  institutionalise it, say so.)
- ⛔ **NOT ONE `.wat` CONVERTED** in the live tree.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole, never re-run to green, name the arm.
- ⛔ **REPORT WHAT A DOOR CANNOT EXPRESS.** Twelve stones running.
- ⛔ **If this brief contradicts the code, THE CODE WINS.** ⭐ **Eighteen corrections across sixteen
  stones**; 255.10 corrected the orchestrator three times AND corrected 255.9's census. **The seed
  list above is the most likely thing here to be wrong. Assume a nineteenth.**
- ⚠ **Reformat in its own commit, or not at all.**
- Work only in `/home/john/work/holon/wat-rs`. **Do not push.**

## Out of scope

The `wat.type` denotation stone (`register_validated` + the two runtime tables) · the 416-test rete
remainder · the shape-B/D read-site sweep · `:wat::keyword::canonical-identity` · 255.8's wrong-join
hole · 8d-ii · 8d-iii.
