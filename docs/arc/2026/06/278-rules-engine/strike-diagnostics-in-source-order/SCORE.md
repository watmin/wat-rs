# SCORE — diagnostics in source order, and a corpus that contains the shape but cannot prove it

> **Written after the orchestrator's own weighing.** The ★ was that half my variant table described
> the wrong fixture, and the sharpest finding is that the tie-break the strike demanded is
> **unprovable from the corpus even though the corpus contains the pair.**

## 24-run stability

| fixture | HEAD | sort removed | after |
|---|---|---|---|
| `c2_mixed_macro_swap` | **14/10** | 9/15 | **24/24** |
| `w2a_kwargs_check_mint_swap` | **14/10** | 12/12 | **24/24** |

`w2a`'s cured order — `40:22, 48:3, 51:41, 51:49` — is a **third** answer, matching neither pre-fix
variant. **The cure beats the defect's absence rather than picking a coin face.**

## ⛔⛔ THE ★ — MY VARIANT TABLE DESCRIBED ONE FIXTURE AND WAS PRESENTED AS BOTH

DESIGN, BRIEF and the rider's prompt all say *"Four errors, the same four… only one error moves."*

**True for `w2a` only.** `c2` emits **NINE** errors (verified: `9 type-check errors`), arriving as two
blocks that swap — 7 match errors at lines 91–127 plus 2 Kwargs at 156. **The gate's own quarantine
row had this right**; my artifacts flattened both fixtures onto `w2a`'s shape. A rider trusting it
would have pinned a 4-row expectation against a 9-error file.

## ★★ THE SHARPEST FINDING — THE CORPUS CONTAINS THE SHAPE AND STILL CANNOT PROVE THE TIE-BREAK

I wrote the tie-break trap expecting the corpus might lack a same-span pair. **It has one** — `c2`
carries two `TypeMismatch`es for params `#1`/`#2`, both at `156:5..158:53`, same variant.

**And it is inert.** Both members come from a *single deterministic intra-function walk*, so their
relative order never varies. Measured: with the key truncated (end + payload dropped), the tie-break
test REDs while **both 24-run fixtures stay GREEN**. **A partial key would have shipped green through
every fixture in the corpus.** Only a constructed pair — fed in both input orders, required to
produce one output — reddens it.

This is a sharper C9 hole: not an **absent** shape, a **present-but-inert** one. A corpus census that
asked "does a same-span pair exist?" would have answered yes and been useless.

## ★ A TRAP NEITHER ARTIFACT COULD HAVE SEEN COMING, AND THE CURE TURNS ON IT

**`Span::eq` returns `true` unconditionally** and `Hash` is a no-op (`crates/wat-reader/src/span.rs:137`)
— deliberate, for structural AST identity. So **any `Ord for Span` consistent with that `Eq` must
return `Equal` always**: a sort keyed on `Span`'s own ordering would be a **silent no-op**, passing
compilation and changing nothing.

The cure is therefore a key *extraction* — `CheckError::source_order_key` →
`(file, line, col, end, format!("{:?}", kind))` — not an `Ord` impl. Neither artifact flagged this,
and the obvious implementation would have failed silently.

## Mutations — four, all driven

| # | mutation | result |
|---|---|---|
| 1 | sort removed | 9/15 and 12/12 over 24 runs, hashes exactly the two pre-fix variants; both 24-run tests RED **and** shards 09/10 of the re-admitted corpus RED — the de-quarantine is load-bearing |
| 2 ★ | partial key | tie-break test RED, **both 24-run tests still GREEN** — see above |
| 3 | reverse | stability green, **order** assert RED plus 4 goldens/cli — the tests read the order, not the set |
| 4 | quarantine row restored | without the constant: RED `left: 1, right: 0`. With it bumped: **17/17 GREEN over a cured file** — the pin cannot tell cured from broken, exactly as its own header says |

## Goldens — 13 moved, every one order-only

Proven mechanically: same multiset of top-level `:errors` records, order changed, everything outside
the `:errors` vector byte-identical. **The instrument was anchored on a known positive** — one field
value mutated → `CONTENT CHANGED` — before its negatives were trusted.

Each new order is nondecreasing in `(line, col)`. `enums__cross_enum_variant_pattern_rejected` now
reads `5:3, 5:4, 5:22, 7:6, 8:6` — a real reader improvement, which is the point of row 2.

## `QUARANTINE_LEN` 2 → 0, and the evidence is not the zero

The evidence is `tests/services/probe_arc278_c20_check_errors_in_source_order.rs`: two tests × **24
fresh processes** pinning the whole span sequence, plus the same-span pair's order, plus the
constructed-pair tie-break. That is the shape the gate's own header demands of a de-quarantine.

## Honest deltas — six more corrections to my artifacts

- **"`check.rs:744-747` — the single site every check error returns through" is false.**
  `freeze/env.rs:189` builds its own multi-error `CheckErrors`. It is a `Vec` walk, already source-
  ordered and outside the measured defect — but "the single exit" does not survive a grep.
- **The root is four map-ordered walks, not two** — `functions_iter()` at `:649`/`:738` **and**
  `function_values()` at `:601`/`:678`. The cure covers all four (they share the exit); the diagnosis
  was incomplete.
- **Blast radius under-stated by an order of magnitude.** I said `check.rs` + the gate. Actual: **15
  tests RED on the first floor** — 13 goldens plus two `wat::cli` positional assertions. ★ Those two
  were passing deterministically **only because their fixture has one function**, so a single-entry
  `HashMap` never exposed the randomisation. **The defect was latent there, waiting for a second
  function.**
- **Neither artifact anticipated a nextest budget.** Two 24-run tests cost ~10 s each against a 30 s
  default kill — over budget at any contention this repo has recorded. A derived 90 s/180 s override
  was added below the rete cohort (first-match ordering preserved); measured under the real floor at
  15.9 s / 13.5 s (1.64x / 1.70x), ~5.7x margin.
- **My trap-door note named the wrong slow test.** I said the floor's `(1 slow)` is
  `reachability_shard_0_of_6`; across the rider's three floors the sole SLOW row was
  `no_broken_intra_doc_link`. Same conclusion — a timing annotation on a PASS — but **a rider told to
  ignore one name could dismiss a genuinely new one.** My instruction was more specific than my
  evidence supported.
- **Minor:** `value/symbol_table.rs:34` is the comment; the field is `:33`.

## What the rider did that is worth copying

- **It forced a clippy recompile** after noticing the first call came back cached — *"which reads
  identical to a green."*
- **It ran a third full floor** after four comment-only citation fixes, rather than report a green
  that predated the text it was shipping.
