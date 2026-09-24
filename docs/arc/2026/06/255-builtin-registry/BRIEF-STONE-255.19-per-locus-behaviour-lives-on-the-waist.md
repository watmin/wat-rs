# BRIEF — STONE 255.19: per-locus behaviour lives on the waist

**Drawn 2026-09-24 against `main` @ `f8851be76`.** Floor 6032/6032, clippy 0, census `no STOP-8`, delta
**3 / RECOVERY 0**, ledger 220. **Executor tier: Opus.**

## Why

255.18 made `Locus :- [T]` and stopped: `:wat::spawn::runner-count` (`wat/spawn.wat:181`) and
`:wat::spawn::with-label` (`:471`) are **defclauses keyed on the concrete loci**. A generic
`(Locus :- [T])` cannot narrow into them. Their own comment says *"a new locus type joins as one more
clause here"*, a per-locus list outside the surface, which the narrow-waist ruling forbids. On top of
that, `with-label` returns the bare `Locus`, and a bare `Locus` is accepted as any `(Locus :- [T])`, so a
process locus passed through it checks as a `Shared` launch.

The four questions (`WEIGH-STONE-255.18-…`) ruled **N2: both become `Locus` surface methods,
implemented in each locus's `extend-type`.**

## The work

1. Add `runner-count` and `with-label` to the `Locus :- [T]` surface:
   - `(runner-count [self <- (Locus :- [T])] -> :wat::core::i64)`
   - `(with-label :- [R] [self <- (Locus :- [T]) r <- :R] -> (Locus :- [T]))`

   `with-label` **keeps the transport**. Match the surface's existing method style.
2. Move each clause body into its locus's implementation, `ThreadOpts` → `(Locus :- [Shared])` and
   `ProcessOpts` → `(Locus :- [Wire])`. Choose the implementing `extend-type` (`spawn.wat` or
   `bracket.wat`) by the method's home, and say why in the SCORE.
3. Retire both defclauses. There is **one way**: no alias and no forwarding defn.
4. Migrate every caller (`with-label` 26 in 9 files, `runner-count` 7 in 6; census them per site) to the
   surface call (`:wat::spawn::Locus/runner-count`, `:wat::spawn::Locus/with-label`). A repeated
   structural rewrite is a **wat-fix codemod** (dry-run, diff, apply, committed with a replay fixture).
   ⚠ Run it with `./target/release/wat`: `cargo wat` in `~/.cargo/bin` is a stale Aug-29 binary.
5. Now finish 255.18's stopped step. The sites that kept a bare `Locus` because of the defclauses —
   defservice's `start$impl`/`resume$impl` locus parameter (`wat/service.wat` ~:2585), `bracket.wat`
   `map-worker`, `wat-scripts/probes/arc-170/probe-s2-runner-count.wat` `read-blind`, and the
   `with-label` callers annotated `-> :wat::spawn::Locus` — take `(Locus :- [T])` with `T` declared, or
   the concrete transport. Remove each comment pointing at the gap.
6. **Rows:**
   - The erasure witness `wat-scripts/scratch-pad/255-18-with-label-erases-the-transport.wat` must now
     be **refused** (a process locus through `with-label` claimed `Shared`). Move it to a `.wat.bad`
     negative fixture, with a positive twin claiming `Wire`.
   - 255.18's narrowing pin `…_generic_narrowing.wat.bad` asserts the gap. **The gap is closed by
     removal, not by the checker**, so rewrite the row as a positive: a generic `(Locus :- [T])` reads
     its runner count.

## STOP triggers

1. The defservice locus parameter going generic needs the Status/Handle/launch-tp-ann free letter
   changed (C-b2 ground) → STOP, report the sites and errors verbatim.
2. A caller depends on the defclause's runtime dispatch in a way a surface method cannot express →
   STOP, report it.
3. `spawn-program` and `:wat::test::spawn-peer` are **out of scope**: their program and result types
   differ per locus. If this stone cannot close without them → STOP, report the shape.

## Expectations — fixed before the strike

| what | command | expected |
|---|---|---|
| defclauses retired | `git grep -n 'defclause :wat::spawn::\(with-label\|runner-count\)'` | 0 |
| erasure refused | the new `.wat.bad` | rc=1, Wire ≠ Shared |
| erasure twin | the positive twin | rc=0 |
| generic reads its count | the rewritten row | rc=0 |
| bare-`Locus` census | per-site census of `:wat::spawn::Locus` in type position, excluding `(Locus :- [..])` | report the count and every survivor, each with its reason |
| floor · clippy · census · delta · ledger | the usual | green · 0 · `no STOP-8` · NEW 3 / RECOVERY 0 · ≤ 220 |

Runtime prediction: 2–3 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name
  the arm. (`harvest_wrap_split`: cite it and report it.)
- ⭐ Prove every fixture can say BOTH words; each new negative is shown accepted on the pre-stone binary.
- Leave the checker's transport special cases in place (C-b5).
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add
  -- <paths>`; **do not push.** Spawn no subagents.

## Out of scope

`spawn-program` and `spawn-peer` (per-locus program types); refusing a bare `Locus` as any
instantiation (C-b5); C-b1b; C-b2 through C-b5.
