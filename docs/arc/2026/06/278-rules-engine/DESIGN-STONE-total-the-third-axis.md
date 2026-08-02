# DESIGN-STONE — `total?`, the third fence axis

> **Status: DESIGNED, blocked on `BRIEF-the-fence-names-the-head.md`.** Builder-ruled 2026-08-02:
> *"draw the fence naming strike, then total? behind it."*

## The hole, measured

A rete `where` predicate must be pure ∧ deterministic. Both hold for verbs that are **partial** — defined
on some inputs and undefined on others — so the fence admits them:

| verb | in `purity.rs`'s pure∧det list? |
|---|---|
| `first` / `second` / `third` | **yes — admitted** |
| `i64::/` `i64::mod` `i64::rem` | **yes — admitted** |

`first`-on-empty is the dangerous shape: it compiles, fires correctly for as long as no rule meets an
empty vector, and then **one empty vector kills the entire fire** — a raising predicate aborts the whole
fire on both engines (measured, `SEAM-2026-08-01`). No amount of green testing surfaces it.

## Not a fourth purity flavour — a third axis

Arc 299's thesis is that `Impure` **fused effect and entropy**, and 299.3 splits it to
`Pure | Effectful | Entropic`. That trio is the three inhabited cells of purity.rs's existing 2×2
(pure × deterministic). **Totality is not a cell in that grid** — `first` is `Pure` by every measure the
trio can take. The grid asks *what does this touch?*; totality asks *is this defined on all its inputs?*

There are **two failure sources**, and the builder's own contribution to 299 (welding entropic to
*cannot-world-fault*) is what makes the split exact:

| | question | fails how | remedy |
|---|---|---|---|
| effect | touches the world? | **world-fault** | an outcome enum |
| entropy | same input → same output? | cannot fault | — |
| **domain** | **defined on all inputs?** | **domain-fault** | **an `:else`** |
| termination | does it halt? | doesn't return | — (see below) |

**And "total" was already a fusion of two of these.** The `total?` designed a month ago
(`NOTE-overlay-read-path` Part 5) was about **recursion/termination** — the seam records that it *would
not have caught* `first`-on-empty, because `first` halts fine and is simply undefined on an input. This
stone is the **domain** axis. Termination is a fourth, separate, and not in scope.

## Why no redesign is needed first

`purity.rs` is **already a record of named axes** — `OpMeta { pure, deterministic }`, one shared walk,
independent predicates. It has never been a flattened flavour. `total` is a third named field.

The flattening question belongs to `types.rs::Purity` (the 2→3-variant enum 299.3 widens, guarding **277**
`defenum` wire-purity markers). **Different population, different question, not a blocker here.**

And the hand-managed map is *already* the declared interim: its own doc says it is *"the explicit v1
projection of the queryable registry that arc 255 will eventually own… when 255 lands, delete this map."*
Adding an axis to it is using the scaffolding as designed, not erecting new scaffolding.

Measured cost: **7 `OpMeta` construction sites** (`purity.rs:101 :105 :113 :298 :317 :327 :347`) — the 110
verbs live in a single `matches!` arm at `:116` feeding one of them.

## The strike

1. `Axis::Total` beside `Pure` / `Deterministic`; `OpMeta.total`; `is_total_expr`;
   `eval_total_predicate` — each a mirror of the two that exist.
2. Register `:wat::rete::total?` beside its siblings (`check.rs:19227-19245`).
3. Split the `matches!` list: the partial verbs (`first`, `second`, `third`, `nth`, `i64::/`, `mod`,
   `rem`, `quot`, `Option/expect`, `Result/expect`) leave the total group.
4. A third conjunct at the fence (`rete.wat:563`), and — because of the strike ahead of this one — a
   message that **names the offending head**.

**Default-deny is the whole method.** Do NOT mass-assert `total: true` across the vetted block: those 110
were vetted for a *different* property, and carrying the claim over is the hand-audit stem the file's own
doc condemns. Everything unproven; run the gate; **the corpus enumerates itself**; classify only what a
live row demands. (Builder, on the namespacing wall: *"they self identify on enforcement"* — and there it
out-enumerated a grep that had been wrong four times.)

## ⛔ Enumerate first, arm last

A refused `first` with nowhere to go locks a user out of arithmetic. The builder's ruled remedy is
rete-namespaced **total variants with a mandatory fallback** — `(rete.i64// n 0 :else -1)`,
`(rete/first n :else …)` — with the partial forms disallowed in a `where`.

So the order is: **enumerate on a branch → mint the `:else` variants → migrate → arm.** Do not ship the
refusal before the destination exists.

## What this buys beyond the hole

R62 records that the corpus's **absolute** column — STOP-1 rejections, the half a peer cannot bound — is
**empty**: 98 rows, zero. Every green row says *we agree with Clara*; only a refusal is a fact about our
substrate alone. This axis is the first thing that would put entries in that column, filled by the
substrate's own answers rather than by hand.

## Owed before it lands

- The fence-names-the-head strike (`BRIEF-the-fence-names-the-head.md`) — hard prerequisite.
- **intueri on the `:else` variant names** — the builder ruled the shape, explicitly not the names.
- The mint list: which verbs get an `:else` sibling. **Allow-list, not deny-list.** Some totals already
  exist (`PersistentVector/get` + `match`, `foldl` with a seed).
