# ⛔ CLOSED — NOTHING HERE WAS A DEFECT. Kept only for the lesson at the bottom.

> **Both claims this file made are struck.** The accumulate behaviour matches Clara exactly (§1).
> The duplicate-derived-fact divergence (§2) is **already known, already named, and already
> CLOSED AS JUSTIFIED** — with the record assigning the verdict OPPOSITE to the direction I was
> heading:
>
> - `wat-scripts/perf/grid/retract-multiplicity.wat:8` — *"the **justified** derived-multiplicity
>   split (Clara bag vs wat set on derived facts) cannot dominate"*. An existing grid cell,
>   deliberately shaped so this divergence cannot affect its verdict.
> - `strike-retract-multiplicity-proof/REVIEW.md:18` — *"the derived-multiplicity divergence — **the
>   one CLOSED AS JUSTIFIED**"*, with the table: zero retracts, wat `[0 1 2]`, Clara `[0 0 1 1 2 2]`.
> - The same REVIEW, on which engine is right: *"the `1 1`/`2 2` is **Clara over-deriving (wat
>   right, closed)**."*
>
> **I was treating Clara as the referee. The record had already ruled the other way.** There is no
> grid cell to add — the population is covered, and covered deliberately.

# FINDING — ⛔ ~~`acc::count` returns a WRONG COUNT~~ — STRUCK. Clara agrees. The real divergence is duplicate DERIVED FACTS.

> # ⛔⛔ THE TITLE'S CLAIM IS FALSE AND THE WHOLE ORIGINAL FINDING IS WITHDRAWN.
>
> **I filed an engine-defect finding before measuring the reference.** The builder's ruling —
> *"as we find these flaws, we must grow the grid to confirm how clara handles these, then we fix
> the wat-oracle then the wat-native"* — is exactly the discipline that caught it, and I had
> committed the finding before following it.

**Re-measured 2026-09-10 at `2d3f4b40e`**, all three engines, same inputs.

## 1. The accumulate behaviour is CORRECT. Clara does the identical thing.

Binding `?v` in an accumulate's `:from` source is **not an inert bind** — it is a **grouping
variable**, in both engines. Three inputs, three engines:

| readings | Clara rows / `n` | wat-oracle rows | wat-native rows |
|---|---|---|---|
| `10 20 30` (all distinct) | 3 → `(1 1 1)` | 1 | 1 |
| `7 7 7` (all equal) | 1 → `(3)` | 1 | 1 |
| `10 20 20` (two equal) | 2 → `(1 2)` | 2 | 2 |

`plain` (no extra bind) is **3** in every engine, every row. ⭐ **The partitioning matches Clara
exactly.** The original "3 readings → n=1" was me reading `first` of a multi-row result and calling
it a count.

## 2. The real divergence, isolated to four lines and NOT about accumulate

```wat
(:wat::rete::defrule :dd::r :when [(:dd::In (?k <- :k))] :then [(:dd::Out :n 1)])
;; insert In{1}, In{2} — the rule derives the SAME Out{n:1} twice
```

| engine | `Out` rows |
|---|---|
| **wat-native** | **1** |
| **Clara** | **2** |

No accumulate, no binding, no grouping. **wat collapses identical derived facts; Clara keeps
both.** Every row of the table in §1 falls out of this: three partitions each deriving
`Extra{n:1}` are three identical facts, and wat shows one.

## 3. It is already documented — as a MASKING property, not as intended semantics

`wat-scripts/perf/grid/CLARA-TRANSLATIONS.md:179`, on why an earlier defect hid:

> *"Three properties must ALL hold or the defect hides again: … it must be observed **through a
> query** (`production_delta` dedups derived facts by value and masks token multiplicity)"*

So the collapse is known and is described as something that **hides defects**. ⚠ That is a
different claim from *"wat intends set semantics for derived facts."* Which of the two it is,
this finding does not establish.

## ⛔ What is NOT measured

- **Whether the FACT STORE or the QUERY collapses.** Both observations above went through a query,
  and `production_delta` is a query-side dedup. There may be two `Out{n:1}` facts in memory with
  the query showing one. **Until that is separated, "wat collapses derived facts" is a statement
  about what a query returns, not about what the engine stores.** That is the first measurement.
- **Whether the divergence is intended.** See §3.
- **Blast radius.** Any rule that can derive one fact from two activations is affected — which is
  a large class, not an accumulate corner.

## ⭐ What the grid needs, now precisely specified

Not an accumulate cell. **A cell whose rule derives the SAME fact from two activations** — the
four-line program in §2. The A5 axis already carries `count`-with-one-bind and `sum`-with-two-binds;
what no cell in the grid has ever produced is a **duplicate derived fact**, which is why 19
accumulate cells compared three ways against Clara have never seen this.

⭐ And by `check-grid-three-way.sh`'s own diagnosis table, `oracle == native != clara` reads as
**the SPEC diverges from Clara** — the pairing its header calls *"the pairing NOTHING has ever
run."*
