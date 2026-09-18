# DESIGN — `root_join_delta`: two costs per element, and I will not guess which is live

> Drawn 2026-09-06 at HEAD `6b4a9fa86`. Source: vigilia 2026-09-05 `temperare` §2.
> **Shape verified on disk. Quantities are the strike's first job.**

## Why this DESIGN asserts no counts

Two `temperare` rows have now been measured and **my reading was wrong both times, in opposite
directions**:

| | my reading | driven |
|---|---|---|
| §1 `join_extend` | 3 lookups per pair | **1** — the span is skipped when the element carries binds |
| §4 `col_field_of` | "probably vacuous" | **1.00 per element** — occupancy *is* empty binds |

Both hinged on the same question — *which population reaches the branch* — and I got it backwards
each time. §2 hinges on it again, so the counter decides, not me.

## The site

`fire/pass/root_join.rs:59-79`, nest `for node_id in &arm.kind_ids.alpha` → `for child_id` →
`for ei in news`:

```rust
let binds = if el.binds.len > 0 { seed_token_binds(&el) }
            else { span_from_row(&mut wm.bind_pool, &el, *node_id, …, &wm.bind_only, &wm.cond_key_ids) };
…
record_token(&mut wm.beta, d_beta, &arm.beta_readers, *child_id, tok);
```

**Two independent costs, and they answer to different loops:**

1. **`span_from_row`** — two `HashMap` lookups keyed on `*node_id`, the **outermost** loop variable.
   **Conditional** on `el.binds.len == 0`. `temperare` claimed every batched leaf element takes this
   branch (`alpha.rs`'s `make_element(idx, 0, 0)`). **That is the claim I have been wrong about
   twice. It is question one.**
2. **`record_token`** — `beta_readers.contains` + `beta.entry` + `d_beta.entry`, three hash ops on
   `*child_id`, the **middle** loop variable. **Unconditional, once per element.** This one does not
   depend on any population question.

## ★ The batched door already exists, and its doc names this cost

`record_tokens` (`fire/pass/mod.rs`) is `record_token`'s batch twin. Its doc:

> *"a join can emit a whole bucket at once, and **growing a `Vec` one token at a time inside the fire
> loop is the cost that `reserve` exists to avoid**."*

`hash_join.rs` uses it. Root-join — the most element-dense loop in the engine — calls the singular
form per element. **Seventh time in this arc the tree stated the rule and applied it in one place**,
and the sixth was `production_delta`, cured two strikes ago with this same buffer-then-flush shape.

## The one contract decision, pinned

**Instrument both costs separately, drive, then cure what the numbers justify.**

- **Cost 2 needs no permission** — it is unconditional and the batched door exists. Buffer the
  tokens per `(node_id, child_id)` and flush once through `record_tokens`. Predicted: three hash ops
  per `(node, child)` pair, not per element.
- **Cost 1 is decided by the count.** If `span_from_row` runs on a meaningful fraction, hoist its
  two `*node_id` lookups to the node scope. If it is rare, **say so and leave it** — and that half
  of §2 is refuted, which is a result.

⛔ **NO WALL-CLOCK CLAIM.** Counts only. The grid is a regression check, not evidence.

## ⛔ Order is the risk, and `root_join` is where it bites

`production_delta` was safe to buffer because nothing read `wm.production` mid-nest — I verified
`encode_view`'s field list to establish it. **Here the buffer holds `Token`s destined for `wm.beta`
and `d_beta`, which later passes read.** Buffering must not change the sequence tokens land in, and
`push_match` mutates `wm.match_pool` per element in the same loop.

That is STOP-1, and it is the one thing that would make this strike not worth doing.

## Scope

**IN:** two counters, the readings, the batch cure, and — only if earned — the span hoist. Floor GREEN.

**OUT, affirmatively cut:** `temperare` §5 (`ensure_gather`); the FxHashMap question; census B, D–M;
A4, D2p, F2.
