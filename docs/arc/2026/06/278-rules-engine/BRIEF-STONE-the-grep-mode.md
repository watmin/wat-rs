# BRIEF — STONE: `--grep` (part B — the CLI mode)

DESIGN: `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-grep-mode.md` — read it whole, first.

**Part A shipped** (`78e8004f5`, `00b28bc37`): `:wat::grep::run` is live in the stdlib and proven —
the loop, the per-file isolation with a perturbation control, and a `Match` whose every field is
real. **Part B is the plumbing that lets a user reach it.** After this stone,
`echo '["a.wat"]' | wat --grep prog.wat` works.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every command in the FOREGROUND and block on it. Your turn ends when the
numbers are in your hands, not when a command is launched.

**You may not spawn sub-agents.**

Anchor: `/home/john/work/holon/wat-rs`. `pwd` first. You do not commit, push, stash, revert, or
checkout — leave your work uncommitted and report.

`cargo build --release` is yours (~19s). `cargo nextest`, `scripts/floor.sh` and clippy are NOT —
the orchestrator takes those centrally. Cap every command:

```
systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=6G  -p MemorySwapMax=0 timeout 180 ./target/release/wat <args>
```

## The work in one paragraph

Add a `Grep` mode to the CLI. It parses like `--repl`/`--mcp` do, shares the Run path's file read and
freeze, then diverges at exactly two points: it validates `:user::grep` instead of `:user::main`, and
it invokes `:user::grep` then hands the result to `:wat::grep::run` instead of calling
`invoke_user_main`.

## ⛔ THE ONE THING THAT WILL BITE IF YOU MISS IT

`src/distribution/mod.rs:443` calls `validate_user_main_signature(&world)` **unconditionally** on the
Run path and exits `EXIT_MAIN_SIGNATURE`. Its own comment explains why it lives there:

> *"`startup_from_source` imposes it only WHEN `:user::main` is declared … A program that declares
> NO main therefore freezes clean and must be caught here."*

**A `--grep` program has no `:user::main`.** If Grep flows through that call it is refused by a wall
about a function it is not supposed to have. Grep needs its own arm with its own wall on
`:user::grep`. **Row 3 is the negative control that proves the bypass is real rather than
accidentally satisfied** — do not skip it.

## The rooms — read in this order

1. **`src/distribution/argv.rs:60-73`** — the `Mode` enum, and the doc comment on `Run` explaining
   why trailing args are not carried in the variant. Yours is `Grep { entry_path: String }`.
2. **`src/distribution/argv.rs:79-135`** — `parse`. Flags at `:93-95` are recognised only BEFORE the
   entry path; `--grep` joins them there. `:113-118` is where `--check-output`-style
   cross-flag validation lives. `:120` is the usage message — it gains a `--grep` line.
3. **`src/distribution/argv.rs:136-162`** — the mode-selection tail: `mcp` → `repl` → `check_only` →
   Run. Each refuses conflicting flags rather than picking one silently, and says so in a comment.
   **Follow that pattern**: `--grep --repl`, `--grep --mcp`, `--grep --check` and `--grep` with no
   positional are all usage errors, not precedence rules.
4. **`src/distribution/mod.rs:230-242`** — the destructure that turns a `Mode` into
   `(entry_path, check_output_format, check_only)`. `Grep` behaves exactly like `Run` here: it has a
   real entry file that must be read and frozen.
5. **`src/distribution/mod.rs:438-457`** — the divergence. `:443` is the main wall you must not
   reach; `:455` is `invoke_user_main`, which hardcodes `:user::main` (`src/freeze.rs:1381`) and is
   NOT reusable.
6. **`src/freeze.rs:1156-1176`** — `call_beside_value`. A test fixture, so not your call site, but it
   shows the exact shape: `symbols().get(name)` → `apply_function(func, vec![], symbols, span)`.
7. **`src/runtime.rs:25785`** — `pub fn apply_function`. The primitive. Read its signature.
8. **`wat/grep.wat`** — `:wat::grep::run [rules] -> nil`, already live. This is what you call.

## The shape

