# DESIGN — STONE: `wat/grep.wat` — wat-grep becomes a wat feature

> Supersedes the scratch-pad scoping in `DESIGN-STONE-the-grep-match.md`. That stone's RECORD
> (the shapes, the intueri ruling, the `:end` provenance argument, what a `:then` can construct)
> stands and is not repeated here; its HOME and SCOPE are replaced by this file.

## The ruling that moved it

Builder, 2026-08-24: *"grep moves out of wat-scripts, that's where we host our repo's scripts,
wat-grep is maturing into a wat feature."* And on the CLI: *"it must be `--grep` to match with
`--repl` and `--mcp`."*

A vocabulary the language ships cannot live in `wat-scripts/`. The reserved-prefix wall
(`src/resolve/registration.rs:129`) was the substrate saying exactly that, and a rider proved it.

## THE CONTRACT — settled 2026-08-24, with the builder, on the record

```
wat-grep DECLARES   Node · Named · Span      the fact base it inserts, per file
                    Match · Capture          what a rule asserts
                    q-match                  the one query — never written by a user

the USER DECLARES   rules over those, asserting Match

RENDEZVOUS          :user::grep [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])

wat-grep OWNS       compile · the lease · the loop · the reset · the query · the print
```

Three things decided this, and each is mechanical rather than aesthetic:

1. **The rendezvous returns RULES, not a compiled network.** `compile-all` ARMS the intern lease
   (`wat/rete/compile.wat:1149` — the leak the `with-network` prototype paid for). A user program
   that returned a `Session` would have taken the lease, leaving wat-grep to release what it did
   not acquire: ownership split across a boundary by two authors, which is precisely the bug
   `with-network` was built to make unrepresentable. Rules in, `with-network` compiles, one scope.
2. **The user supplies NO queries.** Builder's own contract: *"wat-grep owns ONE query and performs
   NO interpretation."* One query is what makes the printer TOTAL — it renders exactly one type it
   fully knows. User-supplied queries would force wat-grep to print result shapes it has never
   seen, which is the interpretation the contract exists to forbid.
3. **wat-grep produces the facts, so wat-grep owns their types.** This is the consequence that grew
   the stone: `Node`/`Named`/`Span` cannot stay `:fx::*` in a probe if the harness is the one
   inserting them. corpus-03 stops declaring them and becomes what it always claimed to be — a
   probe that consumes the real thing.

### Measured, so nothing here rests on a reading

- **A program with no `:user::main` checks clean** — `(:wat::core::defn :user::grep [] -> :i64 42)`
  alone gives `--check EXIT=0`. The `:user::main` wall (`src/freeze.rs:941`) is guarded by
  `.is_some()`: it fires on a MALFORMED main, not a missing one. So a grep program is a legal wat
  program that simply has no entry point.
- **The invocation pattern already exists.** Rust looks `:user::main` up in the frozen symbol table
  (`src/freeze.rs:1617`) and calls it. `:user::grep` is that same move against a different name.
- ⚠ **A stdlib file CANNOT name `:user::grep`** — it does not exist at stdlib-freeze time. So the
  lookup is Rust's and the rules arrive as an ARGUMENT to the wat driver. `wat/grep.wat` never
  mentions the user's namespace. This is why the split below falls where it does.

## THE SPLIT — Rust stays tiny, the logic is wat

```
Rust (--grep mode)   parse the mode · look up :user::grep · call it · hand the rules to the driver
wat/grep.wat         with-network(rules) → per file: facts-of → insert → fire → q-match → print → reset
```

`282-wat-fix-over-rust` is an arc in this repo for a reason. The loop, the lifetime, the reset and
the printing are wat. Rust contributes a `Mode` variant, a symbol lookup, and one call.

**This stone ships the wat half.** `--grep` — the mode, the stdin list, the driver loop — is the
NEXT stone and dispatches into this one's vocabulary.

## THE FILE — `wat/grep.wat`

