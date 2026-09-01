# BRIEF — STONE: the registry answers FIRST, wave 2 — the named guards

Move seventeen stranded totality facts into their registrations, delete two guards that are pure
duplication, and leave the nine unregistered heads alone. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-registry-answers-first-wave-2.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` for a fast read; that binary does
NOT contain your Rust changes, which is what makes it the right tool for capturing BEFORE behaviour.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything.

## Read in order

1. The DESIGN above — the nineteen-verb census, and why the nine unregistered heads stay.
2. `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-registry-answers-first.md` — wave 1,
   **especially its "REFUTED" section**. Two things there change how you work: a stranded fact came
   back WRONG (`concat`), and the DESIGN's fence claim was false because of Law A (`is-rete`).
3. `wat-scripts/scratch-pad/255-the-registry-answers-first.wat` — wave 1's probe. Its header
   documents Law A and names `:wat::rete::total?` as the consumer that reads one axis in isolation.
   Your probe extends this one's shape.
4. `src/rete/purity.rs:246`–`:540` — `intrinsic_meta`, whole, before you cut anything from it.
5. `wat/runtime-meta.wat`, the `Totality` `defenum` — what `Total`/`Partial`/`Unreviewed` MEAN.

## The work

### 1 — move the seventeen facts IN

Each already carries `@Totality Unreviewed`. Replace it with the **re-derived** verdict plus
grounding prose saying why, from the body:

```
src/intrinsic/hashmap.rs:206  hashmap::keys        :233  hashmap::values
src/intrinsic/map.rs:172      map::keys            :190  map::values
src/intrinsic/reflect.rs:640  type-params-used-in  :745  type-equal?
src/intrinsic/stream.rs:91    stream::empty        :120  stream::cons
src/intrinsic/rete.rs:139     rete::pure?          :164  rete::deterministic?
src/intrinsic/rete.rs:190     rete::total?         :219  rete::primitive?
src/intrinsic/rete.rs:249     rete::vocabulary-admitted?   :308  cond-has-deferred-constraint?
src/intrinsic/rete.rs:407     rete::alpha-match    :436  -local    :464  -under
```

⛔ **RE-DERIVE FROM THE BODY. The guard is not evidence.** Wave 1 is the proof: `:wat::string::concat`
was asserted `Total` by a reasoned comment and came back `Partial`, because it is variadic and
`check.rs:14944` admits arity 0 so the runtime owns the raise. **Cite the line you read for each of
the seventeen.**

### 2 — delete the two duplicate guards

`:wat::uuid::v4` (`purity.rs:257–259`) and `:wat::stream::next` (`:408–410`) already say exactly what
their registrations say (`uuid.rs:57` → Pure/Nondeterministic; `stream.rs:177` → Effectful/
Nondeterministic). Delete both guards; edit neither registration. They are the stone's free proof
that the registry can carry the answer.

### 3 — delete the seventeen's guards

Remove the blocks at `purity.rs:283–288`, `:369–371`, `:379–381`, `:391–393`, `:429–439`, `:456–461`.
Leave a retirement comment at the cut in the shape wave 1 used.

### 4 — what must NOT change

- **No `@Purity` or `@Determinism` edit anywhere.** All nineteen already agree on those axes.
- **The nine unregistered heads keep their guards, untouched**: `:wat::core::aggregate-new` ·
  `kwargs-construct` · `write-forms` · `with-children` · `macro-error` · `:wat::verify::string` ·
  `http-path` · `s3-path` · `file-path`. They have no registration, so there is nowhere for a fact
  to go.
- **`rete_op_for` (`purity.rs:251`) is untouched.**

### 5 — the probe

`wat-scripts/scratch-pad/255-the-registry-answers-first-wave-2.wat`, extending wave 1's shape.
Capture BEFORE with the pre-existing binary and document it in the header, exactly as wave 1 did.
Cover: several of the seventeen via `:wat::rete::total?`; both duplicates (must not move); and a
control that stays `Unreviewed` (must stay `false`), so the probe can distinguish "the fact moved"
from "everything says true now".

⚠ A committed `.wat` must LOAD. Anything that panics at run is demonstrated out-of-tree and recorded
verbatim in the header.

## Blast radius

`src/rete/purity.rs` (six guard blocks + two duplicates deleted) · `src/intrinsic/{hashmap,map,
reflect,stream,rete}.rs` (seventeen `@Totality` lines) · the new probe. No body moves. No new
registrations. No `.wat` corpus change.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**STOP-1 — the reporters are on the list, and you are about to change what they report.**
`:wat::rete::total?`/`pure?`/`deterministic?` are themselves among the seventeen. Before you edit
them, capture their BEFORE answers with the pre-existing binary for every verb your probe touches.
If re-deriving one of the reporters gives anything other than what it declares today on
purity/determinism, STOP — a reporter that is wrong about itself invalidates the instrument.

**STOP-2 — a verdict you cannot re-derive is a STOP.** Neither "leave it `Unreviewed`" (silently
narrows every consumer) nor "guess `Total` to preserve behaviour" (the lie `Unreviewed` exists to
prevent) is available to you. Report which verb and what blocked you.

**STOP-3 — uniform confirmation is suspect.** Wave 1 ran 10-of-11. If all seventeen confirm their
guard, re-read the two you were least certain of and say in your report which those were and why you
are confident. A clean sweep after a non-uniform precedent is a claim, not a result.

**STOP-4 — the nine unregistered heads are not yours.** If one looks wrong, that is a finding to
report, never a line to change. Homing them is a different stone.

**STOP-5 — measure the fence, do not predict it.** Some of the seventeen are `:wat::rete::`-namespaced
and may clear Law A where wave 1's string verbs could not. **Whether any `where`/`then` behaviour
changes is a MEASUREMENT you take with the pre-existing binary and report** — the DESIGN deliberately
does not predict it, because wave 1's DESIGN predicted exactly this and was wrong.

## Report

Per-file diff summary; the seventeen verdicts **each with the line you read for it**; which (if any)
overturned its guard; the BEFORE/AFTER the probe captured; what STOP-5's fence measurement showed;
and the part the orchestrator cannot reconstruct: **what surprised you** — a body that contradicted
its guard, a reporter that was wrong about itself, or a consumer the brief did not name.
