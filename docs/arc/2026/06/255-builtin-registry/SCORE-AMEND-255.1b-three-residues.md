# SCORE-AMEND — STONE 255.1b: three residues

Folded into `f816b87bf` (amended). **Not pushed.**
Parent: `AMEND-255.1b-three-residues.md`.
Floor / workspace clippy / census / 19-name sweep / one-position probes / 179-file
delta: orchestrator re-runs uncontended.
Do not start 8d-ii. No `.wat` converted.

## RESIDUE 1 — goldens regenerated, message kept

`probe_arc255_the_type_position_has_its_own_authority__unknown_named_type.edn`
via `UPDATE_EDN=1` (not hand-typed):

```
:message "not a member of wat.type: :wat::type::Bogus"
:path ":wat::type::Bogus"
```

The path is what the author wrote. `:line 0` is the rust-sentinel blank (checker
span). Ruling row 4 kept.

The six-row "one golden" finding split: `:wat::core::Bogus` (keyword spelling
fixture) is **not** a wat.type spelling, so it keeps generic unknown-type prose.
New golden `...__unknown_named_type_core.edn`, captured the same way. Same
exit code; different message — that is the non-vacuity row working.

## RESIDUE 2 — Infer is a marker, not a type

Denotation **cannot** express "reachable but not a type": `:wat::core::Infer`
is not a member, so deriving membership would leave Infer out. Putting it in
`classify` as Builtin made `is-type?` true and broke
`the_type_namespace_spelling_now_answers_is_type`.

Removed from classify. Annotation acceptance is a **wall exemption**, not
membership: `consider_named_path` skips `INFER_TYPE_PATH` (same class as type
vars). `the_infer_marker_is_not_a_registered_type_and_must_stay_accepted`
passes. `is-type?` Infer is `false` in both spellings.

## RESIDUE 3 — type-argument through the same door as the head

`type_denotation` is the door. Applied at:

| position | how |
|---|---|
| parametric **head** | identity stored; `format_type` / `parametric_heads_unify` denotate |
| type **argument** | identity stored; `format_type` denotates |
| nested parametric | recursive `parse_type_form` + format |
| tuple element | `(wat.type/Tuple :- […])` denotates to `TypeExpr::Tuple` (constructor special-case uses denotation, not `== "wat::core::Tuple"`) |
| return slot | same parse + format |
| binder / param slot | stone3 fixtures |

`probe_arc251_parametric_target` 4/4. Expected/got both
`(:wat::core::Vector :- [:wat::core::i64])`.

Storing denotation at parse for **heads only** had turned `wat.type/Bogus` into
`:wat::core::Bogus` and deleted the member message. Identity storage + denotation
at unify/format keeps origin for non-members.

## Walls I ran

- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — **0**
- `probe_arc255_the_type_position_has_its_own_authority` — **13/13**
- `probe_arc251_parametric_target` — **4/4**
- `probe_arc251_keyword_to_type_form` — **9/9** (nested parametric, tuple)
- `probe_arc251_stone3_parametric_form` — **3/3** (param / typealias)

Floor + workspace clippy + census + 19-name sweep + 179-file delta: **not run**.
Do not push. Do not start 8d-ii.

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-21. **ACCEPTED.**

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5936/5936 passed** (8 slow), exit 0 |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| no `.wat` converted | ✅ 1 file touched, **A**dded (`probe_255_1_wat_type_membership.wat`) — a fixture |
| the 19 codemod-emitted `wat.type` names | ✅ **0 failures** |
| ⭐ **one position, not two** | ✅ call `rc=0`, annotation `rc=0`, non-member `rc=1` |
| ⭐ **THE GATE — 179-file delta** | **104 → 101 → 97** |

## ⭐ WHAT THIS STONE ACTUALLY BOUGHT

- **Identity is the pair.** `canonical_identity` — one door, both spellings in, one key out.
- **`wat.type` is REAL**, and membership is **derived from denotation**, not hand-listed: *"a future
  core builtin is a wat.type member automatically."* That is the right shape and it is why the 19
  names now pass.
- **`one_param_spec` was routed, not runed** — `is_binder_marker` moved into `wat-reader` as the
  shared door. A second `:-` recogniser is what this arc exists to delete; the executor refused the
  cheap escape.
- **`Infer` came apart from membership.** Asked whether the derivation could express *"reachable but
  not a type"*, the executor answered **no** and said so, then made annotation-acceptance a **wall
  exemption** (same class as type vars) instead of forcing it. **That is the reporting this brief
  asked for, and it is rarer than a fix.**
- **Two freeze hazards documented at their sites** — a rust-scheme path must not be
  clojure-round-tripped; a *rendered* form starting with `(` must not be prefixed.

## ⛔ 8d IS STILL BLOCKED — the residue, classified on the 97

| cause | n | status |
|---|---|---|
| **`unresolved reference`** — gaps 3a + 3b | **66** | ⛔ **untouched by this stone, exactly as predicted** |
| `ReteCheckErrors` | 12 | not identity |
| ⭐ **`malformed :wat::core::defsurface` declaration** | **11** | ⛔ **a declaration form the brief never listed** |
| `ProgramBodyEvalFailed` (gap 2 tail) | 3 | 54 → 3 → 3 |
| other (`expected …`, `UnknownNamedType`) | 3 | |

⚠ **THE COUNT MOVES SLOWLY AND THAT IS NOT A FAILURE.** A file carrying two gaps stays red until
both close, so 104 → 97 understates the work: gap 1 is **closed**, gap 2 went **54 → 3**. The
executor said this first — *"the regression count is not the residue kind"* — and it was right.

## ⛔ THE EIGHTH BRIEF ERROR — `defsurface`

The brief enumerated `defenum`, `typealias`, `defclause`, `defrecord`, `defstruct`. **It missed
`defsurface` — 154 files in the corpus**, now 11 of the 97. The brief's own closing line said *"my
briefs have been corrected seven times across five stones… assume an eighth."* **This is the eighth,
and it is the same defect the brief warned about: an enumerated list instead of a derived one.**
`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`.

⇒ **255.2 must DERIVE the set of declaration forms**, not extend the list by one.

**VERDICT: ACCEPTED.** The stone did what it was drawn to do. **8d remains blocked**; the dominant
residue is the resolution gap (66 of 97), which this stone correctly did not touch.
