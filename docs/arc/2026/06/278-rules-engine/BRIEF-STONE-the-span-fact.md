# BRIEF — STONE: the Span fact (the coordinate a rule can bind)

DESIGN: `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-span-fact.md` — read it first, whole.

## Your role

You are a rider, not the orchestrator. **Ending your turn ENDS you** — it does not suspend you, and
nothing will wake you. There is no notification coming. Run every command in the FOREGROUND and
block on it: your turn ends when the numbers are in your hands, not when a command is launched.

**You may not spawn sub-agents.** Everything in this brief is yours to do directly.

Anchor: `/home/john/work/holon/wat-rs`. `pwd` first; use `git -C /home/john/work/holon/wat-rs` for
any git READ. You do not commit, push, stash, revert, or checkout — the orchestrator owns the tree's
history. Leave your work uncommitted in the working tree and report.

Cap every run: `systemd-run --user --scope -q -p MemoryMax=6G -p MemorySwapMax=0 timeout 180 …`

## The work, in one paragraph

`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat` turns a real `.wat` file into a fact
base and emits two fact kinds — `:fx::Node` (every node) and `:fx::Named` (only nameable ones). It
emits **no coordinates**, so nothing a rule binds can say WHERE it matched. Add a third fact,
`:fx::Span`, emitted **unconditionally beside `Node`**, carrying the start and end line/col that
`ast-span` / `ast-end-span` return. Then prove a rule can JOIN on it and bind a line.

## The rooms — read in this order

1. **`wat-scripts/scratch-pad/probe-ast-span-totality-under-reader-macros.wat`** — the orchestrator's
   probe, run this session. It is your worked reference for the `ast-span` → `HashMap/get` →
   `Option/expect` chain and for the unguarded-walk shape. Its measured output:
   `sigils-inline Node=23 Span=23` · `probe_do_splice Node=33 Span=33` · `wat/fix.wat Node=4316 Span=4316`.
   **`ast-span` and `ast-end-span` are TOTAL — including reader-synthesized nodes.** Emit `Span`
   with NO guard.
2. **`rules-corpus-03-source-to-facts.wat:28-36`** — the `:fx::Node` / `:fx::Named` defrecords. Your
   `:fx::Span` goes beside them, in the same style.
3. **`rules-corpus-03-source-to-facts.wat:38-42`** — `:fx::Acc`. It threads `nodes` and `named`; it
   gains `spans` as a third `PersistentVector`, threaded identically.
4. **`rules-corpus-03-source-to-facts.wat:60-92`** — `:fx::walk`. `id` and `node` are both in hand on
   the exact line `Node` is conj'd. `Span` is conj'd right there, unconditionally.
5. **`rules-corpus-03-source-to-facts.wat:99-102`** — `:fx::empty-acc`. Initialise the third vector.
6. **`rules-corpus-03-source-to-facts.wat:117-125`** — `:fx::report`. It prints `Node=` and `Named=`;
   it gains `Span=` so the control is visible in the output.
7. **`wat/fix.wat:179-193`** — `fix-text-offset-of`, production code, the canonical way to pull an
   i64 out of an `ast-span` map: `(Option/expect (HashMap/get loc :line) "…")`.
8. **`wat/core.wat:2148-2152`** — `:wat::core::Span` = `file` / `line` / `col` / `end <- (Option :- [Pos])`.
   This is what the USER's RHS assembles in row 4. It is NOT what `:fx::Span` is.

## The fact — exactly as drawn

```clojure
(:wat::core::defrecord :fx::Span
  [id        <- :wat::core::i64      ; joins to Node/id — the pre-order identity
   line      <- :wat::core::i64
   col       <- :wat::core::i64
   end-line  <- :wat::core::i64
   end-col   <- :wat::core::i64])
```

Flat, five i64 fields, no nesting, no `:file`. The DESIGN gives the three reasons; the load-bearing
one is that **a rule binds FIELDS, not sub-records** — `(:fx::Span (?l <- :line))` must read directly.

## Implementation sketch

In `:fx::walk`, beside the existing `nodes` binding:

```clojure
sp    (:wat::core::ast-span node)
ep    (:wat::core::ast-end-span node)
spans (:wat::core::PersistentVector/conj (:fx::Acc/spans acc)
        (:fx::Span :id id
                   :line     (:wat::core::Option/expect (:wat::core::HashMap/get sp :line) "Span :line")
                   :col      (:wat::core::Option/expect (:wat::core::HashMap/get sp :col)  "Span :col")
                   :end-line (:wat::core::Option/expect (:wat::core::HashMap/get ep :line) "Span :end-line")
                   :end-col  (:wat::core::Option/expect (:wat::core::HashMap/get ep :col)  "Span :end-col")))
```

and thread `spans` into `acc'` exactly the way `nodes` and `named` are threaded.

## The acceptance rows YOU run

Build nothing. `target/release/wat` is current and is all you need.

- **Row 1 — `Span == Node` on every file `:fx::report` prints.** This is the non-vacuity control and
  the row that catches a stray guard. `Named < Node` must still hold (that guard is correct and
  stays). A `Span` count BELOW `Node` is a failure, not a curiosity — report it, do not fix past it.
- **Row 2 — a rule joins `Node` × `Named` × `Span` and binds a line.** Add one rule + one query to
  corpus-03 alongside `:fx::arrow` / `:fx::head-kw` / `:fx::type-pos`, in their style: match a
  `Node` whose `Named` name is `"<-"`, join `:fx::Span` on the same `?id`, bind `?l <- :line`, and
  assert a fact carrying `?l`. Query it and print the count and one bound line number. Copy
  `:fx::arrow` for the shape and `:fx::classify` for the compile/insert/fire/query chain.
- **Row 3 — `Span` is non-zero on a real file.** `wat/fix.wat` reports `Node=4316` today; the numbers
  must stay comparable to corpus-03's existing output.
- **Row 4 — a RHS builds a `:wat::core::Span` from bound `?line`/`?col` plus a supplied filename.**
  Nested-record construction with LHS bindings flowing into the nested constructor. The recorded
  output to reproduce in shape:
  `#p/Hit {:span #wat.core/Span {:file "a.wat" :line 7 :col 1 :end :wat.core/None} :why "…"}`.
  Use `:end` = None; you do not need `:wat::core::Pos`. **This row was previously measured on a tree
  that had a live writer in it, so it is UNCREDITED — run it fresh and report your own output
  verbatim.** Put it in its own file: `wat-scripts/scratch-pad/probe-rhs-builds-core-span.wat`.
- **Row 5 — `target/release/wat --check <file>` exits 0 for every file you touched or created.**

Report each row's real output verbatim. A row you could not run is reported as not-run, never as
passed.

## Blast radius

- `wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat` — edited
- `wat-scripts/scratch-pad/probe-rhs-builds-core-span.wat` — created (row 4)

Nothing under `src/`. Nothing under `wat/`. No new stdlib verb. No change to `:fx::Named`'s guard.

## STOP triggers — each ships NOTHING and surfaces the gap

1. **`ast-span` or `ast-end-span` raises on any node reached by the walk.** STOP. The stone's central
   measurement is refuted and its guard design inverts. Report the file, the node kind, and the
   raise verbatim.
2. **`Span` count comes out below `Node` on any file.** STOP. A guard crept in where none belongs.
   Report both counts per file; do not adjust the walk to make the numbers agree.
3. **A rete condition cannot bind a field from a five-field record**, or the three-way join
   `Node × Named × Span` will not compile. STOP and report the compiler's message verbatim — that is
   a rete-surface finding, and rete is not yours to change.
4. **Row 4's nested construction fails.** STOP and report the error verbatim. Do not fall back to a
   flat record or to assembling the Span outside the RHS — the whole point of the row is that the
   RHS can do it.

A STOP means: leave the tree as it is, write the report, end your turn. It is never a licence to
ship a smaller version of the row.

## What you own that nobody can reconstruct

Your numbers per file, the exact output of each row, and anything that surprised you — a node kind
you did not expect, a count that moved, a message that read wrong. Those are the honest deltas and
they are the reason to read your report rather than re-run your work.
