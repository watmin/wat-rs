# BRIEF — STONE total-T4b: derive `total` from the registry; leave an 11-name backlog

Read `DESIGN-STONE-total-t4b-the-fence-reads-the-registry.md` first.

## The work, one paragraph

`intrinsic_meta` computes `total` with a 38-name `matches!`. Twenty-seven of those verbs now
declare the answer at their own registration site. **Make the lookup consult the registry, and cut
the `matches!` down to the 11 verbs that have no registration to consult.** First, correct `if` and
`let` to `@Totality Preserving` — their doc blocks are internally inconsistent today.

## Read in order

```
src/rete/purity.rs, the `── `total` —` block   the 38-name matches! and its per-op reasoning.
                                               The reasoning for the 27 has MOVED (T4a) and its
                                               copy here goes with them; the 11's stays.
src/rete/purity.rs, end of intrinsic_meta      `Some(OpMeta { pure: true, deterministic: true, total })`
src/intrinsic/mod.rs:1038                      `matches!(entry.purity, Pure | Preserving)` — the
                                               house convention that Preserving SATISFIES an axis
src/intrinsic/special/control_flow.rs          `if` — @Purity Preserving, @Totality Total. Inconsistent.
src/intrinsic/special/binding.rs               `let` — same.
src/intrinsic/mod.rs, `registry()`             `pub(crate)`, so purity.rs can call it
```

## Implementation sketch

```rust
// FIRST: if/let become @Totality Preserving (doc-line change only).

// THEN, in intrinsic_meta, replacing the 38-name matches!:
let total = match crate::intrinsic::registry().lookup_entry(head).map(|e| e.totality) {
    Some(wat_doc::Totality::Total) | Some(wat_doc::Totality::Preserving) => true,
    Some(wat_doc::Totality::Partial) => false,
    // No registration to consult: the verb is not homed yet. These eleven keep their
    // ruling here until they have a site to carry it.
    Some(wat_doc::Totality::Unreviewed) | None => matches!(head,
        ":wat::core::map" | ":wat::core::mapv" | ":wat::core::filter"
        | ":wat::core::foldl" | ":wat::core::reduce"
        | ":wat::core::=" | ":wat::core::not=" | ":wat::core::and"
        | ":wat::core::or" | ":wat::core::not" | ":wat::core::bool::to-string"
    ),
};
```

⚠ **`purity.rs` does not import `crate::intrinsic` today** — measured. Adding that dependency is
expected and fine; report if it causes a cycle.

★ **Leave a comment on the residual `matches!` explaining what it now is.** A reader finding an
unexplained 11-name list will assume it is the hand-list this stone replaced. It is a *backlog*:
each name is unhomed, and homing it retires the row.

## ⛔ THE INVARIANT — the verdicts do not move

For every one of the 38, `total` must come out **exactly as it does today**. Prove it, do not assume
it: capture all 38 verdicts before, and all 38 after, and diff them.

## Blast radius

`src/rete/purity.rs`, `src/intrinsic/special/control_flow.rs`, `src/intrinsic/special/binding.rs`.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **Any of the 38 verdicts changes.** That is a transcription or mapping defect. STOP and name the
   verb and both verdicts.
2. **The residual `matches!` needs a 12th name** to keep the verdicts stable. It means a verb the
   design believes is registered is not, or an answer did not land in T4a. STOP and name it.
3. **You are about to answer `@Totality` for a verb outside `if`/`let`.** Not this stone. STOP.
4. **You are about to touch `is_pure_total` or `RETE_OPS`.** Other consumers, other stones. STOP.
5. **The `where`-corpus goes red.** It is the live consumer. STOP with the failure verbatim.

## Acceptance

```
 0. ★ CAPTURE THE BASELINE FIRST: all 38 verbs' `total` verdicts BEFORE any edit. A tiny temp test
      over `intrinsic_meta` is fine. Report the 38, or at minimum the count of true/false.
 1. ★ if AND let READ `@Totality Preserving`, and their other two axes are untouched.
 2. ★ THE DERIVATION IS IN, and the residual `matches!` holds EXACTLY the 11 named above.
 3. ★ ALL 38 VERDICTS IDENTICAL to row 0. Diff them and say so explicitly.
 4. ★ PROVE THE REGISTRY IS ACTUALLY BEING CONSULTED: flip one of the 27 (say `:wat::i64::>`) to
      `@Totality Partial`, show `intrinsic_meta(":wat::i64::>").total` become FALSE, restore. If the
      verdict does not move, the derivation is not wired and the residual list is carrying it.
 5. ★ THE 11 ARE GENUINELY UNREGISTERED: show that `registry().lookup_entry()` returns None (or
      Unreviewed) for each. If one IS registered with an answer, it does not belong in the residue.
 6. ★ THE WHERE-CORPUS IS GREEN. Name how you ran it.
 7. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 8. cargo nextest run --release -E 'test(rete) + test(purity) + test(intrinsic) + test(where)'
```

★ **Row 4 is load-bearing.** Rows 1–3 are consistent with a derivation that is wired but never
reached, because the residual list happens to give the same answers. Only flipping a registry
declaration and watching the verdict follow proves the registry is the source.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

The row-0 baseline and the row-3 diff, explicitly. `if`/`let`'s new declarations. The residual
`matches!` quoted in full with its comment. Row 4's flip, the verdict it produced, and the restore.
Row 5's evidence for all 11. How you ran the `where`-corpus. Then the honest deltas — especially
any verb where the registry's answer and the old list DISAGREED, because that is a transcription
error we need to know about.
