# EXPECTATIONS — prose may name a retired form; code may not

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT, AND NO PINNED PHANTOM COUNT

**The floor must be ≥ 5,248 plus every arm you drive.** My scan found 9 candidates of which most are
noise; **the classifier is the finding, not the number.** Report what yours says.

## The scorecard, with pre-values measured at HEAD `51b851c91`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | ★ an invented head runs clean | **`"ran"`** — driven, `:wat::rete::core::THIS-HEAD-NEVER-EXISTED` in a `def` body | the new gate REDs on it |
| 2 | the two phantoms | `map`/`filter` in the codemod AND the scratch probe (driven) | gone from both; gate green |
| 3 | the rename pairs | 41 pairs, **2 targets unresolvable** | 39, with the reason at the table |
| 4 | prose survives | `foldr`/`nth` in comments only | **still there, gate green** — trap 1 |
| 5 | forms not flagged | `:wat::rete::core::defn` in 15 files | not flagged |
| 6 | the gate declares non-vacuity | — | a real floor; `every_walking_gate_declares_non_vacuity` green |
| 7 | `CLAUDE.md`'s claim | *"All wat stays correct, always"* — **driven false** | states what is proven |
| 8 | lints | **134/134** (measured) | green |
| 9 | floor | **5248/5248** (measured) | ≥ 5,248, zero FAIL rows |
| 10 | clippy | **rc=0** (measured) | silent |

## The mutation proofs

1. **Re-introduce a phantom** in a scratch `.wat` code position → gate REDs, naming file and token.
2. **Re-introduce one in a COMMENT** → gate stays **GREEN**. *This is the one that matters: a gate
   that cannot tell code from prose will be "fixed" by deleting accurate history.*
3. **Blind the resolver** (point it at an empty row set) → the gate must RED loudly, not pass
   vacuously — and its non-vacuity floor must be what catches it.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

60–75 minutes. The deletions are minutes; the classifier and its three mutations are the work.

## What would make this strike a failure even if every test passes

**Re-pointing the codemod at `mapv`/`filterv`.** It would look like the obvious repair, leave 41
pairs intact, and turn the mandated migration tool into one that silently swaps a lazy `Stream` for
an eager `Vector`. Row 3 expects **39 pairs**, not 41 corrected ones.

The second: **a gate that flags comments.** Row 4 and mutation 2. The tree's accurate record of what
was retired is exactly what this class needs preserved.
