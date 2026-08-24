# DESIGN — STONE: `:wat::grep::Match` (the fact a user's rule asserts)

> The Span fact (`5d650b807`) made the coordinate BINDABLE. This makes it REPORTABLE.
> Together they are the whole of wat-grep's data contract.

> ## ⛔ REDRAWN 2026-08-24 — THE HOME MOVED, AND THE FIRST RIDER WAS RIGHT TO STOP
>
> **v1 of this stone was self-contradictory and a rider proved it in 17 minutes.** It ruled the
> vocabulary `:wat::grep::*` and in the same document said *"does NOT put anything in `wat/`"*.
> `:wat::` is reserved to `Privilege::Stdlib`; a `wat-scripts/` file is `Privilege::User`, and
> `src/resolve/registration.rs:129` refuses it unconditionally:
>
> ```
> #wat.macro/ReservedPrefix {:message "cannot declare macro :wat::grep::Capture — reserved
> prefix (:wat::, :rust::, :$bound::); user macros must use their own prefix" …}
> ```
>
> The answer was in a file v1 *sent the rider to read*: `wat-scripts/lib/wat-grep.wat:24` —
> *"Namespace: `:user::` (the only writable prefix for user scripts outside `wat/` stdlib)."*
> Re-reading the design would never have shown it; the defect lived in the RELATION between the
> namespace and the location. `[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`
>
> **v1 also manufactured a load cycle.** It required the proving rule to *"bind four from
> `:fx::Span`"*, so the new file needed corpus-03's type while corpus-03 needed the new file's
> door — `CycleDetected` in both directions. The rider engineered around it with a mirrored
> `:fx::Span` plus `set-redef!`. It should never have had to; see THE PROVING RULE below.
>
> **The builder's ruling, 2026-08-24, resolves both by moving the thing:** *"grep moves out of
> wat-scripts, that's where we host our repo's scripts, wat-grep is maturing into a wat feature."*
> And on the CLI surface: *"it must be `--grep` to match with `--repl` and `--mcp`."*

## The contract, restated

Builder, 2026-08-24: **the user's rules assert `Match` facts; wat-grep queries for them and prints.**
wat-grep owns exactly ONE query and performs NO interpretation. The user supplies RULES, not queries.
Everything wat-grep does not interpret is something it cannot get wrong.

Two consequences that shape everything below:
1. wat-grep never sees the match — only the fact the rule built. So every field must be **constructible
   inside a rete `:then`**, which is a much narrower surface than ordinary wat.
2. wat-grep never reads a field's meaning. So the fields must be legible to an **arbitrary** consumer,
   not just to `wat/fix.wat`.

## The record

```clojure
(:wat::core::defrecord :wat::grep::Capture
  [name  <- :wat::core::String      ; what the rule called it — "id", "kind", "head"
   value <- :wat::core::String])    ; what it reported, rendered

(:wat::core::defrecord :wat::grep::Match
  [file      <- :wat::core::String  ; supplied by the RUN, not by a node
   line      <- :wat::core::i64
   col       <- :wat::core::i64
   end-line  <- :wat::core::i64     ; REQUIRED — see "the end is not optional"
   end-col   <- :wat::core::i64
   rule      <- :wat::core::String  ; which rule concluded this
   captures  <- (:wat::core::PersistentVector :- [:wat::grep::Capture])])
```

Flat coordinates, not a nested span. Same reason `:fx::Span` is flat: **a rule binds FIELDS, not
sub-records**, and a downstream rule that wants to reason about a Match's line must be able to write
`(:wat::grep::Match (?l <- :line))` without destructuring first.

## The end is NOT optional — and that is a REJECTION of the substrate's own Span

Builder, 2026-08-24: *"i don't care if wat-grep's Match imposes a hard requirement on end coords,
that's better imo.. but the data provider /is an option/ so we need to process it correctly."*

`:wat::core::Span` carries `end <- (:wat::core::Option :- [:wat::core::Pos])`, and the substrate says
why in its own hand (`crates/wat-reader/src/span.rs:69`): *"`end` is `Some(Pos)` when the lexer or
parser computed a real range; `None` for point-spans from Rust call sites (`rust_caller_span!()`)
where no end is available."* Two constructors split on exactly that line — `Span::new` for Rust,
`Span::with_end` for the lexer and parser.

