# DESIGN-STONE — a doc comment lands on the next item, whatever it describes

> **Origin (2026-09-01).** Class **E3**, the last Class E row, found by `conformare`. Driven at HEAD
> `7ead5953e`. **E4 moved these four variants one commit ago, so the row's line numbers are stale and
> its damage is now larger — re-read at the new site, which is what this stone does.**

## Why — three blocks, one variant, two silences

`signal.rs`'s `ReteCeiling` (post-E4) opens with **three stacked doc blocks**, and Rust accumulates
them all onto the next item:

1. *"The cascade fixpoint ran past its round cap — the rule set does not terminate."* → describes
   **`FixpointRoundCapExceeded`**
2. *"A rule set that cannot be proven to terminate — refused at `compile-all`…"* → describes
   **`RuleSetMayNotTerminate`**
3. *"A `fire-rules` round boundary found the session past `max-session-bytes`."* → describes the
   variant they actually land on

All three render on **`SessionMemoryCeilingExceeded`**. Driven: `RuleSetMayNotTerminate` and
`FixpointRoundCapExceeded` carry **no doc at all**. So the two failures with the most to explain —
a diverging rule set and a refused compile — are undocumented, while a third variant carries a
paragraph about each of them.

The row's sharpest half: the wall's justification for the terminate verdict being **matchable**
(*"its diagnostic names an action the author can take"*) is attached to a **different failure**.

## ⛔ AND E4 — MY OWN STRIKE, ONE COMMIT AGO — BROKE TWO LINKS HERE

Those blocks cross-reference `[`RuntimeErrorKind::FixpointRoundCapExceeded`]` and
`[`RuntimeErrorKind::SessionMemoryCeilingExceeded`]`. **E4 moved both onto `ReteCeiling`**, so
neither path resolves. Driven: `sed -n '/pub enum RuntimeErrorKind/,/^}/p' | grep -c` → **0**.

Nothing caught it. Clippy passed, the floor passed — **broken intra-doc links are a rustdoc lint**,
and:

```
grep -rn 'cargo doc\|rustdoc' scripts/*.sh .github/workflows/*.yml   → nothing
grep -rn 'broken_intra_doc_links' src/lib.rs Cargo.toml              → nothing
```

**Nothing in this tree ever builds docs, and the lint is not enabled.** Every intra-doc link here is
unverified, which is the same shape as the `file:line` citation-rot row filed during E5.

## The measurement, before prescribing the cure

`RUSTDOCFLAGS="-W rustdoc::broken_intra_doc_links" cargo doc --no-deps --workspace` →
**50 unresolved links**, `signal.rs` worst at **9**, then `wat-reader/parser.rs` 4, `resolve/mod.rs`
3, `load.rs` 3, `kernel/address.rs` 3. So `deny` today reddens the build in 50 places across
subsystems this arc does not own.

## ★ THE ONE CONTRACT DECISION

**The doc that describes a variant sits on that variant, and a link that names a path is checked.**
The three blocks are split onto their own variants; the matchability justification moves to the
failure it justifies; and a gate runs rustdoc and compares the unresolved links against a **NAMED
LIST**, seeded with what remains.

⚠ **A NAMED LIST, NOT A COUNT — this tree has already paid for that distinction.** `purity.rs`'s
`KNOWN_UNREVIEWED` doc records the exact failure: *"the gate wanted SET MEMBERSHIP and measured
CARDINALITY… a brand-new unruled verb walked in free whenever a strike also ruled on one."* A count
ratchet here would let a new broken link in every time someone fixes an old one. The list is a
ratchet **in both directions**: an entry not in it is RED, and an entry in it that now resolves is
RED.

## Blast radius

`src/value/signal.rs` (the attribution + its 9 links), and one new gate under `tests/lint/`. **The
other ~41 links are seeded into the list, not fixed** — see the cut.

## Out of scope — AFFIRMATIVELY CUT

- **Fixing all 50.** They span `wat-reader`, `resolve`, `load`, `address`, `types` — subsystems this
  arc does not own, and a 50-site doc sweep is not a rete strike. **Seed them by name**; the ratchet
  makes the set shrink-only.
- **`#![deny(rustdoc::broken_intra_doc_links)]`.** That is the endpoint once the list is empty, not
  the opening move; denying today reddens the build in 50 places.
- **The `file:line` citation-rot row.** Same family, different mechanism (rustdoc cannot see a line
  number in prose). Still its own row.
