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
;; 1f-γ) continues to work after the arc 170 migration. Each helper
;; uses :wat::test::run-hermetic-ast which forks a child via
;; :wat::kernel::fork-program-ast; the child boots
;; invoke_user_main_orchestrated, which spawns the trio services,
;; registers thread-0, and runs the inner :user::main. The inner
;; main's (:wat::kernel::println v) call routes through the trio to
;; the child's fd 1; the parent drains via OS pipe → RunResult.
;; deftest-hermetic (NOT deftest) on every test — the in-process
;; path skips the fd pipeline; only the forked-child path exercises
;; the orchestrator boot + service spawn + dup-fds + drain machinery.
;;
;; Top-down dependency graph (top → bottom; no forward refs):
;;   Layer 0 :test::run-println-string   → run-hermetic-ast { (println "hello") }
;;   Layer 1 :test::run-println-i64      → run-hermetic-ast { (println 42)      }
;;   Layer 2 :test::run-eprintln-string  → run-hermetic-ast { (eprintln "err")  }
;;   Layer 3 :test::run-println-twice    → run-hermetic-ast { 2× println        }
;;   Layer 4 :test::run-readln-echo      → run-hermetic-ast { readln → println }

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
     (:wat::test::run-hermetic-ast
       (:wat::test::program
         (:wat::core::define
           (:user::main -> :wat::core::nil)
           (:wat::kernel::println "hello")))
       (:wat::core::Vector :wat::core::String)))

   ;; ─── Layer 1 helper — run inner program that prints an i64 ───────
   ;;
   ;; Same shape as Layer 0 with a non-string value. EDN encoding of an
   ;; i64 is its decimal literal (no quotes). Proves the trio doesn't
   ;; only handle pre-formatted strings — println renders any T via
   ;; value_to_edn_with.
   (:wat::core::define
     (:test::run-println-i64 -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic-ast
       (:wat::test::program
         (:wat::core::define
           (:user::main -> :wat::core::nil)
           (:wat::kernel::println 42)))
       (:wat::core::Vector :wat::core::String)))

   ;; ─── Layer 2 helper — run inner program that eprints "err" ───────
   ;;
   ;; eprintln routes to fd 2 instead of fd 1. The RunResult separates
   ;; stdout (empty) from stderr ("err"). Proves the two fd pipelines
   ;; don't cross-talk: a single eprintln lands ONLY in stderr.
   (:wat::core::define
     (:test::run-eprintln-string -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic-ast
       (:wat::test::program
         (:wat::core::define
           (:user::main -> :wat::core::nil)
           (:wat::kernel::eprintln "err")))
       (:wat::core::Vector :wat::core::String)))

   ;; ─── Layer 3 helper — run inner program with two println calls ───
   ;;
   ;; Two sequential calls. The trio ack-rx blocks after each Write so
   ;; the lines land in send order. Proves order preservation across
   ;; multiple round trips through the same fd pipeline.
   (:wat::core::define
     (:test::run-println-twice -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic-ast
       (:wat::test::program
         (:wat::core::define
           (:user::main -> :wat::core::nil)
           (:wat::core::do
             (:wat::kernel::println "first")
             (:wat::kernel::println "second")
             :wat::core::nil)))
       (:wat::core::Vector :wat::core::String)))

   ;; ─── Layer 4 helper — readln round trip via stdin pre-seed ───────
   ;;
   ;; Test seeds stdin with one EDN line; inner main reads it via
   ;; (readln -> :String) which parses the EDN line "echo me" (with
   ;; quotes on the wire) into a native :wat::core::String "echo me"
   ;; (without quotes), then prints the form back. Println re-EDN-
   ;; encodes the String to "\"echo me\"" (with quotes). Proves both
   ;; directions of the trio: input parsed in, output rendered out.
   ;;
   ;; Arc 170 slice 1f-iota — readln is polymorphic via the call-site
   ;; `-> :T` annotation. Pre-1f-iota readln returned :HolonAST and
   ;; the stdout assertion expected a tagged HolonAST String render
   ;; (`#wat-edn.holon/String "echo me"`); post-1f-iota readln returns
   ;; the native :String and the stdout assertion sees the canonical
   ;; EDN-quoted form (`"echo me"`).
   ;;
   ;; The stdin vec uses TWO elements so the substrate's
   ;; (:wat::core::string::join "\n" stdin) produces a trailing newline
   ;; — IOReader/read-line in the stdin service blocks until \n
   ;; arrives, and the parent process doesn't close stdin until
   ;; :user::main exits (hermetic.wat docstring § Limitations).
   (:wat::core::define
     (:test::run-readln-echo -> :wat::kernel::RunResult)
     (:wat::test::run-hermetic-ast
       (:wat::test::program
         (:wat::core::define
           (:user::main -> :wat::core::nil)
           (:wat::core::let
             [echoed (:wat::kernel::readln -> :wat::core::String)]
             (:wat::kernel::println echoed))))
       (:wat::core::Vector :wat::core::String "\"echo me\"" "")))
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
;; Stdin pre-seed delivers "echo me" (EDN-quoted) to the trio reader;
;; (readln -> :String) parses the EDN line into a native String "echo
;; me" (without quotes); println re-EDN-encodes it on stdout. Per the
;; arc 170 slice 1f-iota EDN-only contract, the round trip preserves
;; the canonical EDN form: "\"echo me\"" goes in, "\"echo me\"" comes
;; out (the inner String is the same, the outer EDN-quoting is
;; symmetric). The round trip proves the stdin half of the trio is
;; wired into the orchestrator the same way as the stdout half.

(:wat::test::time-limit "15000ms")
(:deftest-ambient :wat-rs::test::test-ambient-stdio-readln-echo
  (:wat::test::assert-stdout-is
    (:test::run-readln-echo)
    (:wat::core::Vector :wat::core::String "\"echo me\"")))
