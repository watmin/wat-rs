# Arc 170 — Program entry-point contracts + `:user::main` argv

**Status:** DESIGN settled 2026-05-09. All open questions resolved.
Ready for slice 1 BRIEF authorship + opus spawn.

**Blocker for:** arc 109 v1 milestone closure (per arc 109
INVENTORY § M-equivalent — argv on `:user::main` is the missing
contract).

---

## The mental model — client / server

Every "spawn a wat program in some context" primitive is, at its
heart, a **client / server** relationship:

- The **client** is the context that wants isolated work performed.
- The **server** is the spawned wat **Program** — a piece of wat
  code with a typed `(I, O)` shape.
- Communication is a `(tx, rx)` pair on each side; client's
  `tx` ↔ server's `rx`, server's `tx` ↔ client's `rx`.

Each spawn variant differs ONLY in the IPC mechanism connecting
client to server:

| Variant | IPC mechanism | Sharing | Substrate primitive | Server entry symbol |
|---|---|---|---|---|
| Thread | Crossbeam channels (in-memory typed Values) | Same vm; same OS process; no isolation between client + server | `(:wat::kernel::spawn-thread src scope)` | `:user::thread` |
| Process | Pipes (stdin, stdout, stderr — byte streams) | Forked OS process; full process isolation; child can mutate its world without touching parent's | `(:wat::kernel::spawn-process src scope)` | `:user::process` |
| Remote | Sockets (UDS / localhost HTTP / TLS / mTLS); multiplexed `Result<T, E>` | Network or sidecar; remote host may have capabilities client doesn't (DDB/EFS/VPC resources) | `(:wat::kernel::spawn-remote-program ...)` *(future)* | `:user::remote-program` |