```rust
// after the freeze, in place of the main wall + invoke_user_main:
let rules = apply_function(sym(":user::grep")?, vec![],      symbols, span)?;
let _     = apply_function(sym(":wat::grep::run")?, vec![rules], symbols, span)?;
```

**Rust does NOT read stdin.** `:wat::grep::run` reads the EDN path vector itself, with the same
`readln` shape every recorded migration uses. If you find yourself reaching for stdin in Rust, stop —
the driver already does it and it is proven.

**The `:user::grep` wall** mirrors `validate_user_main_signature`'s job, not its text: a missing or
wrong-shaped `:user::grep` must produce a located, structured diagnostic naming `:user::grep` and
saying what signature was expected. Read the main wall's message (`src/freeze.rs:1620-1631`) for the
register — it names the arc, the reason, and the canonical signature.

## Your probe

`wat-scripts/scratch-pad/probe-grep-cli.wat` — a program declaring `:user::grep` (and NO
`:user::main`) returning a vector of rules. Copy the rules from
`wat-scripts/scratch-pad/probe-grep-driver.wat`, which is proven. It must load under
`every_wat_scripts_file_loads`; a program with no main checks clean (measured).

## The acceptance rows YOU run

- **Row 1 — end to end.** `printf '["<file>"]' | ./target/release/wat --grep <probe>` prints the
  Matches that program's rules assert. Output verbatim.
- **Row 2 — the file names are real.** Run TWO different files in one invocation and show two
  different `:file` values. Part A's fix made this possible; this row proves it survives the CLI.
- **★ Row 3 — a program with NO `:user::grep` is refused, and the diagnostic names `:user::grep`.**
  The negative control for the bypass. If the message mentions `:user::main`, the Grep path is
  still going through the Run path's wall and the stone has not shipped. Report the message verbatim.
- **Row 4 — a program with `:user::grep` and NO `:user::main` runs clean.** The direct proof that
  `mod.rs:443` is not on this path.
- **Row 5 — facts do not leak between files, THROUGH THE CLI.** Two files, both orderings, exactly
  as part A proved through the driver. Re-proving it at the real surface is the point.
- **Row 6 — the grammar refuses conflicts.** `--grep --repl`, `--grep --mcp`, `--grep --check`, and
  `--grep` with no positional each produce the usage error and exit 64. Report each.
- **Row 7 — the usage message mentions `--grep`.** Print it verbatim.
- **Row 8 — the stop protocol, ANSWERED BY MEASUREMENT.** `invoke_user_main_orchestrated` performs
  a stop-ask-and-await on main's way out that your path skips. Determine whether Grep needs it:
  write a probe whose driver leaves something live, run it, and report what happens. **State the
  answer and the evidence.** A guess here is either a resource leak or a hang.

Report each row's command and output **verbatim** — never a summary, never a `| head`/`| tail`
window. A row you could not run is reported as not-run, never as passed.

## Blast radius

- `src/distribution/argv.rs` — the `Mode` variant, the flag, the validation, the usage message
- `src/distribution/mod.rs` — the destructure arm and the Grep dispatch
- `wat-scripts/scratch-pad/probe-grep-cli.wat` — created

Nothing in `wat/`. No other `src/` file unless a STOP says why.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **`apply_function` cannot be called from the distribution layer** — a visibility, borrow, or
   world-lifetime problem you cannot resolve inside the two files above. STOP and report the exact
   compiler error; do not restructure `freeze.rs` to route around it.
2. **The rules value cannot cross from `:user::grep` into `:wat::grep::run`** — a type or arity the
   call cannot satisfy. STOP and report the runtime error verbatim.
3. **Row 8 shows Grep DOES need main's orchestration** and you cannot get it without calling
   `invoke_user_main`. STOP and report what you observed — that is a real finding about the stop
   protocol's shape, and inventing a partial version of it silently is how a hang ships.
4. **Anything requires editing `wat/` or a third `src/` file.** STOP — that is a finding for the
   orchestrator.

A STOP means: leave the tree as it is, write the report, end your turn. It is never a licence to
ship a smaller version of a row.

## What you own that nobody can reconstruct

Row 3's exact diagnostic text, row 8's evidence and your reading of it, and anything that surprised
you — a place the Run path's plumbing assumed a main, a lifetime that fought you, a message that
read wrong.
