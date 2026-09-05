;; wat-tests/kernel/services/ambient-stdio.wat — consumer-vantage tests for
;; the ambient stdio surface (:wat::kernel::println / eprintln / readln).
;;
;; Arc 170 slice 1f-θ V3. Replaces the deleted implementer-vantage trio
;; tests (stdin.wat, stdout.wat, stderr.wat) — those hand-built
;; Event::Add / Event::Write / Event::Remove wire frames a consumer would
;; never touch. Per /vocare, a consumer of (:wat::kernel::println) writes
;; the println call; routing-table manipulation is mechanism.
;;
;; Slice mission — verify the forked-child orchestrator path (slice 1f-γ)
;; continues to work after the arc 170 migration. deftest-hermetic (NOT
;; deftest) on every test — the run-hermetic' fork boots bootstrap-fn →
;; spawns the trio services → registers thread-0, so the assertion body
;; runs INSIDE a forked orchestrator child (that boot is what's under test).
;;
;; Arc 278 IPC de-prime — the DRIVERS flip onto the PRIMED PEER WIRE. Where
;; each test used to fork a grandchild and SCRAPE its OS stdout/stderr into a
;; byte-stream RunResult (Layers 0-3 via :wat::test::run-hermetic) or wrap its
;; fds as typed channels (Layer 4 via :wat::test::run-hermetic-with-io), it now
;; spawns the child as a process PEER (`spawn-program' (process)`) and reads its
;; output over the wire: each child `println v` crosses to the parent as a
;; `recv'` `RecvOutcome::Message` carrying the DECODED native value v (NOT the
;; EDN-quoted stdout line the scrape produced — "hello", not "\"hello\""). A
;; child that DIES (eprintln is terminal) surfaces as `RecvOutcome::Lost[cause]`
;; / `recv-all'` `Err[cause]`, its LociDiedError NEVER swallowed. The child
;; bodies (println/eprintln/readln) are unchanged — only the driver flips.
;;
;; Arc 278 — the prelude is annihilated. Each test's inner program rides INLINE
;; in the deftest body; the assertion sits at the top of the diagnostic surface
;; so a failure names the layer.

;; ─── Layer 0 — println a String ─────────────────────────────────────────
;; The forked child writes "hello" over the peer wire; on the primed wire the
;; value crosses DECODED (native String "hello", not the scraped EDN line
;; "\"hello\"" the byte-stream model produced). recv' → Message[m]; m == "hello".
(:wat::test::time-limit "15000ms")

(:wat::test::deftest-hermetic :wat-rs::test::test-ambient-stdio-println-string

  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "hello"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m)
        (:wat::test::assert-eq m "hello"))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      ;; arc 278 #73 — a stop, not a close: the child was ALIVE.
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "println-string: stop requested before the child sent its value — child was ALIVE" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "println-string: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ─── Layer 1 — println an i64 ───────────────────────────────────────────
;; Non-string Ts cross the wire through the same peer pipeline — the i64 42
;; arrives DECODED as the native i64 42 (not the decimal EDN line "42").
(:wat::test::time-limit "15000ms")

(:wat::test::deftest-hermetic :wat-rs::test::test-ambient-stdio-println-i64

  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println 42))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m)
        (:wat::test::assert-eq m 42))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      ;; arc 278 #73 — a stop, not a close: the child was ALIVE.
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "println-i64: stop requested before the child sent its value — child was ALIVE" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "println-i64: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ─── Layer 2 — eprintln a String ────────────────────────────────────────
;; eprintln is a TERMINATING form — it emits the value's EDN then CRASHES the
;; child (src/services/verbs.rs `eprintln_terminate`). Over the peer wire the
;; crash surfaces as `recv'` Lost[cause]; the emitted value's EDN rides the
;; crash reason (`LociDiedError/message cause`, proven in
;; tests/process/probe_arc278_init_crash_reason.wat — the reason carries the
;; sentinel). This reproduces the old `assert-stderr-matches ... "err"` (a
;; regex match for the emitted line) off the crash reason instead of OS-stderr
;; capture, which the wire model drops. A Message (no crash) is the failure —
;; a terminal eprintln must never let a following form (or a value) through.
(:wat::test::time-limit "15000ms")

(:wat::test::deftest-hermetic :wat-rs::test::test-ambient-stdio-eprintln-string

  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::eprintln "err"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message _m)
        (:wat::kernel::assertion-failed! "eprintln-string: eprintln is terminal — expected the child to crash before any value, but a value arrived" :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::core::if (:wat::regex::matches? "err" (:wat::kernel::LociDiedError/message cause))
          nil
          (:wat::kernel::assertion-failed! "eprintln-string: crash reason did not carry the emitted value"
            (:wat::core::Some (:wat::kernel::LociDiedError/message cause))
            (:wat::core::Some "err"))))
      ;; arc 278 #73 — a stop, not a close: the child was ALIVE.
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "eprintln-string: stop requested before the child sent its value — child was ALIVE" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "eprintln-string: child closed before crashing" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))

;; ─── Layer 3 — two println calls, order-preserving ──────────────────────
;; Two round trips through the same peer pipeline land in send order. recv-all'
;; drains the peer honestly until it closes → Ok[["first" "second"]] (decoded
;; native Strings), or Err[cause] if the peer died (surfaced, never swallowed).
(:wat::test::time-limit "15000ms")

(:wat::test::deftest-hermetic :wat-rs::test::test-ambient-stdio-println-twice

  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::do
               (:wat::kernel::println "first")
               (:wat::kernel::println "second")
               nil))))]
    (:wat::core::match (:wat::kernel::recv-all p)
      ((:wat::core::Ok outputs)
        (:wat::test::assert-eq outputs (:wat::core::Vector :- [:wat::core::String] "first" "second")))
      ((:wat::core::Err cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None)))))

;; ─── Layer 4 — readln round trip via the bidirectional peer wire ─────────
;; The parent sends a native String "echo me" INTO the child's readln over the
;; peer lineage (`send'`); the child's (readln) reads + decodes it, (println
;; echoed) writes it back; the parent drains the doubled output off the peer
;; (`recv-all'`) → Ok[["echo me"]]. Exercises both halves of the trio + the
;; symmetric wire EDN encode/decode. Bounded I/O: one send' → one output →
;; child exits (Closed → recv-all' returns Ok).
(:wat::test::time-limit "15000ms")

(:wat::test::deftest-hermetic :wat-rs::test::test-ambient-stdio-readln-echo

  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::core::let
               [echoed (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
               (:wat::kernel::println echoed)))))
     _ (:wat::core::match (:wat::kernel::send p "echo me")
         (:wat::kernel::SendOutcome::Sent nil)
         (:wat::kernel::SendOutcome::Closed nil)
         (:wat::kernel::SendOutcome::Stopped nil)
         ((:wat::kernel::SendOutcome::Lost _c) nil))]
    (:wat::core::match (:wat::kernel::recv-all p)
      ((:wat::core::Ok outputs)
        (:wat::test::assert-eq outputs (:wat::core::Vector :- [:wat::core::String] "echo me")))
      ((:wat::core::Err cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None)))))
