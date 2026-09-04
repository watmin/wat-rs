# BRIEF — the reactor grows a seam (v3)

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`912f2dc56`. Read `DESIGN-the-reactor-grows-a-seam-v3.md`, then
`SCORE-the-reactor-grows-a-seam-v2.md` for why v2 died.

⚠ **v2's extraction may still be in your working tree.** It was correct except for *where* the helper
sits. Reuse it; move the `defn` to the end of the file.

## THE WORK

Same extraction as v2. **One change: the helper goes at the END of `wat/service.wat`, after the
macro** — so no line above 896 moves and the four `peers_bijection` goldens stay green.

## ROOMS

1. **`wat-scripts/scratch-pad/probe-template-calls-a-later-defn.wat`** — **run it first.**
   `FORWARD-REFERENCE-OK`, 3/3. A macro's template can call a `defn` defined below it.
2. **`wat-scripts/scratch-pad/probe-send-seam-parametric.wat`** — `SEAM-EXPRESSES`, 3/3. The
   signature: `:- [R O]`, `peer <- (Peer :- [:R :O])`, `payload <- :R`.
3. **`wat/service.wat:1854-1858`** — the shape. Four arms, `Stopped → false`.
4. **`1659 1697 1784 1811`** — the other four callers. Confirm before touching.
5. **`wat/service.wat:64`** — the sentence that *is* the `Closed → true` arm. Do not change it.
6. **`tests/services/probe_arc278_peers_bijection__case1_*.edn`** — carries `:line 896`. **Read one**
   so you can see what the placement is protecting.

## ⛔ DO NOT TOUCH — name each in the report

`1828` · `1939` `1950` · `2006` `2012` — and the nine never-candidates:
`2025 2045 2100 2239 2324 2368 2420 2470 2620`.

## STOP TRIGGERS

1. **A bijection golden goes red.** Something above 896 moved. **Do not patch the `.edn`** — report
   it. That was v2's correct refusal and it stands.
2. **Any arm disposition changes.** STOP.
3. **An excluded site is swept in.** STOP.
4. **You are about to add the drop.** R2. STOP.
5. **You are about to touch `src/` or another `.wat`.** One file. STOP.

## HOW TO WORK

Foreground everything. **Floor after the edit, never before.** Summary line, never a piped exit code.
On an unintended red: **do NOT re-run**, capture whole, name the arm.

⚠ **Do not write `(:wat::core::None <Type>)`** — phantom form, arc-109 NOTE.

Leave your work uncommitted. Prior comparable: `SCORE-the-call-site-reads-as-english.md`.

## REPORT

- both probes, re-run
- **the four bijection goldens: green or red.** This is the placement's proof
- the floor Summary line, run after the edit
- `grep -n 'kernel::send' wat/service.wat` — **expect 15**, each classified
- the circuit, five runs
- what R2 needs from the helper — say it, do not build it
- every STOP that fired
- **the honest deltas.** Ten of my counts have missed this campaign; v2's row 2 predicted six sends
  where there are fifteen. What you find is the fact.
