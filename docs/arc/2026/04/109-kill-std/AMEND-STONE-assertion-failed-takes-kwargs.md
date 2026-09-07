# AMEND — `assertion-failed!` takes kwargs: the probe is WRITTEN, and the count had DOUBLED

The BRIEF and the NOTE stand. This corrects two things measured on the green tree, and discharges
the debt the BRIEF's own banner names.

## ⛔ STOP-1 IS DISCHARGED — THE PROBE IS COMMITTED AND RED

The BRIEF said *"no probe could be run and none is committed … the strike's FIRST act is to write
and verify a RED probe."* **That was the ORCHESTRATOR's debt, not the rider's** (FM 2-bis: the
empirical probe is the orchestrator's earned right to assert the composition). The only reason it
was sequenced forward is that the stdlib did not parse at drawing time. It parses now.

`tests/kernel/probe_arc109_assertion_kwargs.rs` + 4 co-located `.wat` fixtures, verified on
floor-green `ef2c20bf2`:

```
the_control_program_checks_clean ............. PASS   ← the bar, and it must STAY green
a_plain_failure_needs_only_a_message ......... FAIL
the_optional_kwargs_are_still_accepted ....... FAIL
the_retired_positional_form_is_refused ....... FAIL
```

**Every bar is the control, run in the same test — no hand-written exit code.** A row asserting
`EXIT == 0` outright would also pass with a mis-aimed harness that returned 0 for everything; a row
asserting `!= 0` would be satisfied by a typo in its own fixture.

The kwargs rows fail on the gate itself, not on a malformed fixture:

```
#wat.check/ArityMismatch {:callee ":wat::kernel::assertion-failed!" :expected 3 :got 2}
```

That is `src/assertion.rs:135` `args.len() != 3`. And row 4 states the defect as data:

```
assertion `left != right` failed      left: 0      right: 0
```

★ **The positional form checks exactly as clean as a program with no assertion in it.** Row 4 is
the one that proves the migration FINISHED rather than merely ADDED a spelling (STOP-3), and it is
red today in the OPPOSITE direction from rows 2-3.

## ⚠ THE HEADLINE NUMBER WAS AN UNDERCOUNT — IT IS ~2× WHAT THE BRIEF CLAIMS

The BRIEF's title says *"2532 bare `:None`s stop existing"* and its body says *"1266 of those calls
end in TWO BARE `:wat::core::None`s"*. Re-measured today:

```
                          BRIEF        today
assertion-failed! calls    2670         2683
files                       442          450
trailing None PAIRS        1266         2588      ← the pair count DOUBLED; the call count did not
None sites deleted         2532         5176
```

The call count barely moved, so this is **not corpus growth — the BRIEF's pair pattern undercounted
by about half.** Measured directly and validated before publishing: 2,588 adjacent
`:wat::core::None :wat::core::None` occurrences, of which **only 8** have no `assertion-failed!`
within the preceding 120 characters. So ~96% of all `assertion-failed!` calls are plain fails
carrying two placeholders.

**The BRIEF's ARGUMENT was right and is now twice as strong:**

```
bare Option/Result, corpus     None 5816 · Some 625 · Err 560 · Ok 365   = 7366
deleted by THIS stone                                                    ≈ 5176   (70%)
left for the Option/Result completion                                    ≈ 2190
```

★ Taking this stone first shrinks the enum migration from **7,366 sites to about 2,200.** Migrating
Some/None first would rewrite ~5,176 sites into a longer spelling and then delete them.

## Citations re-derived — one had drifted AGAIN

The BRIEF warns its own citations drift. It is right, and one of its corrections is now stale too:

```
src/assertion.rs:127   eval_kernel_assertion_failed          ✓ still
src/assertion.rs:135   the `args.len() != 3` gate            ✓ still
src/check.rs:17239     the registration        ⛔ NOW :17751
src/check.rs:17778     the STOP-5 sibling                    (was cited as :17266)
```

Re-derive with `grep -n`, never cite from any of these three documents.

## Unchanged and still binding

STOP-2 (R21 — `wat-scripts/fixes/positional-to-kwargs.wat` is the recorded shape; no hand-edited
`.wat`), STOP-3 (the positional form must be REFUSED — row 4 IS this row), STOP-4 (the two Nones are
DROPPED, never rewritten as `:actual :None :expected :None`), STOP-5 (the sibling at `check.rs:17778`
stays out unless the compiler forces it).
