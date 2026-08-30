# ⛔ NOTE (arc 301) — a record accessor in VALUE position is typed `Record -> F`, so it cannot be passed where the CONCRETE record is expected

**Found 2026-08-30, live, by probe.** Surfaced by grok mid-strike on stone 2b:

> *"`mapv` won't take a record accessor as a function here. I'll wrap `Row/sk` and
> `IndexRow/isk` in explicit fns."*

Builder's read: *"that feels like a bug..."* — measured, and it is.

The wrapper workaround is correct and cheap; nothing is blocked. This NOTE exists because the
shape is backwards from every intuition and will cost someone an hour later.

## ⚠ It is NOT the same finding as `NOTE-an-intrinsic-cannot-be-passed-as-a-value` (arc 255)

That one: `:wat::core::str` has **no `Function` entry at all**, so the checker reports what the
token literally is — a `keyword` — and no clause of `mapv` matches.

This one: the accessor **does** resolve to a function. It is passable. Its *type* is wrong.

```
:wat::core::mapv: parameter #1 expects [:probe::R :-> :?3492];
                  got [:wat::core::Record :-> :wat::core::String]
```

Same family — a declaration whose type is more general than the thing it describes — but a
different mechanism and a different fix.

## The measurement — three probes, and the third is the perverse one

```wat
(:wat::core::defrecord :probe::R [sk <- :wat::core::String])
```

**A — direct call.** GREEN.
```wat
(:probe::R/sk r)                                    ;; fine, always has been
```

**B — value position, CONCRETE element type.** RED.
```wat
(:wat::core::mapv :probe::R/sk (:wat::core::Vector :- [:probe::R] …))
;; parameter #1 expects [:probe::R :-> :?N]; got [:wat::core::Record :-> :wat::core::String]
```

**C — value position, ABSTRACT element type.** ★ **GREEN.**
```wat
(:wat::core::mapv :probe::R/sk (:wat::core::Vector :- [:wat::core::Record] …))
```

**Being MORE specific about your data makes the program stop compiling.** B fails and C passes,
and the only difference is that C throws away the element type. That is the finding.

And it is not about `mapv` or collections at all — a plain higher-order function reproduces it
with nothing else in scope:

```wat
(:wat::core::defn :probe::apply-to
  [f <- [:probe::R :-> :wat::core::String]  r <- :probe::R] -> :wat::core::String  (f r))

(:probe::apply-to :probe::R/sk (:probe::R :sk "a"))
;; parameter #1 expects [:probe::R :-> :wat::core::String];
;; got [:wat::core::Record :-> :wat::core::String]
```

## The mechanism, as far as it is grounded

The accessor's value-position type carries the **abstract** `:wat::core::Record` as its
receiver, not the concrete record it was generated for. A direct call survives this because
the call path has an explicit narrowing that a value use does not — `src/check.rs:5602–5620`,
whose own comment says so:

> *"Arc 258 cascade — accept any subtype of `:wat::core::Record` (includes specifically-typed
> records like `:myapp::Pt` in addition to the root `:wat::core::Record`). `is_subtype` is
> reflexive so `:wat::core::Record` itself still matches."*

So the call site compensates for the over-general receiver with a subtype check. **Value
position gets no such compensation**, and wat has no parameter contravariance, so
`Record -> String` does not unify with `R -> ?`.

**Honest limit of this NOTE:** I measured the behaviour and located the compensating call-site
path. I did **not** find the line that constructs the accessor's scheme with a `Record`
receiver. Whoever fixes this should start there, not from this paragraph.

## Why this is the "declared optimistically" family again

`~/work/gen-tests/FINDINGS.md` F3 recorded three `check.rs` sites whose static signature names
the common case while the runtime collapses to another type — *"sound-enough"* declarations
that compose into a well-typed program that dies. This is the same shape moved one position
over: a declaration that is over-general in its **parameter**, kept usable by a compensating
check at one use site, and unusable at every other. F3's lesson applies unchanged — **the
compensation is where the type should have been.**

## The fix, and its size

Give the accessor the concrete receiver: `R -> String`, not `Record -> String`. Then the call
site's subtype narrowing becomes redundant rather than load-bearing, and value position works
without help.

**Not drawn.** It is a type-system change, not an arc-301 change, and 301 has a working
one-line workaround (wrap the accessor in a `defn`). Recorded here because 301 surfaced it.

**Filed to its home arc as
`docs/arc/2026/04/109-kill-std/NOTE-a-callable-keyword-in-value-position-has-four-kinds-and-three-answers.md`** —
where scoping the census from the RULE (rather than from this instance) turned one defect into
a four-row table: a user `defn` resolves concretely, an accessor resolves with an ABSTRACT
receiver, and a constructor resolves to a bare `keyword` exactly like an intrinsic. Three
different answers to the same question. Read that one for the taxonomy; this one for the
measurement.

## What to do meanwhile

Wrap it. This is what grok did, unprompted, and it is right:

```wat
(:wat::core::defn :probe::get-sk [r <- :probe::R] -> :wat::core::String (:probe::R/sk r))
(:wat::core::mapv :probe::get-sk rows)     ;; GREEN
```

A user `defn` lands in `sym.functions` with a `TypeScheme` naming the concrete receiver, so it
unifies. One line, no cost at runtime.
