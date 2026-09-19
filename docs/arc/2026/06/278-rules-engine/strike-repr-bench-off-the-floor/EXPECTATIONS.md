# EXPECTATIONS — rune the timing diagnostics

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the four ordering assertions are gone | `grep -nE '^\s+[a-z_]*(trie\|arr) [<>]' binding_repr_bench.rs` → no hits |
| 2 ★ | three runed reasons | `:146` `below-resolution`, `:264` `no-falsifier`, the dominance test `below-resolution` — each reason answering its category's decisive test |
| 3 ★ | the rune is GATED | `excusare` row in `WARD_VOCABULARIES`, and a fourth invented category REDs `no_unknown_ward_rune` — quoted verbatim, then restored |
| 4 ★ | the vocabulary table | `docs/CONVENTIONS.md`, `purgare`'s shape, marking `below-resolution`/`no-falsifier` as proposed-upstream with the request's path |
| 5 ★ | the dated verdict replaces the gate | six samples, median + range, both columns, in the doc comment |
| 6 | the faithfulness gate survives | not deleted — it is the bench's precondition |
| 7 | floor | **5470 run / 22 skipped** — predicted exactly; report both |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 3 is what stops this being a marker with no checker** — the pattern this arc
has spent the day removing. Row 5 is what replaces a live gate with an honest measurement.

## Trap doors, named in advance

- **Shipping the rune without the registry row.** `categories_on` only searches registered wards, so
  an unregistered rune is silently invisible — green, and decorative.
- **A tolerance or min-of-k** to keep the assertion. Rejected: a margin picked to stop a red is a
  threshold tuned from our own noise.
- **Deleting the faithfulness gate** with the timing assertions.
- **A reason that does not answer its category's decisive test** — `below-resolution` without the
  noise floor and margin is exactly the convenience-plea `excusare` exists to strike.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- Any ordering assertion left.
- A rune with no registry row, or a registry row with no mutation proof.
- A verdict recorded from fewer than six samples, or without its range.
- A floor count other than 5470 / 22.
