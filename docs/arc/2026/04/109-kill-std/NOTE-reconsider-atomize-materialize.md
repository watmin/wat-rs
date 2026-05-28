# NOTE (arc 109 vocabulary) — reconsider `atomize` / `materialize` later

**Filed 2026-05-27. NOT a decision — a marker to challenge a prior intueri verdict when
the namespace/vocabulary work (task #565) lands.**

Arc 225 (the bridge-naming intueri cast) chose the **layer-name + direction** family over
`atomize` / `materialize`:

- `:wat::holon::to-holon`   — value → HolonAST
- `:wat::holon::from-holon` — HolonAST → value   (this replaced `:wat::core::atom-value`)
- `:wat::holon::from-wat`   — WatAST → HolonAST   (renamed from `from-watast`)
- (`to-wat` — HolonAST → WatAST)

The rationale recorded in `docs/arc/2026/05/225-atomize-materialize-rename/DESIGN.md`: the
proposed `materialize` was honest as an operation-name, BUT a user observation surfaced an
asymmetry — we say "holon" as the layer name, never "holonast"; `from-watast`/`to-watast`
already used "watast"; the honest family uses layer-names + direction throughout.

**User direction 2026-05-27:** *"`atomize` and `materialize` — those are good names — want to
challenge intueri on it later, not now."*

**The challenge to run later (during #565 / the intrinsic-substrate vocabulary pass):**
re-cast intueri on the bridge family with `atomize` / `materialize` as live candidates again.
The 2026-05-22 cast rejected them on the layer-name-symmetry argument; the user's instinct is
that `atomize` (value → its atom/holon-form) and `materialize` (holon-form → value) read more
naturally than `to-holon` / `from-holon`. Weigh: naturalness/memorability vs the
layer-name-+-direction symmetry that also governs `from-wat` / `to-wat`. Whatever wins, the
WHOLE family stays consistent (HARD CUT, no synonyms — `feedback_wat_llm_first_design`).

Cross-ref: arc 225 DESIGN + EXPECTATIONS-STONE-225.1 (row 5: atom-value → from-holon);
`project_atom_is_holder` + `project_typed_entities_doctrine` (atomize/materialize = substrate
quote/unquote framing); arc 240 (consumer `.wat` migrated to the current names).
