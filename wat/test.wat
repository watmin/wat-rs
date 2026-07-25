;; vigilatum: 2026-06-06T09:50:50Z — UPDATED-vigilia 6-spell test-kind guard
;; L1+L2=0 (cernere/intueri/exigere CONVERGED on the framework; probare
;; 4 hollow tests realified; vocare 1 CANNOT-FAIL test fixed; complectens
;; CONVERGED on the RUNNING corpus [8 L2 all in arc-170-ignored proof files,
;; composition wards affirmatively scoped to the arc-170 reanimation arc])
;; + circumspicere LAST [F5 2-of-3 unexercised public verbs witnessed
;; (run-in-scope honestly skipped — its ScopedLoader-for-load! distinguishing
;; behaviour requires a fixture-file demo outside this stone's scope) + F6
;; 2 corpus filename renames + F7 7 internal helpers marked]. THE WARDED
;; UNIT is the framework (this file); the corpus (wat-tests/**/*.wat) is its
;; demonstrated surface — running 238/0/53 corpus-green. The full clear that
;; preceded this ward inscribed the corpus into the green-gate's integration
;; tier (#151, ef585672) so it can never silently rot again. Canonical record:
;; docs/arc/2026/06/245-wat-corpus-warding/WARD-TEST-SURFACE.md.
;; Declared invariants, each enforced by a living gate:
;; (1) the test-kind 6-spell muster passes at HEAD (green-gate runs the
;;     corpus on every check 3/3);
;; (2) every public sandbox-entry verb (:wat::test::run / run-ast) has at
;;     least one live corpus witness — gated by the corpus run;
;;     run-in-scope is the named honest fixture-demo gap (245 INSCRIPTION
;;     bound), distinct from a phantom witness;
;; (3) the 15 retired-form bombs inside the arc-170-ignored proof files are
;;     DEFUSED — no value-position :wat::core::nil or :wat::core::struct-restricted
;;     survives anywhere in the corpus (contained un-ignore verification on
;;     the trickiest two files moved the panic from test_runner.rs:459
;;     startup to :487 run-phase, proving CHECK CLEAN);
;; (4) the arc-170 ignore-removal gate (circumspicere F4) is OWED a slow-head
;;     design pass — TRACKED as named follow-on stone #181-followon (the
;;     #151-doctrine sibling: gates are decisions, not reflexes); this stamp
;;     does NOT lie about F4 being closed.
;;
;; :wat::test::* — the wat-native test harness (arc 007 slice 3).
;;
;; Pure wat over three primitives:
;; - :wat::kernel::run-sandboxed        (arc 007 slice 2b)
;; - :wat::kernel::run-sandboxed-hermetic (arc 007 slice 2c)
;; - :wat::kernel::assertion-failed!    (this slice)
;; Plus the string/regex basics from :wat::core::string::* and
;; :wat::core::regex::*.
;;
;; Usage shape:
;;
;;   (:wat::core::defn :user::main [] -> :wat::core::nil
;;     (:wat::core::let
;;       [r (:wat::test::run "(:wat::core::defn :user::main [] -> :wat::core::nil ...)"
;;                           (:wat::core::Vector :wat::core::String))]
;;       ;; inspect (:wat::kernel::RunResult/failure r) — the sole field
;;       (:wat::test::assert-true (:wat::core::none? (:wat::kernel::RunResult/failure r)))))
;;
;; An assertion that fails panics internally; the outer run-sandboxed
;; catches the panic and surfaces the failure in its own RunResult.
;; Nested testing: a test file runs sandboxed to TEST a sandboxed
;; program.

;; ─── :wat::test::TestResult — alias of kernel::RunResult ─────────────
;;
;; Tests are sandboxed runs, so a test's return value IS structurally a
;; RunResult. The role-honest name for the test-discovery contract is
;; TestResult: the runner discovers any function returning this type
;; (or its underlying RunResult). deftest expands its function
;; signatures with :wat::test::TestResult — `kernel::RunResult`
;; describes the mechanism (sandbox), `test::TestResult` describes the
;; role (test outcome).
(:wat::core::typealias :wat::test::TestResult :wat::kernel::RunResult)

;; ─── assert-eq<T> ─────────────────────────────────────────────────────
;;
;; Structural equality via :wat::core::=. Failure renders both sides
;; via :wat::core::show<T> (arc 064) — the assertion's actual / expected
;; slots carry the rendered values so the test runner can display them
;; alongside the source location. Used to be `:None :None` (just "the
;; assertion fired"); arc 064 closed the diagnostic gap.
(:wat::core::defn :wat::test::assert-eq<T> [actual <- :T expected <- :T] -> :wat::core::nil
  (:wat::core::if (:wat::core::= actual expected) 
      nil
      (:wat::kernel::assertion-failed!
        "assert-eq failed"
        (:wat::core::Some (:wat::core::show actual))
        (:wat::core::Some (:wat::core::show expected)))))

;; ─── assert-true / assert-false ───────────────────────────────────────
;;
;; The basic boolean assertions — the first tools a test reaches for.
;; assert-true fires unless its argument is true; assert-false unless false.
;; Each carries its own honest message (not delegated to assert-eq, which would
;; mis-report "assert-eq failed"); the actual slot shows the bool, the expected
;; slot the word it should have been.
(:wat::core::defn :wat::test::assert-true [actual <- :wat::core::bool] -> :wat::core::nil
  (:wat::core::if actual 
      nil
      (:wat::kernel::assertion-failed!
        "assert-true failed"
        (:wat::core::Some (:wat::core::show actual))
        (:wat::core::Some "true"))))

(:wat::core::defn :wat::test::assert-false [actual <- :wat::core::bool] -> :wat::core::nil
  (:wat::core::if actual 
      (:wat::kernel::assertion-failed!
        "assert-false failed"
        (:wat::core::Some (:wat::core::show actual))
        (:wat::core::Some "false"))
      nil))

;; ─── assert-contains ──────────────────────────────────────────────────
;;
;; String substring check. Unlike assert-eq, both sides are :wat::core::String so
;; we can populate actual/expected with the real values — the failure
;; in a RunResult shows the user which haystack/needle fired.
(:wat::core::defn :wat::test::assert-contains [haystack <- :wat::core::String needle <- :wat::core::String] -> :wat::core::nil
  (:wat::core::if (:wat::core::string::contains? haystack needle) 
      nil
      (:wat::kernel::assertion-failed!
        "assert-contains failed"
        (:wat::core::Some haystack)
        (:wat::core::Some needle))))

;; ─── assert-coincident ────────────────────────────────────────────────
;;
;; "Are these two holons the same point in HD space?" — the geometry-
;; aware equality. Wraps `:wat::holon::coincident?` (arc 023): cosine
;; clears the substrate's coincident-floor (1 - cosine < threshold).
;;
;; This is what tests should reach for when checking holon identity.
;; `assert-eq` on cosine f64 against `1.0` is wrong: floating-point
;; arithmetic can return `1.0 + 2 ULPs` for cosine of identical
;; vectors, and exact f64 equality fails. The substrate-level
;; coincident-floor is calibrated for "geometrically equal at the
;; encoded d" — exactly the question test code is asking.
;;
;; Mirrors the assert-contains shape (custom message; both sides
;; carried in the failure payload). Tolerance lives in the substrate,
;; not the test.
;; Assertion failure carries the full coincidence explanation in the
;; `actual` slot of the failure payload (arc 069). When the assertion
;; fails, the consumer sees the cosine, floor, dim, sigma, and the
;; smallest sigma at which the pair would coincide — distinguishes
;; "calibration boundary" from "structurally distant" from "encoding
;; shape wrong" without a separate diagnostic round-trip.
(:wat::core::defn :wat::test::assert-coincident [a <- :wat::holon::HolonAST b <- :wat::holon::HolonAST] -> :wat::core::nil
  (:wat::core::let
      [expl
        (:wat::holon::coincident-explain a b)
       ok
        (:wat::holon::CoincidentExplanation/coincident expl)]
      (:wat::core::if ok 
        nil
        (:wat::kernel::assertion-failed!
          "assert-coincident failed — holons not at the same point"
          (:wat::core::Some (:wat::test::render-coincident-explanation expl))
          :wat::core::None))))

;; Helper — turn a CoincidentExplanation into a multi-line, named-
;; field string for assertion failure displays. Each field on its own
;; line, indented, so a developer reading test output sees the full
;; story without horizontal scrolling. Used by assert-coincident;
;; consumers wanting raw values call coincident-explain directly.
(:wat::core::defn :wat::test::render-coincident-explanation [expl <- :wat::holon::CoincidentExplanation] -> :wat::core::String
  (:wat::core::string::concat
      "\n  cosine            = "
      (:wat::core::f64::to-string
        (:wat::holon::CoincidentExplanation/cosine expl))
      "\n  floor             = "
      (:wat::core::f64::to-string
        (:wat::holon::CoincidentExplanation/floor expl))
      "\n  dim               = "
      (:wat::core::i64::to-string
        (:wat::holon::CoincidentExplanation/dim expl))
      "\n  sigma             = "
      (:wat::core::i64::to-string
        (:wat::holon::CoincidentExplanation/sigma expl))
      "\n  min-sigma-to-pass = "
      (:wat::core::i64::to-string
        (:wat::holon::CoincidentExplanation/min-sigma-to-pass expl))))

;; ─── assert-stdout-is / assert-stderr-matches — RETIRED (arc 278 wave 2d) ──
;;
;; These read the DROPPED :wat::kernel::RunResult/stdout and /stderr
;; capture fields. The capture model is gone — the peer wire delivers a
;; child's output via `recv'`, and RunResult now carries only `failure`.
;; The helpers (and their `any-line-matches` fold) are deleted; there is
;; no stdout/stderr to assert over.

;; ─── run / run-in-scope ───────────────────────────────────────────────
;;
;; Thin ergonomic wrappers over :wat::kernel::run-sandboxed. `run` is
;; the common case — no filesystem access at all (InMemoryLoader).
;; `run-in-scope` sets up ScopedLoader when the test uses load! with
;; fixture files.
(:wat::core::defn :wat::test::run [src <- :wat::core::String stdin <- :wat::core::Vector<wat::core::String>] -> :wat::kernel::RunResult (:wat::kernel::run-sandboxed src stdin :wat::core::None))

(:wat::core::defn :wat::test::run-in-scope [src <- :wat::core::String stdin <- :wat::core::Vector<wat::core::String> scope <- :wat::core::String] -> :wat::kernel::RunResult (:wat::kernel::run-sandboxed src stdin (:wat::core::Some scope)))

;; ─── run-ast + program — AST-entry test sandbox ──────────────────────
;;
;; The string-entry path (:wat::test::run above) is what fuzzers /
;; programs-built-at-runtime use. For hand-written tests, the AST-
;; entry path is the honest move — no escape hell, no nested quoting,
;; the inner program reads as s-expressions.
;;
;; Usage:
;;
;;   (:wat::test::run-ast
;;     (:wat::test::program
;;       (:wat::core::defn :user::main [] -> :wat::core::nil <body>))
;;     (:wat::core::Vector :wat::core::String))
;;
;; `:wat::test::program` expands to `:wat::core::forms` — the
;; variadic-quote substrate. Each top-level form captured as
;; `:wat::WatAST`; the result is `:wat::core::Vector<wat::WatAST>` ready to hand
;; to `:wat::kernel::run-sandboxed-ast`.

(:wat::core::defmacro :wat::test::program
  [& forms <- :wat::core::Vector<wat::WatAST>]
  -> :wat::WatAST
  `(:wat::core::forms ~@forms))

(:wat::core::defn :wat::test::run-ast [forms <- :wat::core::Vector<wat::WatAST> stdin <- :wat::core::Vector<wat::core::String>] -> :wat::kernel::RunResult (:wat::kernel::run-sandboxed-ast forms stdin :wat::core::None))

;; ─── deftest — Clojure-style ergonomic shell (arc 007 slice 3b; arc 027 slice 4; arc 031; arc 170 slice 3 phase E V5; arc 170 slice 4a-γ-flip) ───
;;
;; Registers a named zero-arg test function that returns TestResult (= RunResult).
;; The body runs in a cheap in-process THREAD via :wat::test::run-thread
;; (arc 170 slice 4a-γ-flip; the macro's mid-migration name is `run-thread`
;; and retires to `run` in 4c-β). For tests requiring process-level isolation
;; (captured stdio, mutated runtime config, ambient stdio verb calls — see
;; docs/COMPACTION-AMNESIA-RECOVERY.md § FM 7-ter), use `:wat::test::deftest-hermetic`
;; below. The `prelude` list splices top-level forms (loads, type declarations,
;; defmacros, struct/enum definitions) at the deftest's EXPANSION SITE under
;; (:wat::core::do ...), registering them in the outer symbol table and
;; TypeEnv at freeze time.
;; Gap J (arc 170 slice 3) ensures type declarations (struct/enum/newtype/
;; typealias) nested in the outer do are registered in the TypeEnv.
;; Gap F-1 ensures struct/enum accessor stubs are pre-registered.
;; Gap F-3 propagates the outer TypeEnv into the spawned child so the
;; child's hermetic subprocess sees the same types (deftest-hermetic only —
;; for the thread default, types are already shared with the parent runtime).
;;
;; Shape — empty prelude:
;;
;;   (:wat::test::deftest :my::test::two-plus-two
;;     ()
;;     (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
;;
;; Shape — type declarations in prelude:
;;
;;   (:wat::test::deftest :my::test::with-types
;;     ((:wat::core::struct :svc::State (counter :wat::core::i64))
;;      (:wat::core::typealias :svc::Alias :wat::core::i64))
;;     (:wat::test::assert-eq ...))
;;
;; Expansion:
;;
;;   (:wat::core::do
;;     <prelude spliced here — top-level forms registered at freeze time>
;;     (:wat::core::defn :my::test::two-plus-two [] -> :wat::test::TestResult
;;       (:wat::test::run-thread' <body>)))
;; arc 278 — non-prime :wat::test::deftest DELETED (IPC de-prime). The prime
;; :wat::test::deftest' below is reclaimed to this name (0z: prime -> plain).

;; ─── deftest-hermetic — same shape, forked child for isolation ────────
;;
;; Identical to `deftest` except the body runs in a forked child via
;; `:wat::test::run-hermetic-with-prelude` (→ spawn-process → OS fork).
;; Use for tests that exercise services spawning driver threads
;; (Console, Cache) — in-process run-thread uses spawn-thread (no per-
;; thread stdio capture), and cross-thread writes from a driver panic
;; silently. hermetic runs in a child with real thread-safe stdio
;; (PipeReader / PipeWriter; arc 012). The child inherits the caller's
;; SymbolTable (including loaded deps) + committed Config (arc 031) via COW.
;;
;; Arc 170 slice 3 Phase E — Path E migration: prelude declarations
;; land at the fn body's do-prefix; Gap H + I-A + I-B's closure-extraction
;; lift moves them to the spawned child's prologue where they register at
;; top-level. The substrate gap that blocked Gap G ("DefineInExpressionPosition
;; for define-in-fn-body-do") is closed — `is_declaration_form` covers the
;; declaration heads (def / defmacro / defstruct / defenum / newtype /
;; typealias / defalias) and `extract_closure`'s `split_body_prelude`
;; lifts them to the closure prologue before child eval sees them.
;; arc 278 — non-prime :wat::test::deftest-hermetic DELETED (IPC de-prime). The prime
;; :wat::test::deftest-hermetic' below is reclaimed to this name (0z: prime -> plain).

;; ─── Per-test attributes (arc 122) — :ignore + :should-panic ──────────
;;
;; Sibling-form annotations preceding a deftest. The wat::test! proc
;; macro (arc 121's discovery scanner, arc 122's attribute extension)
;; recognizes these forms and emits the matching Rust attribute on the
;; generated `#[test] fn`:
;;
;;   (:wat::test::ignore "reason")
;;   (:wat::test::deftest :my::test ...)
;;     -> #[test] #[ignore = "reason"] fn deftest_my_test() { ... }
;;
;;   (:wat::test::should-panic "expected substring")
;;   (:wat::test::deftest :my::test ...)
;;     -> #[test] #[should_panic(expected = "...")] fn deftest_my_test() { ... }
;;
;; The annotations are valid wat forms — registered here as no-op
;; `String -> unit` defines so the file type-checks. Their RUNTIME
;; presence is irrelevant; their meaning is purely proc-macro-side.
;; An annotation attaches to the IMMEDIATELY NEXT deftest; intervening
;; non-annotation forms clear the pending annotation.
(:wat::core::defn :wat::test::ignore [_reason <- :wat::core::String] -> :wat::core::nil nil)

(:wat::core::defn :wat::test::should-panic [_expected <- :wat::core::String] -> :wat::core::nil nil)

;; Arc 123 — :time-limit annotation. Sibling-form preceding a
;; deftest: when present, the proc macro wraps the generated
;; `#[test] fn`'s body in std::thread::spawn + recv_timeout. If
;; the budget is exceeded, the wrapper panics and cargo reports
;; the test as failed (timeout). The runaway worker thread leaks
;; until process exit — Rust threads cannot be killed safely;
;; honest in the panic message.
;;
;; Duration syntax: `<digits>{ms,s,m}`. Milliseconds is the
;; foundational resolution; finer granularity is not test-scale.
;; Lead with ms in examples; s and m supported but not
;; advertised. Examples:
;;
;;   (:wat::test::time-limit "100ms")     ;; preferred
;;   (:wat::test::time-limit "30s")        ;; supported
;;   (:wat::test::time-limit "5m")         ;; supported
;;   (:wat::test::deftest :my::test () body)
(:wat::core::defn :wat::test::time-limit [_dur <- :wat::core::String] -> :wat::core::nil nil)

;; ── run-thread' / deftest' — the test layer on the NEW substrate (the pipe model) ──
;;
;; Arc 259 S3.5a. A test is a ONE-SHOT computation with an OUTCOME — not a streaming
;; self-peer. With the thread-peer crash-reason IPC fix (S3.5a-0) in place, `recv'`
;; surfaces a crashed peer's reason over the pipe, so the harness is PURE user surface:
;; `spawn-program'` + `recv'`. The body runs in a self-peer and `send'`s a completion
;; signal (0) on success; a failing assertion crashes the peer; `recv'` delivers the
;; reason. NO outcome-capture side-channel, NO internal forms, NO test privilege — the
;; harness dogfoods exactly what users use.
;;
;; CONTRACT (pass-or-raise; test_runner.rs:297-330): a passing test RETURNS a clean
;; RunResult (failure=None) → the runner's Ok(Ok) arm reports pass; a failing test
;; RAISES, the assertion message carried in the raise → the Ok(Err) arm reports it.
;; Siblings of the legacy `run-thread`/`deftest` (which ride spawn-thread +
;; Thread/join-result); these ride spawn-program' + recv'. The legacy retires in
;; S3.5's back-half.

(:wat::core::defmacro :wat::test::run-thread'
  [body <- :wat::WatAST]
  -> :wat::WatAST
  ;; arc 278 the recv'-outcome wall reaches the harness: recv' RETURNS RecvOutcome (a VALUE), never
  ;; raises. The child's failing assertion crashes it → recv' returns `Lost[cause]`. We do NOT re-raise
  ;; (that would bend the value back into a control-flow raise); we RETURN the outcome — the Lost cause
  ;; (a Failure) drops straight into RunResult.failure, and test_runner's failure-slot check reports it.
  ;; A passing child sends its pass-marker → Message → failure=None. Value-based end to end: a failing
  ;; test is a VALUE, never a swallowed `_ (recv' p)` (the masking this arc annihilates).
  `(:wat::core::let
     [p (:wat::kernel::spawn-program' (:wat::spawn::thread)
          (:wat::core::fn [self <- :wat::kernel::ThreadSelfPeer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
            ;; arc 278 the send'-outcome wall — the PARENT faces the outcome via its own
            ;; `recv' p` right below (Message/Lost/Closed all become a RunResult); the
            ;; child's completion-signal send' just needs to proceed regardless.
            (:wat::core::do ~body
              (:wat::core::match (:wat::kernel::send' self 0)
                (:wat::kernel::SendOutcome::Sent   nil)
                (:wat::kernel::SendOutcome::Closed nil)   ;; parent's recv' already faces a gone self-peer
                ((:wat::kernel::SendOutcome::Lost _c) nil)))))]
     (:wat::core::match (:wat::kernel::recv' p)
       ((:wat::kernel::RecvOutcome::Message _m)
         (:wat::core::struct-new :wat::kernel::RunResult :wat::core::None))
       ((:wat::kernel::RecvOutcome::Lost cause)
         ;; arc 278 the LociDiedError stone — the Lost cause is a LociDiedError; RunResult.failure
         ;; is an Option<Failure>, so convert via `LociDiedError/to-failure` (preserves the
         ;; structured actual/expected/location/frames when the death carried an AssertionPayload).
         (:wat::core::struct-new :wat::kernel::RunResult (:wat::core::Some (:wat::kernel::LociDiedError/to-failure cause))))
       (:wat::kernel::RecvOutcome::Closed
         (:wat::core::struct-new :wat::kernel::RunResult
           (:wat::core::Some (:wat::kernel::message-only-failure "run-thread': test child closed before signaling completion")))))))

(:wat::core::defmacro :wat::test::deftest
  [name <- :wat::WatAST
   body <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::defn ~name [] -> :wat::test::TestResult (:wat::test::run-thread' ~body)))

;; ── run-hermetic' / deftest-hermetic' — the PROCESS-tier pipe-model siblings ──
;;
;; Arc 259 S3.5a. The forms siblings of run-thread'/deftest' (the thread pipe-model
;; pair). Same caller — spawn-program' + recv' — different body PACKAGING: a thread
;; shares memory and ships a CLOSURE; a process/remote has SEPARATE memory and ships
;; FORMS (program over the wire). "Separate memory" = same-host-process OR remote-host;
;; this forms interface is the SHARED one with the future deftest-remote — do NOT
;; special-case "process" in a way that would block a (remote) host.
;;
;; CONTRACT (pass-or-raise): the child runs body via :user::main, then
;; (:wat::kernel::println 0) — the pass-marker on fd 1. The parent recv's it.
;; A failing assertion crashes the child → the reason travels over the process Err
;; channel (fd 2) → recv' raises with it (the process tier surfaces crashes over
;; the pipe, which is precisely why the process tier was the WORKING model that
;; exposed the thread gap). Passing → returns a clean RunResult (failure=None).
;;
;; Pass-marker mechanics: println writes EDN "0\n" to fd 1; recv' (permissive
;; read_edn path) decodes it to i64(0); the result is discarded (_). No -> :T
;; ascription is needed for the discard — read_edn handles the raw i64 EDN correctly.
;;
;; STOP-2: do NOT modify deftest'/run-thread' above.
;; STOP-1: keep the FORMS interface (shared with deftest-remote); no process-only
;; special-casing.

(:wat::core::defmacro :wat::test::run-hermetic'
  [body <- :wat::WatAST]
  -> :wat::WatAST
  ;; arc 278 the recv'-outcome wall reaches the harness (see run-thread' above): recv' RETURNS the
  ;; outcome. A failing child crashes → Lost[cause] → RETURNED in RunResult.failure (not re-raised, not
  ;; swallowed as `_`). A passing child prints its pass-marker → Message → failure=None.
  `(:wat::core::let
     [p (:wat::kernel::spawn-program' (:wat::spawn::process)
          (:wat::core::forms
            (:wat::core::defn :user::main [] -> :wat::core::nil
              (:wat::core::do ~body (:wat::kernel::println 0)))))]
     (:wat::core::match (:wat::kernel::recv' p)
       ((:wat::kernel::RecvOutcome::Message _m)
         (:wat::core::struct-new :wat::kernel::RunResult :wat::core::None))
       ((:wat::kernel::RecvOutcome::Lost cause)
         ;; arc 278 the LociDiedError stone — the Lost cause is a LociDiedError; RunResult.failure
         ;; is an Option<Failure>, so convert via `LociDiedError/to-failure` (preserves the
         ;; structured actual/expected/location/frames when the death carried an AssertionPayload).
         (:wat::core::struct-new :wat::kernel::RunResult (:wat::core::Some (:wat::kernel::LociDiedError/to-failure cause))))
       (:wat::kernel::RecvOutcome::Closed
         (:wat::core::struct-new :wat::kernel::RunResult
           (:wat::core::Some (:wat::kernel::message-only-failure "run-hermetic': test child closed before signaling completion")))))))

(:wat::core::defmacro :wat::test::deftest-hermetic
  [name <- :wat::WatAST
   body <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::defn ~name [] -> :wat::test::TestResult (:wat::test::run-hermetic' ~body)))
