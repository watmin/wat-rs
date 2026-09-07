# AMEND — STONE 251.9: the ground, re-measured on a green tree

The BRIEF and EXPECTATIONS stand unchanged. This adds only what was **measured this session**,
after the match-arm campaign closed, so nothing below is inherited from a note.

## Baseline — the precondition the probe was owed

```
HEAD    aac91f89a      tree CLEAN
floor   5226 tests run: 5226 passed, 0 failed, 21 skipped      clippy 0
```

The probe could not honestly be run red until this existed. It exists now.

## The probe is RED, and its assertion names the mechanism

`cargo nextest run --release --run-ignored all -E 'test(probe_arc251_stone9)'`

```
3 tests run: 0 passed, 3 failed
  missing_mandatory_purity_marker_is_refused_under_both_head_spellings   FAIL
  malformed_variant_is_refused_under_both_head_spellings                 FAIL
  a_symbol_headed_declaration_actually_declares                          FAIL

assertion `left == right` failed: a symbol-headed defenum did not register its
type — the reference to :probe::Color is unresolved
  left: 3   right: 0
```

## The defect also reproduces at the FILE level — a second door, not the same one

```
./target/release/wat --check tests/resolve/probe_arc251_stone9__declares_kw.wat    EXIT=0
./target/release/wat --check tests/resolve/probe_arc251_stone9__declares_sym.wat   EXIT=1

#wat.resolve/UnresolvedReferences {:message "1 unresolved reference" …
  :path ":probe::Color::Red"
  :context "call head — not a builtin, not a registered function"}
```

★ The two fixtures differ in **one token** — `(:wat::core::defenum …)` vs `(wat.core/defenum …)`.
That is not an edge case. **It is the crusade's target spelling**, so this stone is a scale model
of the head migration itself.

## ⚠ DO NOT DIAGNOSE THIS FROM `is_declaration_form` ALONE

`parse.rs:199` is the SUBJECT and its `_ => return false` is the defect verbatim — but read its own
doc before theorising from it:

> *"It must be asked of a POST-MACRO-EXPANSION form: `defn` is a macro that expands to `def`
> (`wat/core.wat:1175`), so the raw surface head a user typed is not what lands in residue."*

So the surface `defenum` a user writes does **not** reach that predicate as itself, and the
`--check` failure above travels the load/preregister path rather than `eval-with-defs!`. **This is
why the BRIEF names 23 head readers across three files and not one site.** A rider that fixes
`is_declaration_form` and stops has fixed one door in a corridor.

The remedy is unchanged and is the whole point: ONE `head_fqdn` door that reads either spelling,
every head reader routed through it, and **the `_` arm deleted at each site converted** — a site
that keeps its catch-all keeps its silent path.

## Why this is the gate, in one line

A symbol-headed declaration evaporates with no error. Run the head migration across 1,869 files
before this lands and every declaration in the corpus stops declaring — surfacing as thousands of
unresolved-reference errors whose actual cause appears in none of them.

## Kin, closed today — the same shape one layer over

`src/resolve/walk.rs` used `items.iter().skip(4)` and never resolved the first arms' bodies. It hid
a malformed reference from arc 278 for three weeks; widening it to `skip(2)` surfaced the defect
instantly. **A reader that answers "nothing here" about something that is very much here.** That is
this defect too. See `109/WEIGH-STONE-the-match-arm-RELAND-5.md`.
