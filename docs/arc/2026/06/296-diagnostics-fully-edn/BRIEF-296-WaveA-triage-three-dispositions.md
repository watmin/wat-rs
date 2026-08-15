# BRIEF — 296 Wave A follow-up: three of the four reds, dispositioned

> Wave A un-ignored 109 dark tests, recaptured 105, and left **4 red on purpose** — the law forbids
> recapturing or re-ignoring a FINDING. This brief closes three of them. The fourth (#3) is held for
> the builder.

**The working tree is RED and uncommitted, and that is your starting state.** It holds Wave A's 105
recaptured goldens plus the 4 deliberate reds. Do not revert it, do not re-ignore anything, and do not
touch the 105.

Current: `4530 tests run: 4526 passed, 4 failed, 154 skipped`.

## ⛔ #3 IS NOT YOURS — `probe_supervisor_select_lost::select_prime_yields_lost_when_process_child_crashes`

Leave it red and untouched. It is a **real regression** under separate ruling: a wat child panics and
the reported `:location` comes back as `src/wat_edn_bridge.rs:442:38` — our own EDN decoder's
`rust_caller_span!()` — instead of the child's crash site. The wire is overwriting the child's
location with the parent's. That is a substrate stone, not a recapture.

---

## #4 — `probe_arc296_2_to_edn_trait::probe_2_span_to_edn_is_structured_map`

**Its golden is already correct** (Wave A recaptured it cleanly). The test still fails on a *second,
independent* assertion in the test body — roughly `assert!(matches!(&edn, OwnedValue::Map(_)), …)` —
which pins the **pre-Span-tag shape**. Stone B made `Span` a tagged record
(`#wat.core/Span {…}`), so a bare `Map` match no longer holds.

Update that assertion to assert the current shape. **Do not weaken it to `assert!(true)`-by-another-
name** — it exists to prove the span renders structured rather than as a string, and that claim
survives the tag. Assert the tagged-record shape, keeping the same subject.

## #1 — `probe_arc296_remediation_collapse::probe_1_type_mismatch_retired_callee_emits_remedies_not_hint`

The golden is stale: the `:note` text differs from what the code now emits, traced to
`src/remedy/retirement.rs:115`.

**Read that site before you capture.** Builder: *"golden is by definition stale — but we need to
understand it."* Capturing is how a wrong message becomes permanent, so the question is whether the
current wording is **intentional** (a deliberate rewording that landed) or **accidental drift** (a
prefix/suffix leaking in from a refactor).

- Intentional → recapture, and say in your report what changed and why you judged it deliberate.
- Accidental → **STOP-1.** Report the wording and the site; do not capture it.

## #2 — `probe_arc296_remediation_collapse::probe_2_type_mismatch_arc114_shape_emits_spawn_thread_remedy_not_hint`

`:remedies` comes back **empty**; the golden expects a populated spawn-thread migration remedy.

**Measured: `shape_remedies` exists nowhere in the source.** It survives only in its own tombstone at
`src/check.rs:94` — *"arc 114's shape_remedies died with the spawn/join/join-result tombstones"*. The
capability was **deliberately deleted**. So the test is not stale — **it asserts a capability we
retired**, exactly like `probe_arc258_dotted_record_field` earlier in this arc.

Ask what the test MEASURES before choosing:

1. **Is the arc-114 shape still constructible?** The test builds a `Thread<…>` where a
   `ProgramHandle<…>` is expected. If that mismatch is still reachable in real code, a caller can
   still hit it.
2. **Does `RETIREMENT_TABLE` produce anything for it?** The collapse absorbed the old hint helpers
   into that table. If it has a row covering this shape, **re-express** the test to assert *that*
   remedy — the subject (a retired shape teaches the caller) survives, only the mechanism moved.
3. **If the shape is reachable and the table has nothing**, then a real diagnostic gap exists: a
   mismatch a caller can hit that teaches them nothing. **That is a finding, not a retirement** —
   report it, leave the test red, and do not delete it to get green.
4. **If the shape is genuinely unreachable** (the types it advised about are tombstoned out of
   existence), the test retires with the capability. Say so with the evidence.

Do not pick (4) by default because it is the quickest path to green.

## STOP TRIGGERS

- **STOP-1 — #1's wording looks accidental.** Report, do not capture.
- **STOP-2 — #2 lands on outcome (3)**, a reachable shape with no remedy. Report the gap; leave red.
- **STOP-3 — any fix requires a `src/` change.** #4 and #1 should not need one. If #2 does, that is a
  substrate stone and belongs in its own strike — report rather than build it here.
- **STOP-4 — a golden you touch fails STOP-2 of the campaign**: it contains `field-N`, a rust-debug
  `{:?}` artifact, or a `#wat-edn.*` tag. The emitting code is wrong, not the golden.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` — read the **Summary line**, never a piped exit code.

Expected: **`4 failed` drops to `1 failed`** (#3, held), with the passed/skipped arithmetic reconciling
against whatever you did to #2 — if it retires, the total test count falls by one and you say so.
Report the arithmetic explicitly.

**On any red you did not intend: do NOT re-run.** Copy the failing test's whole stdout+stderr block
verbatim — never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the FOREGROUND and block on it. Anchor at `/home/watmin/work/holon/wat-rs`;
`pwd` first. Leave the work uncommitted.

Report: each of the three with its disposition and the evidence you based it on, the floor Summary
line verbatim with the arithmetic, every STOP, and the honest deltas — especially anywhere this brief
did not match the disk.
