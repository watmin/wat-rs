# WEIGH — STONE 255.15: ACCEPTED. ⭐ The transport now flows from the locus — and it found a real hole.

Weighed against `7cfa923a0` by independent re-run. Step 1 of the narrow waist (SEAM ruling).

## Re-derived here

| | mine |
|---|---|
| ⭐ my own A/B from this morning, type-variable row | rc=1 → **rc=0** |
| ⭐ my own original positive probe (`t-pos`), **run** | **`"1"` then `"w"`** — `T` flows from each locus through one generic `start` |
| concrete A/B | rc=0 / rc=1, **same diagnostic** as before |
| fixture suite | **9/9** |
| wrong transport (my `t-neg`) | **rc=1 `ReturnTypeMismatch`** — refused for the *right* reason now |
| one `T` given a thread AND a process locus (my construction) | **rc=1**, `#2 expects (Loc :- [Shared]); got Pr` |
| control: same fn, both thread | **rc=0** — the conflict refusal is specific |
| floor | run 1 **RED 6022/1** (its own lint, captured whole), run 2 **6023/6023** |
| clippy | rc=0, **fresh** (6.26 s) |
| delta | **NEW 3, RECOVERY 0** |

⭐ **The permissive direction is bounded**: a wrong transport, a conflicting binding, a non-implementor, a
nested mismatch, a `derive` chain and the ambiguous case are all refused, and **the ambiguity refusal was
shown load-bearing by mutation** (take-the-first → rc=0).

⭐ **The executor refused its own first draft.** Walking the `derive` chain "for parity with `is_subtype`"
accepted a program that dies at run time. It measured that, kept direct edges only, and pinned the refusal.
**A permissive cure that caught itself spreading a hole is the discipline working.**

## ⛔⛔ The hole it found — reproduced here, and it breaks the invariant this whole line exists for

`ambig-lie-run.wat`, both binaries: **`--check` rc=0, run rc=2** — *"expected receiver of class
`:probe::Wire`, got class `:probe::Shared`"*. A type that declares the `Shared` transport with a body and
the `Wire` transport without one passes as `Wire`; at run time the `Shared` body runs.

⛔ **That is a Shared transport masquerading as Wire — a route for a process to hold a shared-memory
address**, the exact rule `a6da457e3` made a type error. It predates this stone and this stone did not
widen it. **It must be closed before the narrow waist relies on transport types** — likely by refusing a
second instantiation of one parametric surface on one type at registration (make it unrepresentable).

The `derive`-chain hole reproduces too: `derive-run.wat` checks rc=0, runs rc=1 `UnknownFunction`.

## ⛔ The brief was wrong twice — and the second is a rot in the code itself

1. **`types.rs:2151` is not the edge store** (it is `Option` prose). ⛔ **The orchestrator copied that line
   number from a code comment** — `check.rs:17374` and `:17473` both say *"types.rs:2151 stores the
   extend-type target keyword VERBATIM"*. The file has drifted; **both comments now point at the wrong
   place** and will mislead the next reader. A line number in a comment is a claim that rots.
2. **`check.rs:5449` is an analogue, not a precedent** — it binds from the receiver's own arguments. The
   real neighbour is 118.3-B's parametric-vs-parametric arm.

## VERDICT

**ACCEPTED.** Pushing. **Next, in order:** close the coherence hole (one surface, one instantiation per
type) · step 2, the transport registry · step 3, the generic `start`/`resume` with the waist wall. The
alias design and probe spec are ready in parallel.
