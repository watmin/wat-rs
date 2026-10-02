# AMEND 5 — STONE 255.81: the twenty-six, ruled by class

**Drawn 2026-10-02.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `e5e26ebaf`
(grok's classification of the 132: 106 spelling, 19 different-error, 7 different-value). Commit locally on `main`;
**do not push**.

The classification is accepted as measured. The orchestrator rules the 26 by mechanism:

- **A. An incidental old spelling in a test's input** fires the retirement (or shifts the first error) before the test's
  subject: `probe_arc242_stone2_value_position_doctrine` contract_01/05, `probe_stone255_71_template_untyped`,
  `stone18a_errors::error_03`, `probe_arc237_stone1_typeunion_substrate` probe_13, `probe_supervisor_select_lost`,
  `check::tests::any_in_fn_rejected_at_parse`, `probe_arc278_deep_cascade` ×2, `probe_arc293_holder_substitution`,
  `probe_arc255_56_operators::eq_generic_refuses_a_function`. **Cure at the input** (recorded codemods; `wat-fix-rust` for
  embedded wat), then each test must pass with **its original error**, and say why 4a missed that input. For any whose
  first error stays different after the input is respelled (eq-generic's `ReturnTypeMismatch`, holder-substitution's
  `kwargs-construct`), trace why the error order changed and say whether this stone caused it: if it did, cure it; if not,
  STOP with the trace.
  - **Doctrine 1 (contract_01/05)** is about a type keyword used as a **value**. Its input becomes `wat.type/nil` in
    value position. If what the checker then says is not Doctrine 1's error, that is a position-rule question (255.66:
    position decides): **STOP-7**, quote it, do not choose.
- **B. A Rust site still on the old AST key:** `probe_arc255_12_macro_identity` ×2, `probe_arc251_8d_macro_member_join`.
  `macro param … declared Path(":wat::WatAST"), but … its type must be :wat::WatAST`: the defmacro parameter check
  compares against a stale literal. Cure it **at the door** (K1: compare canonical keys), find every sibling site with
  the same shape (a hand-typed key compared to a parsed type), and list them.
- **C. Test constants on old keys:** `probe_arc278_value_universal_top` ×2. Move the constants to the registered keys
  (or build them through the type door), and correct the stale `"RED at HEAD"` message.
- **D. A stdlib line moved:** `probe_arc278_peers_bijection` ×4 (span line 909→910, 926→927). Name the edit that moved
  the line (file and commit); then re-capture. If no edit in this stone explains it, STOP with what you found.
- **E. A probe's expected node:** `every_probe_runs::…probe_s3b_extract`. The extracted type node is now spelled as the
  corpus spells it; the probe compares against `(keyword-node ":wat::core::i64")`. Expect the node as it now is, as data
  (and never via a string naming a retired key, which now refuses). The claim is unchanged.
- **F. The lints** (`nested_program_starts` ×2, `no_loose_string_assert`, `no_inlined_wat_in_tests`,
  `one_variant_separator`) fired on this stone's own new code: cure each by its own stated remedy.

Then: the 106 spelling rows re-captured under amendment 2's as-data audit, amendment 4 items 2-3 (the fixture move,
clippy, the `wat-fix-rust` dry runs, `census.sh --diff`), and the floor.

STOPs as before, plus **STOP-7** above. A STOP means STOP. Append to the SCORE, commit, **do not push**.
