# Realizations — slice 1 review (2026-05-09)

## What surfaced

Slice 1 of arc 170 (commit `787c977` + SCORE `bb155ed`) shipped a
working closure-extraction algorithm — 14/14 scorecard rows pass,
Mode A clean, 2108/0 verified locally. The substrate primitive is
sound: free-symbol walker, dep-closure builder, portability
check, topological sort all correct.

But the **public shape of `ClosurePackage` carries the
entry-keyword ceremony DESIGN explicitly killed.**

```rust
// Slice 1 shipped:
pub struct ClosurePackage {
    pub forms: Vec<WatAST>,
    pub entry: String,  // ← the ceremony DESIGN settled to retire
}
```

For inline-lambda input, slice 1 synthesizes
`:__closure::__pkg_<n>`, wraps the fn AST in
`(:wat::core::define :__closure::__pkg_<n> (fn ...))`, appends to
`forms`, exposes the synthetic name as `entry: String`. The
consumer (spawn-process Rust) then looks up that synthetic name
in the frozen world and applies.

**This contradicts the DESIGN-conversation settlement** (DESIGN.md
lines 102-108, 484-509):

> 5. The "name discovery" path (substrate looks up a canonical
>    entry symbol) creates ceremony. The user's preference: **the
>    fn IS the program**; pass it directly; substrate handles
>    closure extraction internally.

> 16. User questioned why entry-keyword is needed: *"why do we
>     even need a name if the forms /are/ the thing that
>     matters?"*

> 17. User refined further: the fn IS the program; spawn-process
>     takes fn directly; no Program wrapper type; closure
>     extraction is internal

The DESIGN killed the entry-keyword at the wat surface. Slice 1
re-introduced it one layer down at the Rust public-API surface.
Same ceremony, different layer.

## Why the deficiency wasn't caught in scoring

The scorecard rows in EXPECTATIONS-SLICE-1.md verified:
- Module + types minted (A)
- Subsystems implemented correctly (B-F)
- Tests pass (G)
- Workspace clean (H)
- No surface added at wat level (I)
- Branch isolation (L)
- Zero Mutex (M)
- Diagnostic UX (N)

What was MISSING from the scorecard: a **DESIGN-intent alignment
row.** A row that asks: *"Does the public shape of this
substrate primitive honor the DESIGN's settled architectural
intent?"* — not just the BRIEF's spec, but the DESIGN's spirit.

The agent shipped exactly what the BRIEF specified (synthetic
name + entry field, per CLOSURE-EXTRACTION.md). The BRIEF was
correct relative to its own spec. **The spec itself was wrong
relative to DESIGN.** The orchestrator (me) drafted the BRIEF
without recognizing that the synthetic-name approach
contradicted the conversation log captured in DESIGN.md lines
484-509.

The agent did its job. The orchestrator's BRIEF was the upstream
defect.

## The honest shape

The fn-form `(fn [stdin :IOReader stdout :IOWriter stderr :IOWriter] :nil ...)`
already evaluates to a fn Value. The substrate's evaluator turns
fn-forms into fn Values directly. We don't need to wrap in a
define + look up by name; we can keep the entry as a fn-form
expression.

```rust
pub struct ClosurePackage {
    pub prologue: Vec<WatAST>,  // type defs + dep defs (the captured environment)
    pub entry_form: WatAST,     // an expression evaluating to a fn Value:
                                //   - inline-lambda input: the fn-form AST itself
                                //   - keyword-path input:  a Symbol AST that
                                //     resolves into prologue's defines
}
```

Consumer (spawn-process Rust):

```rust
let pkg = extract_closure(&fn_value, sym, &types)?;
let frozen = startup_from_forms(pkg.prologue, ...)?;
let fn_value = eval(&pkg.entry_form, env, frozen.symbols())?;
let result = apply_function(fn_value, args, frozen.symbols(), span)?;
```

No synthetic name. No `entry: String`. The fn IS the program at
the structural level too.

## What needs to ship

Slice 1b — structural reshape:

