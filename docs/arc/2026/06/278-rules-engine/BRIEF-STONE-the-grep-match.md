# BRIEF — STONE: `:wat::grep::Match`

DESIGN: `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-grep-match.md` — read it whole, first.
The names in it are RULED (intueri, 2026-08-24); use them exactly as written.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run every command in the FOREGROUND and block on it: your turn ends when the
numbers are in your hands, not when a command is launched.

**You may not spawn sub-agents.** Everything here is yours to do directly.

Anchor: `/home/john/work/holon/wat-rs`. `pwd` first. You do not commit, push, stash, revert, or
checkout — leave your work uncommitted and report. `target/release/wat` is current and is all you
need: `--check <file>` type-checks, `<file>` runs. No cargo, no floor — the orchestrator weighs those
centrally. Cap every run:

    systemd-run --user --scope -q -p MemoryMax=6G -p MemorySwapMax=0 timeout 180 ./target/release/wat <args>

## The work in one paragraph

Three declarations and one function, plus a rule that proves them. `:wat::grep::Capture` is a
name/value pair. `:wat::grep::Match` is the fact a user's rule asserts — flat coordinates with a
**required** end, a file, a rule name, and a vector of Captures. `:wat::grep::Extent` is the four
coordinates in-process, and `:wat::grep::extent-of` is the ONE DOOR that produces it — the only place
in wat-grep that ever unwraps an `ast-span` HashMap.

## The rooms — read in this order

1. **`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat:89-101`** — `:fx::walk`'s four
   inline `Option/expect` calls. This is what `extent-of` replaces: the walk calls the door once and
   spreads its four fields into `:fx::Span` alongside `:id`.
2. **`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat:42-47`** — `:fx::Span` itself. It
   gains a one-line comment cross-referencing `:wat::grep::Extent`, because nothing pins the two
   field lists together and a later rename must be made in both by hand.
3. **`wat/fix.wat:179-193`** — `fix-text-offset-of`. The canonical
   `(Option/expect (HashMap/get loc :line) "…")` chain, and the `-of` naming precedent.
4. **`wat/lint.wat:389`** — *";; extent = ast-span..ast-end-span of the whole concat form"*. Why the
   type is called `Extent`; the word was already here.
5. **`wat-scripts/scratch-pad/probe-rhs-builds-core-span.wat`** — the foot of this file records what a
   rete `:then` can and cannot construct. **Read it before writing the rule**; it will save you the
   two failures it already paid for.

## Where the declarations live

`wat-scripts/scratch-pad/grep-match-vocabulary.wat` — a new scratch-pad file holding the three
declarations, `extent-of`, and the proving rule. This stone does NOT touch `wat-scripts/lib/wat-grep.wat`
and does NOT put anything in `wat/` — the promotion out of scratch-pad is a later stone, once the
walk stops being a probe.

## The shapes — exactly as ruled

```clojure
(:wat::core::defrecord :wat::grep::Capture
  [name  <- :wat::core::String
   value <- :wat::core::String])

(:wat::core::defrecord :wat::grep::Extent
  [line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64])

(:wat::core::defrecord :wat::grep::Match
  [file     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   end-line <- :wat::core::i64
   end-col  <- :wat::core::i64
   rule     <- :wat::core::String
   captures <- (:wat::core::PersistentVector :- [:wat::grep::Capture])])
```

`Match` is FLAT — no nested `Extent` — for the same reason `:fx::Span` is: a rule binds FIELDS, and a
downstream rule that wants a Match's line must write `(:wat::grep::Match (?l <- :line))` without
destructuring first. `Extent` is the in-process return of the door, not a field of anything.

## `extent-of` — the one door

`[node <- :wat::WatAST] -> :wat::grep::Extent`. It calls `ast-span` and `ast-end-span` and consumes
all four `HashMap/get` Options in ONE place. Mirror `fix-text-offset-of`'s `Option/expect` shape; give
each message enough to locate the failure. After this exists, **no other site in the file unwraps a
span** — that is the point of the name, and acceptance row 3 measures it.

## The proving rule

In the same file, a rule that asserts a real `Match`, joining the facts corpus-03 already emits.
Copy `:fx::arrow` for the condition shape and `:fx::classify` for the compile/insert/fire/query chain.
It must bind all five coordinates from the LHS (four from `:fx::Span`, and the file supplied as a
literal in the RHS), and build a non-empty `captures` vector.

⚠ **Two things measured this session that will otherwise cost you a cycle:**
- The vector constructor is **`:wat::rete::core::PersistentVector`**, NOT `:wat::core::PersistentVector`.
  Core's fails the fence with *"is not total"*.
- A **record** constructor takes kwargs; a **tagged enum variant** constructor takes positions. Both
  can appear in one `:then` and they look identical at the call site.

## The acceptance rows YOU run

- **Row 1 — a rule builds a complete `Match` in one RHS.** All five coordinates LHS-bound, `captures`
  a non-empty vector of `Capture` records. Print the fact; report the output verbatim.
- **Row 2 — no `Option` appears in the Match's rendered EDN.** The negative control for the whole
  "the end is not optional" ruling. Grep your own output for `Option/` and report what you find.
- **Row 3 — `extent-of` is the ONLY site that unwraps an `ast-span` HashMap.** Census
  `Option/expect` + `HashMap/get` across the files you touched; report the count and every site.
- **Row 4 — the refactor changed nothing measurable.** Re-run corpus-03 and get `Span == Node`
  unchanged: `wat/fix.wat 4316`, `neg-consumer.wat 435`, `probe_do_splice 33`. A count that moved
  means behaviour moved.
- **Row 5 — `--check` exits 0** on every file you touched or created.

Report each row's command and its output **verbatim** — never a summary, never a `| head`/`| tail`
window. A row you could not run is reported as not-run, never as passed.

## Blast radius

- `wat-scripts/scratch-pad/grep-match-vocabulary.wat` — created
- `wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat` — edited (the door + the comment)

Nothing under `src/`. Nothing under `wat/`. Nothing in `wat-scripts/lib/`.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **A `Match` field cannot be constructed in a `:then`.** STOP and report the compiler's message
   verbatim plus which field. Do not flatten `captures` to a string or drop a coordinate to get past it.
2. **`extent-of` cannot be called from `:fx::walk`** — a type or arity the door's signature cannot
   satisfy. STOP and report; do not inline the unwraps back into the walk.
3. **Row 4's counts move.** STOP. The refactor changed behaviour. Report both sets of numbers; do not
   adjust the walk to make them agree.
4. **Anything requires editing `src/` or `wat/`.** STOP — this stone is scratch-pad only, and a
   substrate gap is a finding for the orchestrator, not work for you.

A STOP means: leave the tree as it is, write the report, end your turn. It is never a licence to ship
a smaller version of a row.

## What you own that nobody can reconstruct

Your exact outputs, the census in row 3 site by site, and anything that surprised you — a construction
that failed for a reason the brief did not predict, a message that read wrong, a count you expected to
move that didn't. That is the part of your report worth reading.