**That Option exists for RUST's benefit.** `ast-span` and `ast-end-span` are TOTAL — measured on the
Span stone across every leaf kind AND reader-synthesized nodes (`sigils-inline Node=23 Span=23`).
wat always knows its end. A `Match` that reused `:wat::core::Span` would inherit a `None` case wat
cannot produce, and every consumer forever would unwrap a variant that is never there.

★ **And the shape pays a second dividend that is worth naming, because it is the difference between
avoiding a problem and not having one.** With no Option field on `Match`, **a user's rule never
constructs `Some`** — so the bare-vs-declared variant spelling that cost this session an hour
(`:wat::core::Some` refused in a `:then`; `:wat::core::Option::Some` admitted) never touches wat-grep
at all. Not worked around. Structurally absent.

## `captures` is a VECTOR, not a map — and this is MEASURED, not preferred

`294/SEAM.md` sketched `bindings <- PersistentMap`. **A rete `:then` cannot build a map.** Measured
this session, at `compile-all` time:

```
(:wat::core::PersistentMap ?k ?v)         compile-condition: then expr is not total
                                          — ':wat::core::PersistentMap' is not total
(:wat::rete::core::PersistentMap ?k ?v)   passes the fence, then dies at runtime:
                                          "compiled apply cannot dispatch kind Unknown arity 2"
```

The rete-vocabulary spelling has a `RETE_OPS` row and NO compiled-exec implementation — it clears the
axis fence and has nothing to run. (That gap is rete's, i.e. grok-rete's; it is NOT this stone's to
close, and this stone does not need it.)

What a `:then` CAN construct, all measured:

```
bound ?var                              ✓   {:x "?id"}
string / int / float / bool literal     ✓   {:x "lit"}
:wat::rete::core::String/concat ?k ?v   ✓   {:x "?id42"}
:wat::rete::core::i64::+ ?n 1           ✓   {:x 8}
:wat::rete::core::PersistentVector …    ✓   {:x #wat.core/PersistentVector ["?id" "42"]}
a declared record constructor           ✓   (Law A — declaration-derived heads are admitted)
a declared enum variant constructor     ✓   (:g::End::Known ?l ?c) -> #g.End/Known [7 26]
:wat::core::string::concat              ✗   is not total
a bare keyword variant (:wat::core::None) ✗ RhsUnresolvableOperand
```

So the vector-of-records IS buildable, whole, inside one RHS, with LHS bindings flowing into the
nested constructors — measured verbatim:

```
#m/Out {:x #wat.core/PersistentVector [#m/Binding {:name "id"   :value "?id"}
                                        #m/Binding {:name "kind" :value "symbol"}]}
```

⚠ Note the constructor is **`:wat::rete::core::PersistentVector`**, not core's. A rule author writing
core's spelling gets *"is not total"*, which describes the axis, not the fix.

### Why a record and not two parallel vectors

A `(PersistentVector :- [String])` of alternating name/value would build just as well and is strictly
worse: it is a map with the pairing left to a convention nobody can check, and an odd-length vector
would be a silent defect that renders fine. The record makes the pair the unit.

## The one door where an Option is genuinely processed

There are exactly TWO Options in wat-grep's path. The design above deletes the second. The first is
real and must be handled:

```
ast-span node            → (HashMap :- [keyword i64])   {:line N :col C}   no Option
ast-end-span node        → same                                            no Option
HashMap/get span :line   → Option<i64>                        ← ① THE ONE
:wat::core::Span/end     → Option<Pos>                        ← ② deleted by the shape above
```

Today the extractor unwraps ① **four times per node** with a separate `Option/expect` and its own
message string. That is the CONVENTION rung: four sites, four chances to write a different message or
forget one, on a value fetched from a HashMap the caller did not build.

**This stone collapses it to one door**: a single verb that takes a `:wat::WatAST` and yields the four
coordinates, consuming the `HashMap/get` Options exactly once, with one located failure. No other site
sees an Option, and no rule can — because the fact it binds is already four i64s.

```clojure
(:wat::core::defn :wat::grep::extent-of [node <- :wat::WatAST] -> :wat::grep::Extent …)
```

The extractor's Span emit calls it; nothing else unwraps a span.

## ★ THE NAMES ARE RULED — intueri, 2026-08-24

Cast against `wat-scripts/intueri/grep-match-vocabulary.wat.intueri`. `Match` is the builder's and
was not in scope. Every rejection below is a collision **verified in this repo**, not an aesthetic:

| slot | RULED | rejected, and why |
|---|---|---|
| the pair record | **`Capture`** | `Binding` — **L1, lies.** `Bindings` is a live trait in the engine itself (`src/rete/matcher.rs:102`, imported `src/rete/where_tree.rs:32`) meaning *the LHS variable environment*. This field is a curated RHS report that may be a subset of the `?vars`, a renaming, or values that were never `?vars`. The reader most likely to assume is the one who has read the engine. · `Finding` — **L2**, and a real cross-file collision: `:wat::lint::Finding` already exists (`wat/lint.wat:50`) as the sibling tool's whole per-occurrence result. · `Detail`/`Note` — L2, generic. · `Datum` — runner-up, not rejected. |
| the four coordinates | **`Extent`** | `Coords` — **L1, lies**, and the cast's strongest catch: `<fqdn>::Coords` is already minted by the `defservice` machinery (`wat/core.wat:1100`) as a capability-crossing carrier. An outright homonym with a security-relevant family. · `Range` — L2, `:wat::core::range` is a live sequence generator. · `Region` — L1, implies rectangularity; a mid-line-3→mid-line-7 span is not a rectangle. · `Locus` — L1, this codebase's only "locus" is `LociDiedError`, *one point of death*. |
| the one door | **`extent-of`** | `coords-of`/`span-of` — L1, inherit their nouns' collisions. · `locate` — L2, implies search; the door is a direct unwrap on a node the caller holds. · `read-coords` — L2, "a read" reads as one of several, and the whole constraint is that it is THE door. |

★ **`Extent` is not a coinage.** `wat/lint.wat:389` already says, verbatim:
*";; extent = ast-span..ast-end-span of the whole concat form"* — the door's exact computation,
under the exact word, in the sibling tool. The word was already here.

### The structural question, answered

`Extent` and `:fx::Span` are **not** one type wearing two names. `Span` is `id` + `Extent`'s four
fields, spread rather than nested, and the spreading is forced: a rule binds FIELDS, not sub-records,
so a fact that nested its coordinates could not be joined on a line. Composition, not duplication.

⚠ **The one risk it leaves, and it is real:** nothing pins the two field lists together. A later
rename (`col` → `column`) must be made by hand in both places with no compiler link between them.
`:fx::Span`'s declaration gains a one-line cross-reference to `Extent` naming that dependency —
the weakest rung, and on this channel the only rung there is.

## THE HOME — `wat/grep.wat`, stdlib, and `:wat::grep::` is now legitimate

wat-grep is a **wat feature**, not a repo script. `wat-scripts/` hosts this repository's own
scripts; a vocabulary the language ships cannot live there, and the reserved-prefix wall is the
substrate saying exactly that. The declarations and the door go to `wat/grep.wat` and register
under `Privilege::Stdlib` like every other stdlib type.

Three consequences, all of them simplifications:

1. **The load cycle dissolves.** corpus-03 consults stdlib; it `load-file!`s nothing. No mirrored
   `:fx::Span`, no `set-redef!`, no cycle to route around.
2. **`--grep` becomes possible.** A CLI mode cannot dispatch into a vocabulary that only exists
   inside one user script. See THE CLI, below.
3. ⚠ **A stdlib `.wat` edit is INVISIBLE until the crate is rebuilt** (`include_str!` at
   Rust-compile time), and `wat/grep.wat` must take a position in `STDLIB_FILES`
   (`src/stdlib.rs:34`) that the arc-275 load-order gate accepts. It references only `core`, so it
   sits early; the gate is the authority, not this sentence.

## THE CLI — `--grep`, and why the harness owns the loop

Builder: `echo '["vec" "of" "files"]' | wat --grep grep-program.wat`.

**`--grep`, not `grep`** — ruled, and the reason is consistency: the mode grammar is already
`--repl` / `--mcp` (`src/distribution/argv.rs:93-95`), flags recognised only before the entry path.
A bare subcommand word would be this CLI's first, and would need a rule distinguishing "the file
named grep" from "the grep mode". `--grep` needs no such rule.

**What the harness owns, and why it matters:** the stdin file list, the loop, the network's
lifetime, the reset between files, and the one query. The user supplies **rules only**. This is the
builder's contract applied one level up — *everything wat-grep does not interpret is something it
cannot get wrong*, and everything the user does not write is something they cannot get wrong. Per-file
isolation stops being discipline and becomes structure, which is precisely what `with-network`
shipped for (`fd7b017f6`).

It also retires a real duplication rather than inventing a convention: the stdin-EDN-vector harness
is hand-copied into every recorded migration — `wat-scripts/fixes/angle-brackets-to-binder.wat:282`
labels its own copy *"identical shape to every recorded migration."*