1. **`closure_extract.rs`**:
   - `ClosurePackage` becomes `{ prologue, entry_form }`
   - Synthetic-name machinery (`__closure::__pkg_<n>` counter,
     wrap-in-define logic) removed
   - For inline-lambda input: emit the fn-form AST as
     `entry_form`; do not wrap or name it
   - For keyword-path input: emit the symbol AST as `entry_form`;
     prologue includes the user's existing define for that symbol
   - Topological sort: types → captures → user deps (NO
     trailing entry define — the entry is `entry_form`, not in
     `prologue`)

2. **Tests `tests/wat_arc170_closure_extraction.rs`**:
   - T1-T15 assertions update to the new shape
   - Regression: re-freezing prologue + evaluating entry_form
     produces a fn Value with behavior equivalent to the
     original input fn

3. **CLOSURE-EXTRACTION.md** amendment:
   - Steps 1, 6 reshape (entry resolution + assembly)
   - Synthetic-name section retired
   - Invariants update (I2 retires; new invariant for
     entry_form evaluating to a fn Value)
   - Test cases update to assert prologue + entry_form roundtrip

4. **DESIGN.md slice plan** amendment:
   - Insert slice 1b between slices 1 and 2
   - Slice 2 explicitly depends on slice 1b's reshape

## What does NOT change

- The closure-extraction algorithm (free-symbol walker, dep
  closure, portability check, Value→AST encoder for captures)
- Honest deltas A through F from SCORE-SLICE-1 still apply:
  - Q-impl-2 captured-fn-value gap (closures-of-closures)
  - Value-kind encoding gaps
  - Diagnostic type-name spelling
  - Topological sort edge tracking
  - Auto-accessor short-circuit
- SCORE-SLICE-1.md (immutable historical record per
  `feedback_inscription_immutable.md`)
- The arc 170 client/server framing
- The spawn primitive surface (`(:wat::kernel::spawn-process fn)`)

## Discipline lesson — for future BRIEFs

Add to EXPECTATIONS scorecards a row of the form:

> **DESIGN-intent alignment** — does the shipped public shape
> honor the DESIGN's settled architectural intent (not just the
> BRIEF's literal spec)? If the BRIEF's spec drifted from
> DESIGN, surface as honest delta and STOP.

This catches BRIEFs that drift from the DESIGN they're
supposedly implementing. The orchestrator drafts the BRIEF; the
scorecard is the verification mechanism that the BRIEF didn't
silently quietly diverge.

