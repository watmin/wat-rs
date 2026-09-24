# BRIEF — STONE J: rename the `wat::holon` Value variants to `wat__holon__*`

Read `DESIGN-stone-J-the-holon-family-gets-the-naming-treatment.md` first — its table is the scope
and its "does NOT change" list is the danger map.

## The work

Rename the seven variants in `src/value/value.rs`'s `enum Value` and every `Value::<Name>` use across
`src/`, `crates/`, `tests/`, `benches/`, `examples/`. Pure rename — no behaviour change. Re-derive the
reference counts yourself first; report any delta from the DESIGN's table.

## How — a word-boundary, `Value::`-anchored rewrite, verified

`Vector` collides with `holon::Vector`, `OwnedValue::Vector`, `Edn::Vector` and string literals. Match
ONLY `Value::Vector` (word boundary after), plus the variant's own declaration line. Same discipline
for the other six. Then check: `git diff` contains no change to any `"…"` string literal and no
`OwnedValue::`/`Edn::`/`holon::` line; the build is the proof that nothing was missed.

(The wat-fix codemod doctrine in `CLAUDE.md` is for `.wat` corpora; this is Rust. A throwaway rewrite
script is fine — delete it before commit, and read the whole diff.)

## STOP triggers

1. A `Value::<Name>` use that is NOT the variant (e.g. a type alias, a macro that builds the name
   from tokens, `paste!`-style concatenation) — report it rather than guessing.
2. A test asserting `Debug` output that contains a variant name — report it (`Value` derives `Debug`).
3. Any gate reddens that you did not add — capture whole, name the arm. ⛔ Do not re-run first.

## Prove it

- `grep -rnwE 'Value::(Vector|Hologram|Engram|EngramLibrary|OnlineSubspace|Reckoner|holon__HolonAST)'`
  over the code roots → **0**.
- No string literal changed (show the check).
- Floor 0 failed; clippy clean.
- ⚠ No mutation proof applies to a rename — the compiler is the gate. Say so; do not stage a fake one.

## Mechanics — ⛔ read

- Floor: `nohup scripts/floor.sh > <file> 2>&1 &`, then repeated foreground
  `until grep -qE '^ *Summary' <that file>; do sleep 30; done` blocks. Do not end your turn while it
  runs. Never `pgrep -f 'cargo …'`. Read `Summary` lines, never piped exit codes.
- `cargo fmt` reformats the whole workspace — `rustfmt <file>` or neither. Stage explicit paths.
