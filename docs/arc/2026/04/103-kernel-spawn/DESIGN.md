# Arc 103 — `:wat::kernel::spawn` (kernel pipes for sandboxed runs) — DESIGN

**Status:** OPEN — drafted 2026-04-29 mid-arc-093 follow-up. Kills
the `Vec<String>`-shaped sandbox stdio surface. The substrate stops
trafficking in `Vec<String>` at the kernel boundary; real kernel
pipes are the surface for real work; `Vec<String>` survives only
inside the test convenience layer where collected output is the
assertion target.

**Predecessors:** arc 007 (`run-sandboxed`), arc 012
(`fork-program-ast` — already does this exact shape over `fork(2)`
+ pipes; this arc is its in-process sibling).

**Foundational reference:** ZERO-MUTEX.md §"Mini-TCP via paired
channels — the canonical mutex-replacement pattern" (named during
arc 089). The user invented this term for the Console / Service
shape: bounded(1) request channel + bounded(1) ack channel,
producer writes-then-blocks-on-read, mutual blocking IS the
synchronization. Arc 103 transports the SAME discipline over
kernel pipes with EDN+newline as the framing.

---

## What's wrong today

`:wat::kernel::run-sandboxed` is `(src, Vec<String>, scope) ->
RunResult { stdout: Vec<String>, stderr: Vec<String> }`.

- **Buffer-in / buffer-out.** Caller hands all input up front;
  child runs to completion on the calling thread; caller harvests
  all output afterward.
- **No back-pressure either way.** Child can't pace its consumer;
  caller can't pace the child.
- **No interleaving.** A bidirectional protocol — child asks, parent
  answers, child asks again — is impossible.
- **Dishonest serialization.** `stdin` arrives as `Vec<String>`,
  gets `join("\n")`'d, gets handed to a `StringIoReader` backed by
  an in-memory `Vec<u8>`. Three representations for one stream.
  Real `pipe(2)` substrate has existed since arc 012; the
  `Vec<String>` shape was a workaround that outlived its excuse.

The principle moving forward: **`Vec<String>` is banned at the
substrate stdio boundary. Real kernel pipes are the surface for
real work. `Vec<String>` survives only inside the wat-level test
helper where collecting output to a vector is the entire point of
the helper.**

---

## What ships

One new substrate primitive. One struct value. Thread driver.
Drop-cascade. Existing primitives (`IOWriter/println`,
`IOReader/read-line`, `:wat::edn::write`, `:wat::edn::read`)
compose into the mini-TCP idiom at every wat callsite. **No
adapter layer.** The OS pipe IS the bounded channel.

### `:wat::kernel::spawn-program`

Name disambiguates from existing `:wat::kernel::spawn` (function-on-
thread, shared symbol table). This primitive spawns a whole wat
PROGRAM with its own `:user::main` and its own frozen world.

```scheme
(:wat::kernel::spawn-program
  (src   :String)
  (scope :Option<String>)
  -> :wat::kernel::Process)

;; :wat::kernel::Process is a struct:
;;   { stdin   :wat::io::IOWriter      ;; we write here, child reads
;;     stdout  :wat::io::IOReader      ;; child writes, we read
;;     stderr  :wat::io::IOReader      ;; child writes, we read
;;     join    :wat::kernel::ProgramHandle<()> }
```

Implementation: alloc three `pipe(2)` pairs; spawn a `std::thread`;
the thread builds the inner `FrozenWorld` via `startup_from_source`
and calls `invoke_user_main` with the **child ends** of the three
pipes (PipeReader for stdin, two PipeWriters for stdout/stderr);
return the **parent ends** as a `Process` struct value.

Sibling primitive `:wat::kernel::spawn-program-ast` mirrors today's
`run-sandboxed-ast` for AST entry. Two Rust dispatch arms; identical
thread driver underneath.

**No `spawn-program-hermetic-ast`.** Today's hermetic distinction
means **separate-OS-process isolation** — `wat/std/hermetic.wat`'s
`run-sandboxed-hermetic-ast` is already a wat-level wrapper over
`fork-program-ast`, which gives the inner program its own address
space and fresh frozen world. For an in-thread spawn the only
remaining sense of "hermetic" is "don't inherit the caller's
Config" — and that's a wat-level discipline (have the inner forms
declare their own `(:wat::config::set-*!)` preamble), not a
substrate primitive. Two substrate primitives plus the existing
`fork-program-ast` cover the matrix; nothing called "hermetic"
needs to live at the Rust layer.

