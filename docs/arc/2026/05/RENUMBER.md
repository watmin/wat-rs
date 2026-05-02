# 2026/05 — Renumber pending

**Status:** queued cleanup. **Do NOT execute mid-flight.** Wait until all in-flight May arcs have closed (INSCRIPTION shipped, no open work).

## The mistake

May 2026 was opened with arc numbers continuing from April (which ended at 119). The convention should have been `2026/05/NNN-slug` starting at `001`. The current 16 May arcs are numbered 120-135.

```
2026/04/  →  ends at 119
2026/05/  →  120-135  ← WRONG: should start at 001
```

## The fix

Rename each May arc dir to start the month at 001:

| Current | Slug | New |
|---|---|---|
| 120-parametric-user-enum-match | parametric-user-enum-match | 001 |
| 121-deftests-as-cargo-tests | deftests-as-cargo-tests | 002 |
| 122-per-test-attributes | per-test-attributes | 003 |
| 123-time-limit | time-limit | 004 |
| 124-hermetic-and-alias-deftest-discovery | hermetic-and-alias-deftest-discovery | 005 |
| 125-rpc-deadlock-prevention | rpc-deadlock-prevention | 006 |
| 126-channel-pair-deadlock-prevention | channel-pair-deadlock-prevention | 007 |
| 127-thread-process-symmetry | thread-process-symmetry | 008 |
| 128-check-walker-sandbox-boundary | check-walker-sandbox-boundary | 009 |
| 129-time-limit-disconnected-vs-timeout | time-limit-disconnected-vs-timeout | 010 |
| 130-cache-services-pair-by-index | cache-services-pair-by-index | 011 |
| 131-handlepool-scope-deadlock | handlepool-scope-deadlock | 012 |
| 132-deftest-default-time-limit | deftest-default-time-limit | 013 |
| 133-tuple-destructure-binding-check | tuple-destructure-binding-check | 014 |
| 134-scope-deadlock-origin-trace | scope-deadlock-origin-trace | 015 |
| 135-complectens-cleanup-sweep | complectens-cleanup-sweep | 016 |

(Append future May arcs as they open — keep this table current.)

## Why wait until end

In-flight references inside DESIGN / BRIEF / EXPECTATIONS / SCORE / INSCRIPTION docs name "arc 130", "arc 131", etc. Renaming directories mid-flight would break ~hundreds of cross-references in committed text. Most safely done as one atomic sweep after every active arc has shipped its INSCRIPTION.

## Execution plan (when the time comes)

1. Confirm no arc dir has uncommitted work (no `M` / `??` under `docs/arc/2026/05/`).
2. Build the rename map (above table — refresh against current dir listing first).
3. `git mv` each directory. Atomic.
4. **Sweep cross-references** in committed prose:
   - Search wat-rs repo: `grep -rn "arc 1[2-3][0-9]" docs/ src/ crates/ wat/ wat-tests/ tests/ examples/ .claude/`
   - Search holon-lab-trading + scratch dirs for references too.
   - For each match: replace `arc 12X` → `arc 00Y` per the table.
5. Search commit messages? — committed git history is immutable; we only renumber going forward. Old commits keep their `arc 130` refs. The renumber is a forward-looking convention.
6. Update memory pointers (`memory/MEMORY.md` + entries that reference May arcs).
7. Run `cargo test --release --workspace` + `git push origin main` to confirm nothing broke.
8. Write the renumber as its own commit:
   ```
   chore: renumber 2026/05 arcs to start at 001
   
   Convention: each YYYY/MM/ starts at NNN=001. May had been
   continuing from April's 119 → 135; renumbered to 001 → 016.
   ```

## Warning

The cross-reference sweep is the load-bearing step. Missing references degrade discoverability. Any agent doing this work should:

- Use `grep -rn` exhaustively before declaring done.
- Spot-check the README + INSCRIPTION + REALIZATIONS files of each renamed arc.
- Verify the newly-named directories load correctly under any tooling that walks the arc tree.

## Cross-references to watch

Especially load-bearing references that will need updating:

- `docs/SUBSTRATE-AS-TEACHER.md` — references arcs 109/110/111/112/113/115/117 (April arcs — unchanged).
- `docs/arc/2026/04/109-kill-std/J-PIPELINE.md` — references downstream arcs 110-117 (April — unchanged) AND any May arc spawned from arc 109's orbit.
- `docs/WAT-CHEATSHEET.md` § 10 + § 11 — references arc 117 + arc 126 + arc 131 + arc 134.
- `docs/USER-GUIDE.md` — references arcs 121-124 + 132.
- `.claude/skills/complectens/SKILL.md` — references arc 130 paths.
- All May arcs' own `INSCRIPTION.md` cross-references.
- `memory/feedback_*.md` — entries referencing recent arcs.

## When considered done

- Every May arc directory starts with a 3-digit number under 100.
- `grep -rn "arc 1[2-3][0-9]" docs/ ...` returns ZERO matches in `docs/arc/2026/05/`.
- `cargo test --release --workspace` exit=0.
- A single commit captures the rename + cross-reference sweep.
- This RENUMBER.md gets either deleted OR moved to a "history of cleanups" record.
