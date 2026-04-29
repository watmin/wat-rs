# wat-edn — IPC Bridge Vision

**Status:** vision doc. Not shipped behavior. Captures the
destination so when we build the bridge later, the intent is
recoverable.

**Author note** *(2026-04-28)*. wat-edn ships with a JSON bridge
and a Clojure companion. Both sides speak EDN; both sides are
type-fidelity-preserving. The natural next move — and the one
that closes the loop on "Clojure orchestrates, wat computes" —
is process-level IPC: a Clojure parent spawns a wat child, they
exchange EDN forms over the kernel pipes (stdin / stdout /
stderr), and neither side has to think about the wire format
because wat-edn and wat-edn-clj already agree.

This is the doc that names that future and the open questions it
carries.

---

## 1. Why

> *I needed wat to do what Clojure couldn't. But I want Clojure's
> access to wat to be simple.* — the framing.

The two languages occupy complementary slots:

```
Clojure                         wat
───────────────────             ──────────────────────────────
fast iteration                  deterministic substrate
REPL-driven exploration         holon algebra primitives
dynamic / late-bound            spec-strict, type-checked
ubiquitous ecosystem            zero-mutex concurrency,
JVM heap, GC pauses             arena-friendly, predictable
data slinging, shaping          heavy compute, simulation,
                                cryptographic determinism
```

The bridge that already exists: **wire-format agreement**. Both
sides read and write the same EDN bytes. Same wire convention for
JSON. Same `.wat` schema files.

The bridge that's missing: **process-level coordination**. A
Clojure program can produce EDN bytes today, but there's no
ergonomic way to:

1. Spawn a wat program as a subprocess
2. Stream EDN values to its stdin
3. Read EDN values back from stdout
4. Read EDN log/diagnostic values from stderr
5. Manage the child's lifecycle (start, signal, drain, wait)

That's what this vision doc describes.

---

## 2. The picture

```
              Clojure parent process
              ┌──────────────────────────┐
              │  application logic       │
              │  ┌────────────────────┐  │
              │  │ wat-edn-clj.proc   │  │ wraps
              │  │   spawn / send! / │  │ ProcessBuilder
              │  │   recv / close!    │  │
              │  └────┬───┬───┬───────┘  │
              │       │   ▲   ▲          │
              └───────┼───┼───┼──────────┘
                      │   │   │
                  stdin  stdout stderr
                      │   │   │      ← kernel pipes
                      ▼   │   │
              ┌───────────┼───┼──────────┐
              │           │   │          │
              │   :user::main   ←── EDN  │
              │   reads stdin            │
              │   writes stdout / stderr │
              │                          │
              │   wat child process      │
              │   (cargo-built binary)   │
              └──────────────────────────┘
```

Each pipe carries newline-or-self-delimited EDN forms. Both
sides treat the channel as a stream of typed values, not bytes.

---

## 3. What exists today

### Wire format
- **wat-edn** — `parse`, `parse_owned`, `parse_all`, `Parser`
  for incremental reading; `write`, `write_to`, `write_pretty`
  for emission. ✅ Solid.
- **wat-edn-clj** — `read-str`, `read-stream` (consumes a
  `PushbackReader`), `to-json-string`, plus all the schema-driven
  helpers. ✅ Solid.
- Schema sharing via `.wat` files — `load-types!` reads the same
  artifact wat-rs's type checker consumes. ✅ Solid.
- Cross-language byte-level handshake — verified by
  `interop-tests/` (four pipelines green). ✅ Solid.

### wat I/O primitives
- `:user::main` signature: `(stdin :IOReader) (stdout :IOWriter)
  (stderr :IOWriter) -> :()`. ✅ Established.
- `:wat::io::read-line!` / `write-line!` — line-by-line ops.
  ✅ Exists; used by examples.
- `:wat::edn::read-str` / `write-str` — wat-edn integration into
  wat (when wat-rs takes the dep). ⚠ Pending — that's the next
  arc on the wat-rs side.

