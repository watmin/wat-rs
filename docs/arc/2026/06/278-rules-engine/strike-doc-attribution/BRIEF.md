# BRIEF — put each doc on its own variant, and make a broken link impossible to add

Three doc blocks stack onto `ReteCeiling::SessionMemoryCeilingExceeded`; two of them describe
`FixpointRoundCapExceeded` and `RuleSetMayNotTerminate`, which carry **no doc at all**. Two of the
links in those blocks were broken by E4 one commit ago and nothing noticed, because nothing in this
tree runs rustdoc. Split the docs onto their variants, fix `signal.rs`'s links, and add a gate that
compares unresolved links against a **named list**. Read `DESIGN.md` first — its ⚠ says why a count
ratchet is the wrong shape here, and its "out of scope" cuts three things including fixing all 50.

## Read in order

1. `src/value/signal.rs`, `pub enum ReteCeiling` (~`:220`) — the three stacked blocks and the two
   silent variants. This is the site.
2. The same blocks' cross-references to `[`RuntimeErrorKind::FixpointRoundCapExceeded`]` and
   `[`RuntimeErrorKind::SessionMemoryCeilingExceeded`]` — both now live on `ReteCeiling`.
3. `src/rete/kernel/outcome.rs` header — the wall's matchability doctrine, so you can put the
   *"names an action the author can take"* justification on the failure it actually justifies.
4. `src/rete/purity.rs`, the `KNOWN_UNREVIEWED` doc — **the ratchet shape to copy**, including its
   own account of why a cardinality check failed.
5. `tests/lint/no_stale_path_in_doc.rs` — an existing doc-checking gate in this tree; its file
   walking and reporting style are the model.

## Sketch

```rust
pub enum ReteCeiling {
    /// A `fire-rules` round boundary found the session past `max-session-bytes`. …
    SessionMemoryCeilingExceeded { … },
    /// `insert` / `insert-all` grew its session past `max-session-bytes`. …
    SessionMemoryCeilingExceededOnInsert { … },
    /// A rule set that cannot be proven to terminate — refused at `compile-all`… (block 2)
    RuleSetMayNotTerminate { … },
    /// The cascade fixpoint ran past its round cap… (block 1)
    FixpointRoundCapExceeded { … },
}
```

The gate: run rustdoc with `-W rustdoc::broken_intra_doc_links`, parse the unresolved-link
diagnostics, compare the set against a `KNOWN_BROKEN_DOC_LINKS` list. **Not in the list → RED. In
the list but now resolving → RED.**

## Blast radius

`src/value/signal.rs` + one new file under `tests/lint/`. The other ~41 links are **seeded, not
fixed**.

## Traps named in advance — each with its step

1. **★ Rust doc comments accumulate onto the NEXT item.** That is the whole defect; do not
   reproduce it. **Step:** after splitting, run `cargo doc --no-deps` and confirm each of the four
   variants renders its own paragraph — that is the only check that sees attribution.
2. **A named list, never a count.** DESIGN's ⚠ and `purity.rs`'s own record. **Step:** entries are
   `(file, link-target)` or the diagnostic's own identifying text; a bare number is the defect this
   tree already paid for.
3. **The gate must run rustdoc, which is slow and not nextest-shaped.** **Step:** check how long
   `cargo doc --no-deps --workspace` takes (it was under the 900s timeout for me) and decide whether
   the gate shells out or parses a committed artifact. If shelling out makes the floor materially
   slower, **say so and propose the alternative** rather than shipping a slow floor.
4. **~41 seeded entries must be reproducible.** **Step:** record the exact command that produced
   them beside the list — `RUSTDOCFLAGS="-W rustdoc::broken_intra_doc_links" cargo doc --no-deps
   --workspace`. A list with no instrument is the failure this arc logged twice.
5. **Two of the nine `signal.rs` links are E4's.** **Step:** fix all nine; note which two were
   introduced one commit ago, because that is the evidence the gate is needed.
6. **New test code trips `wat::lint`.** **Step:** run `binary_id(wat::lint)` before reporting.

## STOP triggers

- **STOP-1** — if the gate cannot run rustdoc within the floor's budget, STOP and report the timing
  with your proposed alternative. A slow floor is a floor people stop running.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if the unresolved-link set is not stable between two runs, STOP and say so. A gate
  over a non-deterministic set is worse than none.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-ceiling-closed-set/` — the strike that broke these two
links, one commit ago.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Twenty riders before you each returned a prescription of mine
that did not survive contact. The last found my sketch would have **defanged a live gate** by
renaming the strings it matches on. If a step here is wrong, unnecessary, or impossible, say it
plainly.