```clojure
;; ── the fact base wat-grep inserts, one set per file ────────────────────────────────
(:wat::core::defrecord :wat::grep::Node
  [id <- :i64  parent <- :i64  index <- :i64  kind <- :String])

(:wat::core::defrecord :wat::grep::Named          ; ONLY for a nameable kind — the absence IS the guard
  [id <- :i64  name <- :String])

(:wat::core::defrecord :wat::grep::Span           ; EVERY node — Span == Node is the non-vacuity control
  [id <- :i64  line <- :i64  col <- :i64  end-line <- :i64  end-col <- :i64])

;; ── what a rule asserts ─────────────────────────────────────────────────────────────
(:wat::core::defrecord :wat::grep::Capture  [name <- :String  value <- :String])

(:wat::core::defrecord :wat::grep::Match
  [file <- :String  line <- :i64  col <- :i64  end-line <- :i64  end-col <- :i64
   rule <- :String  captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])])

;; ── the in-process coordinate, and THE ONE DOOR ─────────────────────────────────────
(:wat::core::defrecord :wat::grep::Extent
  [line <- :i64  col <- :i64  end-line <- :i64  end-col <- :i64])

(:wat::core::defn :wat::grep::extent-of [node <- :wat::WatAST] -> :wat::grep::Extent …)

;; ── source → facts ──────────────────────────────────────────────────────────────────
(:wat::core::defrecord :wat::grep::Facts
  [nodes <- (:PersistentVector :- [:wat::grep::Node])
   named <- (:PersistentVector :- [:wat::grep::Named])
   spans <- (:PersistentVector :- [:wat::grep::Span])])

(:wat::core::defn :wat::grep::facts-of [src <- :String] -> :wat::grep::Facts …)

;; ── the ONE query ───────────────────────────────────────────────────────────────────
(:wat::rete::defquery :wat::grep::q-match :params [] :when [(?fact <- :wat::grep::Match)])
```

(Types abbreviated above for reading; the file spells every FQDN in full.)

`Match` is FLAT — no nested `Extent` — because a rule binds FIELDS, not sub-records. `Extent` is
the door's return value, never a field of anything. `Span` is `id` + `Extent`'s four fields, spread
for the same reason; intueri ruled that composition, not duplication, and named the residual risk:
nothing pins the two field lists together, so `Span`'s declaration carries a cross-reference.

## WHAT MOVES, AND FROM WHERE

`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat` is the proven source. Its walk is
measured and shipped (`5d650b807`): `wat/fix.wat Node=4316 Span=4316`, `neg-consumer 435`,
`probe_do_splice 33`, with `Named` strictly below `Node` in each. **This is a MOVE of working code,
not a rewrite** — the migration is `:fx::` → `:wat::grep::`, `:fx::Acc` becomes `Facts`, and the
four inline `Option/expect` calls collapse into `extent-of`.

corpus-03 keeps its rules and its report, drops its declarations, and consumes the stdlib verbs.
It stays a probe; that is honest, and it becomes the stone's regression check.

## ACCEPTANCE

1. **`(:wat::deporder::verify-stdlib)` returns `[]`.** `wat/grep.wat` takes a position in
   `STDLIB_FILES` (`src/stdlib.rs:34`) that the arc-275 gate accepts. It references `core` and
   `rete` (for `defquery`), so it sits after both. **The gate is the authority, not a reading** —
   placing a file where it reads tidy and violates the gate is exactly how the scoped-work stone
   went red.
2. **corpus-03 still reports the same numbers** — `Node=4316 Span=4316` / `435` / `33`, `Named`
   below `Node` in each. A count that moved means the move changed behaviour.
3. **A rule builds a complete `Match` in one RHS** — five coordinates LHS-bound, `file` a literal,
   `captures` a non-empty vector of `Capture`. Output verbatim.
4. **No `Option` appears in the Match's rendered EDN.** The negative control for the `:end`
   provenance ruling: grep the output for `Option/` and find nothing.
5. **`extent-of` is the ONLY site that unwraps an `ast-span` HashMap.** Census `Option/expect` +
   `HashMap/get` across `wat/grep.wat` and corpus-03; exactly one pair, in the door.
6. Floor green, clippy 0, `every_wat_scripts_file_loads` green.

⚠ **A stdlib `.wat` edit is INVISIBLE until the crate rebuilds** (`include_str!` at Rust-compile
time). Measured this session: an incremental rebuild after touching `wat/core.wat` is **19s**.

## OUT OF SCOPE — affirmatively cut

- **`--grep` itself** — the `Mode` variant, the stdin list, the driver loop, the print. Next stone.
- **Retiring `wat-scripts/lib/wat-grep.wat`.** Its own comment (`strip-useless-mains.wat:22`)
  already calls its two verbs `:wat::fix::wat-grep*` — a home that does not exist. That comment is
  right about the domain: both build span-DELETE edits, which is `fix`'s business. So the old lib
  does not move into grep; its capability belongs to `fix` and the name frees up. One consumer,
  measured. Its own stone.
- **Walking deep in `wat-scripts/lib/wat-grep.wat`.** That file is untouched here.
- **Promoting corpus-03 out of scratch-pad.** It stays a probe and becomes the regression check.

## ⚠ intueri — OWED on two nouns only

`Node` / `Named` / `Span` / `Match` / `Capture` / `Extent` / `extent-of` are already ruled or the
builder's. **`Facts` and `facts-of` are new and unruled** — they follow `extent-of`'s ruled `-of`
precedent, which is why they are written this way rather than invented. Cast before the NEXT stone;
the builder's *"we'll refine as we go"* covers shipping them provisionally, and a rename inside one
stdlib file is a codemod, not a migration.
