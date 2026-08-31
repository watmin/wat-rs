# excursus 002 — the handle-lifetime wall

**What this is:** commissioned 2026-08-31 ("i say we do it"), but no arc was asked for, so it lives
here. `docs/excursus/` is the sibling of `docs/arc/`; see `docs/excursus/README.md`.

**The rule:** a `Peer` may not escape a scope that CREATES its service's `Handle`.

## Where it came from

`probe_arc278_self_scheduling` was `#[ignore]`d for **38 days** on a cause that was never measured
— the symptom `recv': peer closed` was reasoned into "the timer's `remove-at` is evicting the client
peer" and written into the ignore reason, then into a DESIGN's scout note, where it read as
measured. The timer was innocent: the fixture was releasing its own service, because its drive sat
in tail position and that ends the scope holding the handle.

That is fixed (`ca405009b`, both loci green), and the owner-drop now names itself at runtime —
`LociDiedError::Severed` (`5d803e407`). But the runtime notice is **measured racy**, 6/10 in the
tightest shape, so it can only ever be a backstop. A compile-time wall does not race. Hence this.

## The stones

| stone | what | state |
|---|---|---|
| 1 | **creation-scope escape** — a peer escaping a `let` (1a) or a function (1b) that created the handle | ✅ struck — `CheckErrorKind::HandleCreationEscape`, floor 5132 |
| 2 | **tail escape** — a peer leaving via a tail call | ◀ drawn, not struck |

## Two lessons this excursus has already paid for

**Rune the INSTRUMENT. Never rune the ACCEPTANCE CRITERION.** Stone 1 came back with a red floor
because the wall correctly rejected its own target, which I had placed under `wat-scripts/` where
the loader gate demands every file pass. The executor refused to silence it — *"a rune there would
make a green floor that fires on nothing"* — and was right. A gate that must construct the
forbidden state to measure it (`probe_severed_reaches_the_client`, `probe-self-sched-bisect`) earns
a rune; the red probe proving the wall fires never does.

**A must-be-rejected `.wat` lives in `probes/`, not `wat-scripts/`.** The convention was already in
the tree at `docs/arc/2026/06/278-rules-engine/probes/red-*.wat`. Stone 2's collision has the
opposite answer to stone 1's, though: the bisect probe is a program that RUNS, and a rejected file
cannot run, so it is runed rather than moved.

## Feasibility, probed before anything was drawn

- `wat-scripts/scratch-pad/probe-handle-to-surface-relation.wat` (`798188570`) — the checker CAN
  derive a service's surface types from a `Handle`, precisely enough to tell two services apart.
  Also settles where stone 1b lives: a `ReturnTypeMismatch` names a param type and a return type
  together, so both facts are co-present at `check.rs:1805`.
- `wat-scripts/scratch-pad/probe-tail-scope-sees-bindings.wat` (`e4ad7ee0d`) — the checker holds a
  let's binding types while inferring its tail expression, so stone 2 is buildable in principle.
  The same probe measured the severed notice losing its race 4 times in 10.

Both probes carry their acceptance criteria as live code: a function that must stop compiling
beside a near-identical one that must keep compiling.

## ⛔ One correction already recorded, because it is the trap

The first draft keyed the rule on **the parameter** — "param is a Handle, return is a Peer". That
rejects every `conn` helper in the corpus, including three in the stdlib. The discriminator is
**who created the handle**. `probe-handle-to-surface-relation.wat` carries the mislabeled function
under its corrected name with the reason inline, rather than being quietly renamed.

## Measured blast radius

18 functions return a `Peer`; **16 take an `Address`/`Handle` param and must keep compiling** (all
three stdlib `stdio-connect-*` among them); **2 create-and-escape and must be rejected**, both
deliberate probe targets. Re-run the census after any move; never take it from a report.