### Clojure subprocess primitives
- `clojure.java.shell/sh` — fire-and-forget; not streaming. ❌
  Wrong shape for our use case.
- `java.lang.ProcessBuilder` — full control, low-level. ✅
  Available, but raw.
- `babashka.process` — modern wrapper, streaming-friendly,
  graceful shutdown. ✅ Available as a dep; would be the
  foundation for a wat-edn-clj.process namespace.

---

## 4. What's missing

### The wat side

**A streaming EDN reader for `:user::main`'s stdin.** Today
`(:wat::io::read-line!)` reads one line as a String; we'd need
`(:wat::edn::read-stdin!)` that reads exactly ONE EDN form
(parsing across newlines if the form spans them) and returns the
parsed `Value`. Same shape as Clojure's `(clojure.edn/read
*in*)`.

Implementation route: Once wat-rs takes the dep on wat-edn, this
is a thin wrapper over wat-edn's `Parser`. The Rust side
maintains a `Parser` over the stdin byte stream; the wat side
calls `read-stdin!` to pop one form.

**Service-loop helper.** A common pattern:

```scheme
(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::edn::serve! stdin stdout
    (:wat::core::lambda (req :MyReq) -> :MyResp
      ; ... compute ...)))
```

The `serve!` form is sketched: it loops, reads one EDN form per
iteration, calls the handler, writes the response, exits cleanly
on EOF. Single-threaded; bounded; handles parse errors by
emitting an error tagged-element on stderr.

**Buffered vs line-buffered stdout.** Today's emission is fine
for batch programs. For interactive IPC, the parent expects
prompt response — wat-edn's writer should flush after each form,
or expose a `:flush-after-write` mode.

### The Clojure side

**A `wat-edn-clj.process` namespace.** Wraps `babashka.process`
to provide a typed-channel API:

```clojure
(require '[wat-edn.process :as wp])

;; Start a wat program as a subprocess. Returns a record with
;; :in (writer for stdin), :out (reader for stdout EDN forms),
;; :err (reader for stderr EDN forms), :proc (the underlying
;; process handle).
(def proc
  (wp/spawn ["./target/release/my-wat-program"]
            {:env {"LOG_LEVEL" "info"}
             :workdir "."}))

;; Send an EDN form to wat's stdin (blocking until pipe accepts).
(wp/send! proc {:command :compute :input [1 2 3]})

;; Read one EDN form from stdout (blocking until form arrives).
(def result (wp/recv proc))

;; Non-blocking poll on stderr for log forms.
(when-let [log (wp/poll-err proc)]
  (println "wat says:" log))

;; Graceful shutdown: close stdin (signals EOF to wat), wait for
;; exit, return the exit code.
(wp/close! proc)
```

Implementation route: ~200 LOC of Clojure over `babashka.process`
plus `wat-edn.json/clojure.edn` for parsing.

### Protocol decisions still open

These haven't been chosen yet — see `§5 Open design questions`
below.

---

## 5. Open design questions

### Q1. Framing — newline-delimited or self-delimiting?

EDN forms are self-delimiting (paren-balanced); a streaming
reader knows when a form ends without external markers. Both
clojure.edn's `read` and wat-edn's `Parser::parse_next` work
this way.

**Option A — newline-delimited (one EDN form per line).**
Parent and child agree: every form ends with `\n`. Simpler to
debug (`tail -f` works), but multi-line EDN values need
escaping (or a strict "compact mode").

**Option B — self-delimiting streaming.** Either side reads one
form at a time, paren-balanced. Multi-line values flow naturally.
Slightly harder to debug in raw form.

**Option C — length-prefixed.** Each form preceded by a 4-byte
big-endian length. Robust against partial reads, easy
to skip-malformed-and-recover. More complex framing.

Recommendation when this gets built: **B** for primary, **A**
as an opt-in mode for human-readable streams. **C** only if
robustness against corrupted streams becomes a requirement.

### Q2. Error signaling — three channels for three concerns?

Current shape candidate:

```
stdout    →  EDN responses (the answer to a request)
stderr    →  EDN diagnostic/log forms (telemetry, warnings)
exit code →  process-level success / failure
```

Errors during request processing — does the response on stdout
get a `#error/...` tagged-element, or does the error go on
stderr? Both shapes have ecosystem precedent.

Recommendation: **errors-as-data on stdout** (tagged
`#myapp/Error {...}` or use `Result<T, E>`-shaped EDN), so the
parent's recv loop sees one form per request regardless of
success/failure. stderr stays for telemetry that has no
correspondence to a specific request.

### Q3. Single-request or long-lived service?

Two shapes coexist in the wild:

- **Short-lived:** parent spawns child, sends one request, reads
  one response, child exits. Clean for build tools, scripts,
  one-shot transformations.
- **Long-lived:** child runs a service loop, parent streams many
  requests, child exits only on EOF or explicit shutdown.
  Cleaner for hot-path scenarios where spawn cost matters.

Both should work; the API doesn't need to choose. `wp/spawn`
returns a process; the parent decides whether it sends one or
many requests.

### Q4. Backpressure — who blocks?

Kernel pipes have a fixed buffer (typically 64 KiB on Linux).
When wat child can't keep up, parent's `send!` blocks. When
parent can't keep up reading stdout, wat child's `write-stdout!`
blocks. Standard kernel-pipe semantics; both sides need to
tolerate it.

Open question: should the Clojure side expose async / future
shapes (`(send-async! proc v)` returning a deferred) for parents
that want to overlap I/O with compute? Probably yes, but v0.2.

### Q5. Schema discovery

Today the schema lives in shared `.wat` files. The parent
loads it via `(wat/load-types! "shared.wat")`. The child has it
implicitly (it's wat code). Same file, two readers — the
header-file pattern.

Open question: should the wat child be able to **announce** its
schema on startup? E.g., emit a single
`#wat.ipc/Schema {...}` form on stdout before entering the
service loop, so dynamically-spawned wat programs can
self-describe. v0.2 polish.

### Q6. wat's existing `:wat::io::*` maturity

The user's framing: *"our std in,out,err setup is mature enough
enough — but it's close."*

What's good:
- `:user::main` signature is established
- `read-line!` / `write-line!` work for line-shaped data

What might need work for streaming EDN:
- Reading across newlines (multi-line EDN forms)
- Buffered vs unbuffered writes
- Signal handling (SIGPIPE on parent close, SIGTERM lifecycle)
- Non-blocking reads (for the parent's poll-err pattern)

These are wat-rs concerns, not wat-edn concerns. This vision doc
flags the dependency without scoping the wat-rs work.

---

## 6. Sketched API (both sides)

When the bridge ships, the API surface should look approximately
like this. This is illustrative — actual implementation may
differ.

### wat side

```scheme
;; Read one EDN form from stdin. Returns :wat::edn::Value or
;; signals EOF via :Result error variant.
(:wat::edn::read-stdin!) -> :Result<:wat::edn::Value, :wat::edn::EofOrError>

;; Write one EDN form to stdout. Auto-flush so the parent sees
;; the form promptly.
(:wat::edn::write-stdout! v)
(:wat::edn::write-stderr! v)

;; High-level service loop. Reads forms from stdin, applies the
;; handler, writes responses to stdout. Exits on EOF.
(:wat::edn::serve! stdin stdout handler)

;; Hello-world service:
(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::edn::serve! stdin stdout
    (:wat::core::lambda (req :myapp::Req) -> :myapp::Resp
      (:myapp::compute req))))
```

### Clojure side

```clojure
(ns my-app.core
  (:require [wat-edn.core :as wat]
            [wat-edn.process :as wp]))

;; Schema is shared.
(wat/load-types! "shared.wat")

;; Spawn the wat child.
(def proc (wp/spawn ["./target/release/my-wat-service"]))

;; Synchronous request/response.
(defn ask [req]
  (wp/send! proc req)
  (wp/recv proc))

;; Streaming pipeline.
(defn process-batch [items]
  (doseq [item items]
    (wp/send! proc item))
  (wp/close-stdin! proc)              ; signal EOF
  (loop [results []]
    (if-let [r (wp/recv proc)]
      (recur (conj results r))
      results)))

;; Telemetry tail.
(future
  (loop []
    (when-let [log (wp/recv-err proc)]
      (println "[wat]" log)
      (recur))))

(wp/close! proc)
```

---

## 7. What would prove the bridge works

Same template as `interop-tests/` for the EDN/JSON bridges:
empirical end-to-end pipeline tests that anyone can run.

```sh
cd interop-tests/process

# Start a wat echo service; pipe in EDN forms; read responses.
clojure -M:test process-bridge

# Expected: 100 forms in → 100 forms out, byte-equivalent
# round-trip, stderr captures the wat program's logs.
```

Acceptance bar:
- Round-trip 1000 EDN forms through the pipe with zero data loss
- Graceful shutdown (parent closes stdin → child exits cleanly)
- Both sides handle large multi-line forms (a 1 MB tagged map)
- Backpressure works (parent fills pipe → blocks → child drains
  → parent unblocks)

---

## 8. Adjacent work this enables

Once the bridge ships:

- **REPL-driven wat development.** Clojure REPL spawns a wat
  program, sends forms interactively, sees results. The wat
  program's hot-reload loop is the kernel pipe.
- **wat as a Unix tool.** `cat data.edn | my-wat-tool > out.edn`
  becomes natural. Pipelines of wat programs compose.
- **Mixed-language workers.** Clojure orchestrates a fleet of
  wat workers via spawn-pool patterns. Heavy compute farmed to
  wat; result streams back to Clojure for visualization /
  storage / further composition.
- **Zero-deploy multi-language services.** Any container that
  has Clojure + wat-rs binaries can host the bridge. No
  network, no serialization layer beyond the pipe, no schema
  drift (shared `.wat` file).
- **Cross-process holon algebra.** The wat substrate's
  determinism + EDN's content-addressing + process-level
  isolation = a pipeline of holon transformations where each
  stage is independently verifiable.

---

## 9. Reading list (when this gets built)

Existing prior art worth studying:

- **`babashka.process`** — Clojure-side subprocess wrapper with
  a Clojure-shaped API. Likely the foundation.
- **`clojure.tools.deps.alpha`** — uses subprocess EDN
  communication for sub-tasks; pattern reference.
- **nREPL** — Clojure's network REPL protocol; uses bencode (not
  EDN) but the request/response shape and the
  status-message-as-stream are educational.
- **JSON-RPC and LSP** — line-based JSON protocols for
  process-level IPC. Same shape as what we'd build, different
  wire format. The framing decisions (Q1) and error-signaling
  decisions (Q2) have lots of precedent there.
- **GraalVM polyglot embeddings** — same goal (Clojure calls
  another language), different architecture (in-process). The
  contrast is informative.

---

## 10. When to revisit

Build it when one of these triggers fires:

- A real Clojure consumer wants to drive a real wat program (not
  a synthetic test) and the manual ProcessBuilder dance is the
  friction.
- The wat side gets a big-compute use case (engram libraries,
  cryptographic search) that Clojure consumers want to drive.
- The "wat as Unix tool" use case (`cat | wat | grep`) gets a
  concrete user.
- We hit a third or fourth rebuild of the same ad-hoc spawn
  pattern in different consumer code.

Until then: this doc is the placeholder. The destination is
named; the gaps are named; the design questions are named. When
the trigger fires, the bridge ships in one focused commit cycle
because the thinking is already on disk.

---

*Last updated: 2026-04-28. wat-edn was at commit `ef352d8` when
this was written.*
