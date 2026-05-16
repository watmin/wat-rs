;; wat-tests/kernel/services/ambient-stdio.wat — consumer-vantage tests for
;; the ambient stdio surface (:wat::kernel::println / eprintln / readln).
;;
;; Arc 170 slice 1f-θ V3. Replaces the deleted implementer-vantage trio
;; tests (stdin.wat, stdout.wat, stderr.wat) — those hand-built
;; Event::Add / Event::Write / Event::Remove wire frames a consumer would
;; never touch. Per /vocare, a consumer of (:wat::kernel::println) writes
;; the println call; routing-table manipulation is mechanism. Per
;; /complectens, each layer composes top-down and carries its own
;; deftest so the failure trace names the broken layer.
;;
;; Slice mission — verify the forked-child orchestrator path (slice
;; 1f-γ) continues to work after the arc 170 migration. Layers 0-3
;; use :wat::test::run-hermetic (Layer 1, byte-stream RunResult);
;; Layer 4 uses :wat::test::run-hermetic-with-io (Layer 2, typed
;; channel RunResultIO<O>). Both layers call :wat::kernel::spawn-
;; process underneath, which forks a child that boots bootstrap-fn
;; → spawns the trio services → registers thread-0 → runs the
;; user-supplied body. The body's (:wat::kernel::println v) / readln
;; calls route through the trio to/from the child's fd 0/1/2; the
;; parent drains via OS pipe — as raw lines for Layer 1, as typed-
;; EDN-decoded values for Layer 2. deftest-hermetic (NOT deftest) on
;; every test — the in-process path skips the fd pipeline; only the
;; forked-child path exercises the orchestrator boot + service
;; spawn + dup-fds + drain machinery.
;;
;; Top-down dependency graph (top → bottom; no forward refs):
;;   Layer 0 :test::run-println-string   → run-hermetic         { (println "hello") }
;;   Layer 1 :test::run-println-i64      → run-hermetic         { (println 42)      }
;;   Layer 2 :test::run-eprintln-string  → run-hermetic         { (eprintln "err")  }
;;   Layer 3 :test::run-println-twice    → run-hermetic         { 2× println        }
;;   Layer 4 :test::run-readln-echo      → run-hermetic-with-io { readln → println }

;; ─── Prelude — layered helpers spliced into each deftest ────────────
;;
;; make-deftest-hermetic generates a configured deftest-hermetic that
;; splices the prelude before the synthesized :user::main. Each helper
;; below returns a :wat::kernel::RunResult that the deftest body
;; asserts against. The helpers do NOT assert internally — the
;; deftest body's assert-stdout-is / assert-stderr-matches sits at
;; the top of the diagnostic surface so a failure names the layer.

(:wat::test::make-deftest-hermetic :deftest-ambient
  (
   ;; ─── Layer 0 helper — run inner program that prints "hello" ──────
   ;;
   ;; The narrowest proof: a forked child whose orchestrator routes a
   ;; single println call to fd 1. EDN encoding of the wat::core::String
   ;; "hello" is the quoted form "hello" (with literal double quotes)
   ;; — that's what the trio writes line by line.
   (:wat::core::define
     (:test::run-println-string -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic
       (:wat::kernel::println "hello")))

   ;; ─── Layer 1 helper — run inner program that prints an i64 ───────
   ;;
   ;; Same shape as Layer 0 with a non-string value. EDN encoding of an
   ;; i64 is its decimal literal (no quotes). Proves the trio doesn't
   ;; only handle pre-formatted strings — println renders any T via
   ;; value_to_edn_with.
   (:wat::core::define
     (:test::run-println-i64 -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic
       (:wat::kernel::println 42)))

   ;; ─── Layer 2 helper — run inner program that eprints "err" ───────
   ;;
   ;; eprintln routes to fd 2 instead of fd 1. The RunResult separates
   ;; stdout (empty) from stderr ("err"). Proves the two fd pipelines
   ;; don't cross-talk: a single eprintln lands ONLY in stderr.
   (:wat::core::define
     (:test::run-eprintln-string -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic
       (:wat::kernel::eprintln "err")))

   ;; ─── Layer 3 helper — run inner program with two println calls ───
   ;;
   ;; Two sequential calls. The trio ack-rx blocks after each Write so
   ;; the lines land in send order. Proves order preservation across
   ;; multiple round trips through the same fd pipeline.
   (:wat::core::define
     (:test::run-println-twice -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic
       (:wat::core::do
         (:wat::kernel::println "first")
         (:wat::kernel::println "second")
         :wat::core::nil)))

   ;; ─── Layer 4 helper — readln round trip via Layer 2 typed I/O ────
   ;;
   ;; Layer 2 (run-hermetic-with-io) wraps the child's fd 0/1 as typed
   ;; channels: parent-side Sender<I>/from-pipe over Process/stdin and
   ;; Receiver<O>/from-pipe over Process/stdout. The parent passes a
   ;; native :wat::core::String "echo me"; Sender/from-pipe EDN-encodes
   ;; it onto fd 0 (the wire form is `"echo me"` — the canonical EDN
   ;; quoted form). The child body still runs through the trio
   ;; orchestrator the same way as Layer 1: ambient (:wat::kernel::
   ;; readln -> :String) reads from fd 0 (the StdInService routes the
   ;; line up), parses the EDN-quoted wire form back to a native
   ;; String "echo me", then (:wat::kernel::println echoed) renders
   ;; the native String to its EDN-quoted form on fd 1. On the parent
   ;; side, Receiver/from-pipe over Process/stdout decodes the EDN
   ;; line back to a native String, which lands in RunResultIO/outputs.
   ;;
   ;; The round trip exercises both halves of the trio (stdin reader +
   ;; stdout writer) and the symmetric EDN encode/decode at the typed-
   ;; channel boundary. What changed from the legacy Layer 0 wire
   ;; format: stdin pre-seed used to be a raw EDN string Vec with an
   ;; explicit trailing-newline element to drive IOReader/read-line;
   ;; Layer 2's Sender/from-pipe writes a complete EDN line per typed
   ;; send (newline framing built in), and child exit closes fd 1 so
   ;; the parent's Receiver/from-pipe drain sees EOF cleanly. T18
   ;; bounded I/O: one send → one recv → child exits.
   (:wat::core::define
     (:test::run-readln-echo -> :wat::test::RunResultIO<wat::core::String>)
     (:wat::test::run-hermetic-with-io
       :wat::core::String                                  ;; input element type
       :wat::core::String                                  ;; output element type
       (:wat::core::Vector :wat::core::String "echo me")   ;; native String (Sender/from-pipe EDN-encodes)
       (:wat::core::let
         [echoed (:wat::kernel::readln -> :wat::core::String)]
         (:wat::kernel::println echoed))))
   ))

