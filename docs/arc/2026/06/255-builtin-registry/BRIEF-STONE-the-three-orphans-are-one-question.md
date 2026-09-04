# BRIEF — STONE: register the three remaining bare type constructors

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Any path containing `.claude/worktrees/`
is harness state — never operate on it. Do not commit, push, stash, or revert. Do not run the full
floor; the orchestrator runs it centrally.

Read `DESIGN-STONE-the-three-orphans-are-one-question.md` (sibling) first.

## The work in one paragraph

`:wat::core::Vector`, `:wat::core::HashMap` and `:wat::core::HashSet` are the last three bare type
constructors with no registry row, while `List`, `PersistentVector`, `PersistentMap` and `Tuple` —
same shape — were registered by earlier stones in this campaign. Register the three. Their
`intrinsic_meta` residue entries then become shadowed and the 1c-c gate will demand their deletion.
`cond` and `reduce`, the other two orphan targets, are out of scope: no mechanism exists to register
a stdlib macro or a wat-side `defalias`.

## Rooms, in order

1. **`src/runtime.rs`, `:wat::core::PersistentVector`'s registration** — the shipped precedent, same
   file, same shape. Read its whole doc block before writing anything: the prose, the per-axis
   ground paragraphs, and the `@arg`/`@ret`. This is what you are mirroring.
2. **`src/intrinsic/list.rs`, `:wat::core::List`** — a second precedent, a different file, and worth
   comparing: where the two differ, the difference is informative about what a constructor's row
   must carry.
3. **The three handlers themselves** — find where `Vector`/`HashMap`/`HashSet` are dispatched today
   and annotate in place if the signature fits `NativeHandler`; add a thin delegate only if it does
   not, the way `quote`/`forms` needed one.
4. **`src/rete/purity.rs`, `intrinsic_meta`'s `pure_det` list** — all three are in it. They become
   unreachable once registered (the registry consult answers first). Delete them and leave a
   departure marker in the house style already used there for `reduce`/`str`/`u8`/`do`.
5. **The same file's comment** calling bare type constructors "unhomed" / "stays unhomed" — it
   describes a rule this campaign stopped following four stones ago. Correct it, naming the
   registered siblings.

## ⛔ MEASURE EACH AXIS — do not copy PersistentVector's

This is the one way this stone can ship a lie. Each of the three gets its **own** argument for each
of `@Purity`/`@Determinism`/`@Totality`/`@ExpandTime`/`@Category`, grounded in **its own**
constructor's implementation, cited by symbol. This arc has corrected four wrong `@Totality` grades
that came from reasoning-by-resemblance rather than reading the code.

In particular do not assume `Total`. Read each constructor's body and ask what it does on a bad
argument — a capacity overflow, a non-hashable key, an odd-length key/value list. If it can raise on
an input the checker admits, the honest grade is `Partial`, and saying so is worth more than a
matching set of `Total`s.

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — if a constructor's axes cannot be argued from its own implementation (the handler is
  shared, the behaviour depends on a caller, the code does not decide it), STOP and report which
  axis and why. Do not grade by resemblance to `PersistentVector`.
- **STOP-2** — do not touch `:wat::core::cond` or `:wat::core::reduce`. Neither can hold a registry
  row today; attempting one is the FOURTH-registry fork, which this stone does not open.
- **STOP-3** — do not edit `RETE_OPS`, its `core_name` fields, or `no_dangling_or_chained_aliases`.
  Registering `Vector` clears one orphan as a consequence; nothing in `src/rete/vocabulary.rs`
  needs a hand.
- **STOP-4** — if the residue gate does NOT demand the three deletions after you register them,
  STOP and report. That gate is how this stone proves the rows are really shadowed, and a silent
  pass means the registration did not take.

## Verification

```
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::collection)'
cargo nextest run --release -E 'binary_id(wat::types)'
cargo nextest run --release -E 'binary_id(wat::rete)'
cargo clippy --release --all-targets -- -D warnings
```

Then re-run the campaign census and report the new row count:

```
cargo build --release --bin wat
./target/release/wat wat-scripts/scratch-pad/255-registry-census.wat
```

## What to report

The five `@` axis lines for each of the three, with the symbol each was grounded on; whether any
came back `Partial` and why; the residue gate's demand before you satisfied it (verbatim); the
census's row count before and after; the Summary line per scoped run; and anything that surprised
you.
