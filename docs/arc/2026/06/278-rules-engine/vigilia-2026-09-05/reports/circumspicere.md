# ward `circumspicere` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

All claims verified. Writing up.

---

# `circumspicere` — vigilia perimeter report
**HEAD `21530efab`, branch `grok-rete`. Read-only; tree unmodified; no cargo build run.** Everything below was driven against the prebuilt `target/release/wat` or read at `file:line` this session.

---

## L1 — defects, ranked by what they cost

### L1-1. `--check` is a broken oracle in *both* directions — the automation-facing mode

`src/distribution/mod.rs:351` short-circuits `--check` **fifty lines before** the stack raise at `:401-408`. The mode's own comment (`:346-350`) sells it as *"side-effect-free verification suitable for editor save hooks and agent sweep loops."*

**It passes programs that cannot start:**
```
$ : > empty.wat
$ ./target/release/wat --check empty.wat ; echo rc=$?
rc=0
$ ./target/release/wat empty.wat ; echo rc=$?
[#wat.kernel.LociDiedError/MainSignature ["#wat.kernel/MainSignatureError {:message \":user::main not defined — a wat program needs an entry point\" ...}"]]
rc=4
```

**And it crashes on programs that run fine.** Same file, freeze-time non-tail recursion at depth **1000**, 6 samples each:
```
wat fd_1000.wat         : 0 0 0 0 0 0
wat --check fd_1000.wat : 134 134 134 134 134 134

$ ./target/release/wat --check fd_1000.wat
thread 'main' (977867) has overflowed its stack
fatal runtime error: stack overflow, aborting
EXIT=134
```
`--check` ceiling bisected, 6 samples per depth: 200 ✓, 400 ✓, **600 ✗**, 800 ✗. The full runtime clears 70,000.

A verifier that both green-lights unstartable programs and aborts on startable ones is worse than no verifier, because a sweep loop believes it.

### L1-2. The MCP server: arbitrary code, no sandbox, and 1/100th the stack

`src/distribution/mod.rs:317-319` returns to `mcp::serve()` — also before `:401`. The MCP `eval` tool takes caller-supplied wat source and evaluates every form.

**Arbitrary file read, driven through the wire:**
```
$ ./target/release/wat --mcp < read.jsonl | tail -1
{"id":2,...,"result":{"content":[{"text":"#wat.mcp/Turn {... :value \"reason\\n\"}",...}],"isError":false}}
$ cat /etc/hostname
reason
```

**And any turn deeper than ~600 kills the session.** Anchored first (trivial eval returns `3`; depths 10/100/300 return correct values), then bisected 6 samples per depth: 550 ✓ 560 ✓ 570 ✓ 580 ✓ — **600 → `134` ×6**, `fatal runtime error: stack overflow, aborting`. The identical program via the CLI file path survives 70,000. A ~100× gap produced by nothing but which side of line 401 the mode returns on.

Context that raises this: this server is registered globally and `mcp__wat__eval` is live in this session — reachable by prompt injection, outside the harness's own file-permission system. Note the installed binary `/home/john/.cargo/bin/wat` is dated **Aug 23** and is **not** the HEAD build (`cmp` differs), so its behaviour is unmeasured.

Also: `mcp.rs:319/320/418` hand-write `#wat.mcp/…` tags, and `src/error_ns.rs` — which calls itself *"THE single source of truth for error tag namespaces"* — does not list `wat.mcp`.

### L1-3. The arc-261 stopgap removes the *diagnostic*, and its own comment says it doesn't

`src/distribution/mod.rs:397-399`: *"This only RAISES the ceiling; it does **not** remove the class."* The block sets `rlim_cur = min(1 GiB, rlim_max)`.

Single-variable experiment (same binary, same program, only the hard limit differs), 6 samples per arm:

| RLIMIT_STACK hard | exit | stderr |
|---|---|---|
| unlimited → raise takes effect (**the default**) | `139` ×6 | **0 bytes** |
| capped 8 MiB → raise is a no-op | `134` ×6 | 95 bytes: `thread 'main' has overflowed its stack / fatal runtime error: stack overflow, aborting` |

