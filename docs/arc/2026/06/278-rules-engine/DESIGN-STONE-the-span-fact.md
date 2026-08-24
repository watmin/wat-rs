# DESIGN — STONE: the Span fact (the coordinate a rule can bind)

> The single blocking prerequisite for wat-grep. `rules-corpus-03-source-to-facts.wat` turns real
> files into facts and emits `Node` and `Named` — **and no coordinates.** Nothing a rule binds can
> say WHERE it matched, so no rule can produce a `Match`.

## Why this became blocking, and it is the contract's fault — correctly

Builder's contract, 2026-08-24: **the user's rules assert `Match` facts; wat-grep queries for them
and prints.** wat-grep owns exactly one query and performs no interpretation.

Under the earlier model wat-grep could have assembled coordinates itself, because it saw the query
results. Under this contract **it never sees the match** — only the fact the user's RHS built. So the
coordinates must be *bindable from the fact base*, or a user rule cannot construct a `Match` at all.

The contract is right (it is the honest, minimal shape). The consequence is that a `Span` fact stops
being ergonomic and becomes load-bearing.

## Measured before drawing

**`ast-span` and `ast-end-span` are BOTH TOTAL.** Probed across every leaf kind — `keyword` `int`
`string` `keyword` `vector` `map` `symbol` `float` `bool`, plus `list` at top level. Every one
returned a span; none raised.

```
:a::b  start {:line 1 :col 2}   end {:line 1 :col 7}
1      start {:line 1 :col 8}   end {:line 1 :col 9}
```

★ **This is the OPPOSITE of `ast-name`, and it inverts corpus-03's guard design.** That file's
central lesson is that `ast-name` is PARTIAL — it raises on any non-nameable node, and the fix was
structural: emit a `Named` fact ONLY for a nameable kind, so *"the absence IS the guard"*. It cites
the cost of getting this wrong: a hand-rolled codemod died on **43 of 1392** files by guarding on
arity and calling `ast-name` anyway.

`Span` needs **no guard**. It is emitted unconditionally beside `Node`.

And that flips the non-vacuity control, which must be stated so nobody copies `Named`'s:

```
Named  <  Node      only nameable nodes get one   (corpus-03's existing control)
Span   ==  Node     EVERY node gets one           (this stone's control)
```

**A `Span` count that is less than `Node` means a guard crept in that does not belong.**

## The fact

```clojure
(:wat::core::defrecord :fx::Span
  [id        <- :wat::core::i64      ; joins to Node/id — the pre-order identity
   line      <- :wat::core::i64
   col       <- :wat::core::i64
   end-line  <- :wat::core::i64
   end-col   <- :wat::core::i64])
```

**Flat, not nested, and NOT `:wat::core::Span`.** Three reasons, and the third is the real one:

1. `ast-span` returns `(HashMap :- [keyword i64])` — keyword→i64, so it **cannot carry `:file`**.
   `:wat::core::Span` has a `file` field; a fact built from `ast-span` could not fill it honestly.
2. The file is a property of the RUN, not of a node. Repeating it on every fact in a 4316-node file
   is 4316 copies of one string.
3. **A rule binds FIELDS, not sub-records.** `(:fx::Span (?l <- :line) (?c <- :col))` is how a
   condition reads. Nesting the coordinate inside a record would force every rule to destructure
   before it could use a line number, which is the ergonomics this whole surface exists to get right.

The user's RHS then assembles `:wat::core::Span` — which DOES have `file` — from bound `?l`/`?c` plus
the filename wat-grep supplies. **That assembly is proven**: a rete RHS can construct a record with a
nested record field, with LHS bindings flowing into the nested constructor. Probed this session,
output verbatim:

```
#p/Hit {:span #wat.core/Span {:file "a.wat" :line 7 :col 1 :end :wat.core/None} :why "…"}
```

⚠ That probe ran on a tree a rider was mid-flight in. The finding is a property of rete's compiler,
not of the rider's edit — but it is RE-RUN on a quiescent tree as this stone's row 4, not credited.

## The rooms

1. **`wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat:64-90`** — `:fx::walk`. `id`, and
   the node itself, are already in hand exactly where `Node` is conj'd. `Span` goes beside it.
2. **`:fx::Acc` (~:41)** — gains a third vector, threaded like `nodes` and `named`.
3. **`:fx::report` (~:117)** — prints the counts; gains `Span=` so the control is visible.

## Acceptance

1. **`Span == Node` on every file probed.** The non-vacuity control, and it is the row that catches a
   stray guard.
2. **A rule joins Node × Named × Span and binds a line.** Proves the coordinate is reachable from a
   condition, which is the entire point.
3. **`Span` count is non-zero on a real file** — corpus-03 already reports over `wat/fix.wat`
   (Node=4316), so the numbers stay comparable to its existing output.
4. **A RHS builds a `:wat::core::Span` from bound `?line`/`?col` plus a supplied filename** — the
   nested-construction probe, RE-RUN on a quiescent tree.
5. The file still loads (`every_wat_scripts_file_loads`), floor green, clippy 0.

## Out of scope — affirmatively cut

- **`:wat::grep::Match` itself, and wat-grep's loop.** This stone makes the coordinate bindable;
  the Match record and the utility are the next stone.
- **Promoting the walk out of scratch-pad.** corpus-03 is still a probe. When it becomes the real
  slurp it gets a home and a name; not yet.
- **Byte offsets.** `fix.wat` derives them from `{:line :col}` + the file's lines
  (`fix-text-offset-of`). Carrying an offset would give one position two sources of truth that drift
  the moment anything re-reads the file.
