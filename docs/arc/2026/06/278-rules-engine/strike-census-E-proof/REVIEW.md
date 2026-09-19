# REVIEW — ⛔ STOP-1 FIRED. The mutation did NOT red; the gate cannot prove census E.

Grok stalled mid-strike with the mutation still applied. **The tree was left with `matcher.rs`
carrying census E's defect** (bump back below `alpha_pattern`'s `?`) — EXPECTATIONS row 6 exists for
exactly that, and a blind commit would have silently reverted `c91121e5e`. Restored to HEAD;
`git diff --quiet src/rete/matcher.rs` is clean.

Because the mutation state is **perishable**, it was measured before restoring.

## The result — captured from grok's own mutated tree

`compiled_cond_failure_path_allocates_no_binding_keys_at_50_100`, `--release --no-capture`, with
`census_count("match:calls")` BELOW the `?`:

```
  ROW 3 — call-counter differential, same corpus
  match:calls    = 10000  (hand interp_calls = 10000)
  compiled:exec  = 10000  (hand calls = 10000)

  PASS  wat rete::kernel::tests::alpha_discrimination::compiled_cond_failure_path_...
```

**PASS, not RED.** `match:calls == interp_calls` holds *with the defect restored*.

## What that means

The DESIGN's trap fired exactly as written: **the corpus contains no `cond` that fails
`alpha_pattern`.** All 10,000 calls carry an alpha pattern, so the `?` never returns early, and the
equality is identical before and after census E's move.

**So the assertion cannot prove census E.** Census E's widening — invocations whose `cond` is not an
alpha pattern — remains unproven, and is now demonstrably unprovable *by this corpus*.

Per STOP-1 this comes back rather than being forced: **do not add facts to the corpus to manufacture
the red.** That is a corpus decision, and it is the next strike's subject.

## ⚠ ONE CONTROL IS STILL OPEN, AND IT IS LOAD-BEARING

A mutation that does not mutate reads exactly like a negative result
(`[[a-mutation-that-does-not-mutate-reads-as-a-negative]]`). Two explanations fit the PASS above:

1. **The mutation was real but inert** — the `?` never fires on this corpus. (The DESIGN's trap.)
2. **The mutation never reached the binary.**

These are distinguished by one cheap control: **delete the bump entirely.** `match:calls` then reads
0 through `unwrap_or(0)`, against a hand count of 10,000 — so the assertion MUST red. If it does
not, the test is not observing the counter at all and everything above is void.

**Run that control before believing this REVIEW.** It was set up and not run; the credit stall
interrupted it.

## What the three assertions ARE worth — keep them

Independent of census E, they are the first thing in the tree that gates these counters at all:

- `match:calls == interp_calls` — the counter is **live** and counts every call the loop makes.
- `compiled:exec == calls` — same for the compiled half.
- `calls == interp_calls` — the two loops share a corpus, so the counters are comparable.

Before this, `match:calls` had two sites in the whole tree and deleting it changed nothing
observable. That hole closes here even though the E-proof does not.

## Disposition

- `src/rete/kernel/tests/alpha_discrimination.rs` — grok's +31 lines: KEEP, pending the control.
- `src/rete/matcher.rs` — restored to HEAD, byte-identical. **Do not commit it mutated.**
- Census E stays landed and stays **unproven**, and the record says so.
- Next: the control, then a corpus decision — whether this world should carry a non-alpha `cond` so
  the population census E widened is representable at all.
