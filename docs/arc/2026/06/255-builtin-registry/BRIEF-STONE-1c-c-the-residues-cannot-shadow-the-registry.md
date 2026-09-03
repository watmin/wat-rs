# BRIEF — STONE 1c-c: impose the gate, delete what it names

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-c-the-residues-cannot-shadow-the-registry.md`

## The work, in one paragraph

Two functions consult the intrinsic registry first and fall through to a hand-written `matches!`
list. Both state, in their own headers, that a name may appear in that list **only** if the
registry does not answer for it. **53 rows now violate that rule** — they are unreachable dead
text shadowing a real registration. Write one gate that asserts the rule, let it name the
offenders, delete exactly those rows, and correct the two stale count comments. **No name is
registered or unregistered by this stone; no behaviour changes.**

```
src/macros/eval.rs   is_expand_time_legal   registry consult :425, arms from :476   34 of 55 shadowed
src/rete/purity.rs   intrinsic_meta         registry consult :472, arms from :504   19 of 42 shadowed
```

## Read in order

1. **`src/macros/eval.rs`, `is_expand_time_legal`** — read the residue's header in full. It states
   the rule, and it records the last time this happened (four Option/Result unwrappers, deleted
   2026-08-31, found by a rider who happened to notice). Its count claim of **58** is stale.
2. **`src/rete/purity.rs`, `intrinsic_meta`** — same shape, its own wording of the same rule.
3. **`src/intrinsic/mod.rs`**, `registry_first_door_owns_every_handler_row_no_literal_arm_survives`
   — **the precedent for your gate.** It reads `runtime.rs` as data via `include_str!`, bounds
   itself to one function, strips comments, and requires a hit to look like a real match arm
   rather than a prose mention. Copy that discipline exactly; its own doc explains each guard and
   why it exists.

## The gate

One `#[test]`, asserting: **no FQDN named in either residue hand-list resolves in
`registry().lookup_entry`.** On failure it must name every offender and which list it came from —
a count is not enough (`[[feedback_a_gate_freezes_names_never_a_count]]`).

⛔ **The gate's own instrument must be proven, or it returns a vacuous green.** A source-parsing
test that silently parses nothing passes for the wrong reason, and that failure mode has already
bitten this campaign's orchestrator twice today — once by running past a function's closing brace
and collecting names from its neighbours, once by testing only a subset of names and reporting the
subset as the population.

So the gate carries its own non-vacuity assertions, before the real one:
- it found a plausible number of FQDNs in **each** list (assert a sane lower bound per list, not a
  combined total — a lower bound that one list alone could satisfy proves nothing about the other);
- it can see a **specific name known to be present** in each list — pick one you have verified by
  eye, and say in a comment which line you read it from.

`[[feedback_a_green_test_can_prove_nothing]]` is the standing precedent: a probe that never
invokes the thing returns a meaningless green.

## Then delete what it names

The named rows are **unreachable by construction** — the registry consult above the list answers
first for any registered name — so deleting them changes no behaviour. That is precisely why
nothing caught them.

⚠ **Verify that construction rather than trusting this brief:** confirm in BOTH functions that the
`registry().lookup_entry` consult really precedes the `matches!` arms and returns unconditionally
for a hit. If it does not in either one, that is **STOP-2** — the rows would then be live, deletion
would change behaviour, and I need to know before anything is removed.

Leave a short retirement comment in each list, in the style those lists already use for the
2026-08-31 deletion, naming this stone and stating that the registry answers for the removed names.
Correct both stale counts to what you actually measure.

## Blast radius

`src/macros/eval.rs` (residue rows + its count comment) · `src/rete/purity.rs` (residue rows + its
count comment) · one new test file, or the gate added alongside its precedent in
`src/intrinsic/mod.rs` — your call, but say which and why. **No registration. No ledger edit. No
change to either function's logic, only to its dead arms.**

## STOP triggers — halt and report, do not improvise

- **STOP-1.** Your gate's non-vacuity assertions do not hold — it cannot find a name you can see
  by eye. **Do not weaken the assertion to make it pass.** The parser is wrong; report it.
- **STOP-2.** In either function the registry consult does NOT precede the arms, or does not
  return unconditionally on a hit. The rows are then live. Report; delete nothing.
- **STOP-3.** The gate names a row whose deletion you believe WOULD change behaviour. Report the
  name and your reasoning rather than deleting it.
- **STOP-4.** Any ledger (`GAP_A`, `GAP_B`, `DEBT`, `KNOWN_UNREVIEWED`) moves. None should — this
  stone registers nothing.
- **STOP-5.** A test other than your new gate goes red. Copy its entire stdout and stderr block
  verbatim from `.floor/latest/raw.log`, name the exact assertion that fired, and report — before
  re-running anything.

## Verification, in this order

```bash
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

★ Run the floor **once with the gate added and the rows still present**, and report its failure
message verbatim — that red is this stone's proof that the gate fires. A gate that has only ever
been seen green has not been shown to work.

## Acceptance — derived, not estimated

```
shadowed rows            53 → 0     34 expand-time + 19 intrinsic_meta
is_expand_time_legal     55 → 21    named FQDNs remaining
intrinsic_meta           42 → 23
stale count comments      2 → 0
GAP_A / GAP_B / DEBT   49 / 52 / 111 — ALL UNCHANGED, nothing is registered here
KNOWN_UNREVIEWED             14 unchanged
floor            5127/5127 → 5128/5128   ⬅ +1, because this stone DOES mint a #[test] fn
clippy                        0
```

⚠ The `+1` is derived from the mechanism, not from arithmetic on a nearby number: registration
stones cannot move the floor count because they add no test; this one adds exactly one.

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. Report the gate's red output verbatim, the full list of rows you deleted
per file, and the non-vacuity assertions you wrote with the line you verified each against.
