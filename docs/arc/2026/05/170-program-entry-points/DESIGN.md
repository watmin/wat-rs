# Arc 170 — Program entry-point contracts + `:user::main` argv

**Status:** DESIGN settled 2026-05-09 across the conversation thread.
Ready for slice 1 authorship.

**Blocker for:** arc 109 v1 milestone closure.

---

## The mental model — client / server

> **Companion concept doc:** [`TIERS.md`](./TIERS.md) — runtime
> tiers 0 (eval env) → 1 (threads) → 2 (processes) → 3 (remote
> programs). Each tier shares less than the previous. Hermeticness
> is the ambient property of tier ≥ 2 — not a label or flag, but
> what the OS-process boundary inherently provides (memory + signal
> + global-state + runtime-sealing isolation, all at once because
> they're all manifestations of the same boundary).

Every "spawn a wat program in some context" primitive is, at its
heart, a **client / server** relationship:

- The **client** is the context that wants isolated work performed.
- The **server** is the spawned wat **fn** — code with a typed
  `(I, O)` shape per the variant's contract.
- Communication is a `(tx, rx)` pair on each side; client's `tx` ↔
  server's `rx`, server's `tx` ↔ client's `rx`.

Each spawn variant differs ONLY in the IPC mechanism connecting
client to server:

| Tier | Variant | User-visible IPC | Substrate transport | Sharing | Substrate primitive | Server contract |
|---|---|---|---|---|---|---|
| **1** | Thread | `Sender<T>` / `Receiver<T>` | crossbeam channels (in-memory typed Values) | memory shared | `(:wat::kernel::spawn-thread fn)` | `:user::thread` — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| **2** | Process | `Sender<T>` / `Receiver<T>` | EDN-over-pipes (substrate encodes/decodes) | host shared, memory boundary (hermetic ambient) | `(:wat::kernel::spawn-process fn)` | `:user::process` — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| **3** | Remote *(future)* | `Sender<T>` / `Receiver<T>` (Q-channel multiplex) | EDN-over-sockets | network shared, host boundary | `(:wat::kernel::spawn-remote-program fn endpoint)` | `:user::remote-program` — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |

(Tier 0 — `(f x y)` direct invocation in the current eval env — isn't a "spawn variant"; it's the base layer. See [`TIERS.md`](./TIERS.md).)

**Same user-visible abstraction at every tier.** The user writes
the same `(rx :Receiver<I> tx :Sender<O>)` shape regardless of
tier; the substrate handles transport encoding (none for tier 1;
EDN-over-pipes for tier 2; EDN-over-sockets for tier 3). WatAST
serializes to EDN by nature; users never see strings flowing
through these channels.

**The OS-boundary exception — `:user::main`:** the wat-cli IS the
OS-boundary, and the OS shell speaks bytes. `:user::main` keeps
`IOReader`/`IOWriter` for stdin/stdout/stderr; argv stays
`:Vector<String>`. This is the ONE place strings remain at the
user-visible level — because it's where wat meets the OS, and the
OS is bytes. Wat-internal spawning (tier 2 + tier 3) is wat-to-wat
and works in forms.

Every server fn returns `:nil` (Program contract per arc 114 —
"values flow only through channels; return is a panic-free
marker"). The IPC mechanism is implementation detail.

The **CLI** (`wat my-file.wat ...`) is a fourth, special-case
context. It's not a Program-server in the client/server sense —
it's the OS-level boundary where a wat binary runs against the
shell. The OS expects an exit code; the shell may pass `argv`.
That's `:user::main`'s contract — distinct from the three Program
contracts.

---

## The API — `spawn-* fn`

Every spawn primitive takes a fn directly. The fn IS the program.
No discovery, no Program wrapper type, no entry-keyword — the
substrate uses the fn's own definition + its closure as the
program description.

```scheme
;; Thread — runs in parent's world; closures over let-scope work
(:wat::core::let
  [worker-fn (:my::factory my-config)
   thr (:wat::kernel::spawn-thread worker-fn)]
  ...)

;; Process — substrate extracts fn + dep closure; forks OS process
(:wat::core::let
  [worker-fn (:my::factory my-config)
   proc (:wat::kernel::spawn-process worker-fn)]
  ...)

;; Remote (future arc) — substrate serializes fn + closure;
;; ships over wire; remote freezes + invokes; returns RemoteProgram<I,O>
(:wat::core::let
  [worker-fn (:my::factory my-config)
   rp (:wat::kernel::spawn-remote-program worker-fn endpoint)]
  ...)
```

The asymmetry is **internal**:

- spawn-thread runs the fn in parent's world. No serialization needed.
- spawn-process closure-extracts the fn (free symbols, deps, captured
  values, portability check), bundles into Vec\<WatAST\>, forks via
  the existing `fork-program-ast` pathway.
- spawn-remote-program (future) does the same closure extraction,
  serializes the resulting Vec\<WatAST\> to EDN bytes, ships over a
  socket, remote freezes + invokes.

**Closure extraction is substrate-internal Rust plumbing.** It does
NOT surface as a wat-level value type or verb in arc 170. Future
remote-program arc may expose it (for serialization to disk) at
that arc's discretion.

---

## Why this arc is bigger than "add argv"

The conversation thread settled the architecture across roughly a
dozen exchanges. The chain:

1. `:user::main` getting argv requires changing its return to
   `:wat::kernel::ExitCode` (CLI must communicate exit code to OS
   beyond panic-vs-no-panic).
2. Per arc 114, Program\<I,O\> bodies return `:nil`; values flow
   through channels/pipes. CLI's `:ExitCode` return is incompatible
   with the Program contract.
3. Therefore `:user::main` (CLI) and Program-spawned entries must be
   DIFFERENT symbols with different signatures.
4. spawn-process, fork-program, fork-program-ast all currently
   invoke `:user::main`. They must switch to a Program-shaped entry
   contract.
5. The "name discovery" path (substrate looks up a canonical entry
   symbol) creates ceremony. The user's preference: **the fn IS the
   program**; pass it directly; substrate handles closure extraction
   internally.
6. spawn-thread already accepts a fn (or keyword path); it stays
   structurally the same. spawn-process gains the same surface — fn
   in, Process out.
7. fork-program* renames to spawn-process* for verb family
   consolidation.
8. spawn-program (in-thread fresh-world variant) retires entirely —
   two-mode taxonomy (parent's world OR forked) is honest.
9. wat-cli passes std::env::args() to `:user::main` (pure
   passthrough; no flag filtering).

Each decision implies the next. No subset can ship honestly without
leaving the substrate in a half-state.

---

## What ships

### 1. `:user::main` signature update

```scheme
;; before
(:wat::core::defn :user::main
  [stdin  <- :wat::io::IOReader
   stdout <- :wat::io::IOWriter
   stderr <- :wat::io::IOWriter]
  -> :wat::core::nil
  ...)

;; after
(:wat::core::defn :user::main
  [stdin  <- :wat::io::IOReader
   stdout <- :wat::io::IOWriter
   stderr <- :wat::io::IOWriter
   argv   <- :wat::core::Vector<wat::core::String>]
  -> :wat::kernel::ExitCode
  ...)
```

argv layout (pure passthrough — what the binary received is what
the program sees):

| Position | Contents |
|---|---|
| `argv[0]` | Path to the wat binary |
| `argv[1]` | Path to the wat source file |
| `argv[2..N]` | Subsequent whitespace-delimited args |

`:user::main` is the **only strict name** in arc 170. Substrate
enforces this exact name + signature at freeze.

### 2. `:wat::kernel::ExitCode` typealias

```scheme
(:wat::core::typealias :wat::kernel::ExitCode :wat::core::u8)
```

POSIX truth (0-255). Bodies write `(:wat::core::u8 0)` for success;
non-zero values propagate to OS.

### 3. `:user::process` and `:user::thread` — documentation contracts

These are NOT strict names. They're contract names used in
documentation; the substrate enforces only structurally (signature
match). Programs name their actual entries whatever they want
(`:my::accountant::loop`, `:my::ddb::worker`, etc.); fns satisfying
the signature satisfy the contract.

```scheme
;; :user::process contract — the SHAPE the user's fn must satisfy
;;   (post arc 170 + typed-channel substrate)
[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>]
  -> :wat::core::nil

;; :user::thread contract — same shape
[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>]
  -> :wat::core::nil

;; example: a defn satisfying :user::process
(:wat::core::defn :my::worker
  [rx <- :wat::kernel::Receiver<i64> tx <- :wat::kernel::Sender<i64>]
  -> :wat::core::nil
  (... loop receiving from rx, sending to tx ...))
```

### 4. `spawn-process` / `spawn-process-ast` — verb consolidation + fn input

Renames + reshapes:

| Pre-arc-170 | Post-arc-170 |
|---|---|
| `(:wat::kernel::fork-program src scope)` invoking `:user::main` | `(:wat::kernel::spawn-process fn)` — fn satisfies `:user::process` contract; substrate closure-extracts and forks |
| `(:wat::kernel::fork-program-ast forms scope)` invoking `:user::main` | `(:wat::kernel::spawn-process fn)` — same canonical surface; AST-of-fn IS the input |
| `(:wat::kernel::spawn-program src scope)` (in-thread Process) | DELETED — two-mode taxonomy (parent's world OR forked) |
| `(:wat::kernel::spawn-program-ast forms scope)` | DELETED |

Note the consolidation: after arc 170, there's ONE spawn-process
verb. It takes a fn. Substrate does the heavy lifting (closure
extraction; fork). No `-ast` variant — the fn's body IS the AST
already; no separate AST-input verb needed.

### 5. `spawn-thread` — contract naming, no behavioral change

```scheme
;; UNCHANGED behavior
(:wat::kernel::spawn-thread my-fn)   ;; -> :Thread<I,O>
```

spawn-thread already accepts a fn (or keyword path). The fn's body
runs in parent's world; closures over let-scope work; services
pattern preserved.

The arc 170 change to spawn-thread is naming the contract
`:user::thread` in the documentation. Substrate's existing
structural enforcement is unchanged.

### 6. wat-cli passes `std::env::args()` to `:user::main`

```rust
// crates/wat-cli/src/lib.rs::run
let argv: Vec<String> = std::env::args().collect();
// ... existing flag parsing ...
let main_args: Vec<Value> = vec![
    Value::io__IOReader(stdin),
    Value::io__IOWriter(stdout),
    Value::io__IOWriter(stderr),
    Value::vector_of_strings(argv),  // ← arc 170 addition
];
let exit_code = invoke_user_main(&world, main_args)?;
std::process::exit(exit_code as i32);
```

Pure passthrough. wat-cli's flags (`--check`, `--check-output`)
appear in argv if passed; they short-circuit before `:user::main`
runs (so user's main never sees them when they're set).

### 7. Substrate-internal closure extraction (Rust capability)

Not exposed at wat level in arc 170. Internal Rust capability that
spawn-process uses to package a fn for the child process.

Detailed in [`CLOSURE-EXTRACTION.md`](./CLOSURE-EXTRACTION.md) — the
algorithm, the portability check, the test strategy.

The same capability sets up future remote-program (which will
serialize the extracted closure to EDN bytes for over-the-wire
transport).

---

## Substrate impact (full table)

| Surface | Pre-arc-170 | Post-arc-170 |
|---|---|---|
| `:user::main` signature | `(IOReader IOWriter IOWriter) -> :nil` | `(IOReader IOWriter IOWriter Vector<String>) -> :wat::kernel::ExitCode` |
| `:user::process` contract | n/a | documentation contract — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| `:user::thread` contract | n/a | documentation contract — `[rx <- :wat::kernel::Receiver<I> tx <- :wat::kernel::Sender<O>] -> :wat::core::nil` |
| `:wat::kernel::ExitCode` | n/a | typealias for `:wat::core::u8` |
| `:wat::kernel::Program<I,O>` typealias | aliases `:Process<I,O>` (arc 109 § J slice 10a) | UNCHANGED — kept as a typealias for now; future arc may unify Thread + Process under a real Program supertype |
| `:wat::kernel::fork-program` | live; calls `:user::main` | DELETED → spawn-process |
| `:wat::kernel::fork-program-ast` | live | DELETED → consolidated into spawn-process(fn) |
| `:wat::kernel::spawn-program` | live; in-thread Process | DELETED |
| `:wat::kernel::spawn-program-ast` | live | DELETED |
| `:wat::kernel::spawn-process` | n/a | minted; takes fn; invokes via closure-extracted forms in forked process |
| `:wat::kernel::spawn-thread` | takes fn; runs in parent world | UNCHANGED behaviorally; contract conceptually named `:user::thread` |
| Closure extraction (Rust internal) | n/a | minted; substrate Rust capability used by spawn-process |
| `validate_user_main_signature` | enforces 3-arg + nil | enforces 4-arg + ExitCode |
| wat-cli argv plumbing | empty Vec | std::env::args() |
| wat-cli exit code handling | exit 0 / 1 | exit u8-from-Value / 1 |

---

## Settled design (from conversation thread 2026-05-09)

### ExitCode = `:wat::core::u8` typealias

Per "u8 is the honest path." POSIX truth (0-255). Substrate's
existing u8 with range-checked cast suffices. Typealias adds
semantic clarity at signatures without minting a new value type.

### argv only at the CLI boundary

Argv is OS-level concept. Substrate-spawned programs have client/
server semantics with channel/pipe IPC. `:user::process` and
`:user::thread` don't take argv.

### Server-by-fn, not server-by-name

Per "convenience broke through incorrectly... brutal rigidity
brings the paradoxical unbounded flexibility if you play by the
rules" + "the shape of those functions /is/ the contract."

Each spawn primitive takes a fn directly. The fn IS the program.
No canonical lookup name; no entry-keyword; no Program wrapper
type. Substrate uses the fn's own definition + its closure as the
program description.

### Client / server framing

The unifying mental model. `:user::thread`, `:user::process`,
`:user::remote-program` are all server roles; each has its own
entry contract reflecting its IPC mechanism. The spawning context
is the client.

### Closure extraction is Rust-internal in arc 170

Per "we can build this first then use it as a dependency in the
rest of the re-work." The closure extraction logic is the
load-bearing substrate capability. Internal-only at arc 170; future
remote-program arc may expose it at the wat level if useful.

### The fn IS the program

Per "i think 'forms' is just equivalent of what :user::main is.
the forms /are/ the function being called on the inputs it must
receive." spawn-process takes a fn; substrate handles all
internals; no Program wrapper type at wat level.

---

## Out of arc 170 scope (affirmative)

Each item below is a future arc. Numbers reserved when the work
begins.

### `:user::remote-program` entry contract + spawn-remote-program primitive

scratch/2026/05/007-remote-program/ articulates the full
RemoteProgram architecture: typed-capability-bridge framing,
Q-channel multiplexed `Result<T, E>` wire protocol, four-tier IPC
(UDS / localhost HTTP / TLS / mTLS), seven settled questions.

Arc 170's closure extraction substrate capability sets up the
remote-program work — once a fn can be turned into portable
Vec\<WatAST\>, those bytes can go anywhere bytes go (forked process
in arc 170; over a wire in the remote-program arc).

The remote arc adds: transport (sockets, auth, mTLS), wire
protocol (Q-channel multiplex), endpoint addressing, the
`:user::remote-program` contract.

### Real `Program<I,O>` supertype unification

Today `Program<I,O>` is a typealias for `Process<I,O>`. Thread\<I,O\>
+ Process\<I,O\> remain separate concrete types. A future substrate
arc may unify them under a real Program supertype (abstracting over
IPC). Arc 170 keeps the existing types unchanged.

### wat-cli-options DSL + `user:` subcommand convention

scratch/2026/05/019-wat-cli-options/ captures both. Out of arc
170 — argv parameter lands; users hand-roll parsing or build their
own helpers until that arc opens.

### Wat-level closure extraction verb

If users want to serialize a fn to disk or build their own
transport (other than fork or remote-program), exposing the closure
extraction primitive at the wat level becomes useful. Arc 170 keeps
it Rust-internal; later arc may surface it.

---

## Slice plan

Mirrors the substrate-as-teacher pattern from arcs 167 / 168 /
169. Seven slices (six original + slice 1b reshape).

### Slice 1 — closure extraction (Rust substrate; zero callers) — SHIPPED

Build the foundation primitive in Rust. Zero wat-level callers
initially; testable in isolation.

- Free-symbol walker
- Dep-closure builder (recursive)
- Value→AST encoder (extending existing struct→form)
- Portability type-check (channel-bearing values refused)
- Rust integration tests verifying extracted forms re-freeze
  correctly

Detailed in [`CLOSURE-EXTRACTION.md`](./CLOSURE-EXTRACTION.md).

**Shipped**: commit `787c977` + SCORE `bb155ed` (14/14 pass,
Mode A clean). **However** — review surfaced that the public
shape of `ClosurePackage { forms, entry }` carries the
entry-keyword ceremony DESIGN explicitly killed (lines 102-108
+ 484-509). See
[`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md) for the
discipline lesson + slice 1b origin.

Predicted (original): 90-180 min opus. Actual: ~150 min Mode A
clean.

### Slice 1b — `ClosurePackage` reshape ("the fn IS the program") — SHIPPED

Restructures `ClosurePackage` from `{ forms, entry: String }` to
`{ prologue: Vec<WatAST>, entry_form: WatAST }`. Retires the
synthetic-name machinery (`:__closure::__pkg_<n>` counter +
wrap-in-define). The fn-form AST evaluates to a fn Value
directly; no naming required.

**Shipped**: commits `a23acf3` + `365343f` + SCORE `84b6ca6`
(17/17 pass, Mode A clean, ~40 min — under predicted 60-120 min
band). One substantive honest delta surfaced: keyword-path
`entry_form` ships as `WatAST::Keyword`, not `WatAST::Symbol`,
because eval resolves bare-Symbol via `env.lookup` (lexical only)
while top-level defns require keyword resolution via `sym.get`.
Spec intent ("name reference that evaluates to fn Value")
preserved; surface adjusted for substrate-fit.

### Slice 1c — typed-channel-over-EDN-pipes substrate + Process<I,O> reshape

Tier 2 substrate plumbing. Mints the EDN-encoded-pipe transport
behind `Sender<T>` / `Receiver<T>` so the user-visible abstraction
is uniform across tier 1 (crossbeam) and tier 2 (pipes). Reshapes
`:wat::kernel::Process<I,O>` to expose typed-channel handles
instead of byte-pipe handles, per the doctrine that strings stay
at substrate-internal transport boundaries.

- `Sender<T>` / `Receiver<T>` Value variants extended (or
  multimethod-dispatched) to support pipe-fd transport with
  EDN encoding/decoding at the boundary
- `:wat::kernel::Process<I,O>` struct reshape:
  `{ stdin :IOWriter, stdout :IOReader, stderr :IOReader, handle }`
  → `{ tx :Sender<I>, rx :Receiver<O>, handle :ProgramHandle }`
- All Process-callers in `src/fork.rs`, `src/spawn.rs`, etc.
  updated for new field shape
- Errors propagate via `Process/join-result` (existing arc 113
  cascade pattern); stderr-as-separate-channel drops at the
  user-visible level
- Rust integration tests verifying typed-channel-over-pipes
  round-trips end-to-end (parent sends typed Value; child
  receives typed Value; bytes are EDN-encoded transport detail)

Zero wat-level surface change in this slice — pure substrate
plumbing. The wat-level verbs that USE this infrastructure
(`spawn-process`, etc.) land in slice 2.

Predicted: 90-180 min opus.

### Slice 2 — substrate consumer (uses slices 1b + 1c)

- `:wat::kernel::ExitCode` typealias
- `:user::main` signature update + validator
- `eval_kernel_spawn_process` minted; uses slice 1's closure
  extraction; reaches today's fork-program-ast pathway internally
- `eval_kernel_fork_program*` arms renamed → `eval_kernel_spawn_process*`
- `eval_kernel_spawn_program*` deleted (Q1 settled)
- wat-cli pass-through of `std::env::args()`
- Substrate-as-teacher walkers fire on legacy main signature +
  legacy fork-program/spawn-program verb usage
- `tests/wat_arc170_program_contracts.rs` — integration tests

Predicted: 90-180 min opus.

### Slice 3 — consumer sweep + tooling rebuild

Arc 170 changes the spawn surface (tier 1/2/3 framework; fn-input
contract; closure-extraction tier-bridging at tier ≥ 2). Every
existing user-facing thing that interfaces with the spawn family
MUST reach its polished form on the new substrate — not just
functionally work, but **as good as the new substrate allows**.

**Mechanical sweep (sonnet):**

- All `:user::main` definitions:
  - CLI-only programs: migrate to new signature (add argv,
    change return to ExitCode)
  - Substrate-spawn-target programs: rewrite as anonymous fn
    satisfying `:user::process` contract; callers pass the fn
    directly to spawn-process
  - Both contexts: define both
- `fork-program*` callsites → `spawn-process(fn)` shape
- `spawn-program*` callsites → migrate to `spawn-process(fn)` (real
  fork) OR `spawn-thread(fn)` (parent world; services pattern)

**Tooling rebuild — testing-lib three-layer API (orchestrator + sonnet pair, more than mechanical):**

The substrate (`spawn-process fn`) is the full-power form. The
testing lib's job is to hide constant ceremony for typical test
usage. Three layers per [`TIERS.md`](./TIERS.md):

- **Layer 1 — `(:wat::test::run-hermetic body)`** — the 90% case.
  `run-hermetic` is a macro; user writes the body directly; the
  fn-wrapper (`(:wat::core::fn [] -> :wat::core::nil body)`) is
  generated. No channels in the signature, no inputs, no scope,
  no fn ceremony at all. Returns `RunResult`.
- **Layer 2 — `(:wat::test::run-hermetic-with-io<I,O> inputs body)`** —
  the 9% case. Macro introduces `rx :Receiver<I>` and
  `tx :Sender<O>` as bindings in scope of `body`; harness feeds
  Values via rx, drains Values via tx, returns parsed outputs.
  **Typed channels, not byte streams.** Substrate handles
  EDN-over-pipes encoding internally.
- **Layer 3 — `(:wat::kernel::spawn-process fn)`** — the
  substrate; full surface for production code. Caller writes an
  explicit fn-form with typed-channel signature. Tests don't
  reach here unless they really need it.

What disappears from EVERY testing layer:

- `scope :Option<String>` — leaked substrate plumbing; today's
  hermetic.wat errors on `:Some`; not functional anyway. Drops.
- `forms :Vector<WatAST>` — caller writes a fn directly; no AST
  construction.
- **`stdin :Vector<String>` and stdout/stderr as `Vec<String>`** —
  string-shaped IO drops from every testing layer. The user
  works in typed Values; the substrate handles EDN encoding at
  the pipe boundary.
- **`IOReader`/`IOWriter`** — byte-stream types drop from every
  testing layer. Layer 2's fn takes `Receiver<I>`/`Sender<O>`
  (typed channels); the substrate's pipe-fd plumbing is hidden.

What disappears from Layer 1:

- The fn's channel parameters (Layer 2 has them when needed)
- The input data parameter (Layer 2 has it when needed)
- **The fn-form wrapper itself** — `run-hermetic` is a macro;
  user writes the body directly; macro generates
  `(:wat::core::fn [] -> :wat::core::nil body)` internally. The
  empty params + nil return are constant ceremony hidden from
  every Layer 1 caller.

What disappears from Layer 2:

- **The fn-form wrapper** — `run-hermetic-with-io` is a macro;
  it introduces `rx` and `tx` as bindings in the body scope.
  User writes body using rx/tx directly; macro generates the
  fn-form with typed-channel signature.

**Migration scope:**

- **`wat/std/hermetic.wat` retires.** Replaced by
  `:wat::test::run-hermetic` (Layer 1) + `:wat::test::run-hermetic-with-io`
  (Layer 2) under the testing namespace. Path may move to
  `wat/test/` or similar; slice 3's BRIEF settles it.
- **`wat/test.wat`** — references fork-program-ast; same
  three-layer treatment per its own surface.
- **All callers of `run-sandboxed-hermetic-ast`** — classify by
  which layer they need; migrate to the appropriate layer.
  Expected distribution: most → Layer 1 (massive UX collapse);
  some → Layer 2; rare/none → Layer 3.
- **Any other stdlib that wraps the spawn family** — same
  three-layer treatment; reach the polished form.

The slice is sonnet-mechanical for the verb renames + signature
updates, but the hermetic tooling rebuild + caller migration is
orchestrator-judged. Slice 3's BRIEF (drafted post-slice-1b +
post-slice-2) splits these explicitly.

Predicted: 90-180 min sonnet for the mechanical sweep; orchestrator
time for the hermetic rebuild + caller migration TBD when slice 3
is briefed.

### Slice 4 — substrate retirement (opus + sonnet pair)

**This slice retires every bandaid arc 170 carried during its
sweep window.** The arc cannot close with bandaids in place;
INSCRIPTION reflects the final correct shape. Arc 170 INSCRIPTION
must be free of any "future arc retires X" deferral language
per FM 11.

Walker + dispatch-arm retirements:
- `BareLegacyMainSignature` walker variant + Display + Diagnostic + body
- `BareLegacyForkProgram` walker variant + Display + Diagnostic + body
- `BareLegacySpawnProgram` walker variant + Display + Diagnostic + body
- Old `eval_kernel_fork_program*` / `eval_kernel_spawn_program*` arms
- `validate_user_main_signature` legacy 3-arg fall-through
- Vacuous walker-firing tests retired

**`:wat::kernel::Process<I,O>` legacy field retirement (slice 1c
bandaid retirement) — load-bearing for arc 170 close:**

Slice 1c shipped Process<I,O> ADDITIVE — legacy 4 fields
(stdin :IOWriter, stdout :IOReader, stderr :IOReader, handle)
PLUS new typed-channel fields (tx :Sender<I>, rx :Receiver<O>).
The legacy 3 byte-pipe fields (stdin, stdout, stderr) MUST
retire in slice 4 — that's the arc-close discipline. Slice 3's
testing-tooling rebuild migrates ALL callers to typed-channel
accessors; slice 4 destructively removes the legacy fields:

- `:wat::kernel::Process<I,O>` struct in `src/types.rs` —
  remove stdin / stdout / stderr fields; final shape is
  `{ tx :Sender<I>, rx :Receiver<O>, handle :ProgramHandle }`
- All Rust callers updated for 3-field shape (workspace breaks
  at substrate destructive-edit; sonnet sweeps any remaining
  callers slice 3 missed)
- All wat-side accessors (`Process/stdin`, `Process/stdout`,
  `Process/stderr`) retired
- Atomic-commit pattern: opus destructive (don't commit) →
  sonnet sweep (don't commit) → orchestrator commits both as
  ONE atomic commit when workspace = 0-failed

The bandaid was tolerable during the sweep window (kept slice 2
unblocked, kept stdlib tests green during slice 1c → slice 3
transition). It is NOT tolerable past arc close.

Predicted: 60-120 min opus + 30-90 min sonnet sweep =
~90-210 min total.

### Slice 5 — closure paperwork (orchestrator)

- SCORE-SLICE-1, SCORE-SLICE-2, SCORE-SLICE-3, SCORE-SLICE-4
- INSCRIPTION
- 058 changelog row (lab repo)
- USER-GUIDE update (Program client/server section + entry
  contracts + ExitCode + argv + spawn primitives + closure
  extraction note)
- ZERO-MUTEX cross-ref (no new mutex)
- CONVENTIONS doc update (entry-point naming convention)
- Atomic squash-merge to main

When slice 5 ships, **arc 109 v1 milestone closure unblocks**.

---

## Settled questions (conversation log compressed)

### Q1. spawn-program retires entirely. — settled (option A)

Per "A — retire." spawn-program / spawn-program-ast deleted. Two-
mode taxonomy: spawn-thread (parent's world) + spawn-process
(forked).

### Q2. *(retired — was based on wrong assumption)*

Q2 v1 asked about spawn-thread freeze cost; user correction:
threads share parent's memory (zero-mutex doctrine; services
pattern). Q retired.

### Q2-replacement. Contract names are documentation; only `:user::main` is strict. — settled

Per "the shape of those functions /is/ the contract.. i think the
only strict one is :user::main?.."

### Q3. argv pure passthrough. — settled

Per scratch/2026/05/019-wat-cli-options: "no silent argv reshaping;
what the binary received is what the program sees."

### Q4. Substrate-then-sweep parallel pattern. — settled

Per arcs 167 / 168 / 169 precedent. Slice 1 substrate ships;
slice 2 substrate consumer ships with walkers; slice 3 sweeps;
slice 4 retires. Same shape.

### Q5. spawn-process input shape — fn, not Program-typed value. — settled

Initial drafts proposed (src, scope, entry-keyword) tuple, then a
Program newtype with forms + entry. User refinement: the fn IS the
program; substrate handles closure extraction internally; no
wat-level Program type or entry-keyword; spawn-process takes a fn
directly.

### Q6. Closure extraction substrate scope — Rust-internal in arc 170. — settled

Per "we can build this first then use it as a dependency in the
rest of the re-work." Closure extraction is the load-bearing
substrate capability that enables both spawn-process AND future
spawn-remote-program. Built as a Rust capability in arc 170;
exposed at wat level later if needed.

---

## DESIGN-time conversation log (2026-05-09)

The architecture settled across roughly 18 exchanges. Key beats:

1. User: "make :user::main accept argv"
2. Three return-type options presented (i64 / u8 / enum); user picked u8
3. User proposed `(:wat::kernel::exit N)` helper; orchestrator pushed back on libc-naming conflict; user picked typealias path (`:wat::kernel::ExitCode = :u8`)
4. Orchestrator scoped arc 170 narrowly (just argv); user pushed: spawn/fork-program don't need argv
5. User raised: forks/threads might not need :user::main at all
6. Orchestrator surfaced Program\<I,O\> + arc 114's "return nil" contract; arc 170 must split CLI from Program contracts
7. User proposed three named entries: `:user::main` (CLI), `:user::thread` (thread Program), `:user::process` (process Program); plus `:user::remote-program` future
8. User confirmed kill spawn-thread fn-ref-pattern; orchestrator surfaced verb consolidation: fork-program* → spawn-process*
9. User locked client/server framing
10. User confirmed "single very large arc with a bunch of slices"
11. User caught Q2 framing error: spawn-thread shouldn't reshape; threads share parent's memory; services pattern
12. User clarified contract-vs-name: only `:user::main` is strict; `:user::thread`/`:user::process` are documentation contracts
13. User settled Q1 (spawn-program retires); two-mode taxonomy
14. User questioned `child-src` exposing substrate plumbing (None scope param + raw String source); orchestrator pivoted toward AST-first
15. User locked AST-first: "we don't communicate strings - we communciate ast"
16. User questioned why entry-keyword is needed: "why do we even need a name if the forms /are/ the thing that matters?"
17. User refined further: the fn IS the program; spawn-process takes fn directly; no Program wrapper type; closure extraction is internal
18. User pressure-tested: does this set up remote-program? Confirmed yes — closure extraction + Vec\<WatAST\> wire form is the foundation; remote arc adds transport + protocol

Each refinement made the design simpler and more honest. The
final shape: spawn-* takes a fn matching its contract; substrate
handles all internals; no wat-level Program type; no entry-keyword;
no discovery; the fn IS the program.

---

## Cross-references

- **arc 109 (kill-std)** — broader substrate refactor wave; arc
  170 is one of its v1 closure blockers
- **arc 112 (inter-process Result\<Option\<T\>,E\>)** — fork-program
  stdin/stdout/stderr wire foundation; the Result.Err on stderr
  framing arc 170 inherits
- **arc 114 (spawn-as-thread)** — established the Program\<I,O\>
  contract: "values flow only through channels; return is a
  panic-free marker"
- **arc 167 (fn-flat-signature)** — substrate-as-teacher walker
  precedent
- **arc 168 (let-flat-shape)** — multi-slice substrate-refactor +
  sweep precedent
- **arc 169 (struct-destructure-form-a)** — most recent arc;
  closest precedent for arc 170's structure
- **scratch/2026/05/019-wat-cli-options/** — full vision for argv
  + wat-cli-options + user: subcommand convention
- **scratch/2026/05/007-remote-program/** — RemoteProgram
  architecture; the future arc that arc 170 sets up via closure
  extraction
- **arc 091 slice 8 (`struct->form`)** — Value→AST infrastructure
  arc 170's closure extraction extends
- **arc 092 (wat-edn)** — EDN serialization for AST; future
  remote-program transport will use this
- **arc 098 (matches?)** — pattern-grammar precedent for
  signature-shape contracts
- **arc 102 (eval-ast!)** — child-side AST evaluation; arc 170's
  closure extraction produces AST that goes through this pathway
- **arc 113 (`:wat::test::program`)** — forms-from-quoted-forms
  capture pattern; conceptual ancestor of closure extraction

---

## Companion docs

- [`CLOSURE-EXTRACTION.md`](./CLOSURE-EXTRACTION.md) — substrate
  primitive deep-dive: algorithm, invariants, portability check,
  test strategy. The load-bearing substrate work for arc 170.
