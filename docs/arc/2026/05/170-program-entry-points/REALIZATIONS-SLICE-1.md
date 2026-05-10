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

Nine framing passes across two days:

1. Wrong-shape (entry-keyword ceremony at Rust API level)
2. Wrong-shape (hermetic as primary subject)
3. Right-shape (tiers as primary; hermetic as ambient)
4. Wrong-shape (under-scoped slice 3; "future arc" framing on
   hermetic.wat rebuild)
5. Wrong-shape (substrate-level types — IOReader / IOWriter /
   Vec\<String\> stdin / scope — exposed in user-facing
   interfaces; user must work in forms not strings)
6. Discipline (bandaid-bounded by arc close)
7. Architecture (ambient runtime model; drop stdio from
   :user::main; mint :wat::runtime::*)
8. Architecture (Server/Client wat-level abstractions; canonical
   server form)
9. Architecture (three substrate services; single-shot panic;
   structured-stderr-only doctrine)

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

### Pass 7 — Ambient runtime (drop stdio params from `:user::main` and `:user::process`)

User direction (2026-05-09 → 2026-05-10):

> *"do we implement (:wat::kernel::panic! ...) in rust or wat...
> if in wat... do we need to have [:wat::runtime::stdin,
> :wat::runtime::stdout, :wat::runtime::stderr] if yes... it
> means we can drop the values from being a required param for
> :user::main and :user::process to accept... we provide them
> via rust... good programs don't use them..."*

The four-questions analysis (run on the ambient runtime model):

- **Obvious?** YES — `:user::main [] -> :wat::kernel::ExitCode`
  reads as "entry point that returns an exit code"; ambient
  `:wat::runtime::*` reads as "the runtime's handles."
- **Simple?** YES — atomic pieces: mint ambient values; drop
  stdio params; update wat-cli/spawn-process; sweep callsites.
- **Honest?** YES — fd 0/1/2 always exist for any POSIX process;
  the param-as-pretend-handed-in shape was lipstick over
  substrate-internal access. Acknowledging the kernel reality
  is more honest than theatrical signature padding.
- **Good UX?** YES — programs that don't use stdio carry no
  ceremony; programs that do reach for ambient handles directly
  from any depth; one canonical path.

Comparing to KEEPING current 4-arg `:user::main`:

| Question | Ambient | Keep |
|---|---|---|
| Obvious | YES | LESS |
| Simple | YES | NO (3 unused params) |
| Honest | YES | NO (lipstick) |
| Good UX | YES | NO (signature pollution) |

Locks in:
- `:wat::runtime::current-thread` ambient — thread-local id
- `:wat::runtime::argv` ambient — set-once at process start
- `:user::main` `[] -> :wat::kernel::ExitCode` (drop stdio params + argv from signature)
- `:user::process` retires; replaced by Server pattern (see Pass 8)

