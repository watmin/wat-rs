# Arc 170 — Build plan

**Status:** authored 2026-05-10 after design lock-in v3 (commit
`6276061`). The endstate is locked in via DESIGN.md +
EXAMPLES.md + TIERS.md + REALIZATIONS-SLICE-1.md (passes 1-13).
This doc captures the path from current state → endstate.

**Endstate:** see [`EXAMPLES.md`](./EXAMPLES.md) for the
canonical user-facing form. Servers run with
`(:wat::kernel::main! :my::handler)` at top of file; clients
use `(:wat::kernel::spawn-process some-fn)` + `Process/println`
/ `Process/readln` / `Process/wait`. Substrate handles every
concern user code drops.

---

## Section 1 — Where we are

### Committed (clean) state on `arc-170-program-entry-points`

```
6276061  arc 170: EXAMPLES.md — full client/server pair demos in pure wat
ef370dd  arc 170: design lock-in v3 — println/readln + graceful nil + signal model
ffba92b  arc 170: design lock-in v2 — nil IS the exit code; helpers + ambient client
39be0c3  arc 170: design lock-in — three substrate services + canonical server form
0136088  arc 170 slice 3: BRIEF + EXPECTATIONS authored — atomic-commit pair (opus + sonnet)
```

Design fully landed in repo. SCORE-SLICE-1.md / SCORE-SLICE-1B.md
/ SCORE-SLICE-1C.md / SCORE-SLICE-2.md inscribed historical
record of what shipped before the architectural pivot — they
stay immutable per FM 11.

### Dirty-tree contents (~60 files)

Three logical pieces, mixed in the working tree:

**(A) Phase A — retire `wat/std/sandbox.wat` + `wat/std/hermetic.wat`:**
- `src/check.rs`, `src/runtime.rs`, `src/stdlib.rs` —
  references retired
- `wat/std/hermetic.wat`, `wat/std/sandbox.wat` — DELETED
- **STATUS: KEEP.** Foundation work; the testing-lib gets
  rebuilt from scratch in revised slice 3; this prep is
  load-bearing.

**(B) Slice 1d — closure-extraction walker substrate fix:**
- `src/closure_extract.rs` — walker handles match-arm pattern
  bindings + wildcards
- `tests/wat_arc170_closure_extraction.rs` — extended tests
- **STATUS: KEEP.** Substrate fix is load-bearing for any future
  fork-program use; doesn't conflict with the architectural pivot.

**(C) Phase B — mass-edit tests to 4-arg `:user::main` + ExitCode:**
- ~50 `tests/wat_*.rs` files + `wat-tests/**` — `:user::main`
  signature changes; fork-program → spawn-process renames; verb
  updates; assertions against walker firing
- **STATUS: BACK OUT.** Pass 10 reverses the 4-arg + ExitCode
  shape — `:user::main` becomes `[] -> :wat::core::nil`.
  Phase B's signature edits are invalidated; the verb renames
  are still valid but get re-applied during revised slice 3's
  fresh sweep. Salvaging selectively is more complexity than
  re-sweeping.

### Workspace baseline

Last measured 2015 passed / 119 failed (post slice-1d, before
architecture lock-in). After dirty-tree disposition + slice 1e,
the baseline shifts. Re-establish before delegating slice 1e.

---

## Section 2 — Decisions locked in

### D1. Dirty-tree disposition: back out phase B; keep phase A + slice 1d

```bash
# Stage phase A + slice 1d files (the keepers); back out phase B
git add src/check.rs src/closure_extract.rs src/runtime.rs src/spawn_process.rs src/stdlib.rs
git add tests/wat_arc170_closure_extraction.rs
git add wat/std/hermetic.wat wat/std/sandbox.wat   # the deletions

# Restore everything else (phase B test sweep + wat-tests + wat/test.wat)
git restore tests/<phase-B-files>
git restore wat-tests/
git restore wat/test.wat
```

Then commit phase A + slice 1d as the foundation commit before
slice 1e starts. Revised slice 3 will sweep tests fresh against
the post-pivot architecture.

### D2. `spawn-process` ergonomics: fn-input only; no `spawn-server` helper in arc 170

