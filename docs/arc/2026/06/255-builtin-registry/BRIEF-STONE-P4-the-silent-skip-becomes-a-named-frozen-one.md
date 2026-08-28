# STONE P4 — the silent skip becomes a named, frozen one

> Row P4 of `WORKLIST-open-stones.md`. Finding **5** of
> `NOTE-an-absence-recorded-as-an-answer-…md`, which recorded the SHAPE as confirmed and the SIZE as
> **open** — deliberately, because the ward's figure came from an instrument that disagreed with mine.

## The work

Two `#[cfg(test)]` gates over the registry each open by skipping any entry the checker has never
heard of, **silently**:

```rust
// src/intrinsic/mod.rs:609  — doc_arg_ret_types_match_checker_scheme
// src/intrinsic/mod.rs:839  — yields_type_matches_fn_arg_param
let scheme = match check_env.get(entry.name) {
    Some(s) => s,
    None => continue,            // not yet in checker — skip
};
```

`doc_arg_ret_types_match_checker_scheme`'s own doc says *"A mismatch is a doc lie — the user reads
one type, the checker enforces another."* **For every skipped entry it verifies nothing at all**, and
says nothing about how many that is. A registration whose `@arg`/`@ret` strings are pure fiction
passes both gates by being absent from the checker.

**You do not merely count them. You make the skip NAMED and FROZEN**, so the population can never
grow silently and its shrinking is visible.

## ⛔ THE INSTRUMENT IS THE DELIVERABLE — and it must not be a grep

The NOTE refused to publish a size because the ward measured *"96 of 384"* by cross-referencing
`:wat::` string literals inside `register_builtins`'s body against `#[wat_intrinsic]` attributes, and
its 384 disagreed with the anchored 380. **Two instruments, two populations, so the number was
worthless.** This arc has now had **six** wrong censuses of one population, the most recent being an
*anchored* grep that still counted a line inside a format string.

★ **So do not grep. Ask the gate's own instrument the gate's own question.** Both gates build
`CheckEnv::with_builtins_and_types(&TypeEnv::new())` and call `check_env.get(entry.name)`. A test in
the same module can build the same `CheckEnv` and call the same method over
`registry().all_entries()`. **A measurement that cannot disagree with the thing it measures.**
`[[feedback_validate_a_search_pattern_before_trusting_its_count]]`

## The ONE CONTRACT DECISION — freeze NAMES, never a count

`[[feedback_a_gate_freezes_names_never_a_count]]`: a ratchet pinned to a number cannot tell
*"+1 new, −1 fixed"* from *"nothing happened"*, and its failure text cannot name the offender.

Ship a `#[test]` holding a **frozen, sorted allowlist of the FQDNs the checker does not know**, and
assert the measured set equals it. Then:

- a **new** registration absent from the checker → RED, naming it. The population cannot grow quietly.
- a name **added** to the checker → RED, telling you to delete it from the list. The debt shrinks
  visibly and the list can only go down.

⚠ **This list is a DEBT LEDGER, not an exemption list.** Say so in the test's own message. Every name
on it is an intrinsic whose declared `@arg`/`@ret` types are verified by nothing. The stone that
drives it to zero is not this one, and this stone must not pretend otherwise.

## What this stone does NOT do

- **It does not add anything to the checker.** Registering the missing names is a separate, much
  larger stone; the census is its input.
- **It does not change either existing gate's behaviour.** They keep skipping. What changes is that
  the skipping is now counted, named, and walled.
- **It does not judge whether a given absence is legitimate.** Some are (the NOTE records
  `spawn-thread`/`spawn-process` as deliberately special-cased through `infer_*_prime`). Recording
  that a name is on the list is not an accusation; report any you can identify as intentional, and
  say so in the list's comment rather than silently excusing them.

## Rooms — verified against `4ce0d6494`

```
src/intrinsic/mod.rs:609    gate 1's skip — doc_arg_ret_types_match_checker_scheme
src/intrinsic/mod.rs:839    gate 2's skip — yields_type_matches_fn_arg_param
src/intrinsic/mod.rs:834    gate 2's OTHER skip (`no @yields`) — NOT this stone's; that is P5
src/intrinsic/mod.rs        `check_env.get(...)`, `CheckEnv::with_builtins_and_types(&TypeEnv::new())`
                            — the exact construction to reuse; both gates build it identically
src/intrinsic/mod.rs        `registry().all_entries()` — the population
src/check.rs                `register_builtins` — what populates CheckEnv; READ ONLY, do not add to it
tests/lint/no_bare_is_err.rs        ★ the frozen-allowlist shape (file + name identity, never a line)
tests/lint/ignore_reason_justified.rs  ★ P3's fresh precedent — a lint that ships self-tests for its
                                    own detector. Do the same if your measurement has any parsing in
                                    it at all.
```

## Blast radius

`src/intrinsic/mod.rs`'s `#[cfg(test)]` module — one new test. **No production code. No change to
either existing gate. Nothing added to the checker.**

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **You reach for a grep, a file scan, or a text census to produce the number.** The gate's own
   `check_env.get` is the instrument. If you believe it cannot answer the question, STOP and say why.
2. **Your count disagrees with a second measurement you take.** Then one instrument is wrong and
   publishing either is the arc's most expensive recurring mistake. STOP and report both.
3. **The frozen list is a count, or is keyed by line number.** See the contract decision.
4. **You add a name to `register_builtins` to shrink the list.** That is the separate stone; this one
   measures. STOP.
5. **The measured set is EMPTY.** Then the NOTE's finding 5 is refuted and that is a large, welcome
   surprise — STOP and report it rather than shipping a vacuous frozen list. (Both gates would then
   be skipping nothing, and their `None => continue` arms would be dead.)

## Acceptance — run each, report the actual output

```
 0. ★ THE NUMBER, AND THE INSTRUMENT THAT PRODUCED IT. How many of the registered entries does
    `check_env.get(name)` return None for, out of how many total? Show the code that measured it.
    Report the total registered as the anchored count too — they must be consistent.

 1. ★ THE NAMES. The full sorted list of skipped FQDNs. Group them by namespace and say which
    namespaces are wholly absent versus partially — that shape is what the next stone needs.

 2. ★ THE WALL GOES RED BOTH WAYS. This is the point of freezing names rather than a count:
      (a) ADD a throwaway `#[wat_intrinsic]` under a name the checker does not know → the test
          FAILS and NAMES it as an unexpected skip. Remove it; green.
      (b) REMOVE one name from the frozen list → the test FAILS and names it as a name that IS
          skipped but is not on the list. Restore; green.
    Confirm each edit LANDED before reading its output. `NISI FRANGAS, NIHIL PROBAS.`

 3. ★ THE FAILURE TEXT NAMES OFFENDERS, NOT A DELTA. Paste both failure messages from row 2. A
    message that says "expected 96, got 97" fails this row.

 4. ★ HOW MUCH IS ACTUALLY UNVERIFIED. For ONE skipped entry, show its `@arg`/`@ret` doc strings
    and confirm nothing checks them — that is the concrete cost the number stands for.

 5. cargo build --release --all-targets — clean.

 6. cargo nextest run --release -E 'test(intrinsic)' — Summary verbatim, and confirm both existing
    gates still pass unchanged.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- **No `git stash`, in any form.** `git show HEAD:<path>` for a pre-image.
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. The number, the instrument, and the list.
Then the honest deltas. The last rider corrected this orchestrator's ignore count — an *anchored*
grep that still counted a string literal, the sixth wrong census in this arc. If a number in this
brief or a claim in the NOTE does not survive contact with the disk, say so.