Per `project_wat_llm_first_design.md` ("reject synonym
features; force naming; one canonical path") — ambient with
explicit `:wat::runtime::*` namespace IS the canonical path;
the `:user::main` 4-arg shape was a synonym for ambient access.

### Pass 8 — Server / Client wat-level abstractions; the canonical form

User framing:

> *"that child needs to make their console-ish service... we
> need a name for this... the current console service doesn't
> do stdin reading... or... it kinda does?... that is the
> program pattern?... the current console uses crossbeam pipes...
> the child program should build a service that uses the stdin
> pipe instead of the crossbeam... that is how the parent (the
> client) talks to the child (the server)... that server program
> could make its own threaded workers who do stuff... that child
> should operate like a good wat program and hide its pipes...
> so we need a :wat::kernel::Server or something... the thing
> who invokes that is considered to be a :wat::kernel::Client"*

The pattern: Server runs in spawned context (thread/process/
remote); processes typed requests; produces typed responses.
Client is the handle the spawning context holds; provides typed
send/recv interface. Transport-polymorphic across tier 1/2/3.

The canonical wat server-program form (captured as memory
`project_arc_170_canonical_server_form.md` per user direction
2026-05-10 "we must not forget this"):

```scheme
(:wat::core::defn :user::main [] -> :wat::kernel::ExitCode
  (:wat::core::let
    [client (:wat::kernel::StdInService/connect)]
    (:wat::kernel::server-loop client my-handler)))

(:wat::core::defn :wat::kernel::server-loop<I,O>
  [client    <- :wat::kernel::Client<I,O>
   handler   <- :wat::core::fn(I)->O]
  -> :wat::kernel::ExitCode
  (:wat::core::match (:wat::kernel::Client/recv client)
    -> :wat::kernel::ExitCode
    ((:wat::core::Some req)
      (:wat::core::let [resp (handler req)]
        (:wat::kernel::StdOutService/send resp)
        (:wat::kernel::server-loop client handler)))
    (:wat::core::None
      (:wat::kernel::ExitCode 0))))
```

Slice 1c's PipeFd Sender/Receiver substrate becomes the
INTERNAL implementation of how Server/Client serialize across
OS-pipe boundaries. wat-level user code never sees Sender /
Receiver in fn signatures; they see Server / Client.

`:user::process` contract retires entirely. There's only:
- `:user::main` `[] -> :ExitCode` for OS-boundary CLI entry
- `Server/run handler` for service-loop pattern (called inside
  `:user::main` OR as the spawn-process child's body)

Today's `:wat::console::Console` (crossbeam-based) becomes the
**tier-1 instance** of the same pattern. Same abstraction;
transport-polymorphic.

User confirmation (2026-05-10):

> *"that is an incredible wat expression - we must not forget
> this... this form is incredible... this arc grows into
> something remarkable now... this is the point of 109... we
> are making the language outstanding..."*

**The canonical form is what users UNDERSTAND. The helper is
what users WRITE.**

Per `project_wat_llm_first_design.md` ("one canonical path per
task; reject synonym features"), the typical user program is
3 lines:

```scheme
;; my-server.wat
(:wat::core::load! "some-lib.wat")  ;; brings in :my::handler

(:wat::kernel::main! :my::handler)  ;; expands to canonical pattern
```

`:wat::kernel::main!` is a substrate-auto-loaded defmacro that
expands to the full canonical server-loop form above. The full
form remains visible (users CAN write it explicitly when they
need to deviate), but the macro is what programs reach for by
default.

User direction (2026-05-10):

> *"should we provide a helper form... that main pattern.. we
> should encourage it..."*

A complementary `:wat::kernel::run!` macro handles the CLI
utility (one-shot) case for programs that don't run a service
loop. It's variadic — wraps the forms in an implicit-do; the
last form is the ExitCode value. Both helpers live in substrate-
auto-loaded stdlib (no explicit `load!` needed; same pattern as
`:wat::core::defn`).

`main!` accepts any expression evaluating to a handler fn —
keyword path, inline fn-form, or factory call. User direction
(2026-05-10):

> *"(:wat::kernel::main! some-fn) ;; :wat::core::fn(I)->O — the
> user my pass a fn here.. they can do so using a function call .."*

> *"(load! 'some-lib.wat'); (:wat::kernel::main! (make-client))"*

The factory pattern is the polished idiom. Library defines
`make-handler` (returns a fn closure with config baked in); user
program calls it once at startup; macro evaluates the call;
server-loop drives the resulting fn. Three lines of user code,
arbitrary handler complexity behind the factory.

### Pass 9 — Three substrate services: structured-stderr-only + single-shot panic

User direction (2026-05-10) — the architecture locks in:

> *"we do the same thing for stdout... console is completely
> reimagined now... it an always on service with the same
> behavior as panic service... they get new names... the user's
> program always boots up with 2 threads runnning... each
> guarding stdout and stderr respectively... :wat::kernel::StdOutService
> and :wat::kernel::StdErrService. this also means we need a
> third thread... :wat::kernel::StdInService"*

> *"we only write structured STDERR - never anything else... the
> only non structured STDERR is going to be cargo doing tests..
> anything in wat land is structured STDERR + exit code"*

> *"when users call (:wat::runtime::panic! ...) it does the
> blocking, it doesn't return until an ack is delivered which
> means concurrent panics are resolved... the thread pool is
> guarded by the server's io loop... they never get a chance to
> be processed as we blow up after processing the first panic
> completely"*

Locks in:

**Three substrate services boot before any user code:**
- `:wat::kernel::StdInService` — owns fd 0; reads bytes;
  decodes EDN line-by-line; serves typed Values to consumers;
  returns `:None` when fd 0 closes.
- `:wat::kernel::StdOutService` — owns fd 1; receives typed
  Values from per-thread message-pipes; serializes EDN; writes
  to fd 1; single-writer guard.
- `:wat::kernel::StdErrService` — owns fd 2; first panic event
  drained wins; serializes structured cascade EDN; writes to
  fd 2; calls `libc::exit(N)`; process dies.

Each service's loop selects over per-thread input pipes +
control-pipe (self-pipe trick for thread-list updates).

**Doctrines:**

- **Structured-stderr-only.** Inside wat-land, fd 2 ONLY ever
  carries structured panic-cascade EDN. No "regular text" on
  stderr. wat-cli has zero direct stderr writes (load failures,
  freeze errors, etc. all route through StdErrService → cascade
  + exit). Pretty-printing is downstream (shell user pipes
  through formatter if they want); substrate is honest.
- **Single-shot panic.** `(:wat::runtime::panic! ...)` blocks;
  thread sends panic event to its registered StdErrService pipe;
  service drains; emits cascade; calls `libc::exit(N)`; process
  dies. Concurrent panickers in other threads are queued at
  their pipes but never drained — process dies after first
  panic. Other threads die with the process. No multiplexing
  multiple panics; no escape paths.
- **Console retires.** Today's `:wat::console::Console`
  (crossbeam-based; arc 109 slice K.console) was a wat-level
  service for in-thread output mediation. The substrate now
  provides this for free via StdOutService. Console-the-concept
  dies; tests using it migrate to StdOutService.
- **`:wat::runtime::stdin/stdout/stderr` ambient handles
  RETIRE.** They were a midpoint in pass 7's design. Users
  interact with the three services; the ambient runtime stays
  as a CONCEPT (always-available) but the user-facing surface
  IS the services. `:wat::runtime::current-thread` and
  `:wat::runtime::argv` survive (they're values, not services).

**spawn-thread register-with-services contract:**

When the substrate spawns a thread (via spawn-thread or other
mechanism), the thread MUST register with all three services
BEFORE returning a handle to caller:

1. Substrate creates per-thread pipes for StdIn (consumer-side),
   StdOut (writer-side), StdErr (panic-emit-side)
2. Substrate sends `:register thread-id reader-end` to each
   service's control-pipe
3. Each service ack's via the control-pipe's response channel
4. Substrate stores per-thread pipe writers in thread-locals
5. ONLY THEN return Thread<I,O> handle to caller

Without ack-before-return, races possible — the new thread
might panic before services know about it; panic dropped.

**The architectural payoff:**

Every wat process boots with 4 threads minimum: main + 3
services. Every shared mutable resource (stdout, stderr, stdin,
+ future caches/databases/etc.) is guarded by a service per
the program-with-mailbox tier of `feedback_zero_mutex.md`.

The user writes intent (handler logic). Substrate provides
everything else (services, registration, panic routing,
exit-code propagation, structured emit). The canonical server
form expresses this in 12 lines.

This IS what arc 109 was building toward. Per the user:

> *"this arc grows into something remarkable now... this is the
> point of 109... we are making the language outstanding..."*

### Implications for arc 170 slice plan

The substrate refactor is substantial:

```
Already shipped/in-flight (mostly stays valid):
  Slice 1   ✓ closure extraction primitive
  Slice 1b  ✓ ClosurePackage reshape
  Slice 1c  ✓ typed-channel substrate (becomes StdIn/Out service internals)
  Slice 1d  ✓ closure-extraction walker fixes
  Slice 2   ✓ initial wat-level surface (will be revised)
  Slice 3   ⊘ phase A + B + 1d uncommitted; needs revision against new doctrine

New work:
  Slice 1e  Ambient `:wat::runtime::*` (current-thread + argv);
            retire `:wat::runtime::stdin/stdout/stderr`;
            drop stdio params from `:user::main` signature;
            wat-cli no direct stderr writes
  Slice 1f  StdInService / StdOutService / StdErrService substrate
            (Rust runtime components; always-on; per-process;
            select-loops; control-pipe; self-pipe trick)
  Slice 1g  spawn-thread register-with-services contract
            (ack-before-return; per-thread pipes; thread-locals)
  Slice 1h  `:wat::kernel::Server` / `Client` wat-level
            abstractions on top of services + slice 1c PipeFd
  Slice 1i  panic-cascade emit via StdErrService (replace slice 2
            "panic: spawn-process body panicked" marker; use full
            arc 113 cascade structure)

Foundation cleanup:
  Slice 3 (revised)  Sweep all tests + retire Console + migrate
                     to services + canonical server form
  Slice 4   Bandaid retirement (Process<I,O> legacy fields,
            walker bodies, legacy dispatch arms, slice 1c PipeFd
            substrate retires if unused at wat level)
  Slice 5   Closure paperwork → INSCRIPTION
```

Per the bandaid-bounded discipline (pass 6) — slice 4 retires
EVERY bandaid carried during sweep windows. Arc 170 closes
with no deferral language; the canonical server form is the
inscribed truth.

### Discipline lesson candidates

**FM 22 (recovery doc § 6 future): When in design-mode, name
the question; ask the user; don't pre-commit.**

This conversation was 15 framing passes across two days. Many
of them were the orchestrator's wrong-shape proposals
corrected by user direction. The pattern: orchestrator drafts;
user pushes back via the four questions or substrate-doctrine
probe; orchestrator pivots.

The user's tools (probes I should use on myself):
- "What does this MASK?"
- "Do I KNOW this or assume?"
- "Why am I using THIS word?"
- "Did we already have X (or part of X) somewhere?"
- "Could this be a new KIND of thing rather than a feature
  extension?"

The four questions (Obvious, Simple, Honest, Good UX) are the
explicit decision compass; the probes above are the
investigation tools.

When the user is sketching architecture (as in passes 7-9),
the right orchestrator move is: confirm shape, surface
implications, ASK direction, capture outcomes. Don't pre-decide
slicing; don't pre-commit doc updates.

This pass-9 lock-in IS that pattern executed: design surfaced;
canonical form recognized; user said "this is incredible";
memory + REALIZATIONS captured before compaction risk.

**FM 23 candidate: When the user says "we must not forget
this," save to memory IMMEDIATELY.**

The auto-memory protocol's explicit-save trigger. User said
"this form is incredible... we must not forget this" —
canonical server form saved as `project_*` memory immediately.

### Pass 10 — `:wat::core::nil` IS the exit code (drop ExitCode entirely)

User direction (2026-05-10):

> *"do we need -> :wat::kernel::ExitCode at all?... we shouldn't
> even expose this.... you just run your program.. if you panic
> it'll go stderr and we'll exit non-zero. we program for the
> case that assumes we'll never panic.. that's how we demonstrate
> our code.. these are our advertised patterns"*

> *"i think the signature should be '[] -> :wat::core::nil' --
> nil /is the exit code/"*

The four-questions analysis (run on **drop ExitCode** vs **keep
ExitCode**):

| Question | Drop ExitCode (`-> :nil`) | Keep ExitCode |
|---|---|---|
| Obvious | YES — `:user::main [] -> :nil` reads as "entry point that does the work; nil is the success marker" | LESS — return type is theatrical when panic governs failure |
| Simple | YES — no typealias, no constructor-from-typealias for primitive ints, no terminal-form-must-be-ExitCode rule | NO — adds a concept whose only purpose is OS-exit-code |
| Honest | YES — panic IS the failure path; clean nil-return IS success. No theatrical gradation. Aligns with arc 114's `Program<I,O> -> :nil` contract | NO — lipstick over what StdErrService already handles |
| Good UX | YES — `:user::main`, `:user::thread`, `:user::process` all share `[] -> :nil` (modulo channel params); one shape across tier 0/1/2/3 | NO — special-cases tier 0 |

**The user's framing is precision: not "no return type" but
"returns nil, which IS the success exit code."** Same precedent
shape as arc 114's `Program<I,O> -> :nil`. The substrate maps
nil-return to libc::exit(0); panic-cascade maps to libc::exit(N).
User code never participates in exit-code arithmetic.

This is the **absence-is-signal** memory pattern flipped: when a
feature seems necessary but only pads signature, it's masking the
real shape. Drop it; the canonical form reaches uniform tier-0/1/2
signature. **Verbose-is-honest** doesn't apply here — ExitCode
wasn't carrying information; it was duplicating panic's role.

**The locked-in canonical form (post-pass-10):**

```scheme
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [client (:wat::kernel::StdInService/connect)]
    (:wat::kernel::server-loop client my-handler)))

(:wat::core::defn :wat::kernel::server-loop<I,O>
  [client    <- :wat::kernel::Client<I,O>
   handler   <- :wat::core::fn(I)->O]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::Client/recv client)
    -> :wat::core::nil
    ((:wat::core::Some req)
      (:wat::core::let [resp (handler req)]
        (:wat::kernel::StdOutService/send resp)
        (:wat::kernel::server-loop client handler)))
    (:wat::core::None
      :wat::core::nil)))
```

**Cascade (what changes from pass 8/9 lock-in):**

- `:user::main` signature: `[] -> :wat::core::nil` (not `-> :ExitCode`)
- `:wat::kernel::ExitCode` typealias **retires from arc 170 scope**
  (was § 2 ship; was new substrate work in slice 1e). If a future
  arc surfaces a CLI tool that genuinely needs `0/1/2` exit-code
  distinction (grep-like), THAT arc mints the helper. Arc 170
  scopes it out affirmatively.
- Slice 1e **drops** the typealias-as-constructor work for
  primitive-aliased values (`:wat::core::nil` already works at
  both type and value positions per WAT-CHEATSHEET; no new
  substrate work needed).
- `:wat::kernel::main!` macro expansion: terminal `:wat::core::nil`,
  not `(:wat::kernel::ExitCode 0)`.
- `:wat::kernel::run!` macro: variadic `do`-wrap; user's last form's
  return value flows through; if the last form returns nil,
  signature satisfied; if it doesn't, freeze diagnostic catches it.
  No magic ExitCode-coercion.
- wat-cli `run`: clean `:user::main` return → `libc::exit(0)`;
  panic-cascade emit → `libc::exit(N)` from StdErrService. wat-cli
  has no plumbed exit-code value from main's return.
- `validate_user_main_signature` enforces `[] -> :wat::core::nil`.
- spawn-process child: returns nil cleanly; child process exit
  status driven by panic-cascade vs clean-return. Parent reads
  exit status without participating in a wat-level ExitCode value.
- `:user::main`, `:user::thread`, `:user::process` ALL three
  return `:wat::core::nil`. Modulo channel params, the entry
  contracts unify across tier 0/1/2/3.

**The advertised pattern is "we never panic."** Programs that
follow it return nil; exit 0. Programs that fail panic; cascade
fires; exit non-zero. The substrate handles the OS-level
exit-code mapping; user code never participates.

User confirmation (2026-05-10):

> *"yeah - we found our shapes... just needed to add argv lol"*

The arc-170 architecture starts as "add argv to :user::main" and
ends at "the Program contract unifies across tiers; nil IS the
exit code." Each refinement made the design simpler and more
honest.

**Connects to memory:**
- `feedback_absence_is_signal.md` — when a feature looks
  necessary but pads signature, ask what it MASKS. ExitCode
  masked the truth that arc 114 already established (`Program<I,O>
  -> :nil` is the entry contract).
- `feedback_verbose_is_honest.md` — anti-pattern check. Verbose
  is honest when verbosity carries info. ExitCode wasn't carrying
  info beyond panic-cascade's role; not honest verbosity.
- `project_arc_170_canonical_server_form.md` — canonical form
  amended to nil-return shape.

### Pass 11 — Drop explicit `client` binding; helpers + ambient + escape-hatch

User direction (2026-05-10):

> *"i think... client ... should come from
> `(:wat::kernel::StdIoClient)` ?... or... what are we trying to
> communicate there.. we could just have...
> `(:wat::kernel::send-stdout! some-forms)` .. the helper writes
> through the service... we can expose the client via function?...
> this function when called uses the client for its thread?... we
> can hide all of this from the user while not rejecting the
> internals exist?..."*

User confirmation (after orchestrator proposal):

> *"outstanding - phenominal - we're back to ourselves again - the
> good UX matters - but we cannot ignore the stepping stones to
> deliver it.. we do the hard work to make the good work easy"*

Probe: what is the explicit `client` binding telling the user?
- It's a stranger they hold but never look inside
- The pass-9 architecture has per-thread clients ALREADY managed
  by the substrate (registered with services at spawn-thread per
  pass 9)
- The binding pads the canonical form with state-management the
  substrate has already done

The four-questions analysis (run on **drop client binding** vs
**keep explicit client binding**):

| Question | Drop (helpers + ambient) | Keep |
|---|---|---|
| Obvious | YES — `(server-loop my-handler)` reads as the loop-with-handler shape; `(StdIn/recv)` reads as "ask stdin for a value" | LESS — `client` is a stranger; what is it? a service? a state machine? a connection? |
| Simple | YES — server-loop takes one param (handler); ambient does the rest | LESS — extra binding to thread through; loop signature carries client AND handler |
| Honest | YES — Client TYPE still exists in the substrate (tier 1/2/3 unification); helpers route through it; escape-hatch exposes it for advanced cases. Internals exist; we don't reject them, we just don't surface them by default | YES — but at the cost of forcing every user through the surface |
| Good UX | YES — typical user never sees Client; advanced user reaches `(StdIn/client)` | LESS — must learn Client even for the canonical case |

**Locks in (the substrate-honest split):**

| Surface | Audience | Shape |
|---|---|---|
| `(:wat::kernel::StdIn/recv)` → `Option<I>` | typical user | helper; uses thread-local stdin client |
| `(:wat::kernel::StdOut/send v)` → `:nil` | typical user | helper; uses thread-local stdout client |
| `(:wat::runtime::panic! ...)` | typical user | helper; uses thread-local stderr client; blocks; libc::exit |
| `(:wat::kernel::StdIn/client)` → `Client<I,...>` | advanced (custom dispatch, monkey-patching) | escape hatch; returns thread-local Client |
| `(:wat::kernel::StdOut/client)` → `Client<...,O>` | advanced | escape hatch |
| `:wat::kernel::Server` / `Client` types | substrate-internal + tier 1/2/3 unification (Server/run, custom protocols) | exists; tested; documented; unsurfaced in canonical form |

**Naming convention** — Type/verb shape per arc 109 § D'
precedent (Option/some?, Result/ok?). `:wat::kernel::StdIn` and
`:wat::kernel::StdOut` are entities (services per pass 9); the
slash reads as "ask the StdIn entity to do recv."

**The locked-in canonical form (post-pass-11):**

```scheme
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::server-loop my-handler))

(:wat::core::defn :wat::kernel::server-loop<I,O>
  [handler <- :wat::core::fn(I)->O]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::StdIn/recv)
    -> :wat::core::nil
    ((:wat::core::Some req)
      (:wat::core::let [resp (handler req)]
        (:wat::kernel::StdOut/send resp)
        (:wat::kernel::server-loop handler)))
    (:wat::core::None
      :wat::core::nil)))
```

The canonical form drops to **9 lines from 12**. The user's
typical 3-line program through the macro reaches:

```scheme
(:wat::core::load! "some-lib.wat")
(:wat::kernel::main! :my::handler)
```

— unchanged from pass 9 user-side (the macro expansion changes
internally; user form unchanged).

**`main!` macro expansion (post-pass-11):**

```scheme
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [handler handler-expr]
    (:wat::kernel::server-loop handler)))
```

**Cascade (what changes from pass 10 lock-in):**

- Canonical form drops the `client` binding
- `server-loop` signature: `[handler <- :fn(I)->O] -> :nil`
  (drops `client` param)
- Mint `:wat::kernel::StdIn/recv` and `:wat::kernel::StdOut/send`
  helpers as user-facing surface
- Mint `:wat::kernel::StdIn/client` and `:wat::kernel::StdOut/client`
  escape-hatch accessors (return per-thread Client from
  thread-locals)
- spawn-thread (slice 1g) sets per-thread Client values into
  thread-locals as part of register-with-services
- `main!` macro expansion drops the `client` binding
- Substrate slice 1h adds the helpers + escape hatches to its
  scope (alongside Server/Client types)

**Stepping-stones reinforcement:** the user's confirmation
emphasizes the discipline. The good UX (3-line user program
reaching `main! :my::handler`) is the reward; the substrate
work is the cost:
- StdIn/Out/Err services (slice 1f)
- spawn-thread register-with-services + per-thread Client
  thread-locals (slice 1g)
- Server/Client substrate types + helpers + escape hatches +
  macros (slice 1h)
- structured-stderr-only + panic-cascade (slice 1i)

Each substrate slice is non-negotiable; the helper surface
DEPENDS on them. "we do the hard work to make the good work
easy" — the canonical 9-line form is honest because the
substrate carries every concern user code drops.

**Connects to memory:**
- `project_wat_llm_first_design.md` — "one canonical path per
  task; reject synonym features." Helpers ARE the canonical
  path; explicit Client is the escape hatch (not a synonym).
- `feedback_absence_is_signal.md` — the explicit `client`
  binding masked the per-thread ambient relationship the
  substrate already manages. Drop the binding; the ambient
  surface emerges.
- `feedback_simple_is_uniform_composition.md` — N identical
  helper calls (StdIn/recv, StdOut/send, etc.) IS simple.
  Don't conflate change-count with complexity.
- `project_arc_170_canonical_server_form.md` — canonical form
  amended to drop client binding + use helpers.
Then captured in REALIZATIONS as the pass-8 record.

### Pass 12 — `println` + `readln` are the canonical verbs; protocol is line-delimited EDN; data IS HolonAST

User direction (2026-05-10):

> *"so the protocol is the newline is the end of the data...
> users /must/ append new line for recv to work.. it doesn't
> choose bytes.. it chooses data and data ends with a new line"*

> *"the ret val of recv /is data/ -> HolonAST and we can do work
> on that.... we can eval it to extract a value .. we can cast
> it to a string.. a vector... it data that we can work on"*

> *"C - we provide both... `(:wat::kernel::println some-forms)`
> exists as an opinionated convenience - it has earned its spot
> as it provides a good UX that leverages all the dependencies
> we've been building.. it blocks, it passes data - its what we
> need... in a fork it provides the data transmission path for
> the client/server architecture... `(:wat::kernel::readln) ->`
> parses the `(:wat::kernel::println some-forms)` into some-forms
> in the local binding that's the UX"*

The pass-11 names `StdIn/recv` / `StdOut/send` retire from the
user surface. Pass-12 locks-in mainstream-named helpers:

| Helper | Type | Behavior |
|---|---|---|
| `(:wat::kernel::println v)` | `:T -> :nil` | write data + newline; blocks until written |
| `(:wat::kernel::readln)` | `() -> :Option<:wat::holon::Atom>` | read line + parse EDN to Atom; blocks; None on fd 0 closed |
| `(:wat::kernel::StdIn/client)` | `() -> :Client` | escape hatch (advanced) |
| `(:wat::kernel::StdOut/client)` | `() -> :Client` | escape hatch (advanced) |

**The protocol is line-delimited EDN; data IS HolonAST.** Per
memory `project_pipe_protocol.md` ("line-delimited EDN + kernel
pipes. One protocol; four transports."). Pass-11's parametric
`Sender<T>` / `Receiver<T>` framing was wrong for the OS
boundary — at fd 0 / fd 1, users get `:wat::holon::Atom`, NOT a
typed parametric I/O. The user works on the AST and chooses how
to interpret:

- `(:wat::core::eval req)` — evaluate to a value
- `(:wat::holon::Atom/as-string req)` — cast to string
- pattern-match the AST shape directly

The handler signature changes:
- pass-11: `:fn(I)->O` (parametric)
- pass-12: `:fn(wat::holon::Atom)->wat::holon::Atom`

Per `project_wat_llm_first_design.md` ("reject synonym features"),
`println` + `readln` ARE the canonical verbs; `StdIn/recv` /
`StdOut/send` (my pass-11 terminology) are renamed away — not
preserved as synonyms. Type/verb shape survives only on the
escape-hatch accessors (`StdIn/client`, `StdOut/client`).

### Pass 13 — `:wat::core::nil` IS the graceful "done" message; user owns signal-cleanup path

User direction (2026-05-10):

> *"processes may be intentionally short lived... and we need to
> communicate graceful shutdowns.. we have pid groups enabled...
> so the file descriptors should organically close... so us
> getting a closed file descriptor is normal... the forked
> process was intentionally short (think hermetic tests) or a
> long term process is shutting down from a SIGTERM or SIGINT
> being passed... the fork needs to communicate its done before
> it exists... so it must write `:wat::core::nil` as its final
> message to communicate its over"*

User correction (after orchestrator's first wrong-shape proposal):

> *"the user must be allowed to perform their own clean up - go
> study how we manage signals - your response scares me - you
> have forgotten too much"*

User confirmation (after orchestrator crawled arc 106 + memory):

> *"those expressions are so fucking good - don't forget those -
> they are absolutely arc worthy"*

**Failure-engineering paid lesson:** the orchestrator's first
pass-13 proposal conflated substrate-automatic-`:nil` with
substrate-automatic-signal-handling. Wrong: arc 106 established
a model where the kernel MEASURES (per-process atomic flags
flipped by async-signal-safe handlers) and userland TRANSITIONS
(wat program polls `(:wat::kernel::stopped?)` etc., decides what
to do, returns when ready). The substrate's automatic `:nil` is
only the post-main exit epilogue, NOT a forced shutdown on
signal.

The recovery: orchestrator crawled `src/fork.rs:81-130` +
`src/runtime.rs:51-119` + memory `project_signal_cascade.md`
before re-proposing. The corrected pass-13 lock-in respects the
arc-106 model.

**The protocol's terminal states (post-pass-13):**

| `(readln)` returns | Meaning | Handling |
|---|---|---|
| `Some(:wat::core::nil)` | peer announced graceful done | exit loop cleanly; substrate emits our `:nil` on main-return |
| `Some(other)` | peer sent data | process; respond; loop |
| `None` | fd 0 closed without graceful `:nil` | ungraceful (SIGKILL, panic that escaped cascade); user chooses panic / log / exit |

**The four-layer ownership table:**

| Layer | Owner | Contract |
|---|---|---|
| OS signal arrival | kernel + arc 106 handlers | flip atomic flag; return; async-signal-safe |
| Cascade across pid group | OS (`killpg`) | wat-cli broadcasts; kernel delivers; per-process handlers fire |
| Stopped/sigusr polling | **user wat program** | `(:wat::kernel::stopped?)` / `(sigusr1?)` etc. at safe checkpoints |
| Cleanup logic on observed stop | **user wat program** | drain work, close resources, return nil |
| Final `:nil` emit + libc::exit(0) | substrate | post-main epilogue |

**Substrate-automatic graceful-`:nil` epilogue (option A):**

```
user's main returns :nil
  ↓
substrate epilogue:
    1. emit :wat::core::nil to fd 1     ← protocol-compliance final
    2. close fd 1
    3. libc::exit(0)
```

Independent of WHY main returned. Panic exit skips this path
(StdErrService cascade fires libc::exit(N) directly; consumer
sees ungraceful `None`).

User confirmation:
> *"I agree with A - we adhere to the protocol ourselves -
> that's our leg of the work. we write nil to the stdout and
> exit 0 - that's our protocol compliance for the runtime"*

**The locked-in canonical form (post-pass-13):**

```scheme
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::server-loop my-handler))

(:wat::core::defn :wat::kernel::server-loop
  [handler <- :wat::core::fn(wat::holon::Atom)->wat::holon::Atom]
  -> :wat::core::nil
  (:wat::core::if (:wat::kernel::stopped?)
    -> :wat::core::nil
    ;; stop signal observed; user-side returns nil; substrate emits :nil + exits
    :wat::core::nil
    (:wat::core::match (:wat::kernel::readln)
      -> :wat::core::nil
      ;; peer signaled done
      ((:wat::core::Some :wat::core::nil)
        :wat::core::nil)
      ;; data; process; loop
      ((:wat::core::Some req)
        (:wat::core::let [resp (handler req)]
          (:wat::kernel::println resp)
          (:wat::kernel::server-loop handler)))
      ;; ungraceful close — peer died without :nil
      (:wat::core::None
        (:wat::runtime::panic! "stdin closed without graceful :nil")))))
```

**User-cleanup pattern** — when the user needs pre-exit work on
observed stop:

```scheme
(:wat::core::if (:wat::kernel::stopped?)
  -> :wat::core::nil
  (:wat::core::do
    (my::flush-caches!)
    (my::log-shutdown-state!)
    :wat::core::nil)
  ...)
```

The user owns the path between "stop observed" and "main returns
nil." Substrate stays out.

**Cascade on the slice plan:**

- Slice 1i extends scope to "wat-cli exit-path discipline" —
  panic-cascade on stderr (existing) + graceful-`:nil` epilogue
  on stdout (new). Both are exit-path doctrines.

**Connects to memory:**
- `project_signal_cascade.md` — wat-rs cascades signals via
  POSIX pgid + killpg; cascade is mandatory; kernel tracks
  membership; substrate has no registry
- `project_pipe_protocol.md` — line-delimited EDN; one protocol;
  four transports
- arc 057 (HolonAST schema in holon-rs)
- arc 092 (wat-edn line-delimited serialization)
- arc 106 (substrate-level signal handlers for fork children)
- arc 110 (silent kernel-comm illegal — None must surface in match)
- `feedback_assertion_demands_evidence.md` — the orchestrator's
  first pass-13 proposal asserted "SIGTERM converts to
  main-should-return" without crawling. The user's "your response
  scares me - you have forgotten too much" is exactly the
  assertion-demands-evidence trigger. Crawl restored.

### Pass 13 meta-observation — TCO idealization without prompt

User direction (2026-05-10):

> *"you should note that we reached for TCO without explicit
> direction and its the idealized state"*

The canonical server-loop form was written with recursion in tail
position (`(server-loop handler)` as the let-body's last form,
inside the match arm, inside the if-else branch). The orchestrator
wrote this shape without being prompted to make it tail-recursive
or to think about stack growth. The user verified ("this is a
TCO pattern, right?"); the orchestrator crawled arc 003 + memory;
arc 003's trampoline (RuntimeError::TailCall + apply_function
loop) supports exactly this shape. The 100k match-recursion
benchmark in `tests/wat_tco.rs` is named "the Console/loop shape"
— precisely what we reached for.

**The signal:** the natural shape AND the substrate's idealized
state CONVERGED without orchestrator prompting. This is a
maturity signal — the substrate's idiomatic patterns are now
honest enough that "write the obvious shape" produces "the
substrate's intended form."

Cross-arc precedent reinforcement:
- arc 003 (TCO) — built the trampoline so this shape works
  indefinitely
- arc 167 (fn-flat-signature), arc 159 (let new-shape) — pruned
  syntactic shapes so the obvious form IS the canonical form
- arc 109 (kill-std + FQDN doctrine) — every helper named
  consistently so the obvious name IS the substrate name
- arc 167+159+109+003 together → "the natural way to write a
  server-loop is the TCO-correct way is the canonical way"

This convergence is what we mean by "the foundation is
impeccable" (per `feedback_compaction_protocols.md` §
strategic-context). When the orchestrator writes idiomatic wat
without effort and the substrate's existing test coverage names
that exact pattern, the foundation has settled.

**For the arc-170 INSCRIPTION (slice 5 paperwork):** the
canonical form's tail-recursive shape is INTENTIONAL alignment
with arc 003. Cross-reference `tests/wat_tco.rs` "the
Console/loop shape" benchmark explicitly. Future readers see
the lineage: arc 003 built the runway; arc 170 lands on it.

**Connects to memory:**
- `feedback_simple_is_uniform_composition.md` — the obvious
  composition IS the simple composition. Don't over-think it.
- `feedback_no_speculation.md` — the orchestrator didn't
  speculate "this should be tail-recursive"; the obvious shape
  emerged. The user verified post-hoc; arc 003 supports it.

### Pass 14 — Wire encoding lexical doctrine (position-aware)

User direction (2026-05-10), surfaced mid-slice-1f-ii authoring:

> *"if yes [transmit macros over the wire].. we need to remove
> the comma...
> we have scheme macros now.... we need to swap to clojure
> macros...
> we also need to impose our pending rules...
> type declarations may only be keywords. keywords may not
> containt underscores. underscores are reserved for swapping
> from commas when transmitting EDN ...
> :wat::core::HashMap<wat::core::String_wat::core::i64>
> further... symbols may not contain commas, however they can
> use underscores..."*

The pivot surfaced four threads of foundation work that arc 170
was about to TRANSMIT through the wire without the rules locked:

1. **Lexical class redefinition** — keywords vs symbols allow
   different chars
2. **EDN wire encoding** — comma↔underscore swap for round-trip
3. **Comma→apostrophe in fixed-arity dispatch forms** — swap
   `:foo,2` to `:foo'2` (symbols/keywords no longer carry
   comma in source)
4. **Macro flavor swap (Scheme → Clojure)** — Clojure's
   `'foo` quote, `` `foo `` syntax-quote with auto-namespace
   + auto-gensym-on-`#`, `~foo` unquote, `~@foo`
   unquote-splicing, `gensym`, `&form`/`&env`

**Slicing analysis (per "delivery is the measurement" user
direction):**

| Thread | Coupling to arc 170 transmission | Disposition |
|---|---|---|
| (1)+(2) Lexical + wire | TIGHT — slices 1f-ii / 1f-iii / 1f-iv block | Inline arc 170 as new prerequisite slice 1f-W |
| (3) Comma→apostrophe dispatch | ZERO direct coupling | Future arc 171 (orthogonal) |
| (4) Macro flavor swap | LOOSE — arc 170 macros work either flavor | Future arc 172 (orthogonal) |

**Four-questions analysis on the lexical rule (universal "no
underscores in keywords" vs position-aware "no underscores
inside `<>`"):**

| Question | A — Universal | B — Position-aware |
|---|---|---|
| Obvious | LESS (breaks `:rust::*` Rust-mirror convention OR forces an exemption — two rules dressed as one) | YES (one position-conditioned rule; motivation reads in one sentence) |
| Simple | LESS (composed of forbid + exemption + 7-keyword sweep + memory amendment) | YES (single condition: "am I inside `<>`?") |
| Honest | LESS (exemption hides the lie; OR rename `:rust::crossbeam-channel::Sender` lies about Rust mirroring) | YES (no carve-outs; round-trip unambiguous because special chars only have meaning where used) |
| Good UX | LESS (Rust devs jarred; substrate authors maintain special case) | YES (mirror convention preserved; bracket-syntax has its own char rules) |

**B wins on all four foundational questions. Locked in.**

The rule (locked in 2026-05-10):

- **Inside `<...>` substrings within a keyword body:**
  - Source: `_` is FORBIDDEN; `,` is the type-arg separator
  - Wire: `,` ↔ `_` swap (one-to-one at depth ≥ 1)
- **Outside `<...>`:**
  - Source: `_` is allowed (keeps `:rust::crossbeam_channel::*`
    Rust-mirror convention)
  - Wire: no swap (chars stay verbatim)

The 18 underscore-in-keyword forms in source are ALL outside
brackets. Zero rename needed. `project_wat_rust_interop.md`
doctrine preserved verbatim.

**Slicing disposition (final):**

- Arc 170 absorbs (1)+(2) as **new slice 1f-W** (Wire) —
  prerequisite for slice 1f-ii. Lexer split + position-aware
  wire encoding + tests. Pure substrate work in `wat-edn`
  crate + lexer; no codebase sweep.
- Arc 171 (NEW; future) — comma→apostrophe sweep across
  fixed-arity dispatch forms. Arc 146 follow-up. Sized at arc
  171 author time (most grep hits for `:foo,bar` are tuples
  / parametric type args inside `<>`, NOT dispatch suffixes).
- Arc 172 (NEW; future) — Scheme→Clojure macro flavor swap.
  Substantial macro evaluator rewrite. Orthogonal to arc 170;
  arc 170's `main!`/`run!` macros work in either flavor.

**Cascade on arc 170 slice plan:**
- Slice 1f-W inserts BEFORE 1f-ii
- Slice 1f-i (shipped at `630f621`) needs no source revision —
  it parses incoming EDN; the position-aware un-escape lands
  in the parser's keyword-handling code which slice 1f-W
  ships; slice 1f-i inherits unchanged
- Slices 1f-ii / 1f-iii / 1f-iv proceed unchanged; they get
  the wire encoding for free from slice 1f-W

**Connects to memory:**
- `feedback_four_questions.md` — Obvious + Simple + Honest
  before Good UX; B passed all four; A failed on three
- `project_wat_rust_interop.md` — `:rust::*` Rust-mirror
  preserved by position-aware rule
- `feedback_simple_is_uniform_composition.md` — single
  position-conditioned rule IS simple
- `feedback_absence_is_signal.md` — the universal rule SOUNDED
  cleaner but masked the `:rust::` conflict; the position-
  aware reading respects the actual structure of the problem

### Pass 15 — Services are wat programs (not Rust singletons); runtime is the orchestrator; control-pipe is the optional dynamic-membership extension

User direction (2026-05-10) after a workspace-wide deadlock
surfaced during slice 1f-ii verification:

> *"we hit a deadlock - its been a long time since that
> happened... go study our docs - specifically zero mutex and
> service programs - we failed one of our own rules"*

**The violation, named:**

Slices 1f-i (StdInService, shipped at `630f621`) and 1f-ii
(StdOutService, working tree dirty pre-revert) implemented
substrate services as Rust threads with `OnceLock<&'static
ServiceHandle>` singletons. This violated TWO foundational
doctrines:

1. **ZERO-MUTEX.md tier-3 discipline** — services are
   *Program-owned, message-addressed: state owned by a spawned
   wat program, accessed by clients via bounded channels.* Not
   Rust threads with custom registration APIs.

2. **SERVICE-PROGRAMS.md "the lockstep"** — *outer scope holds
   the `ProgramHandle`. Inner scope owns every Sender. Get the
   nesting right and the program shuts down cleanly without any
   explicit teardown code. Get it wrong and you deadlock.*
   `OnceLock<&'static>` has no Drop; the singleton's worker
   thread runs forever; cross-test concurrency on the global
   handle is undefined; deadlock under workspace test
   conditions.

The deadlock IS the substrate-as-teacher diagnostic.
Per `feedback_attack_foundation_cracks.md`: when a crack
surfaces, the fix is also the diagnostic. Apply, use as compass,
pivot forward into cracks the fix reveals.

**The pass-9 framing was wrong.** Pass 9 said "Three substrate
services boot before any user code" and named them as Rust
runtime components. That invented a NEW Rust-thread-based
service tier — bypassing the existing tier-3 (wat program +
mailbox + lockstep) which the substrate already provides via
`:wat::kernel::spawn`, `make-bounded-channel`, `select`, and
`HandlePool`.

**The corrective architecture (pass 15 lock-in):**

The runtime IS the orchestrator. Services are wat programs.
Helpers are thread-aware contexts that "just work."

User direction continued:

> *"every thread that's spawned get upserted into the file
> handles and when they shutdown they are removed... we handle
> registration in the runtime using the runtime?... the control
> thread for the services to be able to something that like a
> 'sighup' in tranditional services... users just call
> (:wat::kernel::readln) to grab next stdin line,
> (:wat::kernel::println ....) to write next stdout line... the
> runtime is responsible for creating and draining"*

The control-pipe protocol (sighup analog):

```
Runtime → service control-pipe:
   :register thread-id (per-thread channels)  → service adds routing entry; acks
   :reap     thread-id                        → service drops routing entry
   :sighup                                     → service drains pending; exits
```

The active-handle ledger:

The runtime tracks which threads are registered with which
services. spawn-thread increments; thread reap decrements.
When count → 0 AND runtime is in shutdown phase, runtime sends
`:sighup` to each service → services drain → exit. Runtime
joins drivers. wat-cli libc::exit(0).

**Two canonical patterns now in the substrate library:**

| Pattern | When | Worked example |
|---|---|---|
| **Static-membership** | Client set known at construction; clients live for the service's lifetime | `wat/console.wat` (pair-by-index ack), `crates/wat-lru/.../CacheService.wat` (Pattern B data-back), `crates/wat-telemetry/.../Service.wat` (Pattern A unit-ack) |
| **Dynamic-membership** | Clients register/reap during program lifetime | `wat/kernel/services/stdin.wat` etc. (slice 1f's three substrate stdio services — NEW) |

Both share the tier-3 base (driver loop + recv-loop + scope-drop
cascade + mini-TCP ack). The dynamic pattern ADDS a control-pipe
side-channel for register/reap/sighup messages.

**The control-pipe is OPTIONAL.** Static-membership services
(Console etc.) don't need it; senders dropping IS the
shutdown signal. Dynamic-membership services need it because
membership changes during program lifetime — the control-pipe
is the lifecycle side-channel.

User confirmation (2026-05-10):

> *"we demonstrate how to do services by making thread aware
> contexts that just work.. users can still provision their
> crossbeam pipes to use for their own operations, but they
> could provide a control pipe should they need to have it..
> not all services need signaling.. but the option exists and
> we show the option in our code so others can see the
> canonical pattern"*

**The cultural payoff (this is the load-bearing point):**

The substrate's runtime + services become the canonical worked
example for the entire ecosystem:

1. wat-cli's run loop demonstrates the spawn → invoke → sighup → join lifecycle
2. `wat/kernel/services/stdin.wat` etc. demonstrate the canonical service-template + control-pipe handler
3. spawn-thread's contract demonstrates the register/reap protocol
4. User's `(println v)` calls demonstrate the thread-aware helper

Future user services can copy this pattern wholesale. Existing
substrate services (Console, CacheService, Telemetry,
HolonLRU) MAY migrate (opt-in, future arcs) when their use
case warrants dynamic membership; their static-membership
pattern remains valid otherwise.

**The substrate practices what it preaches.** Tier-3
discipline is not just documented; it's IMPLEMENTED at the
most-foundational layer of the runtime. Every service in the
ecosystem inherits a coherent doctrine, expressed via worked
examples in the substrate's own source.

**Slice plan implication (revised):**

Slice 1f reshapes from the original Rust-thread α/β/γ to
wat-program α/β/γ/δ:

| Stone | Scope | Notes |
|---|---|---|
| 1f-α | Substrate primitives `:wat::kernel::println` + `:wat::kernel::readln` (look up thread-local routing populated by runtime register cycle) | small |
| 1f-β | Wat-side service implementations: `wat/kernel/services/stdin.wat`, `stdout.wat`, `stderr.wat` (canonical service-template + control-pipe register/reap/sighup handler) | mostly mechanical from existing service-template.wat |
| 1f-γ | Runtime orchestrator + spawn-thread integration: `:wat::kernel::spawn-thread` emits register/reap; active-handle ledger as Rust state in wat-cli/runtime | substantial; absorbs original slice 1g |
| 1f-δ | wat-cli boot integration: full process-start to libc::exit cascade (spawn services → invoke main → reap main → sighup → join → exit) | small |

Slice 1g (spawn-thread register-with-services) FOLDS into
slice 1f-γ — they were always the same concern.

**Cost of the corrective:**

- `git revert 630f621` (slice 1f-i Rust singleton — wrong shape)
- Working tree slice 1f-ii dirty → discarded
- ~100 min of opus work reverts
- Pass 15 (this section) records the lesson + the architecture

The reverts are NOT regression — they're the substrate teaching
the right pattern. Per `feedback_attack_foundation_cracks.md`:
foundation supersedes ergonomics. The path of least resistance
becomes the lockstep + control-pipe pattern, by demonstration.

**Connects to memory:**

- `feedback_never_deadlock.md` — "I am a user of wat too. Every
  comm site lands deliberately in match-or-expect; the
  SERVICE-PROGRAMS lockstep is mandatory; function-decompose
  multi-driver shutdowns; no clever bind-then-decide." Slice
  1f-i + 1f-ii reached for clever-bind-then-decide via
  OnceLock; deadlock surfaced.
- `feedback_attack_foundation_cracks.md` — fix is also
  diagnostic; pivot forward into the cracks the fix reveals.
- `feedback_assertion_demands_evidence.md` — the agents
  reported "Mode A clean / 16/16 rows" against scorecards I
  authored. The scorecards verified the BRIEF's contract; the
  BRIEF was wrong shape. Future BRIEFs for service work must
  cite ZERO-MUTEX.md + SERVICE-PROGRAMS.md as load-bearing
  references and verify the implementation tier at slice 1f-W
  pre-grep depth.
- `project_signal_cascade.md` — the control-pipe pattern is
  the wat-substrate analog of POSIX signals — side-channel for
  service lifecycle distinct from data flow.
- `feedback_pivot_not_defer.md` — the temptation to "fix the
  deadlock and proceed" is exactly the FM-pivot-vs-defer trap.
  The deadlock SIGNAL says reframe-needed. Reframing is the
  forward move.

### Pass 16 — Two-signal mini-TCP protocol refinement (post-pass-15 settlement)

User direction (2026-05-10) walked through the protocol details
that pass 15 left as WIP. Pass 15 captured the architectural
pivot (services-as-wat-programs); pass 16 captures the protocol
that runs on top.

**The settlement, summarized:**

1. **Two signals, not three.** Pass 15's draft had
   `:register/:reap/:sighup`. Refined to `:add/:remove` deltas.
   The `:sighup` variant rejected ("we have full flexibility
   here"); shutdown is via scope-drop per SERVICE-PROGRAMS.md
   doctrine, not a control-pipe message. The `replace`/`reload`
   variant rejected — FIFO + lockstep + acked deltas can never
   drift, so a snapshot operation was carrying baggage for a
   problem the discipline already prevents.

   > *"the entire process is...... when a thread is repeaped..
   > the same is done... the handles are removed and the io
   > select loop is signaled to restart - its all lock step"*

   > *"add and remove paths are going to TCO with the updated
   > refs.... so we just need two signal kinds... add and
   > remove"*

2. **Per-service Signal enums, not a shared `Roster`.** The
   four-questions verdict failed shared-enum on all four. Three
   services means three independent SDKs. The shape is uniform
   across services; the payload types differ because each
   service's role determines which crossbeam end it guards.

   > *"the services are guarding their mutable reference... the
   > thing being added is a crossbeam... the TCO service loop
   > processes that"*

3. **HashMap routing table, not Vec.** Dynamic-membership pattern
   needs O(1) reap-by-id; Vec was static-membership thinking
   carried over from Console + CacheService.

   > *"i think its a hash map?.. we loop the values and reap by
   > id?"*

4. **Mini-TCP universal — no fire-and-forget anywhere.**
   Including stdin. The substrate primitives all block:

   ```
   (:wat::kernel::println v)  -> :wat::core::nil    ; serialize → send → ack
   (:wat::kernel::eprintln v) -> :wat::core::nil    ; same shape, stderr
   (:wat::kernel::readln)     -> :wat::holon::HolonAST  ; req → reply
   ```

   > *"any comm between a service /must/ be 'mini-tcp' there is
   > no fire and forget here.. its fire and wait for completion
   > signal"*

   > *"the readln, println eprintln all must handle the blocking
   > for the user.. the user just uses those interfaces.. they
   > block as long as they need to... the is the zero mutex
   > pattern completely"*

5. **Polymorphic println, no value→HolonAST lift required.** The
   substrate already has `:wat::edn::write` (`src/edn_shim.rs:71`)
   that takes any wat Value and produces a String via
   `value_to_edn_with`. The println primitive uses this
   internally — caller passes any value, substrate handles
   serialization, sends String through Sender<String> (Console's
   pattern, ported to dynamic membership).

   > *"both of those will /just work/ right?... the entire
   > point is to transmit data as edn and either side just works
   > with data"*

   The asymmetry between stdout/stderr (Sender<String>;
   pre-serialized at call site) and stdin (Sender<HolonAST>;
   parsed by service) is honest — the work lives where the work
   IS: encoding at the producer thread, decoding at the place
   that has bytes.

6. **Console retires as slice 1f-ε.** The new shape supersedes
   Console; existing consumer sweep happens after the new shape
   is proven end-to-end (1f-α through 1f-δ). Console was a
   stepping stone.

   > *"at the end of this console service will be completely
   > replaced by the stdin,out,err services -- we have engineered
   > a better form"*

**The locked Signal enum shapes:**

```
(:wat::core::enum :wat::kernel::services::StdInService::Signal
  (add    (thread-id :wat::kernel::ThreadId)
          (req-rx    :wat::kernel::Receiver<wat::core::nil>)
          (reply-tx  :wat::kernel::Sender<wat::holon::HolonAST>))
  (remove (thread-id :wat::kernel::ThreadId)))

(:wat::core::enum :wat::kernel::services::StdOutService::Signal
  (add    (thread-id :wat::kernel::ThreadId)
          (req-rx    :wat::kernel::Receiver<wat::core::String>)
          (ack-tx    :wat::kernel::Sender<wat::core::nil>))
  (remove (thread-id :wat::kernel::ThreadId)))

(:wat::core::enum :wat::kernel::services::StdErrService::Signal
  (add    (thread-id :wat::kernel::ThreadId)
          (req-rx    :wat::kernel::Receiver<wat::core::String>)
          (ack-tx    :wat::kernel::Sender<wat::core::nil>))
  (remove (thread-id :wat::kernel::ThreadId)))
```

Three independent enums; same shape; per-service payload types.
Routing table per service is `HashMap<ThreadId, (req-rx, reply/ack-tx)>`.
TCO loop builds select-set from values + control-pipe each
iteration; on control fire, recurses with mutated map.

**Slice 1f reshaped to α/β/γ/δ/ε** (was α/β/γ/δ in pass 15;
add 1f-ε for Console retirement):

| Stone | Scope |
|---|---|
| 1f-α | substrate primitives println / eprintln / readln |
| 1f-β | wat-side service implementations (stdin.wat, stdout.wat, stderr.wat) |
| 1f-γ | runtime orchestrator: spawn-thread emits Signal::add/remove + ledger |
| 1f-δ | wat-cli boot integration: full lifecycle |
| 1f-ε | Console retirement + consumer sweep (NEW; folds in via the better-form succession) |

**The four questions, applied:**

The pass-15-to-pass-16 progression followed the four questions
verbatim at every decision point:

- Shared-vs-per-service Signal enum → per-service wins on all four.
- Three-vs-two signals → two wins on Simple + Honest (the snapshot
  operation prevents nothing the lockstep doesn't already prevent).
- Concrete-vs-generic channel payload → concrete wins on Simple
  + Honest (the universal form type already covers everything).
- Mini-TCP universal-vs-asymmetric → universal wins on Honest
  (fire-and-forget would be a discipline gap).

**Connects to memory:**

- `feedback_four_questions.md` — the decision compass that drove
  every protocol settlement in this pass
- `feedback_never_deadlock.md` — universal mini-TCP is the
  enforcement at the discipline level
- `feedback_zero_mutex.md` — the architecture this slice IS
- `feedback_pivot_not_defer.md` — pass 15's reframe was the pivot;
  pass 16 is what survives the pivot
- `project_signal_cascade.md` — the control-pipe is wat-substrate's
  POSIX-signal analog (side-channel for service lifecycle distinct
  from data flow)

### Pass 17 — wat-cli is the OS boundary, not the orchestrator; hermetic is tier-2 by definition

User direction (2026-05-10) corrected two responsibility
ambiguities that surfaced AFTER pass 16 settled. Both are
clarifications, not protocol changes — the architecture didn't
shift; the layer attribution did.

**Clarification 1: wat-cli's responsibility scope.**

> *"i think we have identified that wat-cli is overstepping
> responsibilities... i think the thread management is in the
> vm - not the cli"*

Pass 16's BUILD-PLAN slice 1f-δ assigned service spawning,
ProgramHandle ownership, the active-thread ledger, and the
scope-drop cascade to wat-cli. That conflated layers:

- **wat-cli is the OS boundary** — fork the entry program (arc
  104), proxy stdio between real fds and child pipes (arc 104),
  waitpid + propagate exit code (arc 105), forward signals via
  kill(2) (arc 104d). That's it.
- **The VM (runtime) is the evaluator + thread manager** — owns
  the symbol table + frozen world, evaluates user programs,
  manages spawn-thread cycle, hosts the substrate primitives,
  owns service spawning + active-thread ledger + scope-drop
  cascade.

wat-cli should know nothing about services. It just calls into
the runtime to evaluate a program; the runtime handles
everything in between. This matches arc 104's discipline
("wat-cli is the containment surface; the runtime is the
evaluator").

**Implications for slice 1f:**
- Slice 1f-γ's active-handle ledger lives in `src/runtime.rs`,
  NOT "in src/runtime.rs or wat-cli" (BUILD-PLAN amended).
- Slice 1f-δ reshapes from "wat-cli boot integration" to
  "Runtime boot integration." Service spawning +
  ProgramHandles + scope-drop cascade move into the runtime's
  program-entry path. wat-cli changes are minimal (≤ 5 lines;
  may be zero).

This is what the hermetic-test inheritance requires anyway:
every fork child (production wat-cli OR hermetic test)
instantiates a runtime; the runtime's boot path is THE place
service infrastructure lives.

**Clarification 2: hermetic IS tier 2 — already and necessarily.**

> *"hermetic /must/ be in its own process - that's entire part
> of hermetic.. zero shared memory with the thing who requests
> hermetic evaluation... when we expose more and more rust
> interop, settings may exist in a global rust state that
> pollutes our runtime - heremetic makes this impossible"*

Per DESIGN.md TIERS framing: *"Hermeticness is the ambient
property of tier ≥ 2 — what the OS-process boundary inherently
provides (memory + signal + global-state + runtime-sealing
isolation, all at once because they're all manifestations of
the same boundary)."*

Verified on disk:
- `:wat::test::deftest-hermetic` macro expands to
  `:wat::test::run-hermetic body`
- `:wat::test::run-hermetic` (Layer 1 in `wat/test.wat`) routes
  through `:wat::kernel::spawn-process` (per `src/check.rs:14171`
  — *"run-hermetic routes through `(:wat::kernel::spawn-process
  fn)`"*)
- `:wat::kernel::spawn-process` is the substrate's tier-2 fork
  primitive (arc 104b + 105)
- Arc 124's INSCRIPTION confirms: *"Zero runner changes. The
  wat-side macros encode hermetic vs in-process via the choice
  of `run-sandboxed-ast` vs `run-sandboxed-hermetic-ast` in the
  body's expansion."*

Each hermetic deftest IS a separate OS process. Memory + rust
globals + signals zero-shared with the parent. The fork happens
before the child re-instantiates the runtime, so any rust
statics live only in the parent + are absent (or freshly
initialized) in the child.

**This is what makes hermetic actually hermetic.** As
`:rust::*` exposes more crates with statics / `OnceLock`
registries / `lazy_static` / `thread_local!`, in-process
isolation would be theatre. The fork is the only honest answer.

**Implication for arc 170 wrap state:**

| Layer | Hermetic guarantee |
|---|---|
| Production `wat run script.wat` | Tier 2 (own forked OS process per arc 104) |
| `deftest-hermetic` cargo test | **Tier 2 (own forked OS process)** — verified via run-hermetic → spawn-process |
| `deftest` cargo test (non-hermetic) | Tier 1 (shared cargo test process; explicit author choice — for tests that don't need isolation) |

Hermetic tests inherit production parity in the fork child:
own runtime, own services, own thread pool, full
println/eprintln/readln. The runtime's boot path (slice
1f-δ-revised) IS the wiring that delivers this — the fork
child runs the same boot path; it gets the same services
automatically.

**Connects to memory:**

- `feedback_zero_mutex.md` — the zero-Mutex architecture
  this slice IS
- `feedback_diagnose_before_spec.md` — both clarifications
  followed the discipline of reading actual docs (DESIGN.md
  TIERS framing; arc 124 INSCRIPTION; `src/check.rs:14171`)
  before drafting; my earlier framing was reasoning from
  scratch instead of doctrine
- `feedback_substrate_already_typed.md` — the hermetic
  infrastructure was already in place; my framing inverted
  what the substrate already provides
- `feedback_compaction_protocols.md` — the user invoked the
  recovery doc twice this session because compaction-amnesia
  was surfacing in this same shape (reasoning from scratch
  instead of from doctrine)

### Pass 18 — Unified Event enum collapses heterogeneous T's; per-service Signal becomes per-service Event

Slice 1f-β-i's opus run surfaced a substrate-architecture trap
and the user's correction defined the architecturally-honest
shape.

**The trap (opus's deliverable, working-tree-only; discarded):**

Opus implemented a relay sub-thread to bridge between
`:wat::kernel::select`'s monotyped `Vec<Receiver<T>>` signature
and pass-16's design that had TWO separate channel types
(`Receiver<()>` for thread readln-requests + `Receiver<Signal>`
for control-pipe). The relay converted each Signal arrival
into a `()` pulse on a wakeup channel + buffered the Signal on
a sibling channel; driver's select-set became homogeneous
`Vec<Receiver<()>>`. The classic POSIX self-pipe trick at the
wat layer — the SAME pattern slice 1f-i's singleton used (with
libc::poll + self-pipe) that we explicitly retired in pass 15.

The relay wasn't strictly wrong given the constraint opus
imposed on itself; it was the canonical workaround. But the
constraint was self-imposed.

**The correction** (user direction 2026-05-10):

> *"T is whatever we want it to be, right? including an enum
> with different matchables?... signals can coexist with the
> others?... explain to me why wake up is necessary if the
> input signal is a participating member?... sending an item
> on wake up will wake up the select loop as much as the
> signal bearing do work... the two coexisting makes zero
> sense"*

`:wat::kernel::select` is monotyped only in the sense that all
receivers in one select-set must share T. T itself is our
variable. The relay existed because opus accepted slice 1f-α's
`Sender<()>` (stdin) and `Sender<String>` (stdout/stderr)
channel-end types as fixed, then tried to bridge to the Signal
channel from outside. Once we collapse the two flows into ONE
unified `Event` enum per service, the select is homogeneous by
construction — no relay, no sub-thread, no second channel.

**The corrected protocol — per-service Event enum:**

```
(:wat::core::enum :wat::kernel::services::StdInService::Event
  (Read)                                                        ; thread asks for next form
  (Add    (thread-id :wat::kernel::ThreadId)
          (data-rx   :wat::kernel::Receiver<wat::kernel::services::StdInService::Event>)
          (reply-tx  :wat::kernel::Sender<wat::holon::HolonAST>))
  (Remove (thread-id :wat::kernel::ThreadId)))

(:wat::core::enum :wat::kernel::services::StdOutService::Event
  (Write  (line :wat::core::String))                            ; thread sends serialized line
  (Add    (thread-id :wat::kernel::ThreadId)
          (data-rx   :wat::kernel::Receiver<wat::kernel::services::StdOutService::Event>)
          (ack-tx    :wat::kernel::Sender<wat::core::nil>))
  (Remove (thread-id :wat::kernel::ThreadId)))

(:wat::core::enum :wat::kernel::services::StdErrService::Event
  (Write  (line :wat::core::String))
  (Add    (thread-id :wat::kernel::ThreadId)
          (data-rx   :wat::kernel::Receiver<wat::kernel::services::StdErrService::Event>)
          (ack-tx    :wat::kernel::Sender<wat::core::nil>))
  (Remove (thread-id :wat::kernel::ThreadId)))
```

All channels carry `Event`. Service's select-set is uniformly
`Vec<Receiver<Event>>`. The service matches on the variant to
determine action:
- `Read` / `Write` variants → do the work; reply/ack on the
  paired second-end (looked up via select index → routing
  table)
- `Add` variant → store `(data-rx, reply/ack-tx)` in routing
  table keyed by thread-id; rebuild select-set on next TCO
  iteration
- `Remove` variant → drop the routing-table entry; rebuild
  select-set

**Signal becomes Add/Remove variants of Event.** Pass-16/17's
"per-service Signal enum" language collapses into "per-service
Event enum"; the Add/Remove subset is structurally identical
to pass-16's Signal but lives alongside the data variants.
Pass 17's "per-service" discipline carries forward — three
independent Event types, one per service.

**The principle:** when select-monotyping pressure surfaces,
**collapse the heterogeneous flows into a sum type, not bridge
them with workarounds.** The Event enum is the right
abstraction; the relay was scar tissue from accepting a fixed
channel-end type prematurely.

**Implications:**

1. **Slice 1f-α requires reshape (existing slice retroactively
   modified).** `ThreadIO`'s per-thread sender becomes
   `Sender<Event>` instead of `Sender<()>` / `Sender<String>`.
   `println` / `eprintln` / `readln` construct the
   appropriate Event variant (`Write line` / `Read`) before
   sending. Caller-facing primitives stay polymorphic in value
   type at the user surface; only the channel-payload type
   changes. The 10 test rows in
   `tests/wat_arc170_slice_1f_alpha_helpers.rs` migrate to the
   new Event shape. Per user direction 2026-05-10: *"we fix
   what we break once the idealized shape is realized... they
   are us and we are fixing our patterns."*

2. **Slice 1f-β-i reshapes.** Original opus deliverable's
   relay-sub-thread + two-channel topology retire
   (working-tree-only; never committed; discarded). New shape:
   unified Event enum + homogeneous select + mechanical TCO
   loop matching on variants.

3. **Test-harness rot diagnosed (independent foundation crack).**
   Slice 1e retired the four-arg `:user::main (stdin stdout
   stderr) -> ()` signature; the wat-side `:wat::test::deftest`
   macro at `wat/test.wat:315` was not migrated. Every deftest
   in the workspace fails with the retired-signature error;
   the 855-failure baseline since slice 1e shipped is THIS rot.
   A pre-slice fixes the macro to emit the new `() ->
   :wat::core::nil` shape — expects fail count to drop near
   zero.

4. **Heterogeneous-select substrate primitive NOT needed.** The
   Event-enum approach handles this cleanly within the existing
   monotyped select. The hypothetical `select-heterogeneous`
   primitive surfaces no longer; one less substrate extension
   to maintain.

**Slice 1f stones — re-ordered to address foundation cracks first:**

| Stone | Scope | Status |
|---|---|---|
| 1f-0a | Migrate `:wat::test::deftest` macro at `wat/test.wat:315` to emit `() -> :wat::core::nil` user::main; expect 855-failure baseline → near zero | pending |
| 1f-0b | `src/thread_io.rs` ThreadIO + `println` / `eprintln` / `readln` reshape to per-service Event enum; migrate slice 1f-α's 10 test rows | pending |
| 1f-β-i | wat-side StdInService implementing unified Event protocol (no relay) | pending (was: opus working tree, discarded) |
| 1f-β-ii | StdOutService — mechanical pattern application | pending |
| 1f-β-iii | StdErrService — mechanical pattern application | pending |
| 1f-γ | Runtime orchestrator + spawn-thread integration | pending |
| 1f-δ | Runtime boot integration + scope-drop shutdown cascade | pending |
| 1f-ε | Console retirement + consumer sweep | pending |

**Connects to memory:**

- `feedback_attack_foundation_cracks.md` — opus's relay was a
  bridge over a wrong-framing crack; pivot forward into the
  cleaner type-shape that eliminates the bridge
- `feedback_substrate_already_typed.md` — the substrate's
  `:wat::kernel::select` already accepts any T; the constraint
  was self-imposed by fixing channel-end types prematurely
- `feedback_probe_before_framing.md` — opus's "we need a
  relay" framing was a type-theoretic bridge over what is
  actually a sum-type entity-kind problem; collapse to a sum,
  don't bridge it
- `feedback_collapse_to_llm_in_loop.md` — same shape: when a
  workaround surfaces, check if the underlying type-system
  pressure is real or just unexamined
- `feedback_no_known_defect_left_unfixed.md` — the
  855-failure baseline was diagnosed; foundation slice 1f-0a
  fixes it before extending
