# BRIEF — one counter, one unit; and stop pinning the same number twice

`compiled:calls` is asserted at exactly 80,200 and glossed as a call count. Driven: **every one of
those 80,200 is a product of two lengths**, and the two genuine call sites contribute **zero**. The
number it pins is the alpha-element count — which another test already pins, correctly named.

## Read in order

1. `probe-c14.sh.txt` beside this brief — rename one key, watch `compiled:calls` go to **zero**.
   Run it first; it restores itself by hash.
2. `src/rete/kernel/fire/pass/alpha.rs:195` — `census_count_n("compiled:calls", ids.len() * aids.len())`.
   The product. ⚠ `accum_cost.rs:52` cites this as `alpha.rs:122`; **`:122` is now `pack_i64_row`** —
   the citation rotted when D7's cure moved the site.
3. `src/rete/kernel/tests/accum_cost.rs:29-95` — `accum_matcher_op_census`: the `calls > 0` liveness
   guard at `:41`, the `== 80_200` pin at `:66`, and the long C10 comment at `:51-65` explaining the
   union. **That comment is correct about the union and wrong about the citation.**
4. `src/rete/kernel/tests/accum_alpha_cost.rs:14-45` — `accum_alpha_memory_shape`, which pins
   `alpha_elements == 80_200` with the words *"one element per (fact, matching alpha) pair"*. This is
   the same quantity, correctly named.
5. `src/rete/kernel/fire/delta.rs:78` and `src/rete/compiled_cond.rs:928` — the two genuine per-call
   sites, which fire **zero** times on this workload.

## Driven by the orchestrator at HEAD `d7464c95e`

Renaming only the `alpha.rs:195` key:

```
panicked at accum_cost.rs:41:5:
compiled:calls is zero — occupancy fill / skip-span / exec_compiled never counted
```

## The change

1. **`alpha.rs:195` emits `alpha:leaf-fill-pairs`.** One site, one string. No branch, no hot-path edit.
2. **`compiled:calls` keeps only the two per-call sites** and is therefore an honest call count.
3. **`accum_matcher_op_census` asserts what is true.** On this workload the compiled path is entered
   **zero** times. State it, and state why — that is the finding, not an inconvenience. If a
   call-count assertion needs a workload that enters the path, **name that workload; do not re-dial
   this axis** (its sizes are a recorded artifact).
4. **Fix the rotted citation** at `accum_cost.rs:52` (`:122` → `:195`), and replace the line number
   with something that cannot rot if you can — a symbol name beats a line.

⛔ **This is not the split C10 forbade.** C10 ruled out distinguishing the two **delta arms** — a
hot-path branch for an instrument's benefit, already discriminated in `accum_alpha_cost.rs`. This
separates two **units** at one existing call site. Read C10's comment and satisfy yourself the
distinction holds; **if you conclude it does not, STOP and say so** — that is a finding, not a
blocker to work around.

## Blast radius

`src/rete/kernel/fire/pass/alpha.rs` (one string), `src/rete/kernel/tests/accum_cost.rs`. Possibly a
recorded artifact quoting 80,200 as a call count — grep for it.

## STOP triggers

1. **If `compiled:calls` is non-zero on this workload after the split**, stop and report the value —
   my measurement says zero and yours outranks my inference.
2. **If you conclude this IS the split C10 forbade**, stop and report the reasoning.
3. **If any other test or artifact pins `compiled:calls`**, stop and report before changing its
   meaning — you would be silently re-defining a number someone else asserts on.
4. **If the honest assertion would need the axis re-dialled**, stop. The sizes are the artifact.

## Mutation proofs — run all three, report all three

1. **Delete the `skip_span` arm's bump** (`delta.rs:78`) → after the split, a call-count assertion
   must notice. ★ **Before the split it does not** — show both halves. This is the strike.
2. **Restore the old shared key** → the two assertions pin the same number again; show that
   `accum_matcher_op_census` passes while measuring nothing about the compiled path.
3. **Zero the product** (`ids.len() * 0`) → `alpha:leaf-fill-pairs` goes to 0 and its own assertion
   REDs, while `compiled:calls` is unaffected. Proves the two counters are now independent.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The probe's output, and `compiled:calls` after the split.
- All three mutation results, especially both halves of mutation 1.
- What `accum_matcher_op_census` now asserts, and the words you gave it.
- Anything else in the tree that pins or quotes 80,200.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Five consecutive strikes had their ★ be a
  false claim in a file the brief said to trust — three of those were the orchestrator's own
  artifacts. Assume there is a sixth.

Do not commit.
