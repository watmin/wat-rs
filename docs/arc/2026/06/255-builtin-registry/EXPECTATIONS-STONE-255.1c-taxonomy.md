# EXPECTATIONS — STONE 255.1c-taxonomy

Written before the strike. Rows 6–8 are the ORCHESTRATOR's.

| # | what | who | expected |
|---|---|---|---|
| 1 | ★ the subject line no longer says "computation" | rider + orchestrator, read the diff | names DOING, and covers registration + check-time contracts |
| 2 | five variants APPENDED, in order, none inserted mid-list | read the diff | `:Resource` `:Message` `:Ambient` `:Project` `:CheckGate` after `:Declaration` |
| 3 | each new block states its DOING **and** at least one NOT | read the diff | matches the house style every existing variant follows |
| 4 | ★ `:Message`'s prose comes from the AMENDMENT, not the refuted section | read the diff | contains the "transport is an implementation detail, NEVER the axis" clause; contains NO "two of four tiers" argument |
| 5 | `:Io` gains only the contrast clause | read the diff | otherwise byte-identical |
| 6 | the build consumes the enum | **orchestrator** | `cargo build --release` green |
| 7 | floor | **orchestrator** | 4819 run, 0 FAIL, 19 skipped |
| 8 | clippy · ignores | **orchestrator** | 0 · 13 |
| 9 | no verb re-categorized | orchestrator, `git diff --stat` | no `src/intrinsic/` row edits |

**Row 4 is the one a careless strike fails**, because the stone's own earlier text argues the refuted
case and a rider reading top-down meets it first. This is deliberate: the stone preserves the wrong
argument as history rather than deleting it. The brief says so twice.

**Row 1 is load-bearing for row 2.** `:CheckGate` under the old subject line is a category error —
the ward's objection, satisfied only by the amendment shipping.

## Independent prediction

**25–40 minutes.** Two prose edits and five appends to one file. The time goes into the prose, which
ships as generated Rust `///` docs — not into mechanism.

## Trap-doors named in advance

- **The stone contradicts itself by design.** Refuted argument above, correction below. Anyone who
  stops reading early ships the wrong justification into the language's own documentation.
- **Append-only.** A mid-list insert renumbers the generated enum and is invisible in prose review.
- **Compile-time, not test-time.** The derive macro eats this file during `cargo build`; a malformed
  `defenum` fails as a confusing derive error, not a parse error.
- **`:CheckGate` has exactly ONE member today** (`require-wire-address`) and was minted over the
  ward's "wait" on the builder's forward knowledge of the `must-*` family. That is recorded in the
  stone as an override with a revisit trigger — **do not let a rider quietly re-litigate it**, and do
  not let the empty-ish membership read as an error.
