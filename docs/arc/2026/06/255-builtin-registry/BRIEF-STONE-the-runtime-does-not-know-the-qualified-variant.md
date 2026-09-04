# BRIEF — STONE: teach the runtime the qualified builtin-variant spelling

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Any path containing `.claude/worktrees/`
is harness state — never operate on it. Do not commit, push, stash, or revert. Do not run the full
floor; the orchestrator runs it centrally.

Read `DESIGN-STONE-the-runtime-does-not-know-the-qualified-variant.md` (sibling) first — it carries
the five probes, two of which are the controls that make this a located defect rather than a guess.

## The work in one paragraph

`Value::Option` and `Value::Result` are native `Value` variants, so builtin `Some`/`None`/`Ok`/`Err`
never went through the generic enum path. The checker accepts the fully-qualified
`:wat::core::Option::Some` spelling that every user enum uses; the runtime's six hardcoded guards
only know `:Some` / `:wat::core::Some`. Add the qualified spelling to those six guards. **Additive
only** — nothing is removed.

## Rooms, in order

1. **`src/value/value.rs:138` and `:145`** — `Value::Option` / `Value::Result`. Read them first so
   you know why these four variants are special at all.
2. **`src/runtime.rs:8324`** — the match-pattern guard for `None`, and the arc-109 comment above it
   calling `:wat::core::None` "the FQDN form". That comment needs updating: a third, more qualified
   form now also works.
3. **`src/runtime.rs:8412`, `:8441`, `:8467`** — the `Some` / `Ok` / `Err` pattern guards.
4. **`src/runtime.rs:1715`** and **`:13011`** — the two remaining sites carrying the same spellings.
   Judge each: `13011` is a `matches!` over three names, `1715` is a value-position check. Extend
   them only if the qualified spelling can actually reach them; if it cannot, say so in your report
   rather than adding an unreachable arm.
5. **`src/types.rs:1056`** — the `nil` comment describing *"additive recognition; both spellings
   evaluate to the nil singleton."* That is the house precedent this stone follows; mirror its
   phrasing where you leave a note.

## The acceptance probe — it must FAIL first

Write it in `wat-scripts/scratch-pad/` (durable, loader-gated) covering all four builtin variants
under the qualified spelling: `Option::Some`, `Option::None`, `Result::Ok`, `Result::Err`.

⛔ **Run it BEFORE your change and capture the failure.** It must raise `PatternMatchFailed`. A probe
first seen green proves nothing — this arc has a memory named for it
(`[[feedback_a_green_test_can_prove_nothing]]`), and the round-trip probe in this same campaign once
returned a false perfect score.

Then make the change, rebuild, and run it again. `.wat` stdlib files are `include_str!`ed, so every
`--check`/run must follow a `cargo build --release`.

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — do not remove or narrow any existing spelling. `:None`, `:wat::core::None` and their
  siblings keep working. This stone is additive; the bare bridge comes down in the codemod's stone,
  and demolishing it before the far side is walkable is how an anneal fails in the other direction.
- **STOP-2** — do not touch `src/check.rs`. Its 21 sites carry the same spellings and are already
  correct: probe 4 in the DESIGN proves the checker recognises the qualified form and refuses a
  nonsense one. Changing code that is not wrong is how a small stone grows a blast radius.
- **STOP-3** — do not write the corpus codemod, and do not repoint a single call site. This stone
  makes the target spelling work; migrating to it is a separate stone with its own dry-run and diff.
- **STOP-4** — if a site in room 4 cannot be reached by the qualified spelling, do NOT add an arm
  there to be tidy. Report it. An unreachable arm is exactly the shadowed-residue defect this
  campaign has deleted 52 of.

## Verification

```
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::types)'
cargo nextest run --release -E 'binary_id(wat::value)'
cargo nextest run --release -E 'binary_id(wat::wat_lang)'
cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'
cargo clippy --release --all-targets -- -D warnings
```

## What to report

Your probe's output BEFORE the change (verbatim, showing the raise) and after; which of the six
sites you extended and which you did not, with the reason for each; the Summary line per scoped
run; and anything that surprised you — particularly if any site in room 4 turned out unreachable.
