# EXPECTATIONS — STONE: `metadata-of` answers with the whole row

| # | what | expected — EXACT |
|---|---|---|
| 1 | it builds | clean |
| 2 | ★★★ **the six doc-contract keys are present** | `:args` · `:examples` · `:see` · `:deprecated` · `:alias` present on a registry-branch lookup; `:ret` is the **pair** `[type, description]` |
| 3 | ★★★ **THE ROUND TRIP CLOSES** | `wat_doc::from_metadata(metadata-of <fqdn>)` **succeeds**, and the resulting `DocComment` equals the entry's own doc. **This is the acceptance — a key count is satisfiable by six empty vectors.** |
| 4 | ★★★ **BOTH BRANCHES AGREE, KEY FOR KEY** | the same FQDN through the registry branch and a wat-defined binding through the wat branch emit the **same shape** for every shared key. **The precedent's whole point; six keys on one branch is the defect, not the fix.** |
| 5 | ★★★ **the vectors are POPULATED, not merely present** | on `:wat::rete::step-payload`: `:args` has **5** entries, `:examples` **≥1**, and the example's expr is the real form. **An empty `:args` passes row 2 and fails the stone.** |
| 6 | ★★★ **`:ret` carries BOTH halves** | `[":wat::rete::DerivationStep", "the per-edge explain payload …"]` — type **and** description. Today it is the description alone |
| 7 | ★★ **the existing readers still work** | `:arity` (48 sites) · `:totality` (18) · `:purity` (5) · `:determinism` (5) · `:name` (1) unchanged in shape. **Measured: 0 sites read `:ret`, which is why row 6 is safe** |
| 8 | ★★★ **`:yields` is NOT emitted from the registry branch** | absent. `IntrinsicEntry` has no `yields` field; emitting it from the wat branch only would recreate the two-shapes defect. **Report the asymmetry; do not paper it** |
| 9 | ★★ `:syntax` is NOT added | outside the doc-row contract; `from_metadata` never reads it |
| 10 | ★★★ **a doc row renders from the LOOKUP ALONE** | `metadata-of` → `#wat.doc/Row`, no `:wat::intrinsic::examples` scan. **This is what the stone is FOR** — 576 lookups instead of 576 linear scans over 615 |
| 11 | ★★ a negative control, committed | delete one of the six `put`s → the round-trip test goes **RED**. Not a scratchpad probe |
| 12 | ★★ the formatter still dresses it | the example from the lookup formats to the ruled shape, `widest <= 120` |
| 13 | ★★ nothing else moved | `deporder c69f0460e9f91672` · `spawn 1c4bbb19bee10093` · `io ba89b65cbd61abc6` · `fmt cae5250235f652ec` |
| 14 | floor (ORCHESTRATOR) | `5192+` run, **0 FAILED** |
| 15 | clippy (ORCHESTRATOR) | `0` |

**Runtime prediction:** 60-100 min. Six `put`s is small; the wat-branch parity and the round-trip
acceptance are the work.

## Trap-doors named in advance

- **Row 3 is the acceptance and row 2 is not.** Six keys present with empty vectors passes a key
  count and fails the purpose. The round trip is what proves the shapes are the decoder's.
- **Row 4 is the precedent's own bar.** *"Converging one key only MOVES the defect."* Six keys on the
  registry branch while the wat branch answers differently is the same defect with more keys.
- **Row 5 is the anti-empty-vector row.** `:wat::rete::step-payload` declares five `@arg`s; anything
  less than 5 means the emit is structural rather than real.
- **Row 8 is the honest cut.** `:yields` is read by the decoder and absent from the registry's
  source. Emitting an empty or fabricated `:yields` to make row 3 pass would be exactly the
  papering-over this arc keeps finding.
- **Row 6 is safe BECAUSE it was measured** — 0 of 344 call sites read `:ret`. Do not take that on
  faith; re-run the census before changing the shape.
- **Row 10 is the reason any of this matters.** If the row still needs the enumeration verb, the
  stone did not land.
