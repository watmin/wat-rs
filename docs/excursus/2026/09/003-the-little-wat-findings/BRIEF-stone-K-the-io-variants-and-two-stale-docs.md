# BRIEF — STONE K: finish the naming rule (`io__*`) and fix two stale docs

**Drawn 2026-09-24.** Stone J's executor found the rule broken outside `wat::holon`. Rule (stone J's
DESIGN): a `Value` variant is its wat type path with `::` as `__`.

## The work

1. **Rename** `Value::io__IOReader` → `Value::wat__io__IOReader` and `Value::io__IOWriter` →
   `Value::wat__io__IOWriter` (type paths `:wat::io::IOReader` / `:wat::io::IOWriter`,
   `src/value/value.rs:1548,1553`). 18 + 15 references, measured. Same discipline as stone J: only the
   Rust identifier moves; EDN tag strings (`"wat.io", "IOReader"` …) and type-path strings stay.
   Update messages/comments/Debug expectations that name the variant, as stone J did (ruled
   acceptable, `RULING-a-childs-stdout-is-a-wire.md` §2).
2. **Census the WHOLE enum against the rule** — report every variant that still breaks it, with its
   type path. Stone J's DESIGN claimed "every one is `wat::holon`" and was wrong; do not repeat that
   by assumption. The unprefixed core primitives (`i64`, `String`, `Vec`, `Option`, `Tuple`,
   `Aggregate`, …) are a separate question — list them, do not rename them.
3. **`README.md:222`** cites `Value::crossbeam_channel__Sender`, retired before arc 170 per
   `value.rs:84`. Correct it to what exists today.
4. **`CLAUDE.md:57`** says the floor is `~4189/0`; it is 6247 today. ⛔ Do not replace one number with
   another — a count in prose rots as the suite grows, which is how this one went stale. State the
   invariant instead: the floor is **0 failed**, read from the `Summary` line of
   `.floor/latest/clean.log`.

## STOP triggers

1. A census row you are unsure how the rule applies to — report it, do not rename it.
2. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Prove it

Zero `Value::io__` left in code roots; no EDN tag / type-path string changed (show the check);
floor 0 failed; clippy clean. The compiler is the gate for the rename — no mutation proof.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `wat_edn::Value` is a DIFFERENT enum (it bit stone J's grep); scope every grep to the wat `Value`.
- `cargo fmt` reformats the whole workspace — `rustfmt <file>` or neither. Stage explicit paths.