Candidate addition to `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6
as a new failure mode (FM 17): **BRIEF spec drifts from DESIGN
intent without scorecard catching it.** Worked example: arc 170
slice 1 (this realization).

## Cross-references

- DESIGN.md lines 102-108 (the settled "fn IS the program"
  framing)
- DESIGN.md lines 484-509 (DESIGN-time conversation log)
- SCORE-SLICE-1.md (immutable; documents 14/14 pass against the
  insufficient scorecard)
- CLOSURE-EXTRACTION.md (the spec doc that carried the
  synthetic-name approach; gets amended for slice 1b)
- `feedback_attack_foundation_cracks.md` — fix the crack now,
  before slice 2 leans on the wrong shape
- `feedback_inscription_immutable.md` — SCORE stays as-is; fix
  ships forward
- `feedback_no_known_defect_left_unfixed.md` — bias is "ship
  everything we know how to do," not "ship the smaller win"

---

## Addendum 2026-05-09 — tier framework alignment

A second alignment surfaced the same day in the same review
thread. After agreeing to fix the entry-keyword ceremony, the
conversation pressure-tested the broader pattern.

**First framing attempt** (the orchestrator's): "this is a hermetic
package primitive that generalizes across thread / process / remote."
Wrong-shape — over-emphasized hermetic as a thing-in-itself.

**Second framing attempt** (also orchestrator's): four properties
of the hermetic seal — memory / signal / global-state / runtime-
sealing isolation. Closer, but still wrong shape — these "four
properties" are one property manifesting four ways, not four
separate things.

**The settled framing** (user direction): tiers are the primary
structural concept. Hermeticness is the ambient consequence of
tier ≥ 2. There are four tiers:

- Tier 0 — runtime env (call stack; same eval context)
- Tier 1 — threads (memory shared)
- Tier 2 — processes (host shared, memory boundary)
- Tier 3 — remote programs (network shared, host boundary)

Hermetic = "tier ≥ 2." Not a flag; not a label; what the OS-process
boundary inherently provides. The "four properties" all manifest
because tier ≥ 2 means a separate OS process; one boundary; one
seal.

The tier-bridging primitive (closure extraction package) only
matters at tier ≥ 2 — tier 0 and tier 1 use the fn Value
directly.

User quote, captured as the framing's load-bearing text:

> *"the entire concept is hermetic in nature... threads don't get
> hermetic, just a shared space to run in. processes and remote
> programs are hermetic by nature.... but the interface here...
> its the same, but the 'runtime env' has different properties...
> a thread shares memory, a process shares the host, a remote
> program shares the network"*

> *"tier2 and tier3 are hermetic by the boundary of what's shared...
> not something explicit, just an ambient property of the runtime"*

This framing is captured in [`TIERS.md`](./TIERS.md). It's
load-bearing for arc 170 and any future arc that touches the
spawn family.

### Discipline lesson — for orchestrator framing reflexes

Five framing passes in one conversation thread:

1. Wrong-shape (entry-keyword ceremony at Rust API level)
2. Wrong-shape (hermetic as primary subject)
3. Right-shape (tiers as primary; hermetic as ambient)
4. Wrong-shape (under-scoped slice 3; "future arc" framing on
   hermetic.wat rebuild)
5. Wrong-shape (substrate-level types — IOReader / IOWriter /
   Vec\<String\> stdin / scope — exposed in user-facing
   interfaces; user must work in forms not strings)

Each pass was an orchestrator reach for the wrong word. The user
caught each one through the same probe: *"do you actually know
this, or are you assuming?"* (per `feedback_assertion_demands_evidence.md`)
+ *"what is X masking?"* (per FM 10's self-probe).

The pattern: when the substrate has a clear structural concept
(tiers) AND a derivative property of that structure (hermetic at
tier ≥ 2), the orchestrator's reach is for the derivative property
as if it were primary. That's wrong-shape. The structure is
primary; the property emerges.

Add to recovery doc § 6 candidate failure mode (FM 18 candidate):
**Reaching for derivative property as primary frame.** When the
substrate has a structural concept and an emergent property of
that structure, the right doc-architecture is structure-primary +
property-named-as-emergent. Reaching for the property as primary
is FM-10-adjacent (type-theoretic reach when entity-kind is the
answer) but at a higher level — frame-theoretic reach when
structural-concept is the answer.

### Pass 4 — under-scoping reflex

User caught me marking the hermetic.wat rebuild as "Future arc,
not arc 170" when it absolutely IS arc 170's scope.

User direction:

> *"these arcs cover whatever amount of work is necessary - they
> do not have a defined 'limited scope of work' they are defined
> as 'we have a change to the substrate, we deal with whatever
> implications come from it'"*

> *"arc 109 is making our lang's UX outstandingly good"*

> *"working isn't a polished state"*

The arc-scope doctrine: when the substrate's contract changes,
EVERY existing user-facing thing that interfaces with that
contract must reach its CORRECT polished form on the new
substrate. Not "still works." Not "minimal mechanical update."
**As good as the new substrate allows.**

The arc covers all implications. Splitting "make it work" from
"make it polished" into separate arcs is wrong — the polish IS
the arc's deliverable.

This is the same shape of failure as `feedback_pivot_not_defer.md`:
marking known work as "future arc" when it actually belongs in
the current arc. The reflex is "scope this down to ship faster";
the doctrine is "polish is the bar; arc covers all implications."

Doc updates from this pass:

- TIERS.md — dropped "Future arc, not arc 170" framing on the
  hermetic.wat rebuild; in-scope for slice 3
- DESIGN.md slice 3 — expanded from "mechanical sweep" to
  "consumer sweep + tooling rebuild to polished form"; explicit
  hermetic.wat call-out

Candidate FM 19 (recovery doc § 6): **Under-scoping reflex —
marking polish as future arc.** STOP signal: writing "future arc,
not arc N" or "out of scope; later arc handles it" while the
current arc is the arc that changes the substrate that thing
depends on. The arc covers all implications of its substrate
change; "we'll polish later" is FM-11-adjacent (deferral as
done).

### Pass 6 — bandaid bounded by arc close

Slice 1c shipped Process<I,O> ADDITIVE (legacy 4 fields stay;
typed-channel 2 fields appended) instead of destructively
reshaping to the final 3-field form. The agent's reasoning:
destructive reshape would brick stdlib (sandbox.wat backs every
deftest); additive ships green and unblocks slice 2.

User direction (2026-05-09):

> *"the bandaid is tolerable as long as its short term - the arc
> cannot be closed with bandaids. if the bandaids reduce the
> friction to deliver correctness they are justified - they
> cannot persist beyond the arc"*

> *"i fully intend to break shit all over and mass fix it. half
> measures from 'i don't want to break things' is a behavior i
> do not tolerate."*

> *"we use sonnet to do mass fixes and opus to land the platform
> that enforces some new correct behavior"*

The settled principle:

- **Bandaids are tolerable DURING arc work** when they reduce
  friction-to-deliver-correctness (slice 1c additive Process
  kept slice 2 unblocked while testing tooling rebuild slated
  for slice 3)
- **Bandaids CANNOT persist past arc close** (slice 5 INSCRIPTION
  must reflect the final correct shape; FM 11 "INSCRIPTION =
  DONE" forbids deferral language)
- **Slice 4 (substrate retirement) is the bandaid-retirement
  slice** — every bandaid carried during sweep window
  destructively retires before INSCRIPTION
- **Atomic-commit pattern for destructive substrate work**:
  opus lands new correctness (don't commit); sonnet mass-sweeps
  consumers (don't commit); orchestrator commits both as ONE
  atomic commit when workspace = 0-failed (recovery doc § 7
  atomic-commit pattern)
- **Opus lands platform; sonnet does mechanical mass-fixes** —
  not because opus can't sweep but because the labor split is
  honest about the work shapes (judgment vs mechanical)

The orchestrator failure: my BRIEF-SLICE-1C row G offered the
shim option ("investigate + pick: break-as-substrate-as-teacher
OR shim-with-warn"). Offering the shim path enabled the
silent-additive bandaid. Future BRIEFs for substrate-shape
changes during sweep windows: name the bandaid as TEMPORARY;
explicitly slate the bandaid retirement in the slice-4-equivalent;
require the slice plan to enforce arc close = bandaid free.

DESIGN.md slice 4 amended (this commit) to explicitly include
Process<I,O> legacy 3-field retirement. Bandaid is now bounded
by arc 170 close.

### Discipline lesson candidate FM 21 — bandaids must be bounded

When a substrate-shape change ripples wide enough to brick
stdlib bootstrap or block subsequent slices, an additive
intermediate ("bandaid") is acceptable IF AND ONLY IF the
slice plan explicitly slates the bandaid retirement before
arc close.

STOP signals — phrases that mean you're about to fail this:

- "future arc retires X" while X is the arc's bandaid
- "later" without a named slice
- offering "additive OR destructive" as caller choice without
  bounding the additive option

DO this instead:

- Bandaid ships in slice N (sweep window)
- Slice N+M (substrate retirement, before closure paperwork)
  destructively retires the bandaid
- INSCRIPTION at slice closure reflects the final clean shape
- Atomic-commit pattern (opus → sonnet → bundled commit) for
  destructive substrate transitions

Connects to:
- FM 11 (INSCRIPTION = DONE; no deferral language)
- `feedback_pivot_not_defer.md` (don't write "future arc")
- recovery doc § 7 atomic-commit pattern

### Pass 5 — strings stay at the substrate boundary, not in user-facing interfaces

User direction:

> *"we should further hide away string conveyance - we will /only/
> ever transmit edn... WatAST serializes to a string by its
> nature... we should only ever expose an interface where forms
> are moved around.. the fact that they need to be strings at some
> point (process boundary, network boundary) is the runtime's
> concern - not the user. the user continues working in wat
> natively"*

I had drafted Layer 2 of the testing API as:

```scheme
(:wat::test::run-hermetic-with-io
  (fn [stdin :wat::io::IOReader stdout :wat::io::IOWriter stderr :wat::io::IOWriter] :nil
    ...)
  "input bytes")
