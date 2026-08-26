# STONE A-i REPAIR — the cascade: two gates fired, both correctly

BRIEFED 2026-08-25 against `445b80cb6` + Stone A-i's uncommitted work in the tree.

**The floor is RED and this stone makes it green.** `5059 tests run: 5053 passed, 6 failed`.
Both failures are the substrate teaching, not a crisis — the diagnosis below is already done, so do
not re-derive it. **Do not re-run the floor**; you will not need it.

---

## FAILURE 1 — the rete purity completeness gate (1 test). It is RIGHT.

```
wat rete::purity::completeness_gate::every_dispatched_verb_is_classified_or_disposed
    PURITY COMPLETENESS — 544 dispatched verbs
      UNREVIEWED (the worklist)  250   ledger 233
      17  :wat::i64   e.g. :wat::i64::*, :wat::i64::+, :wat::i64::-, :wat::i64::/
```

**250 − 233 = 17**, exactly the verbs Stone A-i registered. The gate is DEFAULT-DENY by design —
*"a head's property holds only if PROVEN"* — so a newly dispatched verb with no ruling is unreviewed
debt. It caught a real gap in one run.

**The fix is a RULING, not a ledger entry.** The ledger is for *unreviewed debt*, and these are not
debt: they are the same pure, deterministic arithmetic their old spellings already carry a ruling
for. `src/rete/purity.rs:379-383` classifies `":wat::core::i64::+"`, `"::-"`, `"::*"`, `"::/"`,
`"::to-string"` and their siblings. **Add the 17 `:wat::i64::*` names to the same classification.**

⛔ Do NOT add them to the ledger to make the count balance. The gate's own text says the ledger
*"must shrink as the debt is paid"*; growing it to silence a gate is the opposite of paying.

---

## FAILURE 2 — five diagnostics goldens (5 tests). They pin a Rust LINE NUMBER.

```
wat::diagnostics probe_diagnostic_value_snapshot_in_errors::probe_{1,2,6,7,8}
```

The ONLY difference between actual and expected, in all five:

```
actual    :location #wat.core/Span {:file "src/runtime.rs" :line 25647 ...}
expected  :location #wat.core/Span {:file "src/runtime.rs" :line 25614 ...}
```

Stone A-i changed `src/runtime.rs` by **+125/−92 = net +33**, and **25614 + 33 = 25647**. It
reconciles exactly. Nothing about the diagnostics changed — every `:message`, `:producer`,
`:provenance`, and every `.wat` fixture span is byte-identical.

### ⛔ THE RECURRENCE — do not fix this the way it has been fixed four times

`tests/diagnostics/probe_diagnostic_value_snapshot_in_errors.rs` records this happening before, in
its own comments, at lines **67, 197, 227, 256**:

> *"only the internal src/runtime.rs span moved"*

Four occurrences, four golden updates, four notes explaining it away. **A house convention that has
become the mechanism.** `UPDATE_EDN=1` would make it five.

### The class fix

**A span pointing into the substrate's own Rust source is an implementation detail. A span pointing
into user `.wat` is the diagnostic's content.** Normalize the former; keep the latter exact.

In `src/lib.rs`'s `assert_edn_eq!` comparison path (the macro is at `src/lib.rs:231`), normalize —
on BOTH sides, before comparing — the `:line` of any `#wat.core/Span` whose `:file` ends in `.rs`.
**Keep the `:file` exact** (it still proves the error was raised from the expected module) and
**keep every `.wat` span exact, `:line` and `:col` both** (that is the user-facing location the
probes exist to assert).

Five goldens carry such a span; `git grep -l 'src/runtime.rs' --include=*.edn tests/` names them.
Once the normalizer is in, update those five goldens' `:line` values once more — after this they
stop mattering, which is the point.

⚠ Whatever normalization you choose must be visible in the failure message when a test DOES fail —
a normalizer that silently swallows a real difference is worse than the brittleness it replaces.

---

## Your role

cwd `/home/john/work/holon/wat-rs`; run `pwd` first. **Ending your turn ENDS you** — every command
FOREGROUND, blocking. **You may not spawn sub-agents.** Do not commit, push, stash, revert, or
`git checkout`; `git stash@{0}` must never be touched.

⚠ **The tree holds Stone A-i's uncommitted work** (`src/intrinsic/i64.rs`, `src/intrinsic/mod.rs`,
`src/runtime.rs`, two scratch-pad probes). It is correct and I have verified it. **Leave it alone** —
your changes are `src/rete/purity.rs`, `src/lib.rs`, the five goldens, and the stale comments.

You may run `cargo build --release` and **the six named tests only**:

```bash
cargo test --release --lib rete::purity::completeness_gate::every_dispatched_verb_is_classified_or_disposed
cargo test --release --test diagnostics probe_diagnostic_value_snapshot_in_errors
```

Not the floor, not clippy — the orchestrator measures those centrally.

## STOP triggers — each rejects

1. **STOP-1 — the 17 verbs are not all pure/deterministic.** If any i64 op does not deserve the
   ruling its old spelling carries, name it and why; ship the rest.
2. **STOP-2 — the normalizer cannot distinguish a `.rs` span from a `.wat` span** in the EDN
   structure. Report what you found; do not normalize both.
3. **STOP-3 — a room's line number does not hold what this brief says.** Written against `445b80cb6`
   plus the uncommitted Stone A-i work.

## Acceptance

```bash
# 1. the ratchet returns to its floor. BAR: the gate passes AND the ledger count did not GROW.
cargo test --release --lib rete::purity::completeness_gate::every_dispatched_verb_is_classified_or_disposed

# 2. all five probes green.
cargo test --release --test diagnostics probe_diagnostic_value_snapshot_in_errors

# 3. ★ BREAK THE DOOR — the normalizer must not be a blanket. Change one `.wat` span's :line in one
#    golden by hand, confirm the test goes RED and NAMES the difference, then restore.
#    A normalizer that swallows that is the failure this stone is preventing.

cargo build --release && cargo build --release --all-targets
```

## Report back with

- Each command's actual output.
- **The broken-door proof** for the normalizer: what you changed, the RED's verbatim text, proof it
  named the difference, and confirmation you restored it.
- The normalizer's code, and how it decides `.rs` vs `.wat`.
- The four stale comments at `:67/:197/:227/:256` — say what you did with them. They document a
  recurrence that should no longer be possible.
- Anything the brief got wrong. What you did NOT do, and why.
