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
;;     (:wat::core::let*
;;       (((r :wat::kernel::RunResult)
;;         (:wat::test::run "(:user::main ...)" (:wat::core::vec :wat::core::String))))
;;       (:wat::test::assert-stdout-is r
;;         (:wat::core::conj (:wat::core::vec :wat::core::String) "expected-line"))))
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
    -> :wat::core::unit)
  (:wat::core::if (:wat::core::= actual expected) -> :wat::core::unit
    ()
    (:wat::kernel::assertion-failed!
      "assert-eq failed"
      (Some (:wat::core::show actual))
      (Some (:wat::core::show expected)))))

;; ─── assert-contains ──────────────────────────────────────────────────
;;
;; String substring check. Unlike assert-eq, both sides are :wat::core::String so
;; we can populate actual/expected with the real values — the failure
;; in a RunResult shows the user which haystack/needle fired.
(:wat::core::define
  (:wat::test::assert-contains
    (haystack :wat::core::String)
    (needle :wat::core::String)
    -> :wat::core::unit)
  (:wat::core::if (:wat::core::string::contains? haystack needle) -> :wat::core::unit
    ()
    (:wat::kernel::assertion-failed!
      "assert-contains failed"
      (Some haystack)
      (Some needle))))

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
    -> :wat::core::unit)
  (:wat::core::let*
    (((expl :wat::holon::CoincidentExplanation)
      (:wat::holon::coincident-explain a b))
     ((ok :wat::core::bool)
      (:wat::holon::CoincidentExplanation/coincident expl)))
    (:wat::core::if ok -> :wat::core::unit
      ()
      (:wat::kernel::assertion-failed!
        "assert-coincident failed — holons not at the same point"
        (Some (:wat::test::render-coincident-explanation expl))
        :None))))

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
;; Compare a RunResult's stdout to an expected Vec<String>. Equality via
;; :wat::core::=, which is defined over T — for Vec<String> it compares
;; elementwise. Joins both sides with "\n" into the Failure payload so
;; the user sees the diff in a RunResult.
(:wat::core::define
  (:wat::test::assert-stdout-is
    (result :wat::kernel::RunResult)
    (expected :Vec<wat::core::String>)
    -> :wat::core::unit)
  (:wat::core::let*
    (((actual :Vec<wat::core::String>) (:wat::kernel::RunResult/stdout result)))
    (:wat::core::if (:wat::core::= actual expected) -> :wat::core::unit
      ()
      (:wat::kernel::assertion-failed!
        "assert-stdout-is failed"
        (Some (:wat::core::string::join "\n" actual))
        (Some (:wat::core::string::join "\n" expected))))))

;; ─── assert-stderr-matches ────────────────────────────────────────────
;;
;; Regex match (unanchored) against each line of a RunResult's stderr.
;; Any line matching passes. Uses foldl over Vec<String> to OR the
;; matches — a straightforward "any" without a new primitive.
(:wat::core::define
  (:wat::test::any-line-matches
    (pattern :wat::core::String)
    (lines :Vec<wat::core::String>)
    -> :wat::core::bool)
  (:wat::core::foldl lines false
    (:wat::core::lambda ((acc :wat::core::bool) (line :wat::core::String) -> :wat::core::bool)
      (:wat::core::or acc (:wat::core::regex::matches? pattern line)))))

(:wat::core::define
  (:wat::test::assert-stderr-matches
    (result :wat::kernel::RunResult)
    (pattern :wat::core::String)
    -> :wat::core::unit)
  (:wat::core::let*
    (((stderr-lines :Vec<wat::core::String>) (:wat::kernel::RunResult/stderr result)))
    (:wat::core::if (:wat::test::any-line-matches pattern stderr-lines) -> :wat::core::unit
      ()
      (:wat::kernel::assertion-failed!
        "assert-stderr-matches failed — no stderr line matched pattern"
        (Some (:wat::core::string::join "\n" stderr-lines))
        (Some pattern)))))

;; ─── run / run-in-scope ───────────────────────────────────────────────
;;
;; Thin ergonomic wrappers over :wat::kernel::run-sandboxed. `run` is
;; the common case — no filesystem access at all (InMemoryLoader).
;; `run-in-scope` sets up ScopedLoader when the test uses load! with
;; fixture files.
(:wat::core::define
  (:wat::test::run
    (src :wat::core::String)
    (stdin :Vec<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed src stdin :None))

(:wat::core::define
  (:wat::test::run-in-scope
    (src :wat::core::String)
    (stdin :Vec<wat::core::String>)
    (scope :wat::core::String)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed src stdin (Some scope)))

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
;;     (:wat::core::vec :wat::core::String))
;;
;; `:wat::test::program` expands to `:wat::core::forms` — the
;; variadic-quote substrate. Each top-level form captured as
;; `:wat::WatAST`; the result is `:Vec<wat::WatAST>` ready to hand
;; to `:wat::kernel::run-sandboxed-ast`.

