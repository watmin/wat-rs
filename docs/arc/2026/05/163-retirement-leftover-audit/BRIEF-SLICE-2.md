# Arc 163 — Slice 2 BRIEF (remaining retirement surfaces)

**Drafted 2026-05-07.** Slice 2 of arc 163 — applies the same
Bucket A/B/C/D framework arc 162 / 163 slice 1 used to four more
prior retirement surfaces.

## Surfaces in scope

Pre-flight orchestrator audit:

| Surface | Retired by arc | Pre-fix sites |
|---|---|---|
| `:Vec<...>` wat-level keyword | 109 slice 1f | 77 |
| `:wat::core::list` keyword | 109 slice 1g | 53 |
| Queue family (`:wat::kernel::Queue*`, `make-bounded-queue`, `make-unbounded-queue`) | 109 K.kernel-channel | 27 |
| `:wat::std::stream::*` namespace | 109 slice 9d | 11 |

Total ~168 sites combined. Most are Bucket C (retirement context
comments) + Bucket D (walker variants like `BareLegacyVecBracket`,
`LegacyKernelQueuePath`, etc. + diagnostic strings). Some Bucket B
comment text + possibly Bucket A live identifiers/dispatch arms.

**Honest concern from orchestrator pre-flight:** at runtime.rs:3088
the substrate has a LIVE dispatch arm `":wat::core::list" => eval_list_ctor(args, env, sym)`.
Arc 109 slice 1g said `list` retires, but the runtime appears to
still alias it. Sonnet must investigate before classifying:
- If the retirement is "soft alias" (transitional scaffolding,
  intentional) → `:wat::core::list` keyword paths in CURRENT
  consumer source = Bucket A (rename to `:wat::core::vec` or
  whatever the canonical replacement is); dispatch arm KEEPS
  (Bucket C-equivalent transitional scaffolding).
- If the retirement is "hard but the runtime arm got missed" →
  flag in honest deltas; don't sweep until orchestrator confirms.

## Working directory

`/home/watmin/work/holon/wat-rs` on `main` branch.

## Workspace state pre-spawn

- HEAD: `a8cc381` (arc 163 slice 1 shipped)
- Working tree clean
- Workspace: 2041 passed / 0 failed
- Bash verified working for sub-agents (probe `which cargo` returns
  `cargo 1.93.0`)

## Bash availability

VERIFIED working. Per memory `feedback_verify_sonnet_tool_claims.md`
— do not falsely claim Bash denial. If you hesitate, run `which cargo`
once to confirm.

## Bucket framework (recap; full classification in arc 162 BRIEF-SLICE-1)

For each site:
- **A** — live identifier using legacy name as concept → RENAME
- **B** — comment text using legacy name in present tense as live
  concept → UPDATE
- **C** — comments recording arc N retirement (historical context)
  → KEEP verbatim
- **D** — orphaned scaffolding per arc 113 precedent (variant +
  Display, walker fns named for legacy form, test fixtures that
  verify the retirement diagnostic) → KEEP verbatim

## Per-surface guidance

### Surface 1 — `:Vec<...>` (arc 109 slice 1f)

Arc 109 slice 1f retired the wat-level `:Vec<T>` keyword in favor
of `:wat::core::Vector<T>`.

Bucket A targets:
- `:Vec<` literal in current consumer wat code (any `.wat` file
  outside `complected/` archive + outside Bucket D fixture files
  testing the retirement) → `:wat::core::Vector<`
- Doc-comment examples using `:Vec<u8>` etc. as if it's the
  canonical form → `:wat::core::Vector<wat::core::u8>` (Bucket B)
- Test embedded-wat fixtures that aren't testing the retirement
  diagnostic → Bucket A consumer code, sweep

Bucket D / KEEP:
- `BareLegacyVec*` / similar walker variant (search for it)
- `validate_legacy_vec_bracket` walker fn if exists
- `tests/wat_arc115*` if it exists (arc 115 was the slice that
  made `:Vec<:String>` illegal at compile time)

### Surface 2 — `:wat::core::list` (arc 109 slice 1g)

Arc 109 slice 1g retired `:wat::core::list` in favor of `:wat::core::vec`
(or `:wat::core::Vector`?). **VERIFY first** by reading:
- `src/runtime.rs:3084-3088` — comment + dispatch arm
- arc 109 slice 1g INSCRIPTION (search `docs/arc/2026/04/`)

If the retirement is soft-alias: sweep consumer keyword usage as
Bucket A; KEEP the dispatch arm as transitional scaffolding.
If the retirement is hard but missed: flag as honest delta;
DO NOT touch without orchestrator confirmation.

### Surface 3 — Queue family (arc 109 K.kernel-channel)

Arc 109 K.kernel-channel renamed:
- `:wat::kernel::QueueSender` → `:wat::kernel::Sender`
- `:wat::kernel::QueueReceiver` → `:wat::kernel::Receiver`
- `:wat::kernel::QueuePair` → `:wat::kernel::Channel`
- `:wat::kernel::make-bounded-queue` → `:wat::kernel::make-bounded-channel`
- `:wat::kernel::make-unbounded-queue` → `:wat::kernel::make-unbounded-channel`

