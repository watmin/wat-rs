# BRIEF — STONE expand-T4b: annotate two, then derive; leave a named backlog

Read `DESIGN-STONE-expand-t4b-the-gate-reads-the-registry.md` first.

## The work, in order — and the order matters

**1. Annotate `:wat::hashmap::keys` and `:wat::hashmap::values` as `@ExpandTime Legal`.**
They are blessed by the predicate but declare `Unreviewed`; T4a's rider withheld them because a
stale comment claimed they had been removed. That comment is corrected now and states the reasoning
you should carry: *a `HashMap` is pure data and `keys` is a pure projection — the same map yields
the same SET every time, and only the ORDER is unspecified.* See `src/macros/eval.rs`, the block
above their arms.

**2. Then derive.** Replace the 202-name `matches!` with a registry lookup whose fall-back holds
only the allow-list's **unregistered** names.

⛔ **Do step 1 first.** Deriving without it flips `keys`/`values` to refused, which makes
`:wat::core::format` undefinable and takes ~247 tests red. That is measured, not hypothetical.

## Read in order

```
src/macros/eval.rs  fn is_expand_time_legal   the 202-name matches! and its family group comments
src/macros/eval.rs  ~line 605                 keys/values' arms and the corrected comment above
src/intrinsic/hashmap.rs                      where the two annotations go
src/rete/purity.rs  fn intrinsic_meta         ★ THE MODEL — total-T4b did exactly this for
                                              totality: registry first, residue behind
src/intrinsic/mod.rs  registry()              pub(crate); eval.rs may call it
```

## Implementation sketch

```rust
fn is_expand_time_legal(head: &str) -> bool {
    if let Some(e) = crate::intrinsic::registry().lookup_entry(head) {
        return matches!(
            e.expand_time,
            wat_doc::ExpandTime::Legal | wat_doc::ExpandTime::Preserving
        );
    }
    // No registration to consult: the verb is not homed yet. These keep their blessing
    // here until they have a site to carry it — a HOMING BACKLOG, not a hand-list.
    matches!(head, /* the unregistered residue */)
}
```

★ **Derive the residue; do not hand-pick it.** A name stays iff `lookup_entry` returns `None`.
Write a temporary test that partitions the 202 and work from its output. The design says 59 — that
is a prediction to check.

## ⛔ THE INVARIANT

**Every name's verdict identical before and after.** Capture the predicate's answer for every
registered entry AND every name in the list, before touching anything; diff after. After step 1,
`VERDICT_FLIPS` must be **0**.

## Blast radius

`src/macros/eval.rs` and `src/intrinsic/hashmap.rs`.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **Any verdict moves after step 1.** The derivation changes where the answer comes from, never
   the answer. STOP and name the verb and both verdicts.
2. **`:wat::core::format` fails to define.** The canary. STOP with the failure verbatim.
3. **The residue needs a name that IS registered.** It means an answer did not land in T4a. STOP
   and name it.
4. **You are about to rule any of the 288 `Unreviewed` verbs.** Not this stone. STOP.
5. **You are about to keep a "which verbs derive" name-list.** The residue is defined by
   `lookup_entry == None` and nothing else. STOP.

## Acceptance

```
 0. ★ BASELINE FIRST: the predicate's verdict for every registered entry and every listed name,
      captured BEFORE any edit. Report how many names you captured.
 1. ★ keys AND values read `@ExpandTime Legal`, each carrying the reasoning from eval.rs's
      corrected comment. Quote both paragraphs.
 2. ★ VERDICT_FLIPS = 0 after step 1, measured. State it.
 3. ★ THE DERIVATION IS IN, and the residue holds exactly the unregistered names. Report the
      count against the design's 59.
 4. ★ ALL VERDICTS IDENTICAL to row 0. Diff and say so explicitly.
 5. ★ THE RESIDUE IS EXPLAINED — quote the comment you wrote. It must say what the list now IS:
      a homing backlog whose rows retire as their verbs get homes.
 6. ★ EVERY RESIDUE NAME IS GENUINELY UNREGISTERED — show `lookup_entry` is `None` for each.
 7. ★ PROVE THE REGISTRY IS THE SOURCE: flip one registered verb's `@ExpandTime` to
      `RuntimeOnly`, show the predicate return `false` for it, restore. Pick a verb that was NOT
      in the residue, so the old path cannot be what answered.
 8. ★ `:wat::core::format` still defines — name how you checked.
 9. cargo build --release --all-targets — clean; warnings VERBATIM if any.
10. cargo nextest run --release -E 'test(macro) + test(stdlib) + test(intrinsic) + test(reflection)'
```

★ **Row 7 is load-bearing.** Rows 3–4 pass equally well on a derivation that is wired but never
reached, if the residue happens to give the same answers. Only flipping a declaration and watching
the predicate follow proves the registry is the source.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Your baseline size. Both annotation paragraphs. `VERDICT_FLIPS` after step 1. The residue count
against 59, and its comment quoted. The row-4 diff, explicitly. Row 6's evidence. Row 7's flip, the
verb you chose and why, and the restore. How you verified `format`. Then the honest deltas —
especially any name whose registered answer and listed membership DISAGREED, since each of those is
a ruling made twice and differently.
