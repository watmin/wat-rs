# POLICY — a replayed step that changes a codemod's source (RULED 2026-09-15: P1 and Q1)

Measured 2026-09-15 (git objects only). 12 steps change `wat-scripts/fixes/`: #153 #155 #157 #159 #278
#438 #440 #627 #628 #630 #635 #638. Two classes.

## Class A — grok-rete's OWN new recorded migrations (7 tools, none on main)

`wrap-fire-once-in-fireoutcome` (#153), `wrap-fire-rules{,-explain}-in-fireoutcome` (#155),
`wrap-insert-in-insertoutcome` (#157), `wrap-compile-in-compileoutcome` (#159),
`wrap-session-facts-in-factbag` (#438, #440), `hoist-where-into-condition` (#627 #628 #630). Each
migrated grok-rete's own corpus when a rete API became total; 150–370 lines of wat in grok-rete's syntax.
On grok-rete's tip none has a replay fixture, a `rune:replay`, or a `;; SCOPE:` line.
`tests/cli/every_recorded_migration_replays.rs` is MAIN-ONLY (stone 0a, `0d51a1305`, 2026-09-12);
`tests/lint/wat_scripts_fixes_load.rs` is on both — so on main each tool must LOAD in main's syntax AND
carry a fixture. The step's corpus edits (the tool's EFFECT) replay through the chain like any `.wat`.

| option | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **P1 port:** convert the tool through `convert.sh` like any `.wat`; bring any string-literal pattern to today's spelling; add `;; SCOPE:`; its fixture = one corpus file the step migrated — `before.pre` its converted C^, `after.post` its converted C — ORACLE the header spec; the replay gate proves the tool reproduces its own step | YES | YES (one recipe per tool; the fixture is data the step already produces) | YES | YES (a working migration for pre-totality rete code) |
| P2 carry the effect, drop the tool | YES | YES | NO (loses branch work with no author's reason — the doctrine's gate 1) | — |
| P3 verbatim | YES | YES | NO (fails main's load wall and replay gate: knowingly red) | — |
| P4 a `rune:replay(unreadable-preimage)` instead of a fixture | YES | YES | NO (the preimage IS readable — the converted C^) | — |

## Class B — grok-rete's edits to MAIN's tools and probes (on main since before the fork)

`to-faithful-clojure-{rete,net}`, `rete-oracle-sigil`, `rete-where-per-type-spelling`,
`type-query-to-defquery` (+1/−1 … +43/−3), and four `rete-truth-maintenance-probes/*.wat` programs —
adapting main's tooling to grok-rete's rete API changes.

| option | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| **Q1 re-express on main's version** (main owns the tool — the `.rs` rule, for tooling): grok's change in today's syntax; the tool's existing fixture still replays; a behaviour change adds a fixture row | YES | YES | YES | YES |
| Q2 take grok-rete's version | YES | YES | NO (overwrites main's tooling with pre-migration text) | — |

## The boundary rule it implies

A codemod step is no longer a boundary once ruled: STOP-6 becomes "a codemod step the policy cannot
carry". #278 stays a boundary for its OTHER cause (a file main deleted for cause — finding 21).
