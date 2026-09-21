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