Every Program is identical at the wat-program level: receive `I`
via its `rx`, emit `O` via its `tx`, return `:nil` (per arc 114 —
"values flow only through channels; return is a panic-free
marker"). The IPC mechanism is implementation detail.

The **CLI** (`wat my-file.wat ...`) is a fourth, special-case
context. It's not a Program-server in the client/server sense —
it's the OS-level boundary where a wat binary runs against the
shell. The OS expects an exit code; the shell may pass `argv`.
That's `:user::main`'s contract — distinct from the three Program
contracts.

---

## Why this arc is bigger than "add argv"

User direction 2026-05-09 settled the architecture across the
conversation:

1. Adding `argv` to `:user::main` requires changing its return to
   `:wat::kernel::ExitCode` (CLI must communicate exit code to OS
   beyond just panic-vs-no-panic).
2. Per arc 114, Program\<I,O\> bodies return `:nil`; values flow
   through channels/pipes. CLI's `:ExitCode` return is INCOMPATIBLE
   with the Program contract.
3. Therefore, `:user::main` (CLI) and Program-spawned entries must
   be DIFFERENT symbols with different signatures.
4. spawn-program, fork-program, fork-program-ast all currently
   invoke `:user::main`. They have to switch to a Program-shaped
   entry symbol.
5. spawn-thread currently takes an arbitrary fn-ref, runs it in
   the parent's world. User direction:
   > *"i had wanted 'run this func in this thread' but that was
   > short sided... convenience broke through incorrectly...
   > brutal rigidity brings the paradoxical unbounded flexibility
   > if you play by the rules."*
6. The fn-ref pattern dies. spawn-thread reshapes to take a wat
   program source + scope, freezes a fresh world, invokes
   `:user::thread`. Same contract shape as spawn-program.
7. The verb family `(spawn-thread, spawn-process, spawn-remote-program)`
   is the consolidated naming. `fork-program` / `fork-program-ast`
   rename to `spawn-process` / `spawn-process-ast`.

These seven decisions form one coherent unit. No subset can ship
honestly without leaving the substrate in a half-state where some
contracts contradict others.

---

## What ships (the seven changes)

### 1. `:user::main` signature update

**Before:**

```scheme
(:user::main
  (stdin  :wat::io::IOReader)
  (stdout :wat::io::IOWriter)
  (stderr :wat::io::IOWriter)
  -> :wat::core::nil)
```

**After:**

```scheme
(:user::main
  (stdin  :wat::io::IOReader)
  (stdout :wat::io::IOWriter)
  (stderr :wat::io::IOWriter)
  (argv   :wat::core::Vector<wat::core::String>)
  -> :wat::kernel::ExitCode)
```

argv layout (pure passthrough — what the binary received is what
the program sees):

| Position | Contents |
|---|---|
| `argv[0]` | Path to the wat binary (e.g., `/usr/local/bin/wat`) |
| `argv[1]` | Path to the wat source file |
| `argv[2..N]` | Subsequent whitespace-delimited args |

Programs don't need to use argv — they can ignore it. But the
substrate enforces the parameter; every `:user::main` declares
argv in its signature.

### 2. `:wat::kernel::ExitCode` typealias

```scheme
(:wat::core::typealias :wat::kernel::ExitCode :wat::core::u8)
```

POSIX-honest: exit codes are 0-255. Substrate's existing `:u8`
provides range-checked construction (`(:wat::core::u8 0)` for
success; `(:wat::core::u8 1)` for general failure). The typealias
adds semantic clarity at the function signature without minting
a new value type.

Settled via four-questions 2026-05-09. i64 rejected for dishonesty
(loses range info). Enum (`Success | Failure(u8)`) rejected for
ceremony cost without proportional gain.

### 3. `:user::process` — contract name (documentation only)

The Program\<I, O\> contract for the **process** IPC variant has
this signature:

```scheme
;; Any function matching this shape satisfies the :user::process
;; contract. The substrate enforces the SHAPE structurally; it
;; does NOT look up a literal `:user::process` symbol.
(my::process-program
  (stdin  :wat::io::IOReader)
  (stdout :wat::io::IOWriter)
  (stderr :wat::io::IOWriter)
  -> :wat::core::nil)
```

`:user::process` is a CONTRACT NAME used in documentation and
specifications — not a substrate-enforced canonical entry symbol.
Programs name their actual entries whatever they want:
`:my::accountant::loop`, `:my::ddb::worker-process`,
`:my::etl::main`, etc. The signature shape IS the contract.

User direction 2026-05-09: *"the shape of those functions /is/
the contract.. i think the only strict one is :user::main?.."*
— **YES, only `:user::main` is a strict name.** `:user::thread`
and `:user::process` are signature contracts, not name slots.

Caller of `spawn-process` specifies which symbol in the child's
world to invoke; substrate validates the resolved symbol matches
the process contract signature.

### 4. `:user::thread` — contract name (documentation only)

Program\<I, O\> contract for the **thread** IPC variant:

```scheme
;; Any function matching this shape satisfies the :user::thread
;; contract. Substrate enforces shape structurally; no literal
;; `:user::thread` symbol is looked up.
(my::thread-program
  (rx :wat::kernel::Receiver<I>)
  (tx :wat::kernel::Sender<O>)
  -> :wat::core::nil)
```

Same as `:user::process`: `:user::thread` is a CONTRACT NAME for
documentation. Programs spawn arbitrary fns satisfying the shape
— inline lambdas, top-level fns at user-chosen keyword paths,
service-loop bodies. The current spawn-thread behavior is
preserved.

Multi-service programs declare 20 separate fns at unique
keyword paths (`:my::svc::accountant::loop`, `:my::svc::registry::loop`,
etc.); each satisfies the contract structurally.

### 5. Verb consolidation + entry-keyword parameter

`fork-program*` rename to `spawn-process*` AND gain an
entry-keyword parameter (caller specifies which symbol in
child's world to invoke):

| Pre-arc-170 | Post-arc-170 | Notes |
|---|---|---|
| `(:fork-program src scope)` invokes `:user::main` | `(:spawn-process src scope entry-kw)` invokes `entry-kw` of child's world | OS fork(2). entry-kw must resolve in child's world to a fn matching the `:user::process` contract signature. |
| `(:fork-program-ast forms scope)` invokes `:user::main` | `(:spawn-process-ast forms scope entry-kw)` invokes `entry-kw` of child's world | Same shape; takes pre-parsed forms instead of source string. |
| `(:spawn-program src scope)` invokes `:user::main` (in-thread) | DELETED (per Q1 settled) | The in-thread fresh-world variant. Tests using it migrate to spawn-process or spawn-thread during slice 2 sweep. |
| `(:spawn-program-ast forms scope)` invokes `:user::main` (in-thread) | DELETED | Same. |

The entry-keyword parameter is **always required** — no
implicit-default lookup. Caller always specifies which symbol to
invoke. Honest about which fn runs.

User direction 2026-05-09: rigidity = explicit; no
magic-default name lookup at the substrate level.

### 6. `spawn-thread` — contract naming, not reshape

**Correction 2026-05-09 (post-DESIGN-v1):** The earlier draft of
this section proposed reshaping spawn-thread to take (src scope)
and freeze a fresh world per call. That was WRONG.

User direction 2026-05-09: *"it should be typical thread
behavior... they share the process' memory.. i don't get the
question. look at how we do services... services should basically
implement this pattern.. they are request/reply servers already -
just not in a separate process -- this is the zero mutex
doctrine."*

Threads share parent's process memory (typical thread behavior).
Services (telemetry/Service.wat, wat-lru/CacheService.wat,
wat-tests/service-template.wat) already implement the
client/server request/reply pattern using spawn-thread + inline
lambda + closure over let-scope values + channel-based I/O. This
is THE pattern; arc 170 doesn't reshape it.

**The actual change to spawn-thread in arc 170 is naming the
contract.** Today the substrate enforces a structural signature
on spawn-thread's body fn: `(:Receiver<I>, :Sender<O>) -> :nil`
(per arc 114 — "values flow only through channels; return is a
panic-free marker"). Arc 170 names this contract `:user::thread`
in the documentation. Substrate continues to enforce the
signature structurally; the name is conceptual/specification —
NOT a substrate-enforced canonical entry point.

```scheme
;; UNCHANGED behavior: takes inline lambda or fn-ref keyword
(:wat::kernel::spawn-thread
  (:wat::core::fn [rx <- :Receiver<I> tx <- :Sender<O>] -> :nil
    body))                          ;; closes over parent's world
;; -> :wat::kernel::Thread<I,O>

;; Or by keyword path:
(:wat::kernel::spawn-thread :my::worker-fn)
;; where :my::worker-fn satisfies the :user::thread signature contract
```

**No fresh-world freeze. No closure restriction. The services
pattern is preserved.** The "rigidity" is the signature contract;
the "flexibility" is being able to spawn any fn matching it
(inline or named, at any keyword path).

### 7. wat-cli passes `std::env::args()` to `:user::main`

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

Pure passthrough. wat-cli's own flags (`--check`, `--check-output`)
are recognized BEFORE `:user::main` runs (they short-circuit), so
in practice user's main never sees them. But there's no filtering
— what `std::env::args()` returns is what main sees.

---

## Substrate impact (full table)

| Surface | Pre-arc-170 | Post-arc-170 |
|---|---|---|
| `:user::main` signature | `(IOReader IOWriter IOWriter) -> :nil` | `(IOReader IOWriter IOWriter Vector<String>) -> :wat::kernel::ExitCode` |
| `:user::process` | n/a | `(IOReader IOWriter IOWriter) -> :nil` minted |
| `:user::thread` | n/a | `(Receiver<I> Sender<O>) -> :nil` minted |
| `:wat::kernel::ExitCode` | n/a | typealias for `:wat::core::u8` |
| `:wat::kernel::fork-program` | live; calls `:user::main` | DELETED (rename to spawn-process) |
| `:wat::kernel::fork-program-ast` | live | DELETED (rename to spawn-process-ast) |
| `:wat::kernel::spawn-program` | live (in-thread); calls `:user::main` | DELETED (per Q1 settled — option A; two-mode taxonomy) |
| `:wat::kernel::spawn-program-ast` | live | DELETED |
| `:wat::kernel::spawn-process` | n/a | minted; takes (src scope); invokes `:user::process` |
| `:wat::kernel::spawn-process-ast` | n/a | minted; takes (forms scope); invokes `:user::process` |
| `:wat::kernel::spawn-thread` shape | `(fn-or-ref) → Thread<I,O>`; signature `(:Receiver<I>, :Sender<O>) -> :nil` enforced structurally | UNCHANGED — services pattern preserved; signature contract gets the conceptual name `:user::thread` (substrate still enforces structurally, not by canonical name) |
| `validate_user_main_signature` | enforces 3-arg + nil | enforces 4-arg (3-IO + argv) + ExitCode |
| `validate_user_process_signature` | n/a | minted |
| `validate_user_thread_signature` | n/a | minted |
| wat-cli argv plumbing | empty Vec passed to invoke_user_main | std::env::args() passed |
| wat-cli exit code handling | exit 0 on Ok, exit 1 on Err | exit u8-from-Value on Ok, exit 1 on Err |
| Workspace test count | 2091 / 0 (post-arc-169) | TBD post-slice-2 |

---

## Settled design (from conversation thread 2026-05-09)

### ExitCode = `:wat::core::u8` typealias

User: *"u8 it is - that's the honest path."*

POSIX truth (0-255), substrate's u8 already exists with range-
checked cast, no new value type minted. Typealias gives semantic
intent at signature without ceremony at the value level.

Rejected alternatives:
- `:i64` — dishonest (64 bits of fiction; OS truncates to 8)
- enum `Success | Failure(u8)` — ceremony tax without proportional clarity gain

### Argv only at the CLI boundary

User: *"so.. a wat forked program must deal with stdin, stdout,
stderr to communicate with the other side.. but argv... has no
meaning here.."*

Argv is an OS-level concept. Substrate-spawned programs have
client/server semantics with channel/pipe IPC. argv has no
meaning inside the substrate. `:user::process` and `:user::thread`
don't take argv.

### Server-by-name, not server-by-fn-ref

User: *"i had wanted 'run this func in this thread' but that was
short sided... brutal rigidity brings the paradoxical unbounded
flexibility if you play by the rules."*

Each spawn primitive invokes a CANONICAL entry symbol. Not
arbitrary closures from inside the parent's world. Programs
declare their entry; substrate enforces signatures. The
flexibility is "any wat program with the right entry can be
spawned" — not "any closure can be spawned."

### Client / server framing for IPC

User: *"the frame of mind now... the 'thing who requests isolated
work' becomes a client.. and the server is the program.. the
program has a (rx, tx) pair that uses to comm with the 'client'
(tx, rx) pair... they /are/ Program<I,O> but the flavor of each
needs to a different entry point to deal with IPC."*

The unifying mental model. `:user::thread`, `:user::process`,
`:user::remote-program` are all Program\<I,O\> servers; each has
its own entry-point name reflecting its IPC mechanism. Client
sees `Program<I,O>`; concrete IPC is implementation detail of
the spawn variant.

---

## Out of arc 170 scope (affirmative)

Each item below is a future arc. Numbers reserved when the work
begins; arc 170's INSCRIPTION does NOT commit to them.

### `:user::remote-program` entry contract + spawn-remote-program primitive

scratch/2026/05/007-remote-program/ articulates the full
RemoteProgram architecture: typed-capability-bridge framing,
Q-channel multiplexed `Result<T, E>` wire protocol, four-tier IPC
(UDS / localhost HTTP / TLS / mTLS), seven settled questions.
Substantial substrate work; future arc. Arc 170 reserves the
NAMED FUTURE entry symbol `:user::remote-program` in DESIGN docs
without minting it.

### Real `Program<I,O>` supertype unification

Today `Program<I,O>` is a typealias for `Process<I,O>` (arc 109
§ J slice 10a). Arc 170 keeps Thread\<I,O\> and Process\<I,O\> as
separate concrete types; uses Program\<I,O\> as conceptual
umbrella in docs only. Real supertype that abstracts over IPC
mechanism (channels vs pipes vs sockets) is a separate substrate
arc.

### wat-cli-options DSL

scratch/2026/05/019-wat-cli-options/ captures the declarative
argparse-style DSL. Out of arc 170 — argv lands; users hand-roll
parsing (or build their own CLI helpers) until that arc opens.

### `user:` subcommand convention

Same scratch doc. Wat-cli routing convention for user-extended
subcommands (`wat user:deploy ...`). Future arc.

### Substrate-side argv parsing helpers

If/when wat-cli-options ships, wat programs gain declarative
argv parsing. Until then, programs do their own parsing on
the `argv :Vector<String>` they receive.

---

## Slice plan

Mirrors arc 168 shape (substrate refactor + sweep + retirement +
closure). Four slices.

### Slice 1 — substrate consumer (opus)

Mint contracts + walkers + reshape primitives + integration tests.
Substrate-as-teacher walkers fire on every legacy shape during
the sweep window.

**Touchpoints:**

- `src/types.rs` — `:wat::kernel::ExitCode` typealias
- `src/freeze.rs::expected_user_main_signature` — update (4 args + ExitCode)
- `src/freeze.rs::expected_user_process_signature` — minted
- `src/freeze.rs::expected_user_thread_signature` — minted
- `src/freeze.rs::validate_user_main_signature` — update
- `src/freeze.rs::validate_user_process_signature` — minted
- `src/freeze.rs::validate_user_thread_signature` — minted
- `src/spawn.rs::eval_kernel_spawn_program` — verify retire/merge per § 5 open question
- `src/fork.rs::eval_kernel_fork_program` — rename to `eval_kernel_spawn_process`
- `src/fork.rs::eval_kernel_fork_program_ast` — rename to `eval_kernel_spawn_process_ast`
- `src/spawn.rs::eval_kernel_spawn_thread` — UNCHANGED for body invocation (parent world; structural signature enforcement); add doc comment naming the contract `:user::thread`
- `src/check.rs` — walker variants:
  - `BareLegacyMainSignature` — fires on 3-arg `:user::main`
  - `BareLegacyForkProgram` — fires on `fork-program` / `fork-program-ast` verbs
  - *(no `BareLegacySpawnThreadFnRef` walker — spawn-thread shape unchanged per Q2-replacement structural-enforcement lean)*
- `crates/wat-cli/src/lib.rs::run` — pass `std::env::args()` + interpret ExitCode return
- `src/harness.rs` — `invoke_user_main` arity check + argv pass-through
- `src/spawn.rs` (eval_kernel_spawn_program path) — invokes `:user::process` instead of `:user::main`
- `src/fork.rs` (eval_kernel_spawn_process / spawn_process_ast paths) — invokes `:user::process` instead of `:user::main`
- New `tests/wat_arc170_program_contracts.rs` — 15-20 cases:
  1. `:user::main` 4-arg + ExitCode return — works
  2. argv pure passthrough — first 3 args = ["wat", "source.wat", "user-arg-1"]
  3. ExitCode = `(:wat::core::u8 0)` returns 0 to OS
  4. ExitCode = `(:wat::core::u8 42)` returns 42 to OS
  5. `:user::process` 3-arg + nil — works in spawn-process invocation
  6. `:user::thread` 2-arg + nil — works in spawn-thread invocation
  7. spawn-thread (src scope) — freezes fresh world; child's `:user::thread` runs
  8. spawn-process (src scope) — forks; child's `:user::process` runs
  9. spawn-process-ast (forms scope) — fork-from-ast; child's `:user::process` runs
  10. Old `:user::main` 3-arg signature — walker fires `BareLegacyMainSignature`
  11. Old `fork-program` verb — walker fires `BareLegacyForkProgram`
  12. Old `spawn-thread` fn-ref — walker fires `BareLegacySpawnThreadFnRef`
  13. `:user::process` missing — spawn-process freeze fails with naming diagnostic
  14. `:user::thread` missing — spawn-thread freeze fails with naming diagnostic
  15. Both `:user::main` AND `:user::process` declared — both work in their respective contexts
  16. argv carrying special chars — round-trips byte-correct
  17. Empty argv (programmatic invocation) — `:user::main` sees empty Vec
  18. ExitCode out-of-range — caught at substrate u8 cast; clear diagnostic
  19. Wrong return type from `:user::main` (returns nil) — type-check error
  20. Wrong arity of args passed to `invoke_user_main` — runtime error

Predicted: 90-180 min opus. Bigger than arc 167 / 168 slice 1 —
multiple primitives reshaping simultaneously plus three walker
variants.

### Slice 2 — consumer sweep (sonnet)

Mechanical sweep across all `:user::main` definitions + every
`fork-program*` callsite + every `spawn-thread` fn-ref callsite.
Walker diagnostics from slice 1 drive the migration.

**Site classifications:**

- **CLI-only programs** (e.g., `examples/with-loader/wat/main.wat`):
  `:user::main` migrates to new signature. Add `argv` parameter
  (ignore if not needed). Change return to `(:wat::core::u8 0)` for
  success or appropriate non-zero for failure paths.

- **Substrate-spawn-target programs** (e.g., test fixtures invoked
  as inner-src of fork-program in `tests/wat_arc104_fork_program.rs`):
  Rename `:user::main` to `:user::process`. Drop argv concept (was
  never there); keep `:nil` return.

- **Both contexts** (rare; CLI-runnable AND used as substrate
  spawn target): Define BOTH `:user::main` and `:user::process` in
  the same wat source. The substrate dispatcher picks per
  invocation context.

- **`fork-program*` verb sites**: rename verb to `spawn-process*`.
  Mechanical.

- **`spawn-thread` callsites** — UNCHANGED. Services pattern
  preserved per Q2-replacement structural-enforcement lean. No
  sweep needed for spawn-thread.

**Predicted size:** 80-150 sites total across:
- All `:user::main` definitions (~100+ across tests, lab, crates,
  examples, wat-tests)
- All `fork-program` / `fork-program-ast` verb sites (~30-50 per
  earlier arc 104 sweep counts)

Predicted runtime: 60-120 min sonnet. Smaller than initial
estimate now that spawn-thread sites stay unchanged.

### Slice 3 — substrate retirement (opus)

Hard-delete all transitional scaffolding from slice 1.

- `BareLegacyMainSignature` walker variant + Display + Diagnostic + body
- `BareLegacyForkProgram` walker variant + Display + Diagnostic + body
- `BareLegacySpawnThreadFnRef` walker variant + Display + Diagnostic + body
- Old `eval_kernel_fork_program*` arms (renamed to spawn-process)
- Old `eval_kernel_spawn_thread` fn-ref arm
- `validate_user_main_signature` legacy 3-arg fall-through (if any)
- spawn-program / spawn-program-ast — retired entirely (per § 5 open question pending user confirmation)
- Vacuous walker-firing tests in `tests/wat_arc170_program_contracts.rs`

Predicted: 30-60 min opus.

### Slice 4 — closure paperwork (orchestrator)

- SCORE-1 (substrate consumer)
- SCORE-2 (consumer sweep)
- SCORE-3 (substrate retirement)
- INSCRIPTION (arc 170 closure)
- 058 changelog row
- USER-GUIDE update — Program\<I,O\> client/server section + new
  entry-point contracts + ExitCode + argv
- ZERO-MUTEX cross-ref check (no new mutex; spawn primitives already use channels)
- CONVENTIONS doc update (new entry-point naming convention)
- Atomic squash-merge to main

When slice 4 ships, **arc 109 v1 milestone closure unblocks** per
arc 109 INVENTORY § M-equivalent.

---

## Settled questions during DESIGN

All four open questions resolved 2026-05-09. Recorded for the
historical thread.

### Q1. spawn-program retires entirely. *(settled — option A)*

User direction 2026-05-09: *"A - retire."*

`:wat::kernel::spawn-program` and `:wat::kernel::spawn-program-ast`
are deleted in arc 170. The substrate's program-spawn taxonomy
collapses to a clean two-mode model:

- **spawn-thread** (parent's world; services pattern; channels)
- **spawn-process** (OS-forked; fresh child world; pipes)

Tests today using spawn-program (in-thread fresh-world fakery
for fast tests) migrate during slice 2:

- Tests verifying full process isolation → migrate to
  spawn-process (real OS fork; slower but real)
- Tests verifying intra-process channel patterns → migrate to
  spawn-thread (parent world; services pattern)
- Tests where speed-without-fork-cost mattered → accept the
  fork cost as the price of the cleaner taxonomy, OR move the
  test to a Rust-side harness call (which can drive frozen
  worlds without spawn-program)

The "test-only fakery" middle option is dishonest: the wat code
under test ran in-thread but is documented to "spawn a process."
Killing the fakery aligns docs with reality.

### Q2. *(retired — was based on wrong assumption)*

Original Q2 asked about spawn-thread fresh-world freeze cost.
User correction 2026-05-09: spawn-thread does NOT freeze a fresh
world. Threads share parent's process memory (zero-mutex
doctrine; services pattern). The question was meaningless under
the correct framing. Section 6 revised; this Q retired.

### Q2-replacement. *(settled — structural only; documentation contract)*

User direction 2026-05-09: *"the shape of those functions /is/
the contract.. i think the only strict one is :user::main?.."*

Settled: **only `:user::main` is a strict name.** `:user::thread`
and `:user::process` are documentation-level contract names
naming the signature shapes. Substrate enforces structurally —
any fn matching the signature works, named at any keyword path
(or inline). Service-template's inline-lambda + closure-over-
`req-rxs` pattern preserved.

Multi-service programs name their entries at unique paths
(`:my::svc::accountant::loop`, `:my::svc::registry::loop`,
etc.); the contract is the signature, not the name. spawn-thread
already accepts inline-or-ref structurally. spawn-process gains
an explicit entry-keyword parameter so the caller specifies
which symbol in child's world to invoke.

### Q3. argv pure passthrough. *(settled — pure passthrough)*

User direction 2026-05-08 (scratch/2026/05/019-wat-cli-options):
> *"no silent argv reshaping; what the binary received is what
> the program sees"*

Pure passthrough. wat-cli's flags (e.g., `--check`) appear in
argv if passed on the command line. wat-cli's recognized flags
short-circuit before `:user::main` runs (so in practice the
user's main never sees them when they're set), but the substrate
makes no claims about filtering — argv is exactly what the OS
delivered.

### Q4. Substrate-then-sweep parallel pattern. *(settled — arc 167/168/169 precedent)*

Slice plan follows the substrate-as-teacher pattern:
- Slice 1 ships substrate + walkers; old shapes still compile
  during the transitional sweep window.
- Slice 2 sonnet sweep; substrate-as-teacher walkers fire on
  every legacy shape; sweep clears the diagnostic stream.
- Slice 3 retires walker bodies + legacy substrate arms.
- Slice 4 closure paperwork.

Same pattern as arc 167 (fn-flat-signature), arc 168
(let-flat-shape), and arc 169 (struct-destructure). Recommended;
confirmed.

---

## Cross-references

- **arc 109 (kill-std)** — broader substrate refactor wave; arc
  170 is one of its v1 closure blockers
- **arc 112 (inter-process Result\<Option\<T\>,E\>)** — fork-program
  stdin/stdout/stderr wire foundation
- **arc 114 (spawn-as-thread)** — established the Program\<I,O\>
  contract: "values flow only through channels; return is a
  panic-free marker." This is the constraint that forces
  `:user::main` (CLI) to differ from `:user::process` /
  `:user::thread` (Programs).
- **arc 167 (fn-flat-signature)** — substrate-as-teacher walker
  precedent at this scale
- **arc 168 (let-flat-shape)** — multi-slice substrate-refactor +
  sweep precedent; arc 170 mirrors the four-slice shape
- **arc 169 (struct-destructure-form-a)** — most recent arc; arc
  170 spawns off the post-169 main
- **scratch/2026/05/019-wat-cli-options/** — full vision for argv
  + wat-cli-options + user: subcommand convention; arc 170 lands
  the argv contract piece only
- **scratch/2026/05/007-remote-program/** — RemoteProgram
  architecture (typed-capability-bridge); future arc post-170

---

## DESIGN-time conversation log (2026-05-09)

The architecture settled across roughly a dozen exchanges:

1. User asked for argv on `:user::main`; pointed to scratch doc
2. Three return-type options presented (i64 / u8 / enum); user picked u8
3. User proposed `(:wat::kernel::exit N)` helper; orchestrator pushed back on naming (conflicts with libc::exit semantics); user picked typealias path (`:wat::kernel::ExitCode = :u8`)
4. Orchestrator scoped arc 170 narrowly (just argv); user pushed: spawn/fork-program don't actually need argv
5. User raised: forks/threads might not need :user::main at all; might need their own entry contract
6. Orchestrator surfaced Program\<I,O\> + arc 114's "return nil" contract; arc 170 must split CLI from Program contracts
7. User proposed three named entries: `:user::main` (CLI), `:user::thread` (thread Program), `:user::process` (process Program); plus `:user::remote-program` future
8. User confirmed "kill spawn-thread fn-ref pattern"; reclaim spawn-thread name with new (src, scope) shape
9. Orchestrator surfaced verb consolidation: `fork-program*` → `spawn-process*` for naming family alignment
10. User locked client/server framing as the unifying mental model
11. User confirmed "single very large arc with a bunch of slices" — arc 170 takes the whole coherent unit
12. User caught Q2 framing error: spawn-thread shouldn't reshape; threads share parent's memory (zero-mutex doctrine; services pattern). DESIGN section 6 corrected; Q2 retired; Q2-replacement opens on signature-vs-canonical-name enforcement question (lean: stay structural)
13. User clarified the contract-vs-name distinction: only `:user::main` is a strict name; `:user::thread` and `:user::process` are documentation labels for signature contracts. Multi-service programs declare bespoke entry paths satisfying the contracts structurally. spawn-process takes an explicit entry-keyword parameter (no magic defaults). Sections 3, 4, 5 corrected; Q2-replacement settled.
14. User settled Q1 with "A - retire": spawn-program / spawn-program-ast delete entirely; substrate's program-spawn taxonomy collapses to two-mode (spawn-thread parent-world + spawn-process forked-fresh-world). Test-only in-thread fakery dies as a discipline cost.

The settled design IS the conversation. This log preserves it as
historical record per FM 11 inscribe-don't-amend doctrine.