;; ─── Layer 0 — :test::run-println-string ────────────────────────────
;;
;; The forked child writes the EDN encoding of "hello" — the literal
;; characters "hello" (5 chars, no surrounding quotes) — to fd 1, then
;; appends a newline (IOWriter::writeln contract). RunResult/stdout
;; splits on \n and drops the trailing empty element; the captured
;; vec is one element wide.

(:wat::test::time-limit "15000ms")
(:deftest-ambient :wat-rs::test::test-ambient-stdio-println-string
  (:wat::test::assert-stdout-is
    (:test::run-println-string)
    (:wat::core::Vector :wat::core::String "\"hello\"")))

;; ─── Layer 1 — :test::run-println-i64 ───────────────────────────────
;;
;; EDN encoding of the i64 value 42 is the decimal literal "42" (no
;; quotes). Proves value_to_edn_with covers non-string Ts through the
;; same fd pipeline.

(:wat::test::time-limit "15000ms")
(:deftest-ambient :wat-rs::test::test-ambient-stdio-println-i64
  (:wat::test::assert-stdout-is
    (:test::run-println-i64)
    (:wat::core::Vector :wat::core::String "42")))

;; ─── Layer 2 — :test::run-eprintln-string ───────────────────────────
;;
;; eprintln routes through StdErrService → fd 2. Captured stdout is
;; empty; stderr matches the EDN-quoted form. assert-stderr-matches
;; uses regex; the literal pattern "err" anchored neither end matches
;; the line "err" (with EDN quotes — regex doesn't care about the
;; quoted boundary, just the substring).

(:wat::test::time-limit "15000ms")
(:deftest-ambient :wat-rs::test::test-ambient-stdio-eprintln-string
  (:wat::test::assert-stderr-matches
    (:test::run-eprintln-string)
    "err"))

;; ─── Layer 3 — :test::run-println-twice ─────────────────────────────
;;
;; Two println calls in send order. The RunResult/stdout vector
;; carries both lines in the order they were written. assert-stdout-
;; is uses elementwise = on Vector<String> — order matters.

(:wat::test::time-limit "15000ms")
(:deftest-ambient :wat-rs::test::test-ambient-stdio-println-twice
  (:wat::test::assert-stdout-is
    (:test::run-println-twice)
    (:wat::core::Vector :wat::core::String "\"first\"" "\"second\"")))

;; ─── Layer 4 — :test::run-readln-echo ───────────────────────────────
;;
;; Layer 2 (run-hermetic-with-io) round trip: parent sends a native
;; :wat::core::String "echo me" through the typed Sender<String> over
;; Process/stdin (EDN-encoded on the wire); the child's (readln
;; -> :String) reads + parses + binds the native String; (println
;; echoed) writes it back through the trio to fd 1; the parent's
;; Receiver<String>/from-pipe decodes the EDN line into a native
;; String, landing in RunResultIO/outputs. assert-eq compares the
;; outputs vector to (Vector :String "echo me") — a one-element vec of
;; native Strings (no EDN quotes). The round trip proves the stdin
;; half of the trio is wired into the orchestrator the same way as
;; the stdout half, and that the typed-channel EDN encode/decode is
;; symmetric at the Layer 2 boundary.

(:wat::test::time-limit "15000ms")
(:deftest-ambient :wat-rs::test::test-ambient-stdio-readln-echo
  (:wat::test::assert-eq
    (:wat::test::RunResultIO/outputs (:test::run-readln-echo))
    (:wat::core::Vector :wat::core::String "echo me")))