Bucket A:
- Live keyword usage in current consumer wat (any `.wat` outside
  `complected/` and outside arc 109's retirement-fixture tests)
- Comment examples using `Queue*` / `make-*-queue` as if current
  vocabulary

Bucket D / KEEP:
- The retirement-walker comment block in `src/check.rs:445-455`
  documenting the rename map
- Any `BareLegacyKernelQueue*` variant
- Test fixtures verifying the retirement diagnostic fires

### Surface 4 — `:wat::std::stream::*` (arc 109 slice 9d)

Arc 109 slice 9d promoted stream out of the `std` namespace.
Replacement: `:wat::stream::*` (no `std::` prefix).

Bucket A:
- Live `:wat::std::stream::*` keyword in consumer wat
- Comment examples treating the legacy prefix as current

Bucket D / KEEP:
- Walker that fires on legacy prefix (search `src/check.rs:1614`
  area + adjacent walker definition)
- Diagnostic strings shown to users

## Pre-flight crawl

1. Read `docs/arc/2026/05/163-retirement-leftover-audit/DESIGN.md`
2. Read `docs/arc/2026/05/163-retirement-leftover-audit/BRIEF-SLICE-1.md` (let*)
3. Read `docs/arc/2026/05/162-lambda-internal-rename/BRIEF-SLICE-1.md` (canonical Bucket A/B/C/D framework)
4. Read `src/runtime.rs:3080-3100` — `:wat::core::list` dispatch arm
5. Read `src/check.rs:440-460` — Queue family rename comment block
6. Read `src/check.rs:1614` — `:wat::std::stream::*` walker
7. Sample one of each surface (cargo doc-comment vs wat consumer)

## Audit baselines (run BEFORE editing)

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release --workspace 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pre passed:", passed, "Pre failed:", failed}'
echo "Surface 1 :Vec<: $(grep -rn ':Vec<' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
echo "Surface 2 :wat::core::list: $(grep -rn ':wat::core::list' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
echo "Surface 3 Queue family: $(grep -rn ':wat::kernel::Queue\|make-bounded-queue\|make-unbounded-queue' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
echo "Surface 4 :wat::std::stream: $(grep -rn ':wat::std::stream' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
```

## Procedure (per surface, in order: stream → Queue → list → Vec)

Smallest first builds confidence; biggest last lets earlier-surface
patterns inform Vec sweep approach.

For each surface:
1. **Read the retirement context** — find the arc INSCRIPTION + walker
2. **Classify each hit** per Bucket A/B/C/D
3. **Apply Bucket A renames** — for live keyword usage in consumer
   wat, replace via Edit `replace_all: true` per file
4. **Apply Bucket B updates** — comment text using legacy name as
   live concept
5. **Verify** — `cargo build --release` after each surface; if
   anything breaks, STOP and report
6. **Audit grep** — confirm count drops to ~Bucket C/D floor

## Verification (after all four surfaces)

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release 2>&1 | tail -3
cargo test --release --workspace --no-fail-fast 2>&1 | grep -E "test result" | awk '{passed+=$4; failed+=$6} END {print "Pass:", passed, "Fail:", failed}'

echo "Post-fix audits:"
echo "Surface 1 :Vec<: $(grep -rn ':Vec<' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
echo "Surface 2 :wat::core::list: $(grep -rn ':wat::core::list' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
echo "Surface 3 Queue family: $(grep -rn ':wat::kernel::Queue\|make-bounded-queue\|make-unbounded-queue' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
echo "Surface 4 :wat::std::stream: $(grep -rn ':wat::std::stream' --include='*.rs' --include='*.wat' . 2>/dev/null | grep -v complected | wc -l)"
```

Test count must stay 2041 (or higher).

## Constraints

- DO NOT commit. Working tree dirty for orchestrator review.
- "STOP at unexpected red" — stop and report; don't paper over.
- The `:wat::core::list` runtime alias arm is suspect — DO NOT
  touch it without orchestrator confirmation. If sonnet's audit
  confirms it's transitional scaffolding (Bucket C) per arc 109
  slice 1g's INSCRIPTION, KEEP the arm; only sweep consumer keyword
  usage. If unclear, surface it and STOP for orchestrator decision.
- Time-box: 60 min wall-clock.

## Reporting (~250 words)

1. Pre-fix → post-fix per-surface counts
2. Per-surface Bucket A renames count + Bucket B updates count
3. Test pass: pre vs post (must stay 2041 or higher)
4. Path classification: Mode A / B / C
5. Honest deltas — explicitly answer:
   - **`:wat::core::list` runtime arm**: confirmed transitional
     scaffolding (Bucket C, KEEP) or missed retirement (flag for
     orchestrator)?
   - Live consumer wat code in `wat-scripts/` or similar that the
     original retirement arcs missed (analogous to arc 154's
     wat-scripts/ leftover surfaced in slice 1)
   - Any walker variant (`BareLegacy*Vec`, `BareLegacyKernelQueue*`,
     etc.) you preserved as Bucket D — list them
   - Hybrid sentences mixing live + retirement context — how split?

DO NOT commit. Orchestrator commits + scores after.

## Time-box

60 minutes wall-clock.
