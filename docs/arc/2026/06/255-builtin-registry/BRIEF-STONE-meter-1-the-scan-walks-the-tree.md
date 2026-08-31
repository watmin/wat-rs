# BRIEF — STONE meter-1: make the completeness scan recursive, ledger the eleven

Read `DESIGN-STONE-meter-1-the-scan-walks-the-tree.md` first.

## The work, one paragraph

`dispatch_verbs` (`src/rete/purity.rs:2613`) finds `#[wat_intrinsic]` registrations by
`read_dir(".../src/intrinsic")`, files plus one subdirectory level. **Make it walk `src/`
recursively.** Eleven verbs then enter the population with no ruling; give each a
`KNOWN_UNREVIEWED` row carrying its own reason. Nothing else changes.

## Read in order

```
src/rete/purity.rs:2613-2690   `dispatch_verbs` — the union, and the read_dir to replace
src/rete/purity.rs:~2696       the NON-VACUITY assert (`verbs.len() > 400`). ⚠ It fired during
                               the orchestrator's probe when a naive root change lost 146 verbs.
                               It must still pass, and you must NOT lower its floor.
src/rete/purity.rs:~2705       the disposition loop: intrinsic_meta -> RULES -> unreviewed
src/rete/purity.rs:2267        `KNOWN_UNREVIEWED` — where the eleven rows land
src/intrinsic/mod.rs:988       255.1c's ruling on why a gate must not read a copy of the truth
```

## Implementation sketch — probed, not guessed

```rust
// replaces the read_dir over /src/intrinsic. Descends the whole tree.
fn walk(dir: std::path::PathBuf, out: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(&dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() { walk(p, out); continue; }
        if p.extension().is_some_and(|x| x == "rs") {
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            for line in text.lines() {
                if let Some(rest) = line.trim_start().strip_prefix("#[wat_intrinsic(\"") {
                    if let Some(j) = rest.find('"') { out.push(rest[..j].to_string()); }
                }
            }
        }
    }
}
walk(std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")), &mut out);
```

This exact shape was run by the orchestrator and produced the eleven below. Improve it if you see
better; keep the behaviour.

## The eleven, and the reason each gets

```
:wat::form::matches?
:wat::rete::arm-session          :wat::rete::export
:wat::rete::release-session      :wat::rete::import
:wat::rete::collect-rules        :wat::rete::lower
:wat::rete::eval-insert          :wat::rete::step-payload
:wat::rete::eval-test            :wat::rete::axis-violation
```

**Every one already declares `@Purity` AND `@Determinism`, and every one declares
`@Totality Unreviewed`.** Verify that yourself — it is the justification for the ledger row. The
reason each row carries is that the FENCE's question is three axes and the third is open:
the verb's purity is ruled and on disk, its totality is not.

⛔ **Do NOT transcribe a verb's `@Purity` into `intrinsic_meta`.** That answers a three-axis
question with a two-axis answer and invents a `total` verdict nobody made. It is also the exact
shape 255.1c retired. A ledger row is the honest disposition here; a fabricated classification is
not.

★ Write each row's reason from **that verb's own declaration**, not from a template. Eleven
identical strings would be a template wearing a reason's clothes.

## Blast radius

`src/rete/purity.rs` only.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **The non-vacuity assert fires, or you are tempted to lower its 400 floor.** It is the guard
   that caught the orchestrator's own bad fix. STOP.
2. **More or fewer than eleven verbs newly need a disposition.** The design predicts exactly these
   eleven by name. A different set means something moved and the design is stale. STOP and report
   the difference.
3. **You are about to classify a verb in `intrinsic_meta`** rather than ledger it. STOP.
4. **You are about to touch `effectful_by_prefix`, `RULES`, or any verb's `@Purity`/`@Totality`.** STOP.
5. **The `stale` assert fires** (a ledger row for a verb no longer unreviewed). Report which.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: population before, and after the walk change, by running the gate.
      Report both, and the eleven names it reports — compare against the design's list.
 1. ★ THE ELEVEN LEDGERED, each with its OWN reason citing its own declaration. Quote all eleven
      rows in your report.
 2. ★ CONFIRM the premise: all eleven declare @Purity, @Determinism, and @Totality Unreviewed. Show
      the command and its output.
 3. ★ KNOWN_UNREVIEWED 217 -> 228, by the gate's own printed line.
 4. ★ THE NON-VACUITY ASSERT STILL PASSES, unmodified. State the population it saw.
 5. ★ BREAK THE DOOR: delete ONE of your eleven rows, show the gate go red naming that verb,
      quote it, restore. A ledger nobody proved can fail is a list.
 6. ★ `git diff --stat` shows src/rete/purity.rs ONLY.
 7. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 8. cargo nextest run --release -E 'test(purity) + test(rete) + test(intrinsic)'
```

★ **Row 5 is load-bearing.** The whole point of this stone is that a verb can no longer hide from
the meter — so prove the meter now catches one being removed.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.

## Report back with

Both populations. The eleven names the gate reported, compared to the design's list. All eleven
ledger rows quoted. The row-2 premise check with its command. The gate's UNREVIEWED line. The
population the non-vacuity assert saw. Row 5's red, verbatim, and confirmation you restored it.
Then the honest deltas — especially any verb whose declaration made a ledger row feel dishonest.