Out of the box the failure is a **bare silent segfault** — no message, no exit-code signal, nothing to log. Capping the limit *restores* the diagnostic. The stopgap traded a labelled abort for a mute one, and said so nowhere. (Likely mechanism — Rust records the main stack's guard bounds in `lang_start`, before `main` runs the `setrlimit`, so the fault lands outside the range its SIGSEGV handler recognises. **Inferred, not verified.**)

### L1-4. One typo'd config setter silently discards every setter after it

Default `dim-count` is 10000. Driven, all exit 0, no diagnostic:

| entry file | observed `dim-count` |
|---|---|
| `(:wat::config::set-dim-count! 4096)` | `4096` |
| `(:wat::confg::set-dim-count! 4096)` | `10000` |
| `(:wat::config::set-dim-count 4096)` | `10000` |
| `(:wat::config::rete::setmax-fire-rounds! 5)` **then** `(:wat::config::set-dim-count! 4096)` | **`10000`** |

The last row is the sharp one: a missing hyphen in `set-max` fails the shape predicate at `src/config.rs:465-469`, which takes the `_ =>` arm at `:470-473` — `remainder_start = Some(i); break;` — ending the setter section, so the *valid* setter behind it is never processed.

And the guard that exists to catch exactly this is **unreachable**: `remainder_start` is assigned at one site immediately followed by `break`, so `if remainder_start.is_some()` at `src/config.rs:477` can never see `Some`. `ConfigErrorKind::SetterAfterNonSetter` is dead, and the documented rule at `src/config.rs:28-29` (*"A setter appearing after a non-setter is an error"*) is unenforced.

The same doc contradicts itself three lines apart: `:31-32` *"Required fields (`dims`, `capacity-mode`) must be set"* vs `:34-36` *"every field has a default."*

### L1-5. A security-load-bearing invariant asserted enforced by three sites and enforced by none

`crates/wat-reader/src/identifier.rs:78-79`:
> *"The lexer now **REJECTS** all raw control characters in source (Stone 249 scope-closure), so this invariant is **ENFORCED** by the lexer, not merely conventional."*

U+0001 is the separator `env_key` splices between a name and its hygiene ScopeIds (`src/scope/resolution.rs:~95`), so a name containing one can forge a scoped key. Driven — source contains a literal `0x01` inside an identifier (`od -c` shows `a 001 b`):
```
$ ./target/release/wat ctrl.wat
"1"
rc=0
```
The lexer accepts it. The second claimed layer, `Identifier::bare`'s `debug_assert!`, is compiled out — `Cargo.toml` has **no `[profile.release]` section**. And `src/scope/resolution.rs:73-78` explicitly declines to re-assert *"the invariant is guaranteed before this function is called."*

**This is your class #3 (each layer defers to the other) and I am naming it as such.** What is new: the previous instance was an *omission* (nothing rejects a phantom head). This one is a **positive assertion that enforcement exists**, on a separator the macro-hygiene boundary depends on. A reader who greps for the guard finds the sentence saying it is there. The gap is proven *absent*; it is **not** proven exploitable (a collision needs the runtime ScopeId, a global counter already far past small values).

### L1-6. `ed25519-dalek` linked unconditionally; nothing on any real path verifies anything

Root `Cargo.toml` has **no `[features]` table** — `ed25519-dalek`/`sha2`/`base64` always compile in. `resolve_loads` picks the verification mode from the head keyword **the source itself wrote** (`src/load.rs:653-674`): `:wat::load-file!` → `verification: None`, a silent no-op. The check is chosen by the code being checked.

```
$ grep -rn "signed-load!\|digest-load!" wat/ wat-scripts/ wat-tests/ --include=*.wat
(no output)
```
Zero opt-ins in the entire corpus. `docs/arc/2026/06/295-signed-code-only/DESIGN.md:15-16` says it plainly — *"Signing is opt-in today."* The crate ships the dependency and the promise; no path takes it. *(Load-path tracing relayed from the attack-surface sweep; I verified the corpus grep and the absent `[features]` myself.)*

### L1-7. Two shipped diagnostics point at files that are dead or absent

- `src/check/error.rs:706` → *"See `examples/console-demo/wat/main.wat` for the canonical ambient-stdio shape."* That file fails `--check` with **4 errors** (`:wat::core::enum` retired at Stone 241.9, line 29; two `UnknownCallee`; a retired `:wat::core::nil` in value position). Verified by running it.
- `src/check/error.rs:621` → *"File path mirrors: wat/std/stream.wat → wat/stream.wat."* `ls wat/stream.wat` → **No such file or directory**; annihilated 2026-06-27 per `src/stdlib.rs:244`.

`examples/console-demo/src/main.rs` is just `wat::main! {}` — the crate compiles green forever while its `.wat` is dead. That is *why* it rotted: being a `default-member` buys a Rust build, not a wat load.

---

## L2 — real weaknesses

1. **`--help` / `--version` are unimplemented.** Both fall through to the file-open path: `wat: read --help: No such file or directory (os error 2)`, exit **66**. First contact with the tool.
2. **`ThreadOwnedCell::ref_guard`'s safety argument names one caller and has five, in two functions.** `src/rust_deps/custodia.rs:112-113` — *"`eval_peer_select_prime` (the sole caller)"*. Verified: `src/runtime.rs:32867, 33011, 33197` (`eval_peer_select_prime`, fn at `:32753`) **and `:33756, :33854` (`eval_poll_prime`, fn at `:33718`)**. `eval_poll_prime`'s compliance was never argued.
3. **`crates/wat-source-derive` is in `members` but not `default-members`** — while `Cargo.toml:13-18` claims `default-members` *"cover[s] every workspace member … keeps every crate honest at every checkpoint."* One crate outside the gate its own comment declares total.
4. **`Cargo.toml:41` states *"There is no rust-toolchain.toml pin, so the toolchain floats."*** `rust-toolchain.toml` exists and pins `1.97.0`. A manifest comment reasoning from a fact that is false.
5. **Nine doctests are compiled by nothing.** nextest does not run doctests; no job or script passes `--doc`. *(Count relayed, not re-derived.)*
6. **Seven `.wat` files are gated by nothing** — `wat-migrate/fix-decl.wat` (still cited as live doctrine at `src/types.rs:2770`) and six under `tools/`. `wat_scripts_fixes_load.rs:36` walks `wat-scripts` and nothing else. `scripts/floor.sh` is read by zero lints.
7. **Twelve gates carry a false provenance stamp.** e.g. `tests/lint/wat_scripts_fixes_load.rs:40-42`: *"the 445 .wat file(s) this walk finds today … the count comes from `every_walking_gate_declares_non_vacuity.rs`, never from prose."* That gate computes only `files.len()>=25` and `in_scope.len()>=18` over `tests/lint` — it cannot produce 445. Actual today: **452**.
8. **A live tautological assertion outside the 265.** `wat-tests/deporder.wat:71` — `(assert-true (i64::>= (length viols) 0))`. Not ignored, not annotated; can only fail by raising.
9. **`src/kernel/spawn.rs:693` and `crates/wat-macros/src/lib.rs:881` spawn threads that run user wat with no `.stack_size()`.** `RUST_MIN_STACK` appears in exactly two files — `.cargo/config.toml:19` and `.github/workflows/ci.yml:9` — and **zero source files**. The 8 MiB every test measures on is supplied by a cargo config; a library embedder gets 2 MiB. *(Code read; I did not drive a thread peer.)*
10. **`README.md:233` documents `sandbox` as *"the in-process sandbox primitives"*;** `src/sandbox.rs` is a 13-line tombstone with zero items.
11. **`holon = { path = "../holon-rs" }` is a bare path dep with no version** — a user who clones the advertised `repository` cannot build.
12. **The `4381` census figure came from an uncommitted instrument.** Not reproducible by any of ten greps at HEAD or at any commit in the last 400. The gap is now fully explained: **5420 = 4978 Rust + 442 `.wat` deftests** generated by `crates/wat-macros/src/lib.rs:701`→`discover.rs:70` from `wat-tests/`, which exist in no `.rs` file; skips = 15 `#[ignore]` + 5 `default-filter` + 1 wat-side (`wat-tests/lint.wat:97`, which means **the wat-side stdlib lint is currently off**). *(Derivation relayed; arithmetic closes exactly.)*

---

## L3 — what all eighteen structurally failed to see, and why

**The shape: every instrument in this repo measures `wat` as a library under cargo. Nothing measures the program the user runs.**

The 5420 tests reach `startup_from_source` / `apply_function` directly, or run on `wat::test!`-spawned threads. **Not one enters `distribution::run_with_args`** — the actual `main`, and the only place the process gets configured: RLIMIT_STACK, signal handlers, the argv ambient, the exec image fd, and the four-way mode dispatch. `grep run_with_args tests/` returns nothing but comments.

So an entire register is invisible **by construction**, not by oversight:

- **Ordering is not a line.** The `--check`/runtime divergence (L1-1) lives in the fifty lines between `mod.rs:351` and `mod.rs:401`. There is no wrong line there to find. An inward lens asks *"is this line right?"*, and a line can only be wrong relative to a spec the lens can see. A *prefix of setup* has no line to be wrong.
- **Mode is not a line.** Four entry paths — bare file, `--check`, `--mcp`, `--repl` — each run a different prefix of that setup, and nothing anywhere asserts they agree. Two of the four sit before the stack raise. Nobody chose that; the returns were added above it, one arc at a time.
- **Failure *mode* is not a line.** `139`-with-nothing versus `134`-with-a-message is not visible to any assertion in the tree, because no test observes the shipped binary dying.
- **The default is not a line.** `dim-count = 10000`, `--help` → exit 66, a config typo silently voiding its successors: all correct code, no wrong line, an artifact behaving in a way nobody chose.

This is the same root as `temperare`'s closing note, generalised. Its version was *"every counter measures occurrences of an operation the design names; none measures lookups performed."* The general form is: **every gate here measures something the design gave a name to. The process, the mode, the ceiling, the exit code, the ordering are the arrangement the named things run inside — and an instrument built from named things cannot resolve the arrangement.**

Two multipliers made it durable:

**The gates grew where the arcs worked, and coverage was then read as a decision.** Of the ~265: 26 read `src/`, 17 `tests/`, 10 `crates/`, 9 `wat/`, 7 `wat-scripts/`, 5 `docs/`, 1 `benches/`, 1 `examples/` — and **0 read `scripts/`, `tools/`, `workflows/`, or `wat-migrate/`**. Those are not bounded-by-design surfaces with a rune; they are surfaces no arc ever landed on. `floor.sh` — the instrument the entire floor discipline runs through — is gated by nothing.

**And the baseline everyone measured against is 4× smaller than the number.** 265 test fns = **99 gates + 166 unit tests of the gates' own parsers**; 32 of the 99 are `macro_rules!` shards of just two gates. **~67 distinct structural gates.** Every ward that wrote "measured against the 265 lints" was calibrating against a figure that is 63% self-test — the same defect as the `4381`, one level up: *a count nobody re-derived, load-bearing for eighteen coverage claims.*

The extirpare rung available here is the top one, and it is cheap. The whole class collapses if the process setup stops being a prefix of one function: **hoist the RLIMIT_STACK raise (and signal wiring) above every mode return in `run_with_args`, so no mode can be constructed that skips it** — the mistake becomes unwritable rather than caught. Then one gate that drives the real binary in each of the four modes over the same fixture set and asserts they agree on verdict and exit code. That single gate is a surface eighteen wards had no way to reach.

---

## What I could not check, and why

- **I ran no floor and no clippy** (one-build-at-a-time; the orchestrator's numbers stand). I drove only the prebuilt `target/release/wat` (mtime 00:47, after HEAD's commit) — **I could not prove that binary matches HEAD byte-for-byte.** Every driven result is against it.
- **The thread-peer stack gap (L2-9) is a code read, not driven.** I did not construct a `(:wat::spawn::thread)` peer with deep recursion; the syntax cost exceeded the time. The *claim* is only that `.stack_size()` is absent and `RUST_MIN_STACK` is in no source file.
- **The mechanism behind L1-3's silencing is inference**, not verified. The measurement (6/6 both arms) stands on its own; the explanation does not.
- **L1-5 proves the guard absent, not the hole exploitable.** No capture was demonstrated.
- **Relayed, not personally read:** the `unsafe` enumeration (~143 sites), `ScopedLoader`'s canonicalize containment and the location of its tests, the io_uring blocks, the doctest count of 9, the 442-deftest derivation, and the per-file lint census. Everything I put in L1 I read or drove myself.
- **Unswept entirely:** `benches/`, `workflows/vigilia.js`, `crates/wat-edn/`'s `clj`/`interop-tests` trees, `--repl`, and the *installed* Aug-23 binary that is the live MCP server. Do not read these as clean.
- **Convention conflict, flagged not resolved:** `wat-rs/CLAUDE.md` requires scratch `.wat` to live in `wat-scripts/scratch-pad/`; my write-nothing orders put my probe files in the session scratchpad instead. They are therefore **not** covered by `every_wat_scripts_file_loads`. If any probe is worth keeping, it needs re-homing under that gate.
