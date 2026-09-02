# DESIGN — a file whose every "in-engine" claim is about code the engine does not run

## Why

Work-list **C5**. `binding_repr_bench.rs` carries three defects and they are one class: the file
prices a representation decision using arms the engine does not use, anchors them to a figure whose
source measurement has collapsed, and guards the whole thing with an assertion that cannot fail.

### 1 — ★ THE FLOOR ASSERTION IS A TAUTOLOGY, AND ITS COMMENT SAYS WHAT IT SHOULD BE

`token_bindings_representation_dominance` (`:545`, **no `#[ignore]` — it is on the release floor**)
ends:

```rust
// The probe must have measured something; a zero here means it timed nothing.
assert!(
    extend_array_wins + get_array_wins < usize::MAX,
    "unreachable"
);
```

The comment declares the check — *a zero means it timed nothing* — and the assertion written under it
is true for every pair of `usize`. The stated check is `> 0`. This is the third time in this class
that a **declared** check sits above an unenforced one (C3's `unwrap_or(0)`, C6's `println!`).

### 2 — NEITHER ARM IS WHAT THE ENGINE USES

The test compares an **rpds trie** against **`Arc<[(Value, Value)]>`**. The engine uses neither:

```rust
// src/rete/kernel/session.rs:64
pub(crate) struct Token {
    pub(crate) matches: BindSpan,
    pub(crate) binds: BindSpan,
}
```

`HashTrieMapSync` survives only at boundary conversions (`session.rs:406,633,785`,
`fire/delta.rs:853`), not on the token-binding path. The comparison is still **evidence for** the
stone that chose `BindSpan` — `fire/delta.rs:725-726` records the premise, *"a binding map holds 1-2
entries, so an rpds trie is paying trie prices"* — but it is not a measurement of the live path, and
nothing in the file says so.

### 3 — THE "163 ns IN-ENGINE BIND" IS ATTRIBUTED TO CODE THE ENGINE NEVER RUNS, AND ITS SOURCE NOW MEASURES NEGATIVE

`bind_key_construction_vs_map_operation` (`:24`, also on the floor) apportions *"the ~163 ns
in-engine bind"* across three arms, attributing it to `eval_clause`'s
`Value::String(Arc::new(var.to_string()))`. Traced on the current tree:

- `eval_clause` ← `eval_clauses` ← `alpha_match_inner_opts` ← `alpha_match_inner{,_local,_seeded}`.
- `alpha_match_inner`: **0** non-test callers (3 test callers). `alpha_match_inner_local`: **0**.
  `alpha_match_inner_seeded`: **1**, at `compiled_cond.rs:1430` — inside `#[cfg(test)] mod tests`
  (opened at `:1373`).
- `compiled_cond.rs:912` states it outright: that function *"replaced `alpha_match_inner`"*.

**The whole matcher evaluation path is reached only from tests.** It is the retired interpreter, kept
as a differential oracle.

And the anchor figure is gone. `alpha_match_cost_per_binding` (`rank_and_instrument.rs:219`), the
named source of the 163 ns, driven at HEAD:

```
alpha cost per BINDING — 40000 facts, MINIMUM of 3
                 delta  : alpha   -0.87 ms ( -22 ns/fact)
```

**−22 ns/fact.** The binding cost is now below what the harness can resolve, so its source
measurement is negative. `163` is quoted four times in this file and printed in its table.

## The contract decision, pinned

**Every arm declares what it measures and whether the engine runs it; the floor assertion becomes the
non-vacuity check its own comment states.**

- The tautology becomes `extend_array_wins + get_array_wins > 0`, plus the orderings that must hold,
  with the whole table interpolated into the failure message.
- Both tests keep their arms and gain a header sentence naming the live representation (`BindSpan`)
  and the stone the comparison is evidence *for*.
- The `163 ns` anchor is dropped from the prose and the printed table. The header already says
  *"treat the RATIO between the three as the finding and not the absolute nanoseconds"* — the fix is
  to make the output obey that sentence.

Rejected: **deleting either test.** They are the evidence behind a landed design stone, exactly as
C6's `A/B/D/E` headroom arms were. Removing evidence to fix a label is the wrong direction.

Rejected: **re-measuring 163 ns and hard-coding the new value.** That is C6's defect re-created; the
figure is below the instrument's resolution and the ratio is the finding.

## Files

`src/rete/kernel/tests/binding_repr_bench.rs` only. Nothing under `src/rete/`.

## Out of scope = REJECTED

- **Reaping the retired matcher path.** `alpha_match_inner*` → `eval_clauses` → `eval_clause` is
  reached only from tests and may be a genuine `purgare` target — or a deliberately kept differential
  oracle. That question is its own row (**C13**), not this strike. Do not delete it here.
- C9, C10, C11, C12.
