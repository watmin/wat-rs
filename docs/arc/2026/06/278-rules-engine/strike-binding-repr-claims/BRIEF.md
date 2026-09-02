# BRIEF — make each binding-repr arm say what it measures, and make the floor assertion real

One benchmark file prices a representation decision with arms the engine does not use, anchors them
to a figure whose source now measures negative, and guards it all with an assertion that cannot fail.
Keep every arm; make every claim true.

## Read in order

1. `src/rete/kernel/tests/binding_repr_bench.rs:663-668` — **the ★**. The comment says *"a zero here
   means it timed nothing"*; the assertion is `extend_array_wins + get_array_wins < usize::MAX`.
2. `src/rete/kernel/session.rs:64-70` — `Token { matches: BindSpan, binds: BindSpan }`. **This is the
   live representation**, and it is neither arm the dominance test compares.
3. `src/rete/kernel/fire/delta.rs:725-726` — the premise the comparison is evidence *for*: a binding
   map holds 1-2 entries, so a trie pays trie prices. Keep this connection; it is why the test earns
   its place.
4. `src/rete/kernel/tests/binding_repr_bench.rs:544-560` — the dominance test's setup and the two
   arms (`bindings_extend_array` at `:522`, its trie twin above it).
5. `src/rete/kernel/tests/binding_repr_bench.rs:10-22` and `:85` — the `163 ns` prose (four mentions)
   and the printed line *"of the ~163 ns in-engine bind"*.
6. `src/rete/kernel/tests/rank_and_instrument.rs:219` — `alpha_match_cost_per_binding`, the named
   source of that figure. Drive it: it reports **−22 ns/fact** at HEAD.
7. `src/rete/matcher.rs:481,498,516` and `src/rete/compiled_cond.rs:912,1430` — the trace showing the
   matcher eval path is reached only from tests, and the note that its replacement took over.

## The three pieces

1. **Make the assertion the one its comment declares.** `> 0`, plus the orderings that must hold, with
   the whole table interpolated so a red carries its own evidence.
2. **Name the live representation.** Both tests gain a header sentence: the engine uses `BindSpan`
   (`session.rs:64`), neither arm here is that, and this comparison is the evidence behind the stone
   at `fire/delta.rs:725-726`.
3. **Drop the `163 ns` anchor** from the prose and from the printed table, and say why: its source
   measures −22 ns/fact, below the harness's resolution. The header already says to treat the ratio
   as the finding — make the output obey it.

## Blast radius

`src/rete/kernel/tests/binding_repr_bench.rs` only. Nothing under `src/rete/` proper.

## STOP triggers — halt and report

1. **If `extend_array_wins + get_array_wins` is actually 0** once you assert `> 0`, stop and report.
   That means the probe times nothing today and the fix is not a one-line assertion.
2. **If you find yourself deleting either test, or the retired matcher path**, stop. Both are evidence
   or oracle; reaping the matcher is tracked separately as C13.
3. **If re-driving `alpha_match_cost_per_binding` gives a stable positive ns/fact**, stop and report
   the samples — then the figure is measurable after all and dropping it is the wrong call.
4. **If naming `BindSpan` requires reading anything under `src/rete/kernel/fire/`**, that is fine to
   read — but any *edit* there means stop.

## Mutation proofs — run both, report both

1. **Force `extend_array_wins = 0` and `get_array_wins = 0`** before the assertion → it must go RED.
   Proves the new assertion is the non-vacuity check, not another tautology.
2. **Break one ordering you assert** (whichever you choose) → RED. Proves the orderings are load-bearing
   and not decoration.

Restore after each.

## What to report

- Both tests' output before and after, verbatim.
- Your `alpha_match_cost_per_binding` samples.
- Both mutation results.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.
- **Anywhere this brief was thin, wrong, or pointed at the wrong line.** The last four riders each
  found a real defect in the brief — including, last time, a figure I had read off the wrong row of a
  three-block table, and a premise that was simply false. Be blunt.

Do not commit. Leave the work in the tree and report.
