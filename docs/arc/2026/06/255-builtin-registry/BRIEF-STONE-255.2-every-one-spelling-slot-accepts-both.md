# BRIEF — STONE 255.2: every ONE-SPELLING slot accepts BOTH

**Drawn 2026-09-21 against `main` @ `78e564015`** (floor 5936/5936, clippy 0, census `no STOP-8`).
Predecessor: **255.1 LANDED** — identity is the pair, `wat.type` is real, delta **104 → 97**.
⛔ **251.8d is still blocked. This stone is what unblocks it.**

## ⭐ THE RULE — one sentence, and it is DERIVED, not enumerated

> **Any slot that accepts exactly one spelling must accept both.**

⛔ **The previous brief ENUMERATED declaration forms** — `defenum`, `typealias`, `defclause`,
`defrecord`, `defstruct` — **and missed `defsurface` (154 files) and `newtype`.** That was the
**eighth** correction to an orchestrator brief, and it is the defect the brief itself warned about:
a hand-list where a derived set belongs (`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).

⛔ **DO NOT EXTEND THE LIST BY TWO. DERIVE THE SETS.** Two derivable sets are already visible:

| set | how to derive it |
|---|---|
| declaration-name slots | **every caller of `parse_declared_name`** (`src/types.rs:4188`, `:4388` `"newtype"`, `src/types/surface.rs:570`, …). A form that declares and does **not** route through it is itself a finding. |
| arrow slots | **every site that demands a bare `->`** — `src/function/parse.rs:146,1428,1498` · `src/function/metadata.rs:35,46` · `src/types/surface.rs:366` · `src/macros/parse.rs:175` · `src/intrinsic/holon/atom.rs:171` |

**A third set (call-head resolution) is the dominant residue and is measured below.**

## THE RESIDUE — classified on the 97, measured on real codemod output

| cause | n | class |
|---|---|---|
| **`unresolved reference`** | **66** | ⛔ **gaps 3a + 3b — the dominant residue** |
| `ReteCheckErrors` | 12 | rete-specific; classify before assuming |
| **`defsurface`** | 11 | ⭐ **an ARROW slot, not a name slot** (see below) |
| `ProgramBodyEvalFailed` | 3 | gap 2 tail (was 54) |
| other (`expected …`, `UnknownNamedType`) | 3 | |

### ⚠ `defsurface` IS NOT A DECLARATION-NAME GAP — the orchestrator assumed it was

Measured on `tests/services/probe_arc272_rs1_state_must_be_record.wat`:

```
:reason "expected `->` symbol after argspec in method member `increment`; got keyword"
```

The codemod rewrites `->` to `:-`, and `defsurface`'s **method members still demand the bare `->`**.
⭐ The executor named this class in its FIRST score — *"residue: Keyword-only non-name slots (`->` vs
`:-`)"* — before the orchestrator measured it. **Believe that classification over this brief's.**

### The two resolution gaps, from the FINDING

- **3a — a declared NAME walked as a REFERENCE.** `(wat.core/def u/x 1)` → `UnresolvedReference
  ":u::x"`. ⭐ **This is the same defect as the `:restricted-to` whitelist finding**
  (`251/FINDING-8d-a-symbol-whitelist-entry-gets-RESOLVED.md`) — a name in **name position** treated
  as a name in **reference position**. **One cure should close both. Confirm; do not assume.**
- **3b — string-matched builtin heads.** `":wat::config::set-capacity-mode!" =>` in
  `src/config.rs:504` dispatches on a **literal keyword string**, so `is_resolvable_call_head` says
  no and the symbol never normalizes. **43 of 260** such heads fail; ⚠ **217 already resolve — this
  is a REGISTRATION hole, not "symbols don't work".**

⭐ **3b is #95's residual shape.** 8c closed the *type-check* half for slashed heads; this is
*resolution*. The symbol path has a gate the keyword path does not.

## The method — unchanged from 255.1, because it worked

1. **Close one derived set at a time**, starting with the largest (**resolution, 66**).
2. ⛔ **RE-RUN THE DELTA after each** and state the new number **with the comparison**. Baseline
   **97**. A bare number is not the gate.
3. **Report what a set cannot express.** ⭐ In 255.1, asked whether denotation could express
   *"reachable but not a type"*, the executor answered **no**, said so, and used a wall exemption
   instead of forcing it. **That reporting is worth more than a fix — do it again.**

## The gate

- ⭐ **THE DELTA IS THE GATE.** Baseline **97** regressions on the 179-file spread
  (`--check` each converted copy against its unconverted original). State the new number and the
  **classification table** — the count alone understates the work, because a file with two gaps stays
  red until both close.
- `scripts/floor.sh` green; clippy `-D warnings --all-targets --workspace` **0**;
  `census.sh --diff` → `no STOP-8`.
- ⛔ **Run `cargo clippy --release --all-targets -p wat -- -D warnings` YOURSELF before scoring**, and
  run the **lint** suite: 255.1 tripped `one_variant_separator`, `one_param_spec` and
  `no_loose_string_assert`. **Two walls can disagree about one line — rune only where the site
  genuinely is not the thing the wall guards; otherwise ROUTE THROUGH THE DOOR.**
  (255.1's `one_param_spec` fix — moving `is_binder_marker` into `wat-reader` — is the model.)
- Predict the test delta from the diff; confirm with `cargo nextest list`.
- ⛔ **NOT ONE `.wat` FILE CONVERTED.** New fixtures are expected; a converted corpus file is a STOP.

## Doctrine — `wat-rs/CLAUDE.md` does not reach a subagent

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture whole the first time; never re-run first.
- ⛔ **`wat/*.wat` is `include_str!`'d.** An on-disk edit is invisible until `cargo build --release`.
- ⛔ **ASK THE TOOL THAT OWNS THE FACT.** Test counts → `cargo nextest list`.
- ⛔ **If this brief contradicts the code or the runner, THEY WIN.** Orchestrator briefs have been
  corrected **eight times across six stones** — including, in the immediately preceding brief, an
  enumerated list this stone exists to replace with a derived one. **Assume a ninth.**
- Work only in `/home/john/work/holon/wat-rs`. No `git filter-branch`. **Do not push.**

## Out of scope — affirmatively cut

- **8d-ii and 8d-iii.** They wait on the delta, not on this stone's calendar.
- **The `:restricted-to` VALIDATION pass** (builder ruled: exempt from resolution **and** validate).
  ⚠ Its *resolution* half is gap 3a and should close here — **say whether it did.** The validation
  half is not this stone.
- **The `wat.type` NAME SET** — short names, case convention, the 12 missing scalars. Builder
  deferred it: *"i don't know if we're ready for the end state names."*
- **Capacity mode `:panic`** — builder: *"we'll deal with capacity later, its not a now thing."*
