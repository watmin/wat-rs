# BRIEF — a diagnostic may not choose its subject by hash order

A mutual rete-defn cycle is refused correctly every run, but *which* of the two functions is blamed —
and which line the caret lands on — is a coin flip. Make the iteration order unrepresentable, and
prove the flip is gone with enough runs to mean something.

## Read in order

1. `src/rete/purity.rs:1707-1712` — the loop: `declared: &HashSet<String>`, `for name in declared`.
2. `src/rete/purity.rs:1729-1743` — **the same function stating the principle it still breaks**:
   *"`declared` is a HashSet, so `for name in declared` runs in ARBITRARY, run-varying order … A check
   that answers differently depending on hash iteration order is not a check."* That comment is the
   fix for the four AXES (seeding `seen` with every declared name) and it holds. **Read why it does
   not cover `rete_defn_cycle` below it.**
3. `src/rete/purity.rs:1758-1770` — `rete_defn_cycle`, called inside that loop, returning on the
   FIRST failure. This is where the arbitrary entry point becomes the user-visible blame.
4. `src/freeze/env.rs:385` (`extract_rete_defn_names`), `:56`, and `src/freeze.rs:460,519` — every
   site the type lives. `purity.rs:1709` is the sixth.
5. `tests/lint/diagnostic_output_is_deterministic.rs:118-140` — the `QUARANTINE` table and
   `QUARANTINE_LEN`. Its own doc: *"NOT a flake list … it must move `QUARANTINE_LEN`, which is what
   makes the addition deliberate."* Removal is equally deliberate.
6. `probe-c20.sh.txt` beside this brief — the orchestrator's 12-run reproduction, with its own
   sample-size note.

## Driven by the orchestrator at HEAD `04abe37fc`

```
12 runs, same binary, same file:
  6 → :probe::b at :line 5
  6 → :probe::a at :line 8
```

Both reports are truthful — walking from `a` closes the cycle at the call to `b`, walking from `b`
closes it at the call to `a`. **The entry point is what is arbitrary, and it is a hash order.**

## The change

`declared_rete_defns` becomes a **`BTreeSet<String>`** at all ~6 sites. Iteration is ordered by
construction; no `.sorted()` call site can be forgotten because there is no unordered value to sort.
The `seen` set *inside* the loop stays a `HashSet` — membership probe, order-irrelevant.

Then `QUARANTINE_LEN` 3 → 2, removing only the `probe_arc278_rete_defn_recurse_mutual` row. **The
other two rows keep their captured evidence and stay.**

## ⛔ The other two quarantined files are NOT yours

Driven: `probe_arc170_w2a_kwargs_check_mint_swap.wat.bad` over 8 runs hashes two ways while its first
two error kinds are stable every run — check-phase error *ordering*, a different root, and `check.rs`
shows no map iteration feeding it. **If your fix appears to cure them too, STOP and report that** —
it would mean the roots are shared and my scoping was wrong, which is a finding, not a bonus.

## Blast radius

`src/rete/purity.rs`, `src/freeze.rs`, `src/freeze/env.rs` (the type), and
`tests/lint/diagnostic_output_is_deterministic.rs` (the quarantine row + length). Plus a regression
test that drives the fixture enough times to mean something.

## STOP triggers

1. **If the `BTreeSet` change alters which error is reported for any OTHER fixture**, stop and report.
   Determinism must not become a behaviour change; the goldens are the witness.
2. **If any site needs `HashSet`'s API in a way `BTreeSet` cannot serve**, stop and report rather than
   leaving one site unconverted — a single surviving `HashSet` re-opens the hole and makes the type
   change a lie.
3. **If the two check-phase files go deterministic as a side effect**, stop and report (see above).
4. **If the flip survives the change**, stop and report the run distribution — do not add runs until
   it looks green.

## Mutation proofs — run all three, report all three

1. **Revert the type to `HashSet`** → your regression test REDs. State the run count and why it is
   enough (⚠ at p≈0.5, two runs miss the flip 50% of the time; C19's sweep needed 24 runs/file).
2. **Keep `BTreeSet` but iterate `.iter().rev()`** → the blame flips to the *other* member,
   deterministically, and the golden/regression REDs. Proves the test reads the identity, not merely
   "some error was produced".
3. **Restore the quarantine row while the fix is in** → the quarantine's own consistency check REDs
   (a row naming a file that is now deterministic). If it does not, the quarantine cannot tell a
   cured file from a broken one, and that is a finding about the gate.

Verify restores by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The run distribution before and after, with the run count justified.
- All three mutation results.
- Every site the type changed, and confirmation that none was left `HashSet`.
- Whether any golden moved, and if so which and why.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Three consecutive strikes here had their ★ be
  a false claim in a file the brief said to trust — the last one was my own pinned contract decision.
  Assume there is a fourth.

Do not commit.
