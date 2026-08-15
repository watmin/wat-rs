# BRIEF — 296 Wave A (T1): 105 tests come out of the dark

> Read `CAMPAIGN-the-recapture-cascade.md` first — it carries the law this brief enforces.

Baseline: HEAD `437edde1`, floor **4421 / 4421 / 263 skipped**, clippy 0.

## THE WORK

**224 tests** carry `#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face;
unlock: 296 recapture (.edn data-equality flip)"]`. They have been dark since stone B replaced the
Rust `{:?}` face with EDN.

**T1 is the ~105 of them whose assertion is already `assert_edn_matches_file!`** — already converted,
merely parked. H-2a (`437edde1`) just proved the regen path at corpus scale: 208 sites converted, 58
goldens rewritten, all 58 proven data-equal. The unlock condition these ignores name is now met.

Take T1. Leave T2 (`assert_eq!` on an inline literal) and T3 (`assert!(… contains …)`) alone — later
waves, different work.

## ⛔ THE LAW — read before you capture

`UPDATE_EDN=1` writes whatever the code emits **right now**. On a test failing for a real reason, that
**freezes the bug into the golden** and paints it green forever.

These tests have been dark for the life of the cohort. Some of them are dark over a genuine
regression nobody has seen since stone B. That is the whole reason this is worth doing and the whole
reason it is dangerous.

**In this order, and no other:**

1. **Un-ignore the T1 tests.**
2. **Run WITHOUT `UPDATE_EDN`.** Capture the failure list whole.
3. **Triage every failure into exactly one of:**
   - **STALE** — the golden pins the pre-stone-B rust-debug face and an EDN face now arrives. This is
     the expected class and the only one eligible for recapture.
   - **FINDING** — anything else. A panic, a changed value, an arity shift, a message that is EDN but
     says something different, a test that hangs. **Do not recapture it. Do not re-ignore it.**
4. **Recapture ONLY the STALE set**, with `UPDATE_EDN=1`.
5. Re-run clean and report.

**A report of "N recaptured, all green" with no triage list has skipped the only step that separates
this from mass-blessing 105 assertions.** The triage list is the deliverable; the green is a
by-product.

## CLASSIFY FROM THE DISK — my numbers are orientation

I counted 105 / 101 / 16 across the three tiers. **Verify that against the disk before you act on
it.** My counts have been wrong twice today in the identical way — reporting a *file* count as an
*item* count (the cohort as "70 tests" when it is 224; H-2a's sites as "62" when there were 209).
Both were caught by riders, not by me.

The T1 predicate: an `#[ignore = "296-recapture-pending…"]` whose test body asserts through
`assert_edn_matches_file!`. Confirm the tier boundaries yourself and report the counts you actually
find, even where they disagree with the table above — **especially** where they disagree.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — a failure you cannot confidently classify** as STALE or FINDING. Report it verbatim with
  its test name and let the orchestrator rule. Ambiguity resolved toward "recapture" is exactly how a
  bug gets blessed.
- **STOP-2 — a recaptured golden that still contains an annihilated form**: `field-N`, a rust-debug
  `{:?}` artifact, a `#wat-edn.*` tag where an EDN face was expected. The capture wrote something this
  arc already killed, which means the emitting code is wrong, not the golden.
- **STOP-3 — a test that hangs or panics the harness** rather than failing an assertion. Do not
  re-ignore it to move on; that is how it got dark the first time. Report it.
- **STOP-4 — the triage set is larger than you can do carefully.** Say so, recapture only the
  unambiguously-stale subset, and report the rest categorized. **A partial wave is a good outcome; a
  blessed wave is not.** Do not trade care for coverage.

## BLAST RADIUS

The T1 `#[ignore]` attributes and the `.edn` goldens their tests recapture. No `src/` changes — if a
FINDING requires a substrate fix, that is a separate strike and you report it rather than fixing it
here. No `.wat` corpus changes. Do not touch T2, T3, or the 26 non-recapture ignores (RED-at-HEAD,
`unimplemented!()`, perf harnesses — several of those are honest and stay).

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.

The floor's **passed count must RISE** by however many T1 tests you un-ignored and left green, and the
**skipped count must FALL** by the same. Report both numbers and the arithmetic. A count that does not
reconcile is itself a finding.

**On any red you did not classify: do NOT re-run.** A re-run that goes green destroys the only
evidence. Copy the failing test's entire stdout+stderr block verbatim — never a `| head` window —
name the exact assertion that fired, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, and no notification is coming. Run
every build and test in the FOREGROUND and block on it; a rider on this arc already lost a flight to
exactly that. Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. Leave the work uncommitted; the
orchestrator weighs and commits.

Report: the tier counts you measured, the **full triage list** (test name → STALE or FINDING, with the
reason), how many you recaptured, every FINDING with its verbatim failure block, the floor Summary
line and the passed/skipped arithmetic, every STOP, and the honest deltas. Every rider on this arc has
found a defect in the orchestrator's brief — including two of my own miscounts. That is the bar.
