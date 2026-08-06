# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`, which is where history belongs.

## Where the code is — nothing parked, nothing uncommitted

```
HEAD 52d1f73f   pushed   floor 4373 passed / 0 failed / 262 skipped   clippy 0
```

`git status` empty. No patch to re-apply, no fence to restore, no red to recover from.

## ★ WHAT LANDED (2026-08-06, the far side)

| commit | |
|---|---|
| `37494061` | **CORRECTION** — Step 0 HAS run; "nobody has decomposed filter" was false in 3 places |
| `ebabad26` | **#49 step 1** — the Op set drawn (nesting, ruled) |
| `1eb31a2f` | the `Interp` escape hatch **KILLED** — lowering is total or it refuses |
| `e29c0f12` | a `where` can fail to TERMINATE — proven; the fifth axis |
| `5dc70e62` | **RETRACTED** the fifth axis — the termination argument was a false analogy |
| `f4ad0123` · `56de27fd` · `b041a4f0` · `78a4e611` · `38cec0eb` · `46a12e44` | the VERIFIER MODEL — static refusal, bounded-not-derived, how it is measured, iteration, user aggregators, the S0–S5 strike |
| `7b8464c0` · `b39bf553` | **#85 CLOSED** — the Clara grid is 18/18 ALIVE |
| `0436ed66` | **#89 CLOSED** — the 18 axes are now EXERCISED in the floor, not merely parsed |
| `e6e8b7a3` · `52d1f73f` | **#88** — the rete `defn` stone, four-questions 4×YES, **named `:wat::rete::core::defn`** |

## ▶ FIRST ACT — #88 is RULED and NAMED. Build it.

**Read `DESIGN-STONE-the-rete-defn.md`.** Do NOT re-derive it. The builder's own framing is the whole
design: *"we just need it create functions who bind to symbols and those functions are tighter in
expressions than core-defn"* — same registration, same symbol binding, **body checked tighter AT THE
DEFINITION SITE**.

Settled, with reasons on the disk:
- **the membrane, not accept-only** (4×YES vs two NOs; and the migration is a pure re-heading of 27
  already-clean sites, so a phase buys nothing)
- **the name** — `:wat::rete::core::defn`, intueri-cast and weighed against the disk. `cond` is the
  case that decides it: top-level AND `core::` AND a core mirror, so "top-level" predicts nothing.
- **the mechanism** — `Function` has no metadata field; one new field, and `head_ok`'s
  `sym.functions` branch changes from *walk the body* to *consult the marker*. That one branch IS
  the membrane.
- **NOT a `RETE_OPS` row** — that table is what may appear INSIDE a predicate. `defn` is a top-level
  declaration. ⚠ Consequence: the naming-rule tests iterate `RETE_OPS` only, so **the FQDN is a
  convention honored BY HAND** (as `cond`'s is). A convention no test enforces is the kind that rots
  — say so in the brief.

Four STOPs are in the stone. STOP-4 is the one that can bite: the migration is a **re-heading**; if
any of the 27 needs a BODY change, the "already admitted ⇒ already clean" reasoning has a hole —
surface it, do not quietly edit a body.

## Then, in order

- **#87 `bound_expr`** — S0–S5 drawn in `DESIGN-STONE-the-one-expression-core.md`. #88 gives the
  bound its natural home (computed once at a declaration, not re-derived per call site), so land #88
  first and hang this on the marker. **S0 is a RED probe and it is the point**: a walk that misses a
  form returns a number that is TOO SMALL — a limit that passes because it never saw the code. That
  is #82's class exactly.
- **#49 the IR** — the prize. Op set drawn, hatch refused, `match`'s 12-arm grammar grounded. Three
  STOPs remain: the closure frame, `CallUser`, result-vs-value.

## ⛔ THE NUMBERS ARE STILL THE BUILDER'S

`depth` / `nodes` / `fold_nesting` limits are **unset by ruling**. Set them from `bound_expr`'s real
distribution, never from a corpus estimate — mine were wrong twice in one session (source-form
`max depth 7 / nodes 9`; **fully inlined: 33 / 33**, so 16 and 32 would refuse a real predicate
today). **We are the kernel here** — the bound is a decision, not a constraint handed down.

## What the record now knows that it did not this morning

- **A `where` CAN fail to terminate** — one door only (the composition back-edge; a self-referencing
  `let` lambda is unbound at runtime). Tail-recursive → hangs (fine, correct-but-unending);
  **non-tail → SIGSEGV, core dumped, ZERO diagnostic**. That mask is **#58**, substrate-wide, not a
  rete defect. The fifth axis was **retracted** — my "no opcode for non-termination" was a false
  analogy to the raise argument.
- **Bounded, never derived.** A lambda can be chosen by a FACT (`get` into a PV at a `?var` index) —
  measured, fires. So the contract is an upper bound over all paths, and *refuse what you cannot
  bound*. Two preconditions: no recursion, and every callee enumerable at lower time.
- **Data is unbounded by design; the PROCESSING is what we bound.** Iteration cannot be manufactured
  from a scalar — there is no `range`/`repeat`/`iterate`, and no `conj`/`assoc`, so a long fold
  requires a long collection to have *arrived*. The one legitimate exception is the ENGINE's own
  gather (`acc::all`/`distinct`/`gather-vals`/`group-by`), and a user aggregator receives that
  engine-sized `PV<T>` **as an argument**.
- **`user-reduce` carries the corpus's ONLY user aggregator** — dead until today, so that capability
  had never once run under #83's fence.

## The rules today paid for — carry these

- **A real fork can still be MIS-POSED.** Two "predictors" that are one variable (confounded), or a
  migration PHASE judged by the four questions as an end state. Before posing A vs B: name the case
  where they DISAGREE, and check both are at the same level.
  ([[feedback_a_real_fork_can_still_be_mis_posed]])
- **Pick the arbiter by the gap's PHASE.** `--check` said OK on a law-A violation and I was one step
  from reporting a hole in the fence. The fence is a RUNTIME check; only running it is the arbiter.
- **A search that cannot reach is not evidence — FOUR times today.** `--check` (wrong phase); a grep
  that could not see a glob (I reported 7 axes as having "no driver"; `check-where-shapes.sh` globs);
  a fold-nesting census that walked `where` forms and missed user aggregators entirely; and `$?` read
  after a pipe through `head`, which returns *head's* exit — the exact trap the floor rule names, in
  my own verification, an hour after quoting it.
- **A gate that DISCOVERS beats one that LISTS** — `run-all.sh` listed, so a new axis would be swept
  by nobody. Now it discovers and reconciles, failing loud in both directions.
- **The wall fires on the wall-builder.** `no_inlined_edn` caught my own new gate file. NOT runed —
  the literal was an input, not a golden, so the lint's own remedy applied. Taking the rune would
  have been the launder.

## Open, in the order I would take them

**#88** (ruled + named, build it) → **#87** (bound, hangs on #88's marker) → **#49** (the IR).
Also open: #7 chaos engine · #50 the per-TestNode clone · #58 the silent SIGSEGV (raised in priority
by the rete surface's exposure, unchanged in ownership) · #60 · #64 · #67 · #81 · #86.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> HEAD is green, pushed, clean. Nothing is parked. #88 is ruled AND named — read the stone, do not
> re-derive it — and the limits in #87 are the builder's to set.
>
> The line this session cost the most to buy: **a fork can be real and still be mis-posed.** Both
> branches existed both times; one pair was a single variable wearing two names, the other was a
> phase judged as a destination. The four questions ran cleanly on both and answered the wrong
> question — which is worse than a fork that does not exist, because it survives scrutiny.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
