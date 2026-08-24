# DESIGN — STONE: `--grep` (the mode, the loop, the print)

> `wat/grep.wat` (`349a2ea52`) shipped the vocabulary. This ships the harness that dispatches into
> it. After this stone, `echo '["a.wat" "b.wat"]' | wat --grep prog.wat` works.

## The surface

```
echo '["vec" "of" "files"]' | wat --grep grep-program.wat
```

**`--grep`, not `grep`** — builder, 2026-08-24: *"it must be `--grep` to match with `--repl` and
`--mcp`."* The mode grammar is already flag-shaped (`src/distribution/argv.rs:93-95`), flags
recognised only before the entry path. A bare subcommand word would be this CLI's first and would
need a rule separating "the file named grep" from "the grep mode". `--grep` needs no such rule.

## THE SPLIT — Rust is three moves; everything else is wat

```
Rust   parse the mode · freeze the program · wall :user::grep · call it · call the driver
wat    :wat::grep::run — read stdin · with-network · per file: facts-of → insert → fire
                          → q-match → print → reset
```

`282-wat-fix-over-rust` is an arc in this repo. The loop, the lifetime, the reset and the printing
are wat; Rust contributes a `Mode` variant, a wall, and two `apply_function` calls.

**Rust does NOT read stdin.** The driver does, with `:wat::kernel::readln` — the identical shape
every recorded migration already uses (`wat-scripts/fixes/angle-brackets-to-binder.wat:296`). This
stone does not invent a convention; it moves a hand-copied one into the substrate.

## ⛔ THE WALL THAT WOULD HAVE BITTEN — `:user::main` is validated UNCONDITIONALLY on the Run path

`src/distribution/mod.rs:443` calls `validate_user_main_signature(&world)` and exits
`EXIT_MAIN_SIGNATURE` on failure. Its own comment says why it lives there rather than in freeze:

> *"`startup_from_source` imposes it only WHEN `:user::main` is declared … because
> `startup_from_forms` must stay usable by callers that legitimately build worlds without a main.
> A program that declares NO main therefore freezes clean and must be caught here."*

**A `--grep` program has no `:user::main`.** Routed through the Run path it would be refused by a
wall about a function it is not supposed to have. So `Mode::Grep` gets its own dispatch arm with the
mirror wall — on `:user::grep` — and never reaches main's.

Measured, so the mirror is built on fact: a program declaring only
`(:wat::core::defn :user::grep [] -> :i64 42)` freezes clean (`--check EXIT=0`); the freeze-time
main wall (`src/freeze.rs:941`) is guarded by `.is_some()` and fires only on a MALFORMED main.

## THE MECHANISM — `apply_function`, the same primitive the substrate already uses

`src/runtime.rs:25785` — `pub fn apply_function(func, args, symbols, span)`. `freeze.rs:1156` shows
the exact call shape against a symbol-table lookup. The Grep arm is two of those:

```rust
let rules  = apply_function(sym(":user::grep")?,      vec![],      …)?;
let _      = apply_function(sym(":wat::grep::run")?,  vec![rules], …)?;
```

⚠ `invoke_user_main` (`freeze.rs:1381`) hardcodes `:user::main` and is NOT reusable here. Whether
Grep needs main's orchestration (the stop-protocol ask-and-await that `invoke_user_main_orchestrated`
performs on the way out) is the one open question this stone must answer with a measurement, not a
reading — see acceptance row 6.

## THE DRIVER — `:wat::grep::run`

```clojure
(:wat::core::defn :wat::grep::run
  [rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])] -> :wat::core::nil …)
```

1. Read one EDN vector of paths from stdin (`readln`, the codemods' shape).
2. `with-network` over `rules` + the single query `:wat::grep::q-match` — **wat-grep compiles, so
   wat-grep holds the lease, in one scope.** The user never touches it.
3. Per file: `facts-of` the source → insert → fire → `q-match` → print each `Match` → **reset**.
4. The reset is the whole reason `with-overlay` exists: per-file isolation is STRUCTURE, not
   discipline, because the body never holds the base Session.

## THE CONTRACT wat-grep KEEPS, and what it refuses to do

wat-grep prints `Match` facts. It does not rank them, filter them, count them, or interpret their
captures. A rule that asserted nothing produces no output for that file — which is the honest
answer, not an error. **Everything wat-grep does not interpret is something it cannot get wrong.**

## ACCEPTANCE

1. **End to end.** `echo '["<file>"]' | wat --grep <prog>` prints the Matches that program's rules
   assert. Output verbatim.
2. **A program with no `:user::grep` is refused with a located diagnostic** naming `:user::grep` —
   NOT a diagnostic about `:user::main`. The mirror wall, and the negative control proving the Run
   path's wall was actually bypassed rather than accidentally satisfied.
3. **A program with `:user::grep` and NO `:user::main` runs clean.** The direct test that
   `mod.rs:443` is not on this path.
4. ★ **FACTS DO NOT LEAK BETWEEN FILES.** Two files through one run, where a rule fires on file A's
   content and file B does not contain it: B's output must be EMPTY. Then the reverse ordering, so
   the result is not an artifact of which file went first. **This is the load-bearing row** — it is
   what `with-network`/`with-overlay` were built for, and a passing run without it proves nothing.
5. **The lease is balanced.** After the run, the interned network is released — no leak. The
   scoped-work stone's own tests (`src/rete/kernel/tests.rs`, `scoped_work_*`) are the shape.
6. **The stop protocol is answered by MEASUREMENT.** Either Grep needs `invoke_user_main`'s
   orchestration or it does not; run a driver that leaves a service live and observe. Whichever way
   it lands, the answer is recorded in the stone — a guess here is a resource leak or a hang.
7. Floor green, clippy 0.

## OUT OF SCOPE — affirmatively cut

- **Walking deep.** `facts-of` already walks the whole tree; `wat-scripts/lib/wat-grep.wat`'s
  top-level-only limitation is that file's, and that file is untouched here.
- **Retiring `wat-scripts/lib/wat-grep.wat`.** Its two verbs are `fix`'s business (its own comment
  at `strip-useless-mains.wat:22` already calls them `:wat::fix::wat-grep*`). Its own stone.
- **Output formatting options** — `--grep` prints EDN `Match` records, one per line. Any flag that
  shapes the output is interpretation, and interpretation is what the contract forbids.
- **Cross-file joins.** The reset is per-file BY DESIGN (row 4). A corpus-wide join is a different
  tool with a different lifetime, and `with-network` (not `with-overlay`) is what it would use.

## ⚠ intueri — OWED on `run`

`:wat::grep::run` is unruled. `Facts` and `facts-of` are also still owed from the last stone. One
cast covers all three; the builder's *"we'll refine as we go"* covers shipping provisionally, and a
rename inside stdlib is a codemod.
