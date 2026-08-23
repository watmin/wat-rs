# BRIEF — the comma dies in the READER. One clause. Read the cascade.

Builder, after months: *"i say we make the change to wat-edn?… that's the kill strike?.. that reveals
the cascade?"* Yes. This is the strike.

**Your deliverable is a GREEN FLOOR with the wall in.** A huge fail-count is the progress meter, not a
crisis — ③ shipped 543 files off exactly this pattern hours ago. Do not stash, do not revert, do not
step back to re-plan.

## The kill

`crates/wat-edn/src/lexer.rs:431`:

```rust
let body_continue = is_symbol_continue(b) || (in_brackets && b == b',');
                                          // ^^^^^^^^^^^^^^^^^^^^^^^^^^ THE PERMISSION
```

**Delete the clause.** A comma can then never enter a keyword body, at any depth, in any construct.

Then, in the same motion, the machinery that existed only to smuggle it:

```
crates/wat-edn/src/vocab.rs:168        write_keyword_body_to's  ',' → '_'   (wire encode)
crates/wat-edn/src/lexer.rs:438-445    the wire_decode  '_' → ','           (wire decode)
crates/wat-edn/src/parser.rs           reject_underscore_in_brackets        (the RESERVATION)
```

That last one is the prize: `_` inside `<…>` has been **reserved** language-wide as an escape. When
the comma cannot occur, the reservation is unearned and `_` goes back to being an ordinary character.

⚠ Order matters: **encode and decode die together.** Leaving the decoder while removing the encoder
means a `_` on the wire silently becomes an illegal `,`.

## What will scream — measured, so you can tell progress from surprise

```
:(a,b,c)      tuple keyword form   200 sites     → the Tuple `:-` form
:fn(T,U)->R   fn keyword form        5 sites     → [T U :-> R]
Type/method<A,B>  callable turbofish  1 site     → dies; see the sibling brief
```

**Both destinations are ALREADY LIVE in the stdlib** — copy what is there, invent nothing:

```clojure
wat/cache.wat   [filter <- [:wat::core::f64 :-> :wat::core::bool] …]
wat/spawn.wat   [g      <- [:wat::spawn::ThreadLaunch :-> :wat::core::nil] …]
```

Probe-verified destinations, including the nullary case, live at
`wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat`. This is the seam's item **C**,
scoped out of ② when the `Tuple` renderer was mode-blind; ②-i-b closed that.

## ⛔ 205 SITES IS A CODEMOD, NOT A SWEEP — R21

Do **NOT** hand-edit 205 `.wat` files. Write `wat-scripts/fixes/<name>.wat`, copy the shape of
`wat-scripts/fixes/angle-brackets-to-binder.wat` (written hours ago for exactly this class), **dry-run
on a `/tmp` copy and `diff` it**, confirm a second pass is byte-identical, then apply. Commit it as
the recorded migration — it is part of the deliverable.

⚠ That codemod's header records the trap you may hit: a converter that renders through a walled door
cannot run once the wall is up. Read it before you write yours.

Singletons the codemod does not reach — Rust string literals, goldens, the turbofish — go by hand.

## STOP triggers

- **STOP-1 — a construct that CANNOT be expressed without a comma.** Not awkward — cannot. Report it
  with the exact forms you tried. That is a real substrate gap and it outranks this stone.
- **STOP-2 — the wall changes what a comma-free keyword accepts.** Additive refusal only.
- **STOP-3 — a wire payload you cannot migrate.** If some persisted or in-flight encoding carries
  `_`-escaped names that must still be read, STOP and report it. Source you can rewrite; a wire
  format with live readers you cannot.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★ | the comma is refused in a keyword body | `:(a,b,c)` and `:fn(T,U)->R` both rejected, with a message naming the legal form |
| 2★★ | a comma between VALUES still works | `(:wat::core::Vector :- [:i64] 1, 2, 3)` → `[1 2 3]`, EXIT 0 — commas stay EDN whitespace |
| 3★ | the reservation is gone | `_` inside `<…>` is an ordinary char; `reject_underscore_in_brackets` deleted |
| 4★★ | the floor | `scripts/floor.sh` **green** |
| 5 | the codemod is recorded | `wat-scripts/fixes/*.wat`, idempotent, dry-run diffed |
| 6 | clippy | 0 under `-D warnings` |

**Row 2 decides it.** Row 1 goes green for a lexer that rejects commas everywhere — which would break
the whole language. Only `1, 2, 3` still reading as `[1 2 3]` proves you killed the comma **as a
separator inside a name** and left it as **whitespace between values**, which is the entire point.

## Boundaries

- `crates/wat-edn/`, the codemod, and whatever the cascade points at.
- `scripts/floor.sh` IS allowed — it is the progress meter and row 4.
- Do NOT commit, push, stash or amend. Keep the index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.

## Your report

Rows 1 and 2 verbatim, **together** — either alone is meaningless. The waterfall, fail-count per
round. The codemod's dry-run stats and its idempotency check. Whether `_`'s reservation is fully
gone. Any STOP. What surprised you.
