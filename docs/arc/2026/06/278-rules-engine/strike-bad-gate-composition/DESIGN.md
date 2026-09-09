# DESIGN — the `.wat.bad` gate's seven states are proven per-component and once composed

**Status:** drawn 2026-09-09. **Largely REFUTES `4P1`** (peragrare, 6×L1) and replaces it with a
smaller, true finding.

## ⛔ What `4P1` claims, and why most of it does not hold

The row: *"6 of its 7 exemption states are unvisited… **all six are L1**"*, with the sharpest being
`(clean, owner-live)` because the header sells that branch as the gate's reason to exist — *"the day
arc 255 lands and the test is un-ignored, this gate goes RED"* — and *"a bug in the upward attribute
scan (`:190-203`) would let a stale exemption survive silently."*

**Read against the file, six of those seven states are already driven** — by **ten unit tests** in
the same file, synthetically, at the predicate level:

| state | driven by |
|---|---|
| `absent` | `prose_naming_the_words_is_not_a_rune` |
| `bad-category` | `an_invented_category_is_read_not_skipped` |
| `no-owner-field` | `a_reason_with_no_owner_field_names_nobody` |
| `owner-absent` | `a_test_that_is_not_there_is_absent` |
| **`owner-live`** | **`a_running_test_reads_as_live`** — *the state the row calls never-driven* |
| `owner-ignored` | `an_ignored_test_reads_as_banked`, plus 3 real fixtures |

⭐ **And the specific risk the row names is the thing a test already asserts.**
`an_ignore_on_a_different_test_does_not_vouch_for_this_one` drives the upward attribute scan
directly: *"The upward scan must stop at the blank line: `other`'s `#[ignore]` is not `t`'s."*

**What `4P1` actually measured is the CORPUS**, which is `peragrare`'s proper subject and is true:
the real `.wat.bad` fixtures visit exactly one state. The row's error is escalating *corpus
coverage* into *gate coverage* — they are different questions and the file answers the second.

## ⭐ The finding that survives, and it is this repo's own Failure mode 24

**`check_shard` — the composition — is called from nowhere but the generated shard tests**
(`fn $name() { check_shard($idx); }`), which run over the **real corpus**, which visits **one state**.

So: **every state has a per-component proof; the composed path has one.** Walk → `declaration_on`
→ `owner_in` → `owner_state` → verdict is exercised end-to-end only for `owner-ignored`. A defect in
the *wiring* — a state computed correctly and then routed to the wrong branch, an early `continue`
swallowing a failure, a `failures` push that never reaches the assertion — is invisible to both the
unit tests (which never call `check_shard`) and the corpus (which never reaches those branches).

That is precisely `docs/COMPACTION-AMNESIA-RECOVERY.md`'s **Failure mode 24 — "Per-component proofs
that never cross a SEAM."**

## What this delivers

**One test that drives `check_shard`'s composed logic over a SYNTHETIC corpus**, so each exemption
state produces its verdict through the real wiring rather than through its predicate alone.

⚠ **The obstacle is real and is why this was never done:** a fixture that drives a failure branch
would, by construction, make the gate RED if left in the corpus. So the composed path must be
driven over a **temporary corpus the test builds and tears down**, or `check_shard` must be
refactored to take its corpus and return its failures rather than assert on them.

⛔ **PINNED: prefer refactoring `check_shard` to RETURN its failures over building a temp corpus on
disk.** A returned `Vec<String>` is testable, needs no filesystem, cannot leave litter that another
gate walks, and the shard tests become `assert!(check_shard(i).is_empty())`. **A test that writes
`.wat.bad` files into a tree that four other gates walk is a hazard**, and this repo has already been
bitten by a file landing in a gated tree.

## Out of scope = rejected

- **Adding fixtures for the six states.** They would red the gate; that is the whole obstacle.
- **`3P1`'s axis-pair coverage** — same class, different instrument, its own row.
- **Re-litigating `peragrare`'s grid.** Its corpus measurement was correct; only the row's
  escalation to "6 L1" was not.