### Drop-cascade

Same discipline as crossbeam channels, transported:

- Parent drops `proc.stdin` (the IOWriter) → underlying OwnedFd
  drops → `close(2)` on the pipe write-end → child's
  `IOReader/read-line` returns `:None`.
- Child returns from `:user::main` → child's writer Arcs drop →
  pipe write-ends close → parent's `IOReader/read-line` on
  `proc.stdout` / `proc.stderr` returns `:None`.
- Either side panics → its handles drop → the other side sees EOF
  on its next read and unwinds gracefully.
- `(:wat::kernel::join (proc.join))` returns `:()` on clean exit
  or `ThreadDiedError` on inner panic (existing `join-result`
  semantics, arc 060).

---

## Mini-TCP over kernel pipes — the wat-level idiom

Same shape as ZERO-MUTEX.md's pair-by-index Console handle, with
the transport swapped: kernel `pipe(2)` instead of crossbeam
channels, EDN+newline instead of typed Rust values.

```scheme
(:wat::core::let*
  (((proc      :wat::kernel::Process)        (:wat::kernel::spawn-program child-src :None))
   ((_         :())                          (:wat::io::IOWriter/println
                                                (proc.stdin)
                                                (:wat::edn::write request)))
   ((resp-line :Option<String>)              (:wat::io::IOReader/read-line
                                                (proc.stdout)))
   ((response  :MyResponse)                  (:wat::edn::read :MyResponse
                                                              (:wat::core::expect resp-line))))
  ;; control delegated to child during read-line; child wrote response;
  ;; control returns. Repeat for next request.
  ...)
```

The producer's `println` IS the bounded send. The producer's
`read-line` IS the recv that blocks until the consumer's response.
Mutual blocking IS the synchronization. The discipline is the same
as in-process Console with `(Tx, AckRx)` paired by index — the
substrate just swapped the bytes-on-the-wire layer.

**Symmetric on the child side.** `:user::main` reads stdin via
`IOReader/read-line`, deserializes via `:wat::edn::read`, does its
work, writes via `:wat::edn::write` + `IOWriter/println` to stdout.
When stdin EOFs (parent closed the pipe), the child's match arm on
`(read-line stdin)` hits `:None`, returns up through main, exits.

**The protocol is the contract.** Both sides write valid EDN+newline
on every exchange. A misbehaving caller (multi-line EDN, garbage
bytes, missing trailing newline) breaks the protocol; the substrate
does not absorb the consequence. Same discipline as wat already
applies everywhere — the substrate measures, the user decides.

---

## Three-question discipline

**Simple?** One substrate primitive (with two siblings for the AST
/ hermetic entry points — three Rust arms total). One struct
registration. One thread driver. The substrate gets SMALLER on
net: `Vec<String>` exits the kernel; `eval_kernel_run_sandboxed*`
Rust impls (~200+ LOC of buffer juggling) delete; the wat-level
helper that absorbs the test convenience is ~30 LOC.

**Honest?** Pipes are pipes. The OS pipe IS the bounded channel,
not a fake channel hiding a buffer. The `Vec<String>` shape is
allowed only where it earns its keep — the wat-level test helper
that collects output explicitly so tests can assert on it.

**Good UX?** Three callers shape:
- Test author: existing `(:wat::test::run src stdin)` keeps its
  signature. Implementation reroutes through the wat-level helper
  atop `spawn`. Existing tests untouched.
- In-process spawner: `(:wat::kernel::spawn-program src :None)` returns
  Process; the wat-level idiom is `println` + `read-line` + EDN
  serialize/deserialize at the call site. No adapter to learn.
- Cross-process spawner (shell, future Clojure interop): same wat
  program. Already works today via wat-cli's RealStdin/Stdout/Stderr;
  no spawn needed for that direction. Symmetric.

---

## What changes

### New (substrate Rust)

- `src/spawn.rs` (new) — `eval_kernel_spawn_program`,
  `eval_kernel_spawn_program_ast`. Each allocates 3 `pipe(2)` pairs,
  spawns a `std::thread` running `invoke_user_main`, returns a
  `Process` struct value with the parent-side pipe ends.
- `src/runtime.rs` — register `:wat::kernel::Process` as a struct
  value; dispatch arms for the three new primitives.
- `src/check.rs` — schemes for the three new primitives + the
  `Process` struct type.

### Migrated (substrate Rust → wat-level)