(:wat::core::defmacro
  (:wat::test::program & (forms :AST<Vec<wat::WatAST>>)
    -> :AST<Vec<wat::WatAST>>)
  `(:wat::core::forms ,@forms))

(:wat::core::define
  (:wat::test::run-ast
    (forms :Vec<wat::WatAST>)
    (stdin :Vec<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed-ast forms stdin :None))

;; --- run-hermetic-ast — AST-entry hermetic sandbox ---
;;
;; Fork-isolated sibling of :wat::test::run-ast. Use for tests that
;; exercise services spawning driver threads (Console, Cache) —
;; in-process run-ast uses StringIo stdio (ThreadOwnedCell, single-
;; thread) and cross-thread writes from a driver panic silently.
;; hermetic-ast takes the same shape (forms + stdin) and runs the
;; inner program in a forked child with real thread-safe stdio.
;;
;; Arc 012 slice 3: the implementation lives in wat/std/hermetic.wat
;; (pure wat stdlib on top of fork-program-ast + wait-child). The
;; child inherits AST in memory via COW — no subprocess reload, no
;; serialization, no binary-path coupling.
(:wat::core::define
  (:wat::test::run-hermetic-ast
    (forms :Vec<wat::WatAST>)
    (stdin :Vec<wat::core::String>)
    -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed-hermetic-ast forms stdin :None))

;; ─── deftest — Clojure-style ergonomic shell (arc 007 slice 3b; arc 027 slice 4; arc 031) ───
;;
;; Registers a named zero-arg test function that returns RunResult.
;; The body runs inside a sandboxed world that INHERITS the outer
;; test file's committed dims + capacity-mode (arc 031). The
;; `prelude` list splices startup forms (loads, type declarations,
;; defmacros) BEFORE the auto-generated `:user::main`. Empty `()`
;; prelude = no startup forms, the minimal shape.
;;
;; The test file's top-level preamble is the single declaration
;; site for config — needed only when overriding defaults (e.g.
;; switching capacity-mode from :error to :panic, or installing
;; a custom set-dim-router! / sigma-fn). Every deftest below
;; inherits whatever the preamble committed (and the substrate
;; defaults for whatever the preamble omits) through the
;; sandbox's Config-inheritance path. No per-test re-declaration.
;;
;; Shape — empty prelude:
;;
;;   (:wat::test::deftest :my::test::two-plus-two
;;     ()
;;     (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
;;
;; Shape — loads in prelude (arc 027 slice 4):
;;
;;   (:wat::test::deftest :my::test::with-loads
;;     ((:wat::load-file! "wat/types/candle.wat")
;;      (:wat::load-file! "wat/vocab/shared/time.wat"))
;;     (:wat::test::assert-eq ...))
;;
;; Expansion:
;;
;;   (:wat::core::define (:my::test::two-plus-two -> :wat::kernel::RunResult)
;;     (:wat::kernel::run-sandboxed-ast
;;       (:wat::core::forms
;;         <prelude spliced here>
;;         (:wat::core::define (:user::main
;;                              (stdin  :wat::io::IOReader)
;;                              (stdout :wat::io::IOWriter)
;;                              (stderr :wat::io::IOWriter)
;;                              -> :())
;;           <body>))
;;       (:wat::core::vec :wat::core::String)
;;       :None))
(:wat::core::defmacro
  (:wat::test::deftest
    (name :AST<wat::core::unit>)
    (prelude :AST<wat::core::unit>)
    (body :AST<wat::core::unit>)
    -> :AST<wat::core::unit>)
  `(:wat::core::define (,name -> :wat::test::TestResult)
     (:wat::kernel::run-sandboxed-ast
       (:wat::core::forms
         ,@prelude
         (:wat::core::define
           (:user::main
             (stdin  :wat::io::IOReader)
             (stdout :wat::io::IOWriter)
             (stderr :wat::io::IOWriter)
             -> :wat::core::unit)
           ,body))
       (:wat::core::vec :wat::core::String)
       :None)))

;; ─── deftest-hermetic — same shape, forked child for isolation ────────
;;
;; Identical to `deftest` except the sandboxed program runs in a forked
;; child via `:wat::kernel::run-sandboxed-hermetic-ast` (→ wat/std/
;; hermetic.wat → :wat::kernel::fork-program-ast). Use for tests that
;; exercise services spawning driver threads (Console, Cache) —
;; in-process run-ast uses StringIo stdio (ThreadOwnedCell, single-
;; thread) and cross-thread writes from a driver panic silently.
;; hermetic runs in a child with real thread-safe stdio (PipeReader /
;; PipeWriter; arc 012). The child inherits the caller's SymbolTable
;; (including loaded deps) + committed Config (arc 031) via COW.
(:wat::core::defmacro
  (:wat::test::deftest-hermetic
    (name :AST<wat::core::unit>)
    (prelude :AST<wat::core::unit>)
    (body :AST<wat::core::unit>)
    -> :AST<wat::core::unit>)
  `(:wat::core::define (,name -> :wat::test::TestResult)
     (:wat::kernel::run-sandboxed-hermetic-ast
       (:wat::core::forms
         ,@prelude
         (:wat::core::define
           (:user::main
             (stdin  :wat::io::IOReader)
             (stdout :wat::io::IOWriter)
             (stderr :wat::io::IOWriter)
             -> :wat::core::unit)
           ,body))
       (:wat::core::vec :wat::core::String)
       :None)))

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
    (name :AST<wat::core::unit>)
    (default-prelude :AST<wat::core::unit>)
    -> :AST<wat::core::unit>)
  `(:wat::core::defmacro
     (,name
       (test-name :AST<wat::core::unit>)
       (body :AST<wat::core::unit>)
       -> :AST<wat::core::unit>)
     `(:wat::test::deftest ,test-name ,,default-prelude ,body)))

;; ─── make-deftest-hermetic — fork-isolated configured variant ─────────
;;
;; Same shape as make-deftest; generated macro expands to
;; :wat::test::deftest-hermetic. Use when the configured tests
;; spawn driver threads and need subprocess isolation.
(:wat::core::defmacro
  (:wat::test::make-deftest-hermetic
    (name :AST<wat::core::unit>)
    (default-prelude :AST<wat::core::unit>)
    -> :AST<wat::core::unit>)
  `(:wat::core::defmacro
     (,name
       (test-name :AST<wat::core::unit>)
       (body :AST<wat::core::unit>)
       -> :AST<wat::core::unit>)
     `(:wat::test::deftest-hermetic ,test-name ,,default-prelude ,body)))
