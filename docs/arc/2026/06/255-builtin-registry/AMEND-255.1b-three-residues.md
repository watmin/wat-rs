# AMEND — STONE 255.1b: three residues. 22 → 7, and two of them are real.

**Orchestrator's verification row, `f816b87bf` (not pushed).**

## ⭐ THE FOLD WORKED — 22 → 7, and the hard row LANDED

| row | result |
|---|---|
| the 19 codemod-emitted `wat.type` names | ✅ **0 failures** (was 7) |
| ⭐ **ONE POSITION, NOT TWO** | ✅ `(wat.type/Vector 1 2)` in **call** position → rc=0; annotation → rc=0 |
| non-member control | ✅ `(wat.type/nope)` → `UnresolvedReference` |
| clippy workspace | ✅ 0 |
| floor | ⛔ **7 of 5936** (was 22) |

⭐ **Your line was right: *"a complete membership list was what the predicate needed."*** Deriving
membership from **denotation** rather than a hand-list is the correct door, and moving
`is_binder_marker` into `wat-reader` instead of runing `one_param_spec` was the right call — a second
recogniser is what this arc exists to delete.

## RESIDUE 1 — 5 STALE GOLDENS. Your message is RIGHT; do not revert it.

```
expected: "annotation names unknown type :wat::core::Bogus — not a declared type, not a type
           variable, and not a builtin"
actual:   "not a member of wat.type: :wat::type::Bogus"
```

⭐ **The new message is acceptance row 4 of the ruling, verbatim:** *"the refusal must say 'not a
member of `wat.type`' — distinct from 'unknown type', or a typo and a real absence read alike."*
**Reverting it to satisfy a golden would delete a row the builder ruled.**

⚠ Note the `:path` also moved — `:wat::core::Bogus` → `:wat::type::Bogus`. That too is an
improvement: it names **what the author wrote**, not what it canonicalized to. Keep it, and say so.

**Regenerate, do not hand-type.** Files: the 5 `probe_arc255_the_type_position_has_its_own_authority`
goldens (`a_bogus_type_in_return_position` · `..._in_a_type_argument` · `..._in_a_defenum_variant_field`
· `..._in_a_defstruct_field`, plus the one `the_type_namespace_spelling...` below).

## ⛔ RESIDUE 2 — REAL. `Infer` became a type, and it must not be one.

```
the_type_namespace_spelling_now_answers_is_type
  left:  "true true true true"      right: "true true true false"
  "Tuple and i64 answer true in BOTH spellings; Infer is a marker and stays false"
```

Your SCORE says you added `INFER_TYPE_PATH` as a *"wat.type-only sentinel"* member. **That makes
`is-type?` answer `true` for `Infer`, and the marker semantics say `false`.**

⛔ **A sibling test PASSES and pins the other half:**
`the_infer_marker_is_not_a_registered_type_and_must_stay_accepted`. So `Infer` must be **accepted in
an annotation** and **not be a registered type**. Both at once — that is the marker's whole job.

**Membership and is-type must come apart for `Infer`.** ⚠ If deriving membership from denotation
cannot express *"reachable but not a type"*, say so — that is a finding about the derivation, not a
line to patch.

## ⛔ RESIDUE 3 — REAL. The type ARGUMENT does not canonicalize like the HEAD.

Fixture writes **both** as `wat.type/`:

```wat
(:wat::core::defn :user::c01-mk [] -> (wat.type/Vector :- [wat.type/i64]) …)
```

Renders as:

```
:expected "(:wat::core::Vector :- [:wat::type::i64])"     ← head canonicalized, ARG did not
:got      "(:wat::core::Vector :- [:wat::core::String])"
```

⛔ **Two namespaces for the same position inside ONE message.** The head goes through canonical
identity; the type-argument does not. Both were written `wat.type/`.

Hits `probe_arc251_parametric_target::contract_02_parametric_form_rejects_mismatched_element` and
`contract_04_new_form_equiv_to_legacy_angle_bracket`.

⚠ **This is the recurring shape of the whole campaign** — one door reached, a sibling position
missed. Route the type-argument through the same door as the head, then state which other positions
you checked (binder head, nested parametric, tuple element, return slot). ⛔ **Do not fix only the
two failing contracts.**

## The fold

⛔ **Fold into `f816b87bf`.** Not a repair commit — the stone must be green at its own landing.
Nothing is pushed.

## After the fold

State crate clippy + `probe_arc251_parametric_target` + `probe_arc255_the_type_position_has_its_own_authority`.
I re-run floor, workspace clippy, `census.sh --diff`, the 19-name sweep, the one-position probes and
the 179-file delta. **Do not push. Do not start 8d-ii.**
