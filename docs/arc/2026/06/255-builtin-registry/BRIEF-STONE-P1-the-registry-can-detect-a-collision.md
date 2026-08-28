# STONE P1 — the registry can detect a collision

> Row P1 of `WORKLIST-open-stones.md`. Finding 1 of
> `NOTE-an-absence-recorded-as-an-answer-the-class-behind-the-apply-defect.md`, which carries the
> disk citations.

## The work

Two homes claiming the same FQDN is a silent overwrite. `src/intrinsic/mod.rs:348`:

```rust
fn register(&mut self, entry: IntrinsicEntry) {
    debug_assert!(!self.entries.contains_key(entry.name), "duplicate intrinsic registration: {}", entry.name);
    self.entries.insert(entry.name, entry);
}
```

`Cargo.toml` has **no `[profile.release]` section**, so debug assertions are off in release — and
`scripts/floor.sh:96` runs `cargo nextest run --release`. **On the only floor this repo trusts, that
assert does not exist.** Behind it, `HashMap::insert` overwrites silently: last `inventory::iter`
writer wins, and that order is not guaranteed.

You add a `#[test]` that catches a collision **in release**, where the floor can see it.

⛔ **THE OBVIOUS FIX DOES NOT WORK, AND THIS IS THE WHOLE DESIGN OF THE STONE.** A test that walks
`registry().all_entries()` **cannot detect a duplicate**: `IntrinsicRegistry` is a
`HashMap<&'static str, IntrinsicEntry>` (`mod.rs`), so by the time `all_entries()` can be called the
collision has already collapsed — one entry, no trace. The count comes back right and the test
passes while the defect sits there.

**Walk the SUBMISSIONS, not the registry.** `registry()` builds itself by iterating
`inventory::iter::<IntrinsicSubmission>` and `inventory::iter::<SpecialFormSubmission>`. Those
streams carry **every** submission, including both halves of a collision. That is the only place the
duplicate is still visible, and it needs **no production change at all**.

## Rooms — verified against `dd22a6d07`

```
src/intrinsic/mod.rs:347-350   fn register            — the debug_assert; READ IT, do not necessarily change it
src/intrinsic/mod.rs:338       struct IntrinsicRegistry { entries: HashMap<…> }  — why all_entries() is blind
src/intrinsic/mod.rs:372       fn registry()          — the two `inventory::iter` loops you mirror
src/intrinsic/mod.rs:200       IntrinsicSubmission    — has `.name`
src/intrinsic/mod.rs:255       SpecialFormSubmission  — has `.name`; BOTH streams register into ONE map,
                                                        so a collision ACROSS the two kinds is possible too
src/intrinsic/mod.rs (tests)   the existing #[cfg(test)] mod — put it beside its siblings
Cargo.toml                     no [profile.release]; do NOT add one to "fix" this
```

## Implementation sketch

```rust
#[test]
fn no_two_submissions_claim_the_same_fqdn() {
    let mut seen: std::collections::HashMap<&'static str, usize> = HashMap::new();
    for s in inventory::iter::<IntrinsicSubmission> { *seen.entry(s.name).or_default() += 1; }
    for s in inventory::iter::<SpecialFormSubmission> { *seen.entry(s.name).or_default() += 1; }
    let dupes: Vec<_> = seen.iter().filter(|(_, &n)| n > 1).collect();
    assert!(dupes.is_empty(), "…name the offenders and their counts…");
}
```

★ **Cover BOTH streams in ONE map.** They register into the same `entries` map, so an intrinsic and
a special form can collide with each other. A test that checks each stream separately would miss
exactly that case — and it is the one no reader would think to look for.

⚠ **The failure message must NAME the colliding FQDN and its count.** A count-only assert cannot tell
a maintainer which two homes to open, and this arc has already paid for a gate whose error text could
not name its offender. `[[feedback_a_gate_freezes_names_never_a_count]]`

⚠ **Do NOT add `[profile.release] debug-assertions = true` to `Cargo.toml`.** It would "fix" this one
assert by turning on every `debug_assert!` in the workspace in release — a change to how the entire
floor runs, made as a side effect of a registry gate. If you believe that is the right move, STOP and
say so; it is the builder's ruling, not this stone's.

## What to do with the `debug_assert!`

Your call, and say which you chose and why:
- **keep it** — it still fires fast in debug, and the new test covers release; or
- **replace it with the test's guarantee** and leave a comment pointing at the test.

Do not simply delete it and leave nothing pointing at the new gate.

## Blast radius

`src/intrinsic/mod.rs` — one `#[test]`, plus at most a comment on `register`. **No production
behaviour changes.** No `Cargo.toml` change. No new file.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **The test goes RED on the current tree.** Then a duplicate EXISTS today and that is a finding
   far bigger than this stone — STOP, name the colliding FQDNs, and report. (Measured 2026-08-28:
   382 submissions, 382 distinct names, so it is expected GREEN. If it is red, my census was wrong
   and I want to know before anything is changed.)
2. **You cannot reach `inventory::iter` from the test.** STOP and report what blocks it rather than
   falling back to `all_entries()` — a test over the collapsed map is a test that cannot fail for
   the reason it exists.
3. **You find yourself editing `Cargo.toml`.** STOP; see above.
4. **The test passes without being able to fail.** See acceptance row 1: you must MAKE a collision
   and watch it go red. A gate never seen red is a claim.

## Acceptance — run each, report the actual output

```
 0. THE GATE IS GREEN ON THE CURRENT TREE.
      cargo nextest run --release -E 'test(no_two_submissions_claim_the_same_fqdn)'
    Summary line verbatim.

 1. ★ AND IT CAN GO RED — IN RELEASE. Temporarily add a second `#[wat_intrinsic]` handler claiming
    an FQDN that already exists (a throwaway fn in any intrinsic file). Show:
      (a) `cargo build --release` still SUCCEEDS — proving the collision is silent without this gate;
      (b) the new test FAILS, and its message NAMES the colliding FQDN;
    then remove the throwaway and show the test green again. `NISI FRANGAS, NIHIL PROBAS.`
    ⚠ Confirm each edit LANDED before reading its output.

 2. ★ AND THE OLD GUARD COULD NOT HAVE CAUGHT IT. With the throwaway duplicate still in place, show
    that `cargo nextest run --release` (the floor's own command) does NOT fail on the debug_assert.
    This is the row that proves the stone was necessary; without it, someone will later "simplify"
    the test away on the grounds that the assert already covers it.

 3. ★ A CROSS-KIND COLLISION IS CAUGHT TOO. Repeat row 1 with the duplicate declared as a
    `#[wat_special_form]` colliding with an existing `#[wat_intrinsic]` name (or vice versa).
    Both streams register into ONE map; a gate that only checks within a stream is half a gate.

 4. cargo build --release --all-targets — clean.

 5. cargo nextest run --release -E 'binary_id(wat::lint) + test(intrinsic)' — Summary verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing will wake you. Your turn ends when the
  numbers are in your hands, not when a command is launched.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- Do not commit, push, stash, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Which disposition you chose for the
`debug_assert!` and why. Then the honest deltas — what surprised you, what this brief got wrong.
Four riders on this chain have each caught a real defect in an orchestrator brief; one refuted its
opening premise outright. That is the most useful thing you can hand back.
