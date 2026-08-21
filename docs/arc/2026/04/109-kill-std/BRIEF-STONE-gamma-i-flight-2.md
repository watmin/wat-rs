# BRIEF — arc 109 γ-i, flight 2: close rows 3 and 6

Flight 1 delivered the binder. Two rows remain, both MEASURED, both with a working control. Flight
1's five modified files are already in the working tree and are your base — **do not revert them.**
Read `SCORE-STONE-gamma-i-flight-1.md` first; it records what is already green so you do not re-derive it.

## Row 3 — a declaration carrying BOTH spellings must ERROR

```
(:wat::core::defn    :user::f<T> :- [T] [x <- :T] -> :T x)   ⛔ silently CHECKS   ← the bug
(:wat::core::defrecord :user::R<T> :- [T] [v <- :T])         ✅ rejects           ← the control
```

The TYPE side already emits the exact message to mirror, from `take_declared_binder` (`src/types.rs`):

> *declaration carries BOTH a name-embedded `<...>` type-param spelling (…) and a `:- [...]` binder —
> pick one; a declaration with both is a contradiction, never something to silently resolve*

**The obstacle, and it is the whole row:** `try_parse_fn_shape_def` (`src/runtime.rs:3395`) returns
`Option`, so it has no channel to carry a located error — a malformed shape is `None`, which every
caller reads as "not this shape, let another parser try." Flight 1's stopgap unions the two lists
silently and says so in a comment.

Find where the contradiction CAN be surfaced with a span. Prefer the site that already reports
`def`/`defn` shape errors with a location. **Read the callers of `try_parse_fn_shape_def` before
changing its signature** — `None` is load-bearing for the recognizer chain, and turning it into a
hard error could reclassify forms that are legitimately "not this shape."

## Row 6 — the kwargs branch breaks under the binder spelling

```
(:wat::core::defn :user::mk  :- [T] [a <- :T & [b <- :T]] -> :T a)
  ⛔ "malformed :wat::core::fn form: triple is incomplete; expected `name <- :T` but ran out of items"
(:wat::core::defn :user::mk2<T>     [a <- :T & [b <- :T]] -> :T a)
  ✅ checks          ← byte-identical argspec; the difference is the binder alone
```

`wat/core.wat:673`'s `defn` macro keys its kwargs branch on `name-parametric?` and `name-tp` — the
`<…>` string suffix taken off the NAME — and re-attaches it as `{b}::Kwargs{p}` and `:{b}$impl{p}`.
The binder spelling leaves `name-tp` empty, so the branch mis-builds.

Teach `name-parametric?` / `name-tp` / `name-base` the binder spelling: when the form carries
`:- [T U]` after the name, those three must read as if the name had been written `<T,U>`. The macro
already peels nothing — the binder currently rides `rest` straight through — so you will need to
recognise and consume it there, then still forward it into the emitted `fn` (which is what makes
flight 1's Rust-side peel fire).

**Verify the expansion, not your reading of the macro:** dump it with
`(:wat::core::macroexpand '(…))` and look at what is actually emitted. `wat-rs/CLAUDE.md` R4 — and
this stone has already cost two wrong designs written from reasoning instead of expansions.

## Read in order

| where | why |
|---|---|
| `docs/arc/2026/04/109-kill-std/SCORE-STONE-gamma-i-flight-1.md` | what is already green, and the three brief defects flight 1 exposed |
| `git diff` (working tree) | your base — five files from flight 1 |
| `src/types.rs` `take_declared_binder` | the contradiction message and its shape |
| `src/runtime.rs:3395` `try_parse_fn_shape_def` + its callers | row 3's obstacle |
| `wat/core.wat:673` the `defn` macro, the `name-base`/`name-tp`/kwargs branch | row 6 |

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- `src/check.rs` stays at **zero changes**. It is empty today and that is G3's premise.
- Do NOT touch `parse_fn_signature_prefix`'s `&[WatAST; 3]`.
- Do NOT attempt to fix the anonymous-`fn` silent-accept (any stray token in the first slot disabling
  checks) or let-polymorphism. Both are measured, filed, and out of this stone.
- A cascade after a shape change is the substrate naming its next site. Read it; never revert in a panic.

## Your own checks

`target/debug/wat --check <file>` on files under `wat-scripts/scratch-pad/` after
`cargo build --bin wat`. Rows 3 and 6 above are the two you must move, and re-run rows 1, 2a-2e, 4,
5, 7, 8 from the SCORE to confirm you moved nothing else. Prefix long commands with
`systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.

⚠ `wat/core.wat` is the stdlib, baked in by `include_str!` at RUST-compile time — a **rebuild** makes
your macro edit visible, so `cargo build --bin wat` then `--check` DOES exercise it. That is the one
place this stone differs from the usual rider rule; use it, and say which rows you ran after which build.

## STOP triggers — ship nothing and report

- **STOP-1.** If row 3 cannot be surfaced without editing `src/check.rs`, STOP and report where you
  looked. G3's blast radius excludes it; if the contradiction genuinely lives there, the builder
  re-decides rather than you widening scope.
- **STOP-2.** If turning `try_parse_fn_shape_def`'s `None` into an error reclassifies forms that are
  legitimately "not this shape", STOP and report which forms. Silently changing the recognizer chain
  is worse than leaving row 3 open.
- **STOP-3.** If row 6 requires changing what `defn` EMITS beyond the kwargs branch — the plain path
  is verified green and must stay byte-identical — STOP and report.

## Your report

The diff per file. Rows 3 and 6 with verbatim output, before and after. The full expansion you dumped
for row 6. Confirmation that rows 1, 2a-2e, 4, 5, 7, 8 still pass and that `git diff --stat
src/check.rs` is empty. What surprised you. Anything you inspected and deliberately left alone, with
the reason.