`(:wat::kernel::spawn-process some-fn)` where `some-fn`
satisfies `[] -> :wat::core::nil`. For server programs, users
write the wrap explicitly:

```scheme
(:wat::kernel::spawn-process
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::server-loop :my::handler)))
```

A `spawn-server` helper symmetric to `main!` (auto-wrapping
handler in server-loop) is future-arc territory. Arc 170 keeps
the substrate honest: spawn-process takes a fn; `main!` is the
ONE convenience macro that wraps for the CLI case. Wat-level
helpers can compose on top later when demand surfaces.

EXAMPLES.md already shows this explicit form.

### D3. Slice ordering: 1e → 1f → 1g → 1h → 1i → 3 → 4 → 5

Each slice depends on the prior. No reordering possible without
breaking dependencies (named in each slice's section below).

### D4. Atomic-commit per slice (recovery doc § 7)

Substrate slices stay uncommitted while internally inconsistent
(slice 2 → slice 3 pattern from arc 130 slice 2). Final commit
when the slice's load-bearing tests are green. SCOREs land
beside the commit.

---

## Section 3 — The slice path

### Slice 1e — Ambient runtime + drop stdio params + retire ExitCode

**Substrate; opus.**

**Scope:**
- Mint `:wat::runtime::current-thread` (thread-local id) and
  `:wat::runtime::argv` (set-once at process start)
- Update `:user::main` signature: `[] -> :wat::core::nil` (drop
  3 stdio params + ExitCode return)
- Update `expected_user_main_signature` /
  `validate_user_main_signature` in `src/freeze.rs`
- Retire `:wat::kernel::ExitCode` typealias (delete
  `wat/kernel/exit-code.wat`)
- Update wat-cli to plumb `std::env::args()` into ambient
  `:wat::runtime::argv` (no longer a parameter)
- Update spawn-process child invocation (child's fn is
  `[] -> :nil`; no stdio params)
- Walker `BareLegacyMainSignature` updates to fire on the
  4-arg shape (now legacy)

**Dependencies:** post phase A + slice 1d commit; clean tree.

**Ship criteria:**
- 4-arg `:user::main` definitions in arc 170 fixture tests fire
  the walker
- `[] -> :wat::core::nil` definitions parse + freeze
- `:wat::runtime::argv` accessible from a wat program
- `:wat::kernel::ExitCode` no longer registered (any reference
  errors)

**Predicted runtime:** 60-120 min opus.

**Expected workspace impact:** ~50-200 test failures from
ExitCode references + 4-arg signature assumptions. Slice 3
sweep fixes; slice 1e leaves them as substrate-as-teacher input.

### Slice 1f-W — Wire encoding (lexical rule + EDN comma↔underscore swap) — NEW PREREQUISITE

**Substrate; opus.** Inserted 2026-05-10 per REALIZATIONS pass 14
(wire encoding lexical doctrine — position-aware) after slice
1f-ii authoring surfaced that the EDN wire spec wasn't locked.

**Scope:**
- Lexer split: keyword bodies get a position-aware char rule —
  inside `<...>` substrings, `_` is FORBIDDEN; outside `<...>`,
  `_` is allowed (preserves `:rust::*` Rust-mirror convention)
- Wire writer: `wat_edn::write_keyword` swaps `,` → `_` at
  depth ≥ 1 (inside `<...>`); outside, chars pass verbatim
- Wire parser: `wat_edn::lex_keyword` (or post-lex normalize)
  swaps `_` → `,` at depth ≥ 1; outside, chars pass verbatim
- Tests: round-trip cases (basic; with `<>`; with `:rust::*_*`
  outside brackets); rejection case (`_` in source inside `<>`
  fires lexer error with diagnostic)

**Dependencies:** slice 1e shipped (current branch tip); slice
1f-i shipped (parser will inherit the un-escape).

**Ship criteria:**
- Round-trip: `:wat::core::HashMap<wat::core::String,wat::core::i64>`
  → wire `:wat::core::HashMap<wat::core::String_wat::core::i64>`
  → parsed back to source form (keyword equality)
- `:rust::crossbeam_channel::Sender<T>` round-trips verbatim
  (underscore preserved outside brackets; `<T>` has no comma
  to swap)
- Source-position `_` inside `<>` rejected with diagnostic
- All 18 existing underscore-in-keyword forms still parse
  (none are inside `<>`)
- Slice 1f-i tests still green (parser un-escape doesn't break
  existing decode path)
- Workspace cargo test fail count delta ~0 from post-slice-1f-i
  baseline (parallel substrate change; existing test fixtures
  don't use `<>` with commas in keyword positions)

**Predicted runtime:** 60-90 min opus.

**Expected workspace impact:** small — purely additive wire
encoding + lexer split + tests. Existing workspace fail count
unchanged (855 pre-1f-W; ±5 post-1f-W).

**Why this exists:** slice 1f-ii would write EDN with commas in
keyword bodies (parametric types like `HashMap<K,V>`). Without
the wire encoding swap, the receiving side's EDN parser would
treat commas as whitespace (per EDN spec), corrupting the
keyword. Slice 1f-W locks the protocol BEFORE transmission
slices send anything.

### Slice 1f — Three substrate stdio services — REFRAMED PER PASS 15

**Reframed 2026-05-10 per REALIZATIONS pass 15** after a
workspace deadlock surfaced. The original Rust-thread singleton
shape (slices 1f-i/1f-ii/1f-iii/1f-iv) violated ZERO-MUTEX.md
tier-3 doctrine + SERVICE-PROGRAMS.md "the lockstep." Services
in wat-rs are **wat programs**, not Rust threads with custom
APIs. The runtime is the orchestrator; services are wat programs
spawned via `:wat::kernel::spawn`; helpers are thread-aware
contexts that "just work."

**The corrective:**
- `git revert 630f621` — slice 1f-i Rust singleton wrong shape
- Discard slice 1f-ii working tree (uncommitted)
- Slice 1f rebuilds along α/β/γ/δ stones using the existing
  tier-3 substrate primitives + a NEW control-pipe protocol
  for dynamic membership

**The substrate's dual-canonical pattern library after slice 1f:**

| Pattern | Worked example | Use case |
|---|---|---|
| Static-membership | `wat/console.wat`, `crates/wat-lru/.../CacheService.wat`, `crates/wat-telemetry/.../Service.wat` | Client set known at construction |
| **Dynamic-membership** (NEW; minted in slice 1f) | `wat/kernel/services/stdin.wat`, `stdout.wat`, `stderr.wat` | Clients register/reap during program lifetime |

**Atomic commit per stepping stone.** SCORE per stepping stone.

**Dependencies:** slice 1e shipped (ambient runtime). Slice 1f-W
shipped (wire encoding for parametric type keywords). Slice 1g
(spawn-thread register-with-services) FOLDS into slice 1f-γ —
they were always the same concern.

#### Slice 1f-α — `:wat::kernel::println` + `:wat::kernel::readln` substrate primitives

**Substrate; opus.**

**Scope:**
- Mint `:wat::kernel::println` (eval arm in `src/runtime.rs`):
  signature `:T -> :wat::core::nil`; looks up the calling
  thread's stdout-sender from thread-local ambient; sends
  `(value, ack-sender)`; blocks on ack-receiver; returns nil
- Mint `:wat::kernel::readln`: signature
  `() -> :Option<:wat::holon::Atom>`; looks up the calling
  thread's stdin-receiver from thread-local; recv; returns
- Type-check arm registrations
- New thread-local data structure (Rust): `ThreadIO` carrying
  per-thread `(stdout-tx, stderr-tx, stdin-rx)` triple; the
  runtime populates this at spawn-thread time (slice 1f-γ)
- For slice 1f-α, the helpers can have a "no service running"
  fallback that errors with a clear diagnostic (helper called
  before runtime spawned services); the actual service spawn
  lands in slice 1f-γ

**Dependencies:** slice 1e shipped + slice 1f-W shipped.

**Ship criteria:**
- `(:wat::kernel::println v)` evaluates without panic when no
  thread-local is populated (returns clean error or no-op
  diagnostic — doesn't matter the exact behavior; integration
  with services lands in 1f-γ)
- `(:wat::kernel::readln)` evaluates similarly
- Type-check arms register correctly
- Test fixture in `tests/wat_arc170_slice_1f_alpha_helpers.rs`

**Predicted runtime:** 60-90 min opus.

#### Slice 1f-β — Wat-side service implementations

**Substrate; opus + wat-author.**

**Scope:**
- New files: `wat/kernel/services/stdin.wat`,
  `wat/kernel/services/stdout.wat`,
  `wat/kernel/services/stderr.wat`
- Each is a wat program in the canonical `service-template.wat`
  shape PLUS the control-pipe register/reap/sighup handler
- `stdin.wat`: driver loop reads bytes from stdin via
  `:wat::io::IOReader/read-line`; parses with `wat-edn::parse_wire`;
  routes to per-thread consumer based on routing table;
  control-pipe ops update routing table
- `stdout.wat`: driver loop selects across per-thread message
  receivers + control-pipe; receives `(atom, ack-tx)`; serializes
  with `wat-edn::write` (depth-aware swap); writes to stdout +
  newline; signals ack-tx
- `stderr.wat`: similar to stdout but first-panic-wins; emits
  structured cascade EDN; calls `:wat::runtime::libc-exit` (or
  equivalent) after emit
- Substrate-auto-loaded (like `wat/kernel/main.wat` per pass 8)
- Tests in `wat-tests/kernel/services/` exercising the canonical
  service-template patterns + control-pipe protocol

**The control-pipe protocol (locked-in pass 15):**

```
control-pipe carries `:wat::kernel::services::Control`:
  :register :tid (sender / receiver halves)
  :reap     :tid
  :sighup
```

**Dependencies:** slice 1e + slice 1f-W shipped. Slice 1f-α
shipped (helpers exist to drive the test cases).

**Ship criteria:**
- Three wat-side service files compile + load
- service-template patterns hold (lockstep + mini-TCP ack)
- Control-pipe protocol verifiable: register a fake thread-id;
  send data; assert routed; reap; assert dropped; send sighup;
  service drains and exits
- All three services pass standalone deftest exercises (no
  runtime orchestrator yet — that's slice 1f-γ)

**Predicted runtime:** 90-150 min opus + wat-author.

#### Slice 1f-γ — Runtime orchestrator + spawn-thread integration (absorbs slice 1g)

**Substrate; opus.**

**Scope:**
- Active-handle ledger as Rust state in `src/runtime.rs` or
  wat-cli — tracks `(thread_id → (stdin-rx, stdout-tx, stderr-tx))`
- `:wat::kernel::spawn-thread` substrate primitive emits
  `:register thread-id pairs` to each service's control-pipe
  BEFORE returning; awaits ack from each service; populates
  the new thread's `ThreadIO` thread-local; THEN returns the
  thread handle
- Thread reap (on join or panic): runtime emits `:reap thread-id`
  to each service's control-pipe; service drops routing entry
- Per-thread `ThreadIO` cleanup on thread exit
- Integration test: spawn N threads; each calls println; verify
  output ordering per thread + cross-thread interleaving;
  reap; verify control-pipe messages emitted

**Subsumes original slice 1g** (spawn-thread register-with-services).

**Dependencies:** slice 1f-α + 1f-β shipped.

**Ship criteria:**
- spawn-thread emits register; awaits ack-from-each-service
- spawn-thread populates ThreadIO thread-local correctly
- Helpers `println`/`readln` find their thread-locals after spawn
- Reap cycle removes routing entries cleanly
- Active-handle ledger increments/decrements in lockstep

**Predicted runtime:** 120-180 min opus.

#### Slice 1f-δ — wat-cli boot integration + sighup-and-drain cascade

**Substrate; opus.**

**Scope:**
- wat-cli's `run` function spawns the three services BEFORE
  invoking `:user::main`; holds their ProgramHandles
- main thread's spawn-thread cycle (registers main as thread-0)
- Invoke `:user::main`; user code uses println/readln freely
- After main returns: runtime reaps main thread (decrements
  active-handle ledger); when ledger → 0 (no live user threads),
  runtime sends `:sighup` to each service
- Services drain pending; exit
- wat-cli joins each service's ProgramHandle
- Substrate's epilogue (slice 1i): emits `:wat::core::nil` to
  fd 1 + close + libc::exit(0)
- Integration test: wat-cli runs a one-shot script
  (`(println "hello")`); verifies hello on stdout, then `:nil`,
  then exit 0

**Dependencies:** slice 1f-α + 1f-β + 1f-γ shipped.

**Ship criteria:**
- wat-cli boots services successfully
- User's main runs with helpers populated
- Sighup-and-drain cascade fires correctly on graceful exit
- Panic path bypasses sighup (libc::exit fires from
  StdErrService directly per pass 9 doctrine)

**Predicted runtime:** 60-90 min opus.

#### Total slice 1f budget (REFRAMED)

330-510 min opus across α/β/γ/δ. Larger than the original
195-390 prediction because the new architecture is broader —
it builds the dynamic-membership pattern as a NEW canonical
template for the entire ecosystem. The substrate's services
become worked examples for future user services (telemetry
dynamic-membership migration etc.).

**Expected workspace impact:** parallel infrastructure
through slice 1f-α + 1f-β. Slice 1f-γ and 1f-δ replace the
existing wat-cli stdio path; tests using
`spawn_stdin_proxy` / etc. may flip to passing OR may need
slice 3 migration. Substrate-as-teacher continues.

**Original slice 1g RETIRED** — folded into slice 1f-γ. Both
were always the same concern (spawn-thread cycle + service
registration); the pass-15 reframe makes it natural to keep
them together.

**Dependencies:** slice 1f (services must exist to register
with).

**Ship criteria:**
- spawn-thread returns only after all three services ack
- Thread-locals populated in spawned threads
- Panic from spawned thread routes through StdErrService

**Predicted runtime:** 90-180 min opus.

### Slice 1h — Server / Client substrate + helpers + macros

**Wat-level + substrate; opus design + sonnet wat helpers.**

**Scope:**
- Mint `:wat::kernel::Server` and `:wat::kernel::Client`
  substrate types (used internally + tier 1/2/3 unification)
- Mint user-facing helpers (the canonical surface):
  - `(:wat::kernel::println v)` → `:wat::core::nil` — write
    Atom + newline via per-thread stdout Client
  - `(:wat::kernel::readln)` → `:Option<:wat::holon::Atom>` —
    read line + parse EDN via per-thread stdin Client
- Mint Type/verb escape hatches:
  - `(:wat::kernel::StdIn/client)` → `Client` (per-thread)
  - `(:wat::kernel::StdOut/client)` → `Client` (per-thread)
- Mint Process/-verbs for parent-side use (SPECIFIES the API
  EXAMPLES.md flagged as "proposed"):
  - `(:wat::kernel::Process/println proc v)` → `:nil`
  - `(:wat::kernel::Process/readln proc)` → `:Option<:Atom>`
  - `(:wat::kernel::Process/wait proc)` → `:nil` (block until
    child exits)
- Mint `:wat::kernel::server-loop` wat-level helper (the canonical
  service-loop fn body with `(stopped?)` poll + three-branch
  `(readln)` match)
- Mint substrate-auto-loaded macros:
  - `(:wat::kernel::main! handler-expr)` — expands to canonical
    server-program form
  - `(:wat::kernel::run! form1 form2 ...)` — variadic; wraps
    forms in implicit-do for one-shot CLI scripts
- Both macros live in `wat/kernel/main.wat` (or similar);
  substrate auto-loads; users don't `load!` them

**Dependencies:** slice 1g (per-thread Clients must exist for
helpers to route through).

**Ship criteria:**
- `(println v)` / `(readln)` work from any thread
- Process/-verbs work from parent across spawn-process
- main! macro expands correctly; expanded form runs
- Integration test exercising the full pass-13 canonical form

**Predicted runtime:** 90-180 min mixed (opus settles substrate;
sonnet wat helpers).

### Slice 1i — wat-cli exit-path discipline (structured-stderr-only + graceful-`:nil` epilogue)

**Substrate; opus.**

**Scope:**
- wat-cli has zero direct stderr writes (load failures, freeze
  errors all route through StdErrService → cascade)
- panic-cascade emit on fd 2 from Rust (replaces slice 2's flat
  marker); uses arc 113 cascade pattern via StdErrService
- Substrate exit epilogue after `:user::main` returns nil:
  1. emit `:wat::core::nil` to fd 1 (protocol-compliance final)
  2. close fd 1
  3. libc::exit(0)
- Panic exit skips this path (StdErrService cascade fires
  libc::exit(N) directly; consumer sees ungraceful `None`)
- Signal model preserved per arc 106: substrate measures via
  per-process atomic flags; userland transitions; substrate
  does NOT auto-trigger main-return

**Dependencies:** slice 1h (Server/Client + helpers must work
before exit-path discipline is testable).

**Ship criteria:**
- Shell-level test: `wat hello.wat` writes structured EDN to
  stdout; clean exit 0; trailing `:nil` line on stdout
- Panic test: cascade EDN on stderr; non-zero exit; no clean
  `:nil` on stdout
- Hermetic-test-harness can read `Some(:nil)` as graceful-done
  marker

**Predicted runtime:** 90-180 min opus.

### Slice 3 (revised) — Consumer sweep + testing-lib rebuild

**Mechanical (sonnet) + judgment (opus); atomic-commit pair.**

**Scope:**
- Sweep all `:user::main` definitions to `[] -> :wat::core::nil`
  signature
- Sweep fork-program* / spawn-program* callsites → spawn-process(fn)
- Migrate Console-using tests to StdInService / StdOutService
- Replace `IOReader` / `IOWriter` parameter types with helper
  calls
- **Testing-lib three-layer rebuild:**
  - Layer 1: `(:wat::test::run-hermetic body)` — 90% case
  - Layer 2: `(:wat::test::run-hermetic-with-io<I,O> inputs body)` —
    9% case (typed channels via Process)
  - Layer 3: `(:wat::kernel::spawn-process fn)` — 1% case;
    full substrate
- Replace `wat/std/hermetic.wat` (deleted in phase A) with the
  three-layer API under `wat/test/` or similar

**Dependencies:** slices 1e through 1i complete.

**Ship criteria:**
- Workspace = 0 failed
- Testing-lib three layers documented + tested
- Hermetic test isolation property preserved (per-process
  ambient hermetic seal at tier ≥ 2)

**Predicted runtime:** 90-180 min sonnet mechanical + 60-120 min
opus orchestration for the testing-lib rebuild.

### Slice 4 — Substrate retirement (bandaid retirement)

**Substrate destructive + sweep; opus + sonnet atomic-commit pair.**

**Scope (the bandaid inventory):**
- `:wat::kernel::Process<I,O>` legacy 3 byte-pipe fields
  (stdin/stdout/stderr) — retire (slice 1c additive shape)
- `wat/std/sandbox.wat` + `wat/std/hermetic.wat` — already
  deleted in phase A; verify no stragglers
- Walker variants: `BareLegacyMainSignature`,
  `BareLegacyForkProgram`, `BareLegacySpawnProgram` + their
  Display + Diagnostic + bodies
- Old eval arms: `eval_kernel_fork_program*` /
  `eval_kernel_spawn_program*` — deleted
- `validate_user_main_signature` legacy 4-arg fall-through —
  deleted (only the new shape remains)
- Vacuous walker-firing tests retired
- Today's `:wat::console::Console` crossbeam service — retire
  (replaced by StdOutService); migrate any remaining test
  references in slice 3
- Slice 1c PipeFd Sender/Receiver substrate — retire from wat
  level (becomes substrate-internal only; consumers go through
  Server/Client or println/readln)

**Dependencies:** slice 3 complete; workspace = 0 failed.

**Ship criteria:**
- All bandaids retired
- Workspace = 0 failed (atomic with retirement)
- INSCRIPTION free of deferral language per FM 11

**Predicted runtime:** 60-120 min opus destructive + 30-90 min
sonnet sweep = ~90-210 min total.

### Orthogonal future arcs (NOT arc 170 scope; tracked here for visibility)

Per REALIZATIONS pass 14, two threads of substrate-foundation
work surfaced during arc 170 but are orthogonal to arc 170's
transmission services. They get their own arcs.

#### Arc 171 — Comma → apostrophe in fixed-arity dispatch forms

**Scope:** sweep `:foo,2` → `:foo'2`, `:foo,i64-i64` →
`:foo'i64'i64` etc. across the substrate's dispatch registry
(arc 146/148) + every callsite that uses fixed-arity
discrimination.

**Why orthogonal to arc 170:** arc 170 doesn't add fixed-arity
dispatch entries. The lexical rule from slice 1f-W (forbids `_`
inside `<>`) doesn't conflict with comma-suffix dispatch forms
because the comma is OUTSIDE any `<>`. Arc 171 happens to
share the "no commas in keyword bodies" theme but is a
separate sweep.

**Sizing:** TBD at arc 171 author time. Most grep hits for
`:foo,bar` outside `<>` are tuple-args (`:(A,B,C)`) and
parametric type args inside `<>` (`<K,V>`); the actual
comma-suffix dispatch forms need careful counting.

**Dependencies:** arc 170 slice 5 (arc 170 should close
cleanly first; sweep arc 171 against the post-arc-170 state).

#### Arc 172 — Macro flavor swap (Scheme → Clojure)

**Scope:** replace defmacro + quasiquote/unquote infrastructure
with Clojure semantics:
- `'foo` quote (not `(quote foo)`)
- `` `foo `` syntax-quote with auto-namespace-qualify and
  auto-gensym-on-`#` suffix
- `~foo` unquote
- `~@foo` unquote-splicing
- `gensym` for hygiene
- Implicit `&form` and `&env` inside defmacro bodies
- Migrate all existing wat-side macros (Console, harness,
  defn, etc.) to Clojure flavor

**Why orthogonal to arc 170:** arc 170's `main!` / `run!`
helper macros work either flavor. Slice 1h (Server/Client +
helpers + macros) ships them in whichever flavor is current
when 1h spawns; arc 172 then migrates them along with all
other macros.

**Sizing:** LARGE. Macro evaluator rewrite + migration of all
defmacro callsites. Multi-slice arc.

**Dependencies:** arc 170 slice 5 (close arc 170 first); arc
172 then sweeps + migrates against the closed arc-170 state.

---

### Slice 5 — Closure paperwork (orchestrator)

**Orchestrator-side; no agent spawn.**

**Scope:**
- Author SCORE-SLICE-1E.md through SCORE-SLICE-4.md
- Author INSCRIPTION.md (pre-grep per recovery doc § 11)
- Update USER-GUIDE.md (Program client/server section + entry
  contracts + nil-IS-exit-code + argv + spawn primitives +
  closure extraction note + structured-stderr doctrine)
- Update CONVENTIONS.md (entry-point naming convention)
- Update ZERO-MUTEX.md cross-ref (no new Mutex)
- Cross-ref `tests/wat_tco.rs` "the Console/loop shape"
  benchmark per pass-13 meta-observation
- Update 058 changelog row in lab repo
- Atomic squash-merge to main

**Dependencies:** slice 4 complete; INSCRIPTION pre-grep clean.

**Ship criteria:**
- INSCRIPTION pre-grep returns no "deferred" / "future arc" /
  etc. language
- Arc 109 v1 milestone closure unblocks
- Branch merged to main

**Predicted runtime:** 60-120 min orchestrator.

---

## Section 4 — Atomic commit boundaries

| Boundary | Files | Commit message form |
|---|---|---|
| **Foundation** | phase A + slice 1d | `arc 170: phase A + slice 1d — sandbox/hermetic retirement + closure walker fix (foundation for slice 1e)` |
| **Slice 1e** | substrate-only after sweep tests fail | `arc 170 slice 1e: ambient runtime + drop stdio params + retire ExitCode` |
| **Slice 1f** | services landed | `arc 170 slice 1f: three substrate services (StdIn/StdOut/StdErr)` |
| **Slice 1g** | thread-registration contract | `arc 170 slice 1g: spawn-thread register-with-services + per-thread Client thread-locals` |
| **Slice 1h** | helpers + macros | `arc 170 slice 1h: Server/Client substrate + println/readln helpers + main!/run! macros + Process/-verbs` |
| **Slice 1i** | exit-path discipline | `arc 170 slice 1i: wat-cli exit-path discipline (structured-stderr + graceful-:nil epilogue)` |
| **Slice 3** | sweep + testing-lib (atomic with substrate consumers) | `arc 170 slice 3: consumer sweep + testing-lib three-layer rebuild` |
| **Slice 4** | bandaid retirement (atomic with sweep) | `arc 170 slice 4: substrate retirement (bandaid retirement; INSCRIPTION-ready)` |
| **Slice 5** | closure paperwork | `arc 170: INSCRIPTION` + `arc 109 v1 milestone closure unblocks` |

Branch `arc-170-program-entry-points` accumulates all commits;
slice 5 squash-merges to main.

---

## Section 5 — Risk surface

### R1. Slice 1f size

The three substrate services are the heaviest single slice.
180-300 min predicted. Mode B / time-violation possible. If
sonnet-tier delegation, the BRIEF must split into stepping
stones (1f-i, 1f-ii, 1f-iii?). Current prediction assumes opus
single-shot; revisit at BRIEF-author time.

### R2. Per-thread Client thread-local discipline

If spawn-thread doesn't ack-before-return, a panic in the
spawned thread before service registration completes would
drop the panic on the floor (no one to receive it). Slice 1g's
register-with-services contract MUST enforce ack synchronously.
Surface this as a load-bearing test (spawn-thread-then-panic-
immediately).

### R3. Process/-verbs API specification

Slice 1h must settle the parent-side API. EXAMPLES.md flagged
the names (`Process/println`, `Process/readln`, `Process/wait`)
as proposed. The shape (Type/verb on Process; mirror ambient)
is settled per pass-12 + pass-13. The exact verb names + types
land in slice 1h's BRIEF.

### R4. Testing-lib rebuild scope

Slice 3's testing-lib three-layer rebuild is judgment-heavy.
Layer 1 macro expansion (the 90% case) is the biggest UX win;
botching it ripples across all hermetic tests. Allocate
orchestrator time; not a pure sonnet-mechanical sweep.

### R5. Workspace baseline volatility

Each substrate slice 1e through 1i breaks tests cumulatively.
Substrate-as-teacher (FM 15) discipline applies — fail counts
are the progress meter; don't panic. Slice 3 sweep collapses
the cumulative red.

### R6. Bandaid retirement timing

Slice 4 must NOT happen before slice 3's sweep is green —
retiring legacy fields/walkers/dispatch-arms while consumers
still use them = workspace red mid-flight. Atomic-commit-pair
discipline (recovery doc § 7) enforces this.

---

## Section 6 — Pre-flight checklist (before slice 1e starts)

- [ ] Disposition decision committed to repo (D1 above)
- [ ] phase A + slice 1d committed as foundation
- [ ] Phase B test sweep backed out (clean tree post-foundation)
- [ ] `cargo test --release --workspace --no-fail-fast` runs;
      baseline number recorded in slice 1e's EXPECTATIONS
- [ ] Slice 1e BRIEF + EXPECTATIONS authored + committed
- [ ] Recovery doc § Sonnet-delegation-protocol pre-flight
      satisfied (substrate-informed brief; predicted runtime
      band; wakeup at 2× upper bound; etc.)

When all checks pass: spawn slice 1e.

---

## Section 7 — Compaction-amnesia recovery for this plan

When a future session resumes mid-build:

1. Read this BUILD-PLAN.md (you're here)
2. Read the most recent SCORE-* in this arc dir
3. `git log --oneline arc-170-program-entry-points | head -20`
4. `git status --short` — what's mid-flight?
5. Identify which slice (per Section 3) is in progress; resume
   from the next sub-step

Memory cross-references:
- `project_arc_170_canonical_server_form.md` — the polished
  9-line form across passes 8 → 13
- `project_arc_170_user_guide_seeds.md` — the EXAMPLES.md
  seed material (informational; EXAMPLES.md is load-bearing)
- `project_signal_cascade.md` — kernel measures, userland
  transitions
- `project_pipe_protocol.md` — line-delimited EDN; one
  protocol; four transports
- `feedback_compaction_protocols.md` — substrate-as-teacher
  discipline (FM 15)
- `feedback_v1_backout_dependency_arc.md` — dirty-tree
  back-out pattern

The plan stays operational. If reality diverges from the plan,
amend this doc; the plan tracks reality, not the other way
around.
