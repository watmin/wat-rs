# ★ RULING (builder, 2026-08-15) — 255 OWNS the purity domain. rete measures COMPOSITION, nothing else.

> *"rete's purity needs to be rehomed eventually.... it belongs in 255.... uhm..... clean up the
> constant in rete for now... we'll figure out what to do with that later... **255 owns this problem
> domain.. the registry we're building must be the thing we consult for these questions.... rete's
> needs to be reduced to just measuring if the forms used in the rete expressions are composed from
> rete primitives (who are pure, deterministic and total; language level thing, not a rete
> thing...)**"*

## The cut, stated plainly

There are **two questions**, and `src/rete/purity.rs` currently answers both. It should answer only
the second.

| the question | who owns it | today |
|---|---|---|
| **"Is `:wat::foo::bar` pure / deterministic / total?"** — a property of a NAME | **the registry (255)** — a *language-level* fact | ⛔ a hand-maintained map in `rete/purity.rs` |
| **"Is this rete expression composed ONLY of rete primitives?"** — a property of a FORM | **rete** | ✓ correct, and this is all it keeps |

**Purity is a language-level property, not a rete property.** A verb's purity is true whether or not
rete ever sees it — which is exactly why it cannot live in rete's file. rete's legitimate job is the
*composition* check: walk the forms in a `where` / `:test` / accumulator and confirm every head is a
rete primitive. The primitives are pure ∧ deterministic ∧ total **by construction of the language**;
rete asserts membership, not properties.

## What this rules ON — the arc's own decomposition sharpens

`DESIGN.md`'s **255.3 — consumers collapse** already prescribed the deletion:

> *"`src/rete/purity.rs` (379 lines, `:wat::rete::pure?`/`deterministic?`) DELETES → queries the
> baseline. `macros::is_pure_total` (eval.rs:344) DELETES → queries `:expand-time-legal`.
> `runtime::is_effectful_op` (runtime.rs:22731) becomes the registration-time DERIVER that POPULATES
> `:pure`. rete/macro-gate/checker all QUERY the registry; 'rete just calls this.'"*

**The ruling confirms 255.3 and sharpens it in one way the design did not say:** `rete/purity.rs`
does **not** delete wholesale. It **splits** —

- the **name→property tables** (`intrinsic_meta`, the `RULES` namespace dispositions, the
  `KNOWN_UNREVIEWED` ledger, and the `completeness_gate` that polices them) → **rehome to 255's
  registry.** These are language facts.
- the **composition check** (are these forms built from rete primitives?) → **stays in rete**, and
  becomes rete's whole purity surface.

## The measured evidence that forced it, this session

Carving home #2 (`255.1c-time`) put 41 verbs into the registry with declared purity/determinism, and
**three separate hand-maintained structures reacted or failed to react** — the exact fragmentation
the ruling ends:

1. **`rete/purity.rs` `KNOWN_UNREVIEWED`** — the floor went RED at
   `purity.rs:2247`: *"41 verb(s) in `KNOWN_UNREVIEWED` are no longer unreviewed."* A **good** red —
   the ledger freezes NAMES (`[[feedback_a_gate_freezes_names_never_a_count]]`), so it named all 41
   precisely. Cleaned per this ruling; debt 214 → 173.
2. **`rete/purity.rs` `RULES`** — still carries a `:wat::time::` namespace disposition
   (`purity.rs:1883`, *"MIXED — `now` reads the clock … but `epoch-nanos` … is a pure read. Needs
   per-verb review"*). **The registry has now answered exactly that question, per verb**, from the
   handler bodies: **17 Nondeterministic, 24 Deterministic**. The hand-note asking for the review is
   obsoleted by the review having happened elsewhere.
3. **`runtime::derive_pure_deterministic` (`runtime.rs:25255`)** — a *third* hand-derivation, whose
   `NONDETERMINISTIC` residual list contains **only** `:wat::core::Uuid/v4`. It will now report
   `deterministic = true` for all 17 genuinely non-deterministic `:wat::time::` rows — **actively
   contradicting the registry.** Found by the 255.1c rider; deliberately not fixed (out of its
   scope, and this ruling is why).

**Three tables, one question, and after one home they no longer agree.** That is the asymmetry 255
exists to annihilate, observed rather than predicted — and it could not have been observed on home
#1, because every `core::Bytes` row is `Pure`+`Deterministic` and so nothing could disagree.

## Sequencing — this is NOT a new stone, it is 255.3's shape

Nothing here reorders the arc. The carve continues (more homes → `1b-iv` deletes the blanket-accept);
`255.3` collapses the consumers and now has its cut defined in advance rather than discovered during
the strike. **Until 255.3 lands, the three tables stay divergent and that divergence is KNOWN, not
silent** — which is the honest state, and the reason it is written here.

⚠ **One thing this ruling does NOT decide:** whether `rete/purity.rs`'s `completeness_gate` (the
ratchet that catches unclassified dispatched verbs) rehomes with the tables or is retired outright.
Once the registry is the single truth, "unclassified" becomes structurally impossible — a builtin
cannot register without answering, per the LOCKED baseline's forced minimum. The gate may therefore
be scaffolding that dissolves rather than moves. **Not decided here; 255.3's question.**
