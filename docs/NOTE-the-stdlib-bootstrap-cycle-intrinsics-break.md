# ⛔ NOTE — the stdlib has a BOOTSTRAP CYCLE, and Rust intrinsics are what break it

**Found 2026-08-17, at the cost of one rider flight.** Filed at `docs/` rather than in an arc,
because it is a property of the **substrate's load model** and the next person to move a verb from
Rust to wat will rediscover it the same way.

## The rule, in one sentence

> **A verb consumed by a macro body that runs during stdlib load CANNOT be defined in wat.** It must
> be a Rust intrinsic — available from expression zero, before any `.wat` file exists.

Nothing in `src/stdlib.rs`, in the `wat/` files, or in any doc says this. It is enforced only by a
crash, and only after you have already written the code.

## How it was found

Stone 279.3 ruled that `:wat::core::string::join` should become a wat `defn` — a reasoned position,
argued over three rounds, with a green committed exemplar proving the composition. A rider built it
exactly as specified. The stdlib died before `main`:

```
#wat.macro/ProgramBodyEvalFailed — macro :wat::core::defrecord — program body eval failed
  at wat/core.wat:1885
  cause: #wat.runtime/UnknownFunction "unknown function: :wat::core::string::join"
         at wat/Record.wat:172
```

`wat/core.wat:1885` self-invokes `(:wat::core::defrecord :wat::kernel::Location …)` **while
`core.wat` is still loading**. `defrecord`'s body computes a namespace prefix at `Record.wat:172` —
`(:wat::core::string::join "::" ns-lead)` — and that runs at **macro-expansion time**, during the
load. A wat-defn `join` does not exist yet, and will not until position 278.

## ★ THE CYCLE — measured, and it is real

```
stdlib.rs:40    wat/core.wat      ─┐  core.wat's MACRO BODIES need `join`
stdlib.rs:131   wat/Record.wat    ─┼─ three join users load BEFORE string.wat
stdlib.rs:169   wat/bracket.wat   ─┘
stdlib.rs:278   wat/string.wat       string.wat needs `defn` + `keyword` (core.wat), `mapv` (seq.wat)
```

**`core.wat` ↔ `string.wat` is a genuine dependency cycle.** The graph is acyclic **only because
`join` is a Rust intrinsic.** The intrinsic is not an implementation detail here — it is the
cycle-breaker.

Three consequences worth stating, because each one killed a proposed fix:

1. **Routing one file's call sites at a primitive is not enough.** The rider proposed pointing
   `Record.wat` at a `join'`. `core.wat` and `bracket.wat` are also ahead of `string.wat`.
2. **Reordering is impossible, not merely risky.** `string.wat` calls `:wat::core::defn` and
   `:wat::core::keyword`, both defined in `core.wat`. It cannot load first.
3. **A two-tier stdlib fails Honest.** "Early files use the primitive, later files use the surface"
   is an unenforced, unwritten rule whose violation is a crash at position 40. The four questions
   killed it on that axis.

## What this does NOT mean

- **It is not an argument against wat-defined stdlib verbs in general.** Most of `wat/` is wat, and
  correctly so. The constraint binds exactly one population: **verbs reachable from a macro body that
  expands during load.**
- **It is not a defect to fix.** A bootstrap needs a floor written in something that exists before the
  bootstrap. Every language with a self-hosted stdlib has this seam somewhere. The defect is that the
  seam is **undeclared**, not that it exists.
- **It does not affect arc 255's carve.** 255 registers intrinsics into `sym.functions`; it does not
  move them into wat. The carve is unaffected. *(Checked, because the reverse would have been a large
  hidden constraint on that arc.)*

## The rung this is currently on, and the one it could be on

Today: **a convention nobody wrote down** — the weakest rung on the extirpare ladder, and it just
cost a rider flight.

The next rung is **a check that fires at construction time**: a test or lint that, for every verb
referenced from a macro body in a file at load position *N*, asserts the verb is either a Rust
intrinsic or defined at a position *< N*. That is mechanically decidable from `stdlib.rs`'s order
plus a scan of macro bodies, and it would have turned this flight into a red line at edit time.

**Not built. Not briefed. Recorded so the cost is paid once.** If a second Rust→wat move trips this,
that is the signal to build the check rather than write a third note.

## Kin

- `docs/arc/2026/06/279-format/DESIGN-STONE-279.3-join-renders-its-elements.md` — the `⛔ CORRECTION`
  section: where this was found, the four questions run flat on all four options, and the ruling
  (`join` stays an intrinsic and becomes generic).
- `docs/arc/2026/06/279-format/BRIEF-STONE-279.3-…md` — bannered SUPERSEDED; kept as the record of
  what was tried.
- `docs/SUBSTRATE-AS-TEACHER.md` — same family: the substrate's failure IS the diagnostic. Here the
  failure arrived only at runtime-of-the-loader, which is the latest and most expensive moment it
  could have.

---

> The design argument had plausible positions on both sides and ran three rounds. **Neither side had
> measured the load order** — one command. The oscillation itself was the tell: when a question keeps
> flipping on reasoning, the deciding fact is usually one nobody has gone to look for.
