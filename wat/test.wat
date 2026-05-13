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
;;   (:wat::core::define (:user::main
;;                        (stdin  :wat::io::IOReader)
;;                        (stdout :wat::io::IOWriter)
;;                        (stderr :wat::io::IOWriter)
;;                        -> :())
;;     (:wat::core::let
;;       (((r :wat::kernel::RunResult)
;;         (:wat::test::run "(:user::main ...)" (:wat::core::Vector :wat::core::String))))
;;       (:wat::test::assert-stdout-is r
;;         (:wat::core::conj (:wat::core::Vector :wat::core::String) "expected-line"))))
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
(:wat::core::define
  (:wat::test::assert-eq<T>
    (actual :T)
    (expected :T)
    -> :wat::core::nil)
  (:wat::core::if (:wat::core::= actual expected) -> :wat::core::nil
    :wat::core::nil
    (:wat::kernel::assertion-failed!
      "assert-eq failed"
      (:wat::core::Some (:wat::core::show actual))
      (:wat::core::Some (:wat::core::show expected)))))

;; ─── assert-contains ──────────────────────────────────────────────────
;;
;; String substring check. Unlike assert-eq, both sides are :wat::core::String so
;; we can populate actual/expected with the real values — the failure
;; in a RunResult shows the user which haystack/needle fired.
(:wat::core::define
  (:wat::test::assert-contains
    (haystack :wat::core::String)
    (needle :wat::core::String)
    -> :wat::core::nil)
  (:wat::core::if (:wat::core::string::contains? haystack needle) -> :wat::core::nil
    :wat::core::nil
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
(:wat::core::define
  (:wat::test::assert-coincident
    (a :wat::holon::HolonAST)
    (b :wat::holon::HolonAST)
    -> :wat::core::nil)
  (:wat::core::let
    [expl
      (:wat::holon::coincident-explain a b)
     ok
      (:wat::holon::CoincidentExplanation/coincident expl)]
    (:wat::core::if ok -> :wat::core::nil
      :wat::core::nil
      (:wat::kernel::assertion-failed!
        "assert-coincident failed — holons not at the same point"
        (:wat::core::Some (:wat::test::render-coincident-explanation expl))
        :wat::core::None))))

;; Helper — turn a CoincidentExplanation into a multi-line, named-
;; field string for assertion failure displays. Each field on its own
;; line, indented, so a developer reading test output sees the full
;; story without horizontal scrolling. Used by assert-coincident;
;; consumers wanting raw values call coincident-explain directly.
(:wat::core::define
  (:wat::test::render-coincident-explanation
    (expl :wat::holon::CoincidentExplanation)
    -> :wat::core::String)
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

;; ─── assert-stdout-is ─────────────────────────────────────────────────
;;
;; Compare a RunResult's stdout to an expected wat::core::Vector<String>. Equality via
;; :wat::core::=, which is defined over T — for wat::core::Vector<String> it compares
;; elementwise. Joins both sides with "\n" into the Failure payload so
;; the user sees the diff in a RunResult.
(:wat::core::define
  (:wat::test::assert-stdout-is
    (result :wat::kernel::RunResult)
    (expected :wat::core::Vector<wat::core::String>)
    -> :wat::core::nil)
  (:wat::core::let
    [actual (:wat::kernel::RunResult/stdout result)]
    (:wat::core::if (:wat::core::= actual expected) -> :wat::core::nil
      :wat::core::nil
      (:wat::kernel::assertion-failed!
        "assert-stdout-is failed"
        (:wat::core::Some (:wat::core::string::join "\n" actual))
        (:wat::core::Some (:wat::core::string::join "\n" expected))))))

;; ─── assert-stderr-matches ────────────────────────────────────────────
;;
;; Regex match (unanchored) against each line of a RunResult's stderr.
;; Any line matching passes. Uses foldl over wat::core::Vector<String> to OR the
;; matches — a straightforward "any" without a new primitive.
(:wat::core::define
  (:wat::test::any-line-matches
    (pattern :wat::core::String)
    (lines :wat::core::Vector<wat::core::String>)
    -> :wat::core::bool)
  (:wat::core::foldl lines false
    (:wat::core::fn [acc <- :wat::core::bool line <- :wat::core::String] -> :wat::core::bool
      (:wat::core::or acc (:wat::core::regex::matches? pattern line)))))

(:wat::core::define
  (:wat::test::assert-stderr-matches
    (result :wat::kernel::RunResult)
    (pattern :wat::core::String)
    -> :wat::core::nil)
  (:wat::core::let
    [stderr-lines (:wat::kernel::RunResult/stderr result)]
    (:wat::core::if (:wat::test::any-line-matches pattern stderr-lines) -> :wat::core::nil
      :wat::core::nil
      (:wat::kernel::assertion-failed!
        "assert-stderr-matches failed — no stderr line matched pattern"
        (:wat::core::Some (:wat::core::string::join "\n" stderr-lines))
        (:wat::core::Some pattern)))))

;; ─── run / run-in-scope ───────────────────────────────────────────────
;;
;; Thin ergonomic wrappers over :wat::kernel::run-sandboxed. `run` is
;; the common case — no filesystem access at all (InMemoryLoader).
;; `run-in-scope` sets up ScopedLoader when the test uses load! with
;; fixture files.
(:wat::core::define
  (:wat::test::run
    (src :wat::core::String)
    (stdin :wat::core::Vector<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed src stdin :wat::core::None))

(:wat::core::define
  (:wat::test::run-in-scope
    (src :wat::core::String)
    (stdin :wat::core::Vector<wat::core::String>)
    (scope :wat::core::String)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed src stdin (:wat::core::Some scope)))

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
;;       (:wat::core::define (:user::main ...) <body>))
;;     (:wat::core::Vector :wat::core::String))
;;
;; `:wat::test::program` expands to `:wat::core::forms` — the
;; variadic-quote substrate. Each top-level form captured as
;; `:wat::WatAST`; the result is `:wat::core::Vector<wat::WatAST>` ready to hand
;; to `:wat::kernel::run-sandboxed-ast`.

(:wat::core::defmacro
  (:wat::test::program & (forms :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::Vector<wat::WatAST>>)
  `(:wat::core::forms ~@forms))

(:wat::core::define
  (:wat::test::run-ast
    (forms :wat::core::Vector<wat::WatAST>)
    (stdin :wat::core::Vector<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed-ast forms stdin :wat::core::None))

;; --- run-hermetic-ast — AST-entry hermetic sandbox ---
;;
;; Fork-isolated sibling of :wat::test::run-ast. Use for tests that
;; exercise services spawning driver threads (Console, Cache) —
;; in-process run-ast uses StringIo stdio (ThreadOwnedCell, single-
;; thread) and cross-thread writes from a driver panic silently.
;; hermetic-ast takes the same shape (forms + stdin) and runs the
;; inner program in a forked child with real thread-safe stdio.
;;
;; Arc 012 slice 3: the implementation lives in wat/kernel/hermetic.wat
;; (pure wat stdlib on top of fork-program-ast). The
;; child inherits AST in memory via COW — no subprocess reload, no
;; serialization, no binary-path coupling.
(:wat::core::define
  (:wat::test::run-hermetic-ast
    (forms :wat::core::Vector<wat::WatAST>)
    (stdin :wat::core::Vector<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed-hermetic-ast forms stdin :wat::core::None))

;; ─── deftest — Clojure-style ergonomic shell (arc 007 slice 3b; arc 027 slice 4; arc 031; arc 170 slice 3 phase E V5) ───
;;
;; Registers a named zero-arg test function that returns TestResult (= RunResult).
;; The body runs in a hermetic subprocess via :wat::test::run-hermetic
;; (arc 170 slice 3 phase C). Subprocess isolation is the default —
;; every deftest is hermetic. The `prelude` list splices top-level
;; forms (loads, type declarations, defmacros, struct/enum definitions)
;; at the deftest's EXPANSION SITE under (:wat::core::do ...), registering
;; them in the outer symbol table and TypeEnv at freeze time.
;; Gap J (arc 170 slice 3) ensures type declarations (struct/enum/newtype/
;; typealias) nested in the outer do are registered in the TypeEnv.
;; Gap F-1 ensures struct/enum accessor stubs are pre-registered.
;; Gap F-3 propagates the outer TypeEnv into the spawned child so the
;; child's hermetic subprocess sees the same types.
;;
;; Shape — empty prelude:
;;
;;   (:wat::test::deftest :my::test::two-plus-two
;;     ()
;;     (:wat::test::assert-eq (:wat::core::i64::+'2 2 2) 4))
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
;;     (:wat::core::define (:my::test::two-plus-two -> :wat::test::TestResult)
;;       (:wat::test::run-hermetic <body>)))
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do
     ~@prelude
     (:wat::core::define (~name -> :wat::test::TestResult)
       (:wat::test::run-hermetic ~body))))

;; ─── deftest-hermetic — same shape, forked child for isolation ────────
;;
;; Identical to `deftest` except the sandboxed program runs in a forked
;; child via `:wat::kernel::run-sandboxed-hermetic-ast` (→ wat/kernel/
;; hermetic.wat → :wat::kernel::fork-program-ast). Use for tests that
;; exercise services spawning driver threads (Console, Cache) —
;; in-process run-ast uses StringIo stdio (ThreadOwnedCell, single-
;; thread) and cross-thread writes from a driver panic silently.
;; hermetic runs in a child with real thread-safe stdio (PipeReader /
;; PipeWriter; arc 012). The child inherits the caller's SymbolTable
;; (including loaded deps) + committed Config (arc 031) via COW.
;;
;; Arc 170 slice 3 Phase E — Path E migration: prelude declarations
;; land at the fn body's do-prefix; Gap H + I-A + I-B's closure-extraction
;; lift moves them to the spawned child's prologue where they register at
;; top-level. The substrate gap that blocked Gap G ("DefineInExpressionPosition
;; for define-in-fn-body-do") is closed — `is_declaration_form` covers all 8
;; declaration heads (define / def / defmacro / define-dispatch / struct /
;; enum / newtype / typealias) and `extract_closure`'s `split_body_prelude`
;; lifts them to the closure prologue before child eval sees them.
(:wat::core::defmacro
  (:wat::test::deftest-hermetic
    (name :AST<wat::core::nil>)
    (prelude :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::define (~name -> :wat::test::TestResult)
     (:wat::test::run-hermetic
       (:wat::core::do
         ~@prelude
         ~body))))

;; ─── make-deftest — configured-deftest factory (arc 029; arc 031) ─────
;;
;; Register a new deftest variant whose default-prelude is baked in.
;; Each test using the variant drops to just name + body. Dims and
;; capacity-mode come from the test file's top-level preamble via
;; arc 031's sandbox-inherits-config path.
;;
;; Preamble at the top of a test source file:
;;
;;   (:wat::test::make-deftest :deftest
;;     ((:wat::load-file! "wat/vocab/shared/time.wat")))
;;
;; Every test below:
;;
;;   (:deftest :my-test-name
;;     (:wat::test::assert-eq 2 (+ 1 1)))
;;
;; Bare `:deftest` is user territory — only `:wat::*` and `:rust::*`
;; are reserved. An ambient `:deftest` at the root of a test source
;; file is idiomatic, and dedup makes the macro registration
;; file-local in practice (the test file's frozen world has it;
;; other files get their own configured shape).
;;
;; Expansion (outer → inner):
;;   outer generates (:wat::core::defmacro (,name ...) ...)
;;   inner expands to (:wat::test::deftest <name> ((load!)) <body>)
;;
;; Nested quasiquote mechanics (arc 029 slice 1): ,,default-prelude
;; substitutes AT OUTER expansion (the configured forms land as
;; literals in the generated defmacro's body). ,test-name and ,body
;; preserve across the outer pass — they're the inner macro's own
;; parameters and fire when the user calls the configured variant.
(:wat::core::defmacro
  (:wat::test::make-deftest
    (name :AST<wat::core::nil>)
    (default-prelude :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::defmacro
     (~name
       (test-name :AST<wat::core::nil>)
       (body :AST<wat::core::nil>)
       -> :AST<wat::core::nil>)
     `(:wat::test::deftest ~test-name ~~default-prelude ~body)))

;; ─── make-deftest-hermetic — fork-isolated configured variant ─────────
;;
;; Same shape as make-deftest; generated macro expands to
;; :wat::test::deftest-hermetic. Use when the configured tests
;; spawn driver threads and need subprocess isolation.
(:wat::core::defmacro
  (:wat::test::make-deftest-hermetic
    (name :AST<wat::core::nil>)
    (default-prelude :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::defmacro
     (~name
       (test-name :AST<wat::core::nil>)
       (body :AST<wat::core::nil>)
       -> :AST<wat::core::nil>)
     `(:wat::test::deftest-hermetic ~test-name ~~default-prelude ~body)))

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
(:wat::core::define
  (:wat::test::ignore (_reason :wat::core::String) -> :wat::core::nil)
  :wat::core::nil)

(:wat::core::define
  (:wat::test::should-panic (_expected :wat::core::String) -> :wat::core::nil)
  :wat::core::nil)

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
(:wat::core::define
  (:wat::test::time-limit (_dur :wat::core::String) -> :wat::core::nil)
  :wat::core::nil)

;; ─── Layer 1 testing-lib API (arc 170 slice 3 phase C) ─────────────────
;;
;; Three-layer model per TIERS.md:
;;   Layer 1 — run-hermetic (this file, 90% case)
;;   Layer 2 — run-hermetic-with-io (phase D)
;;   Layer 3 — (:wat::kernel::spawn-process fn) directly (substrate)
;;
;; Layer 1 hides all spawn ceremony from the test author. User writes
;; just the body:
;;
;;   (:wat::test::run-hermetic
;;     (:wat::test::assert-eq (:wat::core::i64::+'2 2 2) 4))
;;
;; The fn-wrapper ([_rx _tx] -> nil body) is generated by the macro.
;; The spawned child runs in a hermetic OS process (tier 2 per TIERS.md)
;; — memory isolation, signal isolation, global-state isolation, runtime
;; sealing are ambient by virtue of crossing the OS-process boundary.
;; Returns :wat::kernel::RunResult { stdout stderr failure }.
;;
;; Implementation: Path A (pure-wat, no new substrate verb).
;;   1. run-hermetic-driver : Process<nil,nil> -> RunResult
;;      Drains stdout+stderr via drain-lines (hermetic.wat helper),
;;      joins via Process/join-result, builds RunResult.
;;   2. run-hermetic defmacro : body -> AST
;;      Wraps body in (:wat::core::fn [_rx _tx] -> nil body),
;;      spawns via spawn-process, calls run-hermetic-driver.
;;
;; Honest deltas (arc 170 slice 3 phase C SCORE):
;;   - stdout in RunResult is empty for Layer 1 (child uses typed
;;     channels, not println; assertions propagate via ProcessDiedError
;;     over stderr EDN, not stdout lines). Consistent with slice 4's
;;     planned RunResult reshape to { outputs :Vec<O>, failure }.
;;   - IOWriter/close (stdin EOF) is NOT called before join: the child
;;     fn ignores _rx entirely; leaving the parent's tx alive does not
;;     block the child. No deadlock risk for Layer 1.
;;   - drain-lines, failure-from-process-died, extract-panics are
;;     defined in wat/kernel/hermetic.wat (loaded before test.wat in
;;     stdlib.rs); safe to call here without re-declaration.

;; ── run-hermetic-driver ─────────────────────────────────────────────────
;;
;; Internal driver for Layer 1. Takes an already-spawned Process whose
;; child fn wraps a test body. Joins (waits for child exit), drains
;; stdout+stderr, builds RunResult. Called exclusively from the
;; run-hermetic macro expansion — not part of the public test surface.
;;
;; Join-first pattern (same as hermetic.wat): small assertion bodies
;; fit in pipe buffers (Linux: 64KB+ per direction). Join blocks until
;; the child exits; then drain is safe and single-threaded. No
;; concurrent drain + join ceremony needed for the 90% case.
;;
;; The stderr-chain preference (extract-panics over join-result's
;; singleton) is arc 113 slice 3 symmetry: when the child panics with
;; an AssertionPayload, fork.rs writes the cascade chain as a tagged
;; EDN line on stderr. extract-panics recovers the full chain; the
;; test runner's failure_to_diagnostic extracts actual/expected from
;; the AssertionFailed Failure struct. Falls back to join-result's
;; singleton when no panic-marker is found (clean exit).
(:wat::core::define
  (:wat::test::run-hermetic-driver
    (proc :wat::kernel::Process<wat::core::nil,wat::core::nil>)
    -> :wat::kernel::RunResult)
  ;; Outer scope: proc handle lives here; Process/join-result runs AFTER
  ;; inner scope has dropped both output Receivers.  SERVICE-PROGRAMS.md
  ;; § "The lockstep" — inner-let owns every output Receiver; when the
  ;; inner body returns, stdout-r and stderr-r drop; substrate drain
  ;; threads see EOF; child can exit; outer join-result unblocks cleanly.
  (:wat::core::let
    [drain-pair
      (:wat::core::let
        ;; Inner scope: Receivers + drained lines only.
        ;; Dropping these bindings lets the child's OS pipes drain to EOF.
        [stdout-r       (:wat::kernel::Process/stdout proc)
         stderr-r       (:wat::kernel::Process/stderr proc)
         stdout-lines   (:wat::kernel::drain-lines stdout-r)
         stderr-lines   (:wat::kernel::drain-lines stderr-r)]
        (:wat::core::Tuple stdout-lines stderr-lines))
     stdout-lines   (:wat::core::first drain-pair)
     stderr-lines   (:wat::core::second drain-pair)
     ;; Inner scope has exited; Receivers dropped; child can exit.
     ;; join-result runs in the outer scope and returns immediately.
     joined-result  (:wat::kernel::Process/join-result proc)
     stderr-chain   (:wat::kernel::extract-panics stderr-lines)
     failure
      (:wat::core::match joined-result
        -> :wat::core::Option<wat::kernel::Failure>
        ((:wat::core::Ok _)  :wat::core::None)
        ((:wat::core::Err chain)
         (:wat::core::Some
           (:wat::kernel::failure-from-process-died
             (:wat::core::match stderr-chain
               -> :wat::core::Vector<wat::kernel::ProcessDiedError>
               ((:wat::core::Some sc) sc)
               (:wat::core::None      chain))))))]
    (:wat::core::struct-new :wat::kernel::RunResult
      stdout-lines stderr-lines failure)))

;; ── run-hermetic macro ──────────────────────────────────────────────────
;;
;; Layer 1 entry point. User writes only the body; macro generates:
;;   (:wat::test::run-hermetic-driver
;;     (:wat::kernel::spawn-process
;;       (:wat::core::fn
;;         [_rx <- :wat::kernel::Receiver<wat::core::nil>
;;          _tx <- :wat::kernel::Sender<wat::core::nil>]
;;         -> :wat::core::nil
;;         <body>)))
;;
;; The fn channels are Receiver<nil>/Sender<nil> — Layer 1 bodies
;; do not communicate via channels; they run assertions that panic on
;; failure. The _rx/_tx names (underscore prefix) signal intentional
;; non-use; the child ignores them.
;;
;; DO NOT MODIFY deftest or deftest-hermetic above — this is a new
;; entry point running in PARALLEL to the existing macros. Consumer
;; sweep (migrating deftest callers to run-hermetic) is phase E.
(:wat::core::defmacro
  (:wat::test::run-hermetic
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::test::run-hermetic-driver
     (:wat::kernel::spawn-process
       (:wat::core::fn
         [_rx <- :wat::kernel::Receiver<wat::core::nil>
          _tx <- :wat::kernel::Sender<wat::core::nil>]
         -> :wat::core::nil
         ~body))))

;; ─── Layer 2 — run-hermetic-with-io ────────────────────────────────────
;;
;; Arc 170 slice 3 phase D: the 9% case macro. Builds on Phase C's
;; Layer 1 foundation. Adds typed-channel I/O: the child body has
;; rx :Receiver<I> and tx :Sender<O> in scope; the parent sends all
;; inputs via the Process's tx channel and drains all outputs from
;; the Process's rx channel.
;;
;; Decision D1 — macro type-param shape:
;;   Option A.1 selected: the macro takes the FULL channel type keywords
;;   for the fn signature (4 args: rx-type, tx-type, inputs, body). The
;;   user passes:
;;     rx-type — the full Receiver type keyword e.g. :Receiver<i64>
;;     tx-type — the full Sender type keyword e.g. :Sender<i64>
;;     inputs  — the input Vector<I> expression
;;     body    — the child body AST (with rx and tx in scope)
;;
;;   The aspirational form `(run-hermetic-with-io :i64 :i64 inputs body)` with
;;   inner type args ONLY would require constructing `Receiver<i64>` and
;;   `Sender<i64>` keywords from the inner type at macro-expand time.
;;   No `keyword::from-string` or string->keyword verb exists in the
;;   substrate (honest delta D1). Full channel-type keywords are required.
;;   No separate O type arg is needed — the driver infers O from the
;;   process's typed channels (O flows from the Sender<O> in tx-type).
;;
;; Decision D2 — RunResultIO registration:
;;   Rust-side StructDef in src/types.rs (same as RunResult). Chosen
;;   because it auto-generates accessors via register_struct_methods and
;;   gives the struct first-class status without a wat-side `:struct` form.
;;
;; Decision D3 — send/drain ordering:
;;   Sequential: send all inputs → drain all outputs → join → drain stderr.
;;   Safe for bounded I/O (T18: single send, single recv). The child exits
;;   after processing, dropping its tx, which signals EOF to the parent's
;;   drain. Join then returns immediately.
;;
;;   Honest delta: if the child reads rx to EOF (loop until Ok(None))
;;   and the parent's tx is never closed, the child would deadlock.
;;   Closing the parent's tx requires dropping the Sender Value, which
;;   is not expressible in wat's function-scope bindings. This pattern
;;   (child reads to EOF) is NOT the T18 case — T18's child reads exactly
;;   one value and exits. Threaded drain (concurrent send + recv) is the
;;   correct fix for the EOF-reading pattern; deferred to a future arc.

;; ── run-hermetic-send-inputs ─────────────────────────────────────────────
;;
;; Tail-recursive helper that sends every element of `inputs` to `tx`.
;; Panics on send failure (disconnected — child exited before receiving
;; all inputs; this is a usage error in the caller's test scenario).
;; Called exclusively from run-hermetic-with-io-driver.
;;
;; Note: (:wat::core::first vec) returns Option<I> (arc 047 honest
;; absence design). We use Option/expect to unwrap since we only call
;; first after confirming Vector/empty? is false.
(:wat::core::define
  (:wat::test::run-hermetic-send-inputs<I>
    (tx :wat::kernel::Sender<I>)
    (inputs :wat::core::Vector<I>)
    -> :wat::core::nil)
  (:wat::core::if (:wat::core::Vector/empty? inputs)
    -> :wat::core::nil
    :wat::core::nil
    (:wat::core::let
      [item
        (:wat::core::Option/expect -> :I
          (:wat::core::first inputs)
          "run-hermetic-send-inputs: first of non-empty vector was None (substrate bug)")
       rest (:wat::core::rest inputs)
       _
        (:wat::core::Result/expect -> :wat::core::nil
          (:wat::kernel::send tx item)
          "run-hermetic-send-inputs: send failed — child disconnected")]
      (:wat::test::run-hermetic-send-inputs tx rest))))

;; ── run-hermetic-drain-outputs ───────────────────────────────────────────
;;
;; Tail-recursive drain of a Receiver<O> into a Vector<O>. Mirrors
;; :wat::stream::collect-drain<T> from stream.wat. Reads until the
;; channel is disconnected (child exited; tx dropped) or signals Ok(None).
;; Accumulates outputs into `acc` and returns when the stream is exhausted.
;; Called exclusively from run-hermetic-with-io-driver.
(:wat::core::define
  (:wat::test::run-hermetic-drain-outputs<O>
    (rx :wat::kernel::Receiver<O>)
    (acc :wat::core::Vector<O>)
    -> :wat::core::Vector<O>)
  (:wat::core::match (:wat::kernel::recv rx)
    -> :wat::core::Vector<O>
    ((:wat::core::Ok (:wat::core::Some v))
     (:wat::test::run-hermetic-drain-outputs rx (:wat::core::conj acc v)))
    ((:wat::core::Ok :wat::core::None) acc)
    ((:wat::core::Err _died) acc)))

;; ── run-hermetic-with-io-driver ──────────────────────────────────────────
;;
;; Internal driver for Layer 2. Receives an already-spawned Process<I,O>
;; and the inputs Vector<I>. Orchestrates the I/O round-trip:
;;   1. Send each input to the child via Process/tx.
;;   2. Drain outputs from Process/rx until disconnect (child exited).
;;   3. Join via Process/join-result (child has already exited by step 2).
;;   4. Drain stderr; rebuild panic chain via extract-panics.
;;   5. Assemble :wat::test::RunResultIO with outputs + stderr + failure.
;;
;; D3 ordering — sequential (send → drain → join → drain-stderr):
;;   Works for T18's bounded I/O (one send, one recv). The child exits
;;   after processing all inputs, dropping its tx. The drain-outputs sees
;;   Ok(None) (EOF) and returns. Join then finds the child already exited.
;;
;; Note: Process/stdout is the byte-pipe view of the output channel
;; (legacy arc 170 slice 1c additive field). Layer 2 does NOT drain
;; stdout-as-string-lines — it drains outputs via typed-channel recv.
;; The RunResultIO carries outputs :Vector<O> instead of stdout :Vector<String>.
(:wat::core::define
  (:wat::test::run-hermetic-with-io-driver<I,O>
    (proc :wat::kernel::Process<I,O>)
    (inputs :wat::core::Vector<I>)
    -> :wat::test::RunResultIO<O>)
  ;; Outer scope: proc handle + join-result.  SERVICE-PROGRAMS.md § "The
  ;; lockstep": inner-let owns every output Receiver; when inner body
  ;; returns, stderr-r drops; drain threads see EOF; child can exit;
  ;; outer join-result unblocks cleanly.
  (:wat::core::let
    [tx             (:wat::kernel::Process/tx proc)
     _              (:wat::test::run-hermetic-send-inputs tx inputs)
     rx             (:wat::kernel::Process/rx proc)
     outputs        (:wat::test::run-hermetic-drain-outputs rx (:wat::core::Vector :O))
     ;; Inner scope: stderr Receiver + drained lines.
     ;; Dropping stderr-r lets the child's stderr pipe drain to EOF.
     stderr-lines
      (:wat::core::let
        [stderr-r     (:wat::kernel::Process/stderr proc)
         lines        (:wat::kernel::drain-lines stderr-r)]
        lines)
     ;; Inner scope has exited; Receivers dropped; child can exit.
     joined-result  (:wat::kernel::Process/join-result proc)
     stderr-chain   (:wat::kernel::extract-panics stderr-lines)
     failure
      (:wat::core::match joined-result
        -> :wat::core::Option<wat::kernel::Failure>
        ((:wat::core::Ok _) :wat::core::None)
        ((:wat::core::Err chain)
         (:wat::core::Some
           (:wat::kernel::failure-from-process-died
             (:wat::core::match stderr-chain
               -> :wat::core::Vector<wat::kernel::ProcessDiedError>
               ((:wat::core::Some sc) sc)
               (:wat::core::None      chain))))))]
    (:wat::core::struct-new :wat::test::RunResultIO
      outputs stderr-lines failure)))

;; ── run-hermetic-with-io macro ───────────────────────────────────────────
;;
;; Layer 2 entry point (the 9% case). User writes the body with rx and tx
;; in scope. The macro generates the fn-form wrapper, spawns a hermetic
;; OS process, sends inputs, drains outputs, and returns RunResultIO<O>.
;;
;; Arc 170 slice 3 Gap A — updated to use keyword/of for channel type
;; construction. The caller now passes INNER element types (:i64, not
;; :Receiver<i64>); keyword/of constructs the full channel types at
;; macro-expand time.
;;
;; Canonical call form (Gap A: inner element types, 4 args):
;;
;;   (:wat::test::run-hermetic-with-io
;;     :wat::core::i64                          ;; input element type
;;     :wat::core::i64                          ;; output element type
;;     (:wat::core::Vector :wat::core::i64 21)  ;; inputs Vector<I>
;;     (:wat::core::let
;;       [n (:wat::core::Option/expect -> :wat::core::i64
;;             (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
;;               (:wat::kernel::recv rx) "recv failed")
;;             "stream closed")
;;        _ (:wat::core::Result/expect -> :wat::core::nil
;;             (:wat::kernel::send tx (:wat::core::i64::*'2 n 2)) "send failed")]
;;       :wat::core::nil))
;;
;; Expands to:
;;   (:wat::test::run-hermetic-with-io-driver
;;     (:wat::kernel::spawn-process
;;       (:wat::core::fn
;;         [rx <- :wat::kernel::Receiver<wat::core::i64>
;;          tx <- :wat::kernel::Sender<wat::core::i64>]
;;         -> :wat::core::nil
;;         <body>))
;;     <inputs>)
;;
;; keyword/of constructs the full channel-type keyword at expand time:
;;   (:wat::core::keyword/of :wat::kernel::Receiver ~input-type)
;;   → :wat::kernel::Receiver<wat::core::i64>  (after ~input-type substitution)
;;
;; The driver's return type :RunResultIO<O> is inferred by the type checker
;; from the Process<I,O> argument (O flows from the Sender<O> in tx channel).
;; No separate o-type arg is needed — the driver infers O from typed channels.
;;
;; DO NOT MODIFY run-hermetic (Layer 1) above — this is an ADDITION.
;; DO NOT touch deftest / deftest-hermetic macro definitions (phase E).
(:wat::core::defmacro
  (:wat::test::run-hermetic-with-io
    (input-type  :AST<wat::core::nil>)
    (output-type :AST<wat::core::nil>)
    (inputs      :AST<wat::core::nil>)
    (body        :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::test::run-hermetic-with-io-driver
     (:wat::kernel::spawn-process
       (:wat::core::fn
         [rx <- (:wat::core::keyword/of :wat::kernel::Receiver ~input-type)
          tx <- (:wat::core::keyword/of :wat::kernel::Sender ~output-type)]
         -> :wat::core::nil
         ~body))
     ~inputs))