- `src/sandbox.rs::eval_kernel_run_sandboxed` — **DELETE**.
- `src/sandbox.rs::eval_kernel_run_sandboxed_ast` — **DELETE**.
- `src/sandbox.rs::eval_kernel_run_sandboxed_hermetic_ast` —
  **UNTOUCHED.** It's already a thin wrapper that's been replaced
  in practice by `wat/std/hermetic.wat` over `fork-program-ast`. If
  the Rust impl still exists at the start of slice 103b, that's a
  separate cleanup; arc 103 doesn't need to remove it.
- `src/check.rs` — drop the three `run-sandboxed*` schemes.
- `wat/std/sandbox.wat` (new) — wat-level `run-sandboxed` /
  `-ast` / `-hermetic-ast` definitions atop `spawn`. Each:
  spawn the inner; write each input line via `IOWriter/println`;
  drop the stdin writer; read-all-lines from stdout + stderr;
  join; return RunResult. The `Vec<String>` shape lives here,
  and ONLY here, in service of the test surface that needs it.
- `wat/std/test.wat` — its `run` / `run-with-scope` / deftest
  helpers keep their signatures and now resolve through the
  wat-level `run-sandboxed` instead of the deleted Rust primitive.

### Untouched

- `:wat::kernel::pipe` — already exists, already correct.
- `:wat::kernel::fork-program-ast` — heavyweight OS-process sibling.
  Untouched. The `Process` struct shape mirrors `ForkedChild`'s
  but the lifetimes differ (JoinHandle vs pid + waitpid); we don't
  unify them.
- All existing test files — their assertions hit RunResult.stdout
  / .stderr the same way; only the underlying mechanism changed.

---

## Slices

**103a — substrate primitive.** `Process` struct registration +
`spawn-program` / `spawn-program-ast` Rust dispatch + schemes +
thread driver + drop-cascade unit test (child writes one line,
returns; parent reads the line, second read returns `:None`).

**103b — wat-level `run-sandboxed` helpers.** `wat/std/sandbox.wat`
with `run-sandboxed` / `-ast` / `-hermetic-ast` atop spawn.
`wat/std/test.wat` reroutes. Delete the three substrate Rust
`eval_kernel_run_sandboxed*` arms + their schemes. Run full
`cargo test` — the test suite is the proof.

**103c — dispatcher demo (resumes paused arc 093 follow-up).**
A wat-script that demonstrates `echo '{:db-path
"./demo.db" :query-program "./count-logs.wat"}' | wat
dispatch.wat` end-to-end. Reads one EDN line from stdin, parses,
spawns the named query program with the db-path written to its
stdin, forwards the inner program's stdout. Proves mini-TCP over
kernel pipes via the simplest possible producer (the shell's
`echo`).

**103d — INSCRIPTION + ZERO-MUTEX.md update.** New subsection
under "Mini-TCP via paired channels" titled "Mini-TCP across
kernel pipes" pointing at `:wat::kernel::spawn` and the wat-level
idiom. INSCRIPTION captures the `Vec<String>`-banned-from-substrate
discipline as the load-bearing decision. USER-GUIDE row in §7
(Concurrency) added: spawn alongside send/recv/select. 058 row in
the language spec.

---

## Open questions resolved upfront

1. **Inner thread vs. inner cooperative scheduler.** Use a thread.
   The single-thread shortcut breaks the moment input exceeds the
   OS pipe buffer (~64KB Linux default).

2. **Process value shape.** Struct, not a new `Value` variant.
   Same shape as `ForkedChild`; registers via the existing
   struct-value pathway.

3. **Framing.** Line-delimited EDN. One value per line. Multi-line
   EDN values violate the protocol; caller's responsibility.

4. **Error semantics for malformed-on-the-wire EDN.** Same as
   existing `:wat::edn::read` — returns Result; caller decides.
   No new error wrapper at the spawn layer; spawn deals in bytes.

5. **Capacity / framing adapters.** None. The OS pipe is the
   bounded channel. Adding a bounded crossbeam adapter on top
   would be a second mini-TCP layer underneath the OS-pipe
   mini-TCP — two of the same pattern stacked. Not honest.

6. **Convenience helpers.** Optional, post-103: `:wat::edn::write-line
   (writer value)` and `:wat::edn::read-line :T (reader)` to fuse
   the two-step pattern at every callsite. Wat-level `define`s, not
   substrate. Land if/when a caller's verbosity demands it.