```

— which still leaks substrate-level types (IOReader, IOWriter)
and substrate-level concepts (raw byte stdio) into the user's
view. Wrong-shape.

The settled doctrine: **the user works in forms at every tier;
strings are the substrate's transport detail.** The polished
abstraction is uniform across tiers:

| Tier | User-visible IPC | Substrate transport |
|---|---|---|
| 1 — threads | `Sender<T>` / `Receiver<T>` | crossbeam (in-memory; no encoding) |
| 2 — processes | `Sender<T>` / `Receiver<T>` | EDN-over-pipes (substrate encodes/decodes) |
| 3 — remote | `Sender<T>` / `Receiver<T>` (Q-channel) | EDN-over-sockets |

Same shape at every tier. WatAST serializes to EDN by nature; the
substrate handles encoding at the pipe/socket boundary; users
never see strings flowing through these channels.

The OS-boundary exception: `:user::main`'s stdin/stdout/stderr
stay `IOReader`/`IOWriter` (the OS shell speaks bytes; we can't
pretend otherwise). argv stays `:Vector<String>`. This is the
ONE place strings remain user-visible — because it's where wat
meets the OS.

**This is a substrate-shape change beyond what slice 2's BRIEF
currently spec'd.** Slice 2 is already frozen at v1-shape (per
FM 6) waiting on slice 1b; it now ALSO needs:

- `:user::process` contract change: from
  `(stdin :IOReader stdout :IOWriter stderr :IOWriter) -> :nil`
  to `(rx :Receiver<I> tx :Sender<O>) -> :nil` — same shape as
  `:user::thread`
- `:wat::kernel::Process<I,O>` struct shape change: byte-pipe
  handles (stdin/stdout/stderr IOReader/IOWriter) drop;
  typed-channel handles (tx :Sender<I> + rx :Receiver<O> +
  ProgramHandle) replace them
- spawn-process implementation creates EDN-over-pipe channels
  internally; the byte-pipe + EDN-encoder/decoder pair is
  substrate-internal plumbing

Slice 2 will be redrafted (post-slice-1b) with these typed-channel
+ form-only changes layered onto the closure-extraction reshape.

### Discipline lesson candidate FM 20 — strings as substrate-leakage

When designing user-facing interfaces, **strings (and the byte
streams they imply) are substrate-level types that should not
appear at the user's level unless the user is genuinely AT the
OS boundary.** Wat-internal communication is form-shaped; the
substrate's transport (bytes over pipes/sockets/files) is its
own concern.

STOP signal — phrases that mean you're about to fail this:

- "the user passes a `Vec<String>` for stdin..."
- "the fn takes `(stdin :IOReader stdout :IOWriter stderr :IOWriter)`..."
- "scope :Option\<String\>"
- "we drain the output to `Vec<String>`"

When these surface in a user-facing interface design, ask: is
the user genuinely at the OS boundary? If yes (`:user::main`,
wat-cli's argv), strings stay. If no (any wat-internal spawn,
test harness, IPC), the substrate handles serialization; the
user works in typed Values.

Connects to memory `project_pipe_protocol.md` ("line-delimited
EDN + kernel pipes. One protocol; four transports.") — the
protocol is EDN; the transports vary; the user-visible
abstraction stays form-shaped.
