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
;; continues to work after the arc 170 migration. Layers 0-3 use
;; :wat::test::run-hermetic (Layer 1, byte-stream RunResult); Layer 4 uses
;; :wat::test::run-hermetic-with-io (Layer 2, typed channel RunResultIO<O>).
;; Both call :wat::kernel::spawn-process underneath, which forks a child
;; that boots bootstrap-fn → spawns the trio services → registers thread-0
;; → runs the user-supplied body. deftest-hermetic (NOT deftest) on every
;; test — the forked-child path exercises the orchestrator boot + service
;; spawn + dup-fds + drain machinery.
;;
;; Arc 278 — the prelude is annihilated. Each test's inner program (formerly
;; a helper defn spliced via make-deftest-hermetic's prelude) now rides
;; INLINE in the deftest body; the run-hermetic fork IS the forked-child
;; path under test, and the assertion sits at the top of the diagnostic
;; surface so a failure names the layer.

;; ─── Layer 0 — println a String ─────────────────────────────────────────
;; The forked child writes the EDN encoding of "hello" (the literal chars,
;; no surrounding quotes) to fd 1, then a newline. RunResult/stdout splits
;; on \n and drops the trailing empty element; the captured vec is one wide.
(:wat::test::time-limit "15000ms")
(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel) — leaks/hangs; remove before arc 170 closes")
(:wat::test::deftest-hermetic' :wat-rs::test::test-ambient-stdio-println-string
  
  (:wat::test::assert-stdout-is
    (:wat::test::run-hermetic (:wat::kernel::println "hello"))
    (:wat::core::Vector :wat::core::String "\"hello\"")))

;; ─── Layer 1 — println an i64 ───────────────────────────────────────────
;; EDN encoding of the i64 42 is the decimal literal "42" (no quotes) —
;; value_to_edn_with covers non-string Ts through the same fd pipeline.
(:wat::test::time-limit "15000ms")
(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel) — leaks/hangs; remove before arc 170 closes")
(:wat::test::deftest-hermetic' :wat-rs::test::test-ambient-stdio-println-i64
  
  (:wat::test::assert-stdout-is
    (:wat::test::run-hermetic (:wat::kernel::println 42))
    (:wat::core::Vector :wat::core::String "42")))

;; ─── Layer 2 — eprintln a String ────────────────────────────────────────
;; eprintln routes to fd 2, not fd 1 (no cross-talk). Arc 278: eprintln is a
;; TERMINATING form — it writes "err" to fd 2, then crashes the child
;; (the #wat.kernel/ProcessPanics envelope follows "err" on stderr;
;; RunResult.failure = Some). assert-stderr-matches "err" still holds (it
;; matches the emitted line regardless of the trailing crash envelope).
(:wat::test::time-limit "15000ms")
(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel) — leaks/hangs; remove before arc 170 closes")
(:wat::test::deftest-hermetic' :wat-rs::test::test-ambient-stdio-eprintln-string
  
  (:wat::test::assert-stderr-matches
    (:wat::test::run-hermetic (:wat::kernel::eprintln "err"))
    "err"))

;; ─── Layer 3 — two println calls, order-preserving ──────────────────────
;; The trio ack-rx blocks after each Write so lines land in send order —
;; order preservation across multiple round trips through the same pipeline.
(:wat::test::time-limit "15000ms")
(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel) — leaks/hangs; remove before arc 170 closes")
(:wat::test::deftest-hermetic' :wat-rs::test::test-ambient-stdio-println-twice
  
  (:wat::test::assert-stdout-is
    (:wat::test::run-hermetic
      (:wat::core::do
        (:wat::kernel::println "first")
        (:wat::kernel::println "second")
        nil))
    (:wat::core::Vector :wat::core::String "\"first\"" "\"second\"")))

;; ─── Layer 4 — readln round trip via typed I/O ──────────────────────────
;; run-hermetic-with-io wraps the child's fd 0/1 as typed channels: the
;; parent sends a native String "echo me" over Process/stdin (EDN-encoded);
;; the child's (readln) reads + parses it, (println echoed) writes it back;
;; the parent's Receiver decodes the EDN line into a native String in
;; RunResultIO/outputs. Exercises both halves of the trio + the symmetric
;; typed-channel EDN encode/decode. T18 bounded I/O: one send → one recv →
;; child exits.
(:wat::test::time-limit "15000ms")
(:wat::test::ignore "arc-170 concurrency layer (subprocess spawn / thread-on-channel) — leaks/hangs; remove before arc 170 closes")
(:wat::test::deftest-hermetic' :wat-rs::test::test-ambient-stdio-readln-echo
  
  (:wat::test::assert-eq
    (:wat::test::RunResultIO/outputs
      (:wat::test::run-hermetic-with-io
        :wat::core::String
        :wat::core::String
        (:wat::core::Vector :wat::core::String "echo me")
        (:wat::core::let
          [echoed (:wat::kernel::readln )]
          (:wat::kernel::println echoed))))
    (:wat::core::Vector :wat::core::String "echo me")))
