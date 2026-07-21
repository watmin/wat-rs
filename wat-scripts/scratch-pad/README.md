# `wat-scripts/scratch-pad/`

**Scratch / reconnaissance / throwaway `.wat` programs live here** — not in the ephemeral
session scratchpad (`/tmp/.../scratchpad`).

Why in the repo: a scratch `.wat` is a *durable, loadable* reference, and the
`every_wat_scripts_file_loads` gate (`tests/lint/wat_scripts_fixes_load.rs`) parses +
type-checks **every** `.wat` under `wat-scripts/` — including this dir, recursively — on the
current runtime. So a scratch program that rots (stops conforming to the substrate as it
evolves) goes **RED** and cannot hide as a graveyard that reads like live code. All wat stays
correct, always — even the scratch.

Consequence: scratch here obeys the substrate's current rules (e.g. an op-Response is an
outcome enum carrying `RequestTooLarge`, arc-278 ruling A). If a scratch program is genuinely
dead, delete it; if it's kept, it conforms.

Non-`.wat` temporary files (logs, patches, intermediate data) still belong in the session
scratchpad, not here.
