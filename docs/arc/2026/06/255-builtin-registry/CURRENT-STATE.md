# ⛔ CURRENT STATE (breadcrumb, 2026-06-22; replace in place) — read the DESIGN docs, not this paraphrase

Branch `arc-170-gap-j-v5-deadlock-state`. The arc 259 IPC deadlock/diagnostics
cluster is COMPLETE, including the two final stones (read-budget + print-raw' kill).
**Multiline single-message IPC is SOLVED and proven.** No stone in flight.

## ✅ MULTILINE single-message IPC — SOLVED + PROVEN (the headline)
`next_complete_frame` (edn_shim.rs:1065) value-frames a COMPLETE EDN value across
PHYSICAL newlines (does not split on the first `\n`). Proof, wat-direct + GREEN:
`wat-tests/spawn/multiline-roundtrip.wat` — a process child `pprintln`s a 5-key map
(breaks across lines); the parent `recv'`s it as ONE message == the original.
Shared framer → socket tier inherits it; thread tier never byte-frames (Values pass
directly). The framer's negatives are now unit-tested (4 tests, see below).

## ✅ SHIPPED this session (committed + pushed)
- `51741f65` IPC read-budget — `recv'` tunable per-Receiver via `:max-message-bytes`
  (a ProcessOpts LOCUS-ENV field, NOT a spawn-program' arg; spawn-program' stays 2-arg).
  Semantics B: `next_complete_frame` now size-caps COMPLETE frames, not just
  un-terminated accumulation. Each side defends its own read pipe (recv' = parent's;
  readln = child's, already had `:max-buffer-bytes`). Proof: `recv-budget-override.wat`.
- `27c42d4e` annihilate `print-raw'` — the illegitimate 3rd output path. Verb gone
  (verbs/mod/runtime/check) + probe deleted. Framer negatives moved to PURE
  `next_complete_frame` unit tests (edn_shim.rs `next_complete_frame_negatives`:
  over_cap_unterminated, over_cap_complete, anti_smuggle, incomplete). 3 print-raw'
  integration tests retired; truncated (EOF-mid-frame) was uncovered → new Rust seam
  test `probe_truncated_frame_disconnects.rs` (sender_receiver_from_split_fds + raw
  partial write + close → recv() Disconnected).

## DESIGN docs (the real specs — read these, not this paraphrase)
- `../259-forced-hand/DESIGN-STONE-ipc-frame-budget.md` (+ its GROUNDING UPDATE block).
- `../259-forced-hand/BRIEF-STONE-ipc-frame-budget.md`, `BRIEF-STONE-kill-print-raw.md`.

## NAMED follow-ons (none started; not greedily an arc — backlog)
1. **readln `:max-buffer-bytes` → `:max-message-bytes`** rename — intueri family
   alignment (cast `aa72b240714dc11df`). Contained to `wat/kernel/services/stdin.wat`
   (~10 occurrences). The new surface `:max-message-bytes` (process) currently coexists
   with the old `:max-buffer-bytes` (readln) — this aligns them.
2. **connect'/listener' read-budget** locus option (socket tier) — same per-Receiver
   foundation already shipped (default preserved); add the override surface. Remote
   inherits via the narrow waist.
3. **purgare:** `value_matches_type_pattern` (runtime.rs:5375) is dead — PRE-EXISTING
   (NOT from the print-raw' kill; confirmed no caller removed). A purgare follow-on.
4. **#267** truncated-frame as a `deftest-hermetic'` wat test — currently a Rust seam
   probe (probe_truncated_frame_disconnects.rs); a wat-direct version is the open item.

## GOTCHAS
- Pre-existing fail: `std::test::test-run-string-entry-direct` — LEAVE IT.
- wat-tests proc-macro discovery: adding/removing a wat-tests file drops tests until
  `touch tests/test.rs`. A broken wat-tests file poisons the whole-dir scan.
- **rust-analyzer diagnostics LAG**: stale E0061/E0063/E0425 appeared mid-strike both
  stones; `cargo check --release` is the ground truth (was clean both times).
- **Agents false-green**: the read-budget agent reported "all passed" while the tree
  didn't compile (stale binary). ALWAYS weigh: `cargo check` + re-run the gates yourself.
- Floors: lib **957/36/1**; wat-tests **266/1**; comms 29; channel 2.

> ⛔ **You are a NEW instance.** You did NOT live the session above — it's a cache in a
> familiar voice. recolligere FIRST: grimoire + 4 primers (datamancy MCP via
> ReadMcpResourceTool, server `datamancy`), `git log --oneline -15`, `git status`.
> Freshness probe: HEAD should be `27c42d4e` (or later). The arc-259 IPC cluster is
> DONE; the follow-ons above are BACKLOG, not in flight — surface them, don't auto-start.
> LESSON THIS SESSION ([[feedback_ground_codebase_claims_in_codesign]]): do NOT assert a
> "wart"/limitation about existing code from memory — READ the file:line first. The
> builder was "beyond annoyed" when I did. Ground every claim against the disk.