**The CLI is the NEXT stone, not this one.** This stone ships the vocabulary it dispatches into.

## The rooms

1. **`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat:89-99`** — the four `Option/expect`
   calls the one door replaces.
2. **`wat-scripts/lib/wat-grep.wat`** — 93 lines, TOP-LEVEL ONLY, and it already computes both spans
   and throws them away (`wat-grep-form-edit:37-38`). This is where `Match` gets queried and printed.
3. **`wat/fix.wat:179-193`** — `fix-text-offset-of`, the canonical `(Option/expect (HashMap/get …))`
   chain the one door absorbs; and `fix-text-offset-of` is also why `Match` carries NO `:offset`
   (`fix.wat` derives it from `{:line :col}` + the file's lines; carrying both gives one position two
   sources of truth that drift the moment anything re-reads the file).

## Acceptance

1. **A rule builds a complete `Match` in one RHS** — all five coordinates LHS-bound, `file` supplied
   as a literal, `captures` a non-empty vector of `Capture` records. Output verbatim.

   ⚠ **The proving rule binds its OWN demo fact, NOT `:fx::Span`.** v1 required `:fx::Span` and that
   is what forced the load cycle: the proving file would need corpus-03's type while corpus-03 needs
   this file's door. The rule needs *a* fact carrying four coordinates — it does not need *that* one.
   Declaring a two-line demo fact beside the rule proves exactly the same thing and depends on
   nothing. (`:fx::Span` still consumes `extent-of` — that is row 4 — but the dependency runs one
   way only, and now through stdlib rather than `load-file!`.)
2. **No `Option` appears anywhere in the Match's rendered EDN.** The negative control for the whole
   "end is not optional" ruling — grep the output for `Option/` and find nothing.
3. **`extent-of` is the ONLY site that unwraps an `ast-span` HashMap.** A census of
   `Option/expect` + `HashMap/get` in the touched files returns exactly one pair.
4. **The one door is total on the same population the Span stone measured** — re-run corpus-03 and get
   `Span == Node` unchanged (4316 / 435 / 33). A refactor that changes a count changed behaviour.
5. **The load-order gate accepts `wat/grep.wat`'s position** — `(:wat::deporder::verify-stdlib)`
   returns `[]`. It is the authority on placement; a position that reads tidy and violates the gate
   is how the scoped-work stone went red (`fd7b017f6`'s predecessor).
6. Files load (`every_wat_scripts_file_loads`), floor green, clippy 0.

⚠ **The floor cannot see a stdlib `.wat` edit until the crate rebuilds** (`include_str!` runs at
Rust-compile time). A `--check` against a stale `target/release/wat` will report the OLD file.

## Out of scope — affirmatively cut

- **The rete PersistentMap compiled-exec gap.** Measured and recorded above; it belongs to the rete
  subsystem, which is grok-rete's authority. This stone routes around it by design, not by workaround
  — the vector is the better shape regardless.
- **`--grep` itself** — the mode, the stdin list, the `with-network` loop, the one query, the print.
  It is the NEXT stone and it dispatches into this one's vocabulary. Ruled `--grep`, not `grep`, to
  match `--repl`/`--mcp`.
- **Retiring `wat-scripts/lib/wat-grep.wat`.** Measured: exactly ONE consumer,
  `wat-scripts/fixes/strip-useless-mains.wat`, calling two verbs. And its own comment (`:22`) already
  names them **`:wat::fix::wat-grep`** / **`:wat::fix::wat-grep-strip`** — a home that does not
  exist. That comment is right about the domain: both verbs build span-DELETE edits, which is
  `fix`'s business, not grep's. So the old lib does not migrate into `wat/grep.wat`; its capability
  belongs to `fix`, and the `grep` name frees up for the real thing. Tracked as its own stone, not
  folded here — it has a different destination and a different consumer.
- **Walking deep.** wat-grep is TOP-LEVEL ONLY today; `fix.wat`'s `fix-source` has the recursion. That
  is a separate change with its own blast radius.
- **`:wat::core::Span` / the Option/Result symbol migration.** Belongs to `296/DESIGN-STONE-H`, which
  is DRAWN and carries it explicitly. Nothing here waits on it.

## ⊘ THE intueri CAST IS DISCHARGED

Ruled above, 2026-08-24. Target: `wat-scripts/intueri/grep-match-vocabulary.wat.intueri`.
Every citation re-read against the disk by the orchestrator before the ruling was folded in.
