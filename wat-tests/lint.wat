;; wat-tests/lint.wat — proof tests for wat/lint.wat.
;;
;; Arc 277 Stone 277.1. The linter carries its own proof (complectens):
;; literal SourceFile fixtures, no I/O — pure functions are the targets.
;;
;; Four cases:
;;   1. Detects the ladder — lint-source finds >=1 finding with rule "nested-if-=-ladder".
;;   2. No false positive — clean files produce 0 findings from the ladder rule.
;;   3. The fix applies (end-to-end lint->fix) — DEFERRED to 277.1b (STOP-1: ast-span
;;      returns only the START location; computing old-len for a structural node requires
;;      an ast-end-span substrate primitive not yet present).
;;   4. lint-stdlib runs + rule-zero present — lint-stdlib returns a Vector; a fabricated
;;      out-of-order input produces a rule-zero "load-order" Finding.

;; ─── Case 1: detects the ladder ──────────────────────────────────────

(:wat::test::deftest :wat-tests::lint::detects-nested-if-eq-ladder
  
  ;; A SourceFile whose top-level form is a 3-deep nested-if-=-ladder over
  ;; one var `x` — each branch compares (= x "a"), (= x "b"), (= x "c"),
  ;; all returning true, with `false` as the terminator.
  (:wat::core::let
    [src "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool (:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= x \"b\") true (:wat::core::if (:wat::core::= x \"c\") true false))))"
     sf  (:wat::source::File :path "t.wat" :source src)
     files (:wat::core::Vector :wat::source::File sf)
     findings (:wat::lint::lint-source files)]
    (:wat::core::do
      ;; must find at least 1 finding
      (:wat::test::assert-true
        (:wat::core::i64::>= (:wat::core::length findings) 1))
      ;; the first finding must be the ladder rule
      (:wat::test::assert-eq
        (:wat::lint::Finding/rule
          (:wat::core::first findings))
        "nested-if-=-ladder"))))

;; ─── Case 2: no false positive ───────────────────────────────────────

(:wat::test::deftest :wat-tests::lint::no-false-positive-on-clean-forms
  
  ;; Three clean files that must NOT trip the ladder rule:
  ;;   a) a single `if` (not a chain at all)
  ;;   b) two `if`s over DIFFERENT vars (mixed vars — not a ladder)
  ;;   c) a chain that is only 2 deep (below the >=3 threshold)
  (:wat::core::let
    [;; a: single if — one branch, no chain
     src-a "(:wat::core::if (:wat::core::= x \"a\") true false)"
     sf-a  (:wat::source::File :path "a.wat" :source src-a)
     ;; b: two ifs over different vars — var changes, so not a single-var ladder
     src-b "(:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= y \"b\") true false))"
     sf-b  (:wat::source::File :path "b.wat" :source src-b)
     ;; c: 2-deep chain over same var — below >=3 threshold
     src-c "(:wat::core::if (:wat::core::= z \"a\") true (:wat::core::if (:wat::core::= z \"b\") true false))"
     sf-c  (:wat::source::File :path "c.wat" :source src-c)
     files (:wat::core::Vector :wat::source::File sf-a sf-b sf-c)
     findings (:wat::lint::lint-source files)]
    (:wat::test::assert-eq (:wat::core::length findings) 0)))

;; ─── Case 3: the fix applies (DEFERRED — STOP-1) ─────────────────────
;;
;; The auto-fix (replacement AST → write-forms → span offset/len edit) cannot
;; land cleanly in 277.1 because :wat::core::ast-span returns ONLY the START
;; location (line/col), not the end position. Computing old-len for a structural
;; node (the whole ladder form) requires an ast-end-span primitive not yet
;; present in the substrate. The rule ships REPORT-ONLY (fix = "").
;;
;; This case is deferred to 277.1b, which will add the substrate primitive
;; and populate the fix field as (offset, old-len, new-text) for fix-text-apply.

;; ─── Case 4: lint-stdlib runs + rule-zero present ────────────────────

;; ⛔ UN-IGNORE ATTEMPTED 2026-08-16 AND REVERTED THE SAME HOUR. THE IGNORE IS CORRECT.
;;
;; I lifted this on a measurement of **1.826s**, taken with `--run-ignored ignored-only` —
;; 25 tests on a near-idle box. The full floor then went RED, and the arm named the real limit:
;;
;;     deftest_wat_tests_lint_lint_stdlib_runs: exceeded time-limit of 5000ms —
;;     deftest :wat-tests::lint::lint-stdlib-runs (test thread leaked — process exit will reap)
;;
;; The 5000ms cap is the wat harness's own per-deftest limit (`tests/kernel/test.rs:17`), NOT
;; the nextest 15s/30s profile deadline I checked against. Under the real floor — 12-14
;; concurrent test processes — this blows it. THE MEASUREMENT'S CONDITION DID NOT MATCH THE
;; CLAIM'S CONDITION: isolation is not the regime the ignore was describing, and "it's fast
;; when nothing else runs" is not the same statement as "it is fast".
;;
;; It also took a SECOND test down with it: the leaked thread keeps doing lint work for the
;; rest of the run, and `every_wat_scripts_file_loads_on_the_current_runtime` (normally
;; ~90-120s) hit its 240s TIMEOUT in the same floor. One lifted ignore, two reds.
;;
;; ⚠ SEPARATELY, AND STILL TRUE: this test's assertion is TAUTOLOGICAL. `(length findings) >= 0`
;; holds for every Vector, so the only thing it can catch is `lint-stdlib` raising. Measured:
;; it returns **136** findings, so the "0 violations" comment below is about rule-zero
;; specifically, not about `findings`. And the half of this case's own title that says
;; "rule-zero present" is asserted nowhere. Fixing that needs a ruling on what it SHOULD say.
;;
;; UNLOCK, restated honestly: a PERF fix, measured UNDER FLOOR CONTENTION — not in isolation.
(:wat::test::ignore "296-recapture-pending: lint-stdlib exceeds the harness's 5000ms per-deftest limit under floor contention (measured RED 2026-08-16); unlock: a perf fix verified under a full floor, never in isolation")
(:wat::test::deftest :wat-tests::lint::lint-stdlib-runs

  ;; (:wat::lint::lint-stdlib) must evaluate without error and return a Vector.
  ;; Currently 0 rule-zero violations (arc 275 fixed them all); length >= 0.
  (:wat::core::let
    [findings (:wat::lint::lint-stdlib)]
    (:wat::test::assert-true
      (:wat::core::i64::>= (:wat::core::length findings) 0))))

(:wat::test::deftest :wat-tests::lint::rule-zero-finding-on-out-of-order-input
  
  ;; A fabricated out-of-order file pair: file "a" eval-depends on :t::f which
  ;; is defined in file "b" (a defn, not a defmacro), but "a" loads before "b".
  ;; deporder/verify must produce a violation; violations->findings must map it
  ;; to a Finding with rule == "load-order".
  (:wat::core::let
    [a     (:wat::source::File :path "a.wat" :source "(:t::caller (:t::f))")
     b     (:wat::source::File :path "b.wat" :source "(:wat::core::defn :t::f [] -> :wat::core::i64 1)")
     files (:wat::core::Vector :wat::source::File a b)
     viols (:wat::deporder::verify files)
     rule-zero-findings (:wat::lint::violations->findings viols)]
    (:wat::core::do
      ;; must produce at least 1 violation (a refs b's defn but loads before b)
      (:wat::test::assert-true
        (:wat::core::i64::>= (:wat::core::length viols) 1))
      ;; the rule-zero finding must exist and have rule == "load-order"
      (:wat::test::assert-true
        (:wat::core::i64::>= (:wat::core::length rule-zero-findings) 1))
      (:wat::test::assert-eq
        (:wat::lint::Finding/rule
          (:wat::core::first rule-zero-findings))
        "load-order"))))

;; ─── Case 5: detects concat-abuse ────────────────────────────────────

(:wat::test::deftest :wat-tests::lint::detects-concat-abuse
  
  ;; A SourceFile whose body contains a defn with a concat call that mixes
  ;; string literals ("x: ", " of ") with non-literal args (a, b) — the
  ;; textbook hand-rolled template that `format` cures.
  (:wat::core::let
    [src "(:wat::core::defn :t::g [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat \"x: \" a \" of \" b))"
     sf  (:wat::source::File :path "t.wat" :source src)
     files (:wat::core::Vector :wat::source::File sf)
     findings (:wat::lint::lint-source files)]
    (:wat::core::do
      ;; must find at least 1 finding
      (:wat::test::assert-true
        (:wat::core::i64::>= (:wat::core::length findings) 1))
      ;; there must be a finding with rule == "concat-abuse"
      ;; Arc 118.2a — `filter` flipped LAZY; `length` needs a concrete container, so `filterv`.
      (:wat::test::assert-true
        (:wat::core::i64::>=
          (:wat::core::length
            (:wat::core::filterv
              (:wat::core::fn [f <- :wat::lint::Finding] -> :wat::core::bool
                (:wat::core::= (:wat::lint::Finding/rule f) "concat-abuse"))
              findings))
          1)))))

;; ─── Case 6: no false positive for concat ────────────────────────────

(:wat::test::deftest :wat-tests::lint::no-false-positive-concat
  
  ;; Two clean concat calls that must NOT trip the concat-abuse rule:
  ;;   a) all-literal  (concat "a" "b") — nothing to interpolate
  ;;   b) all-value    (concat a b)     — no literal scaffolding
  (:wat::core::let
    [;; a: all-literal — both args are string literals
     src-a "(:wat::core::string::concat \"a\" \"b\")"
     sf-a  (:wat::source::File :path "a.wat" :source src-a)
     ;; b: all-value — both args are symbols (non-literals)
     src-b "(:wat::core::string::concat a b)"
     sf-b  (:wat::source::File :path "b.wat" :source src-b)
     files (:wat::core::Vector :wat::source::File sf-a sf-b)
     findings (:wat::lint::lint-source files)]
    (:wat::test::assert-eq (:wat::core::length findings) 0)))

;; ─── Case 8: rename-keyword-prefix — type-arg reach + boundary guard (arc 283.1) ───

(:wat::test::deftest :wat-tests::lint::rename-keyword-prefix-type-arg-and-boundary
  
  ;; Renaming :t::Old → :t::New over a source with:
  ;;   - a TYPE-ARG  (Vector<t::Old>)    — must rename → Vector<t::New>
  ;;   - a return type (:t::Old)          — must rename → :t::New
  ;;   - an accessor (:t::Old/make)       — must rename → :t::New/make
  ;;   - a boundary DECOY (:t::OldExtra)  — must NOT rename (boundary guard)
  (:wat::core::let
    [src "(:wat::core::defn :u::f [xs <- :wat::core::Vector<t::Old> y <- :t::OldExtra] -> :t::Old (:t::Old/make xs))"
     result (:wat::fix::rename-keyword-prefix ":t::Old" ":t::New" src)]
    (:wat::core::do
      ;; type-arg must rename
      (:wat::test::assert-true
        (:wat::core::string::contains? result "Vector<t::New>"))
      ;; return type must rename
      (:wat::test::assert-true
        (:wat::core::string::contains? result "-> :t::New "))
      ;; accessor must rename
      (:wat::test::assert-true
        (:wat::core::string::contains? result ":t::New/make"))
      ;; boundary decoy must survive untouched
      (:wat::test::assert-true
        (:wat::core::string::contains? result ":t::OldExtra"))
      ;; old type-arg form must be gone
      (:wat::test::assert-false
        (:wat::core::string::contains? result "Vector<t::Old>")))))

;; ─── Case 8: concat-format-fix — bare-symbol rewrites to format; compound stays; dedup ────

(:wat::test::deftest :wat-tests::lint::concat-format-fix-bare-symbol-rewrites
  
  ;; Part A: (string::concat "x: " a " y: " b) — a,b bare symbols.
  ;; lint-fix-file must rewrite to a format call with {a}/{b} slots and :a a :b b kwargs.
  ;;
  ;; Part B: (string::concat "n=" (i64::to-string n)) — value slot is a compound expr.
  ;; lint-fix-file must leave it unchanged (report-only, no format rewrite).
  ;;
  ;; Part C: (string::concat "pre:" x "-" x) — same symbol x twice.
  ;; The template must have {x} twice but the kwarg list must have :x x only once.
  (:wat::core::let
    [;; Part A: bare-symbol concat
     src-a "(:wat::core::defn :u::g [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat \"x: \" a \" y: \" b))"
     sf-a  (:wat::source::File :path "a.wat" :source src-a)
     fixed-a (:wat::lint::lint-fix-file sf-a)
     ;; Part B: compound-slot concat
     src-b "(:wat::core::defn :u::h [n <- :wat::core::i64] -> :wat::core::String (:wat::core::string::concat \"n=\" (:wat::core::i64::to-string n)))"
     sf-b  (:wat::source::File :path "b.wat" :source src-b)
     fixed-b (:wat::lint::lint-fix-file sf-b)
     ;; Part C: same-symbol-twice dedup
     src-c "(:wat::core::defn :u::k [x <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat \"pre:\" x \"-\" x))"
     sf-c  (:wat::source::File :path "c.wat" :source src-c)
     fixed-c (:wat::lint::lint-fix-file sf-c)]
    (:wat::core::do
      ;; Part A: must contain format call with {a}/{b} slots and kwargs
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-a "(:wat::core::format"))
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-a "{a}"))
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-a "{b}"))
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-a ":a a"))
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-a ":b b"))
      ;; Part A: the concat call must be gone
      (:wat::test::assert-false
        (:wat::core::string::contains? fixed-a "string::concat"))
      ;; Part B: compound-slot must stay report-only (concat must remain)
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-b "string::concat"))
      (:wat::test::assert-false
        (:wat::core::string::contains? fixed-b "(:wat::core::format"))
      ;; Part C: format present, {x} appears, but :x x appears only once
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-c "(:wat::core::format"))
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-c "{x}"))
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed-c ":x x"))
      ;; Part C: concat must be gone
      (:wat::test::assert-false
        (:wat::core::string::contains? fixed-c "string::concat")))))

;; ─── Case 9: concat-fix position gate — defmacro→interpolate, defn→format ─────

(:wat::test::deftest :wat-tests::lint::concat-fix-position-gate
  
  ;; A source with BOTH positions:
  ;;   - a defmacro whose body builds a name via bare-symbol concat (expand-time → interpolate)
  ;;   - a defn whose body has a bare-symbol concat (runtime → format)
  ;; lint-fix-file must rewrite the defmacro-body one to string::interpolate and the
  ;; defn-body one to format; no string::concat survives.
  (:wat::core::let
    [src "(:wat::core::defmacro :u::m [x <- :wat::WatAST] -> :wat::core::String (:wat::core::let [s (:wat::core::ast-name x) nm (:wat::core::string::concat s \"::Op\")] nm)) (:wat::core::defn :u::f [a <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat \"x: \" a))"
     sf  (:wat::source::File :path "t.wat" :source src)
     fixed (:wat::lint::lint-fix-file sf)]
    (:wat::core::do
      ;; defmacro-body concat must become interpolate
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed "(:wat::core::string::interpolate \"{s}::Op\" :s s)"))
      ;; defn-body concat must become format
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed "(:wat::core::format \"x: {a}\" :a a)"))
      ;; no string::concat must survive
      (:wat::test::assert-false
        (:wat::core::string::contains? fixed "string::concat")))))

;; ─── Case 7: ladder-autofix rewrites to contains? + clean file round-trips ────

(:wat::test::deftest :wat-tests::lint::ladder-autofix-rewrites-and-round-trips
  
  ;; Part A: a 3-deep nested-if-=-ladder over `x` — lint-fix-file must rewrite it
  ;; to a (contains? (HashSet …) x) call. The fixed source must contain "contains?"
  ;; and no longer contain the nested "(:wat::core::if (:wat::core::= x".
  ;; Part B: a clean file (no findings) must round-trip byte-identical through
  ;; lint-fix-file (no edits applied means source is returned unchanged).
  (:wat::core::let
    [src-ladder "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool (:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= x \"b\") true (:wat::core::if (:wat::core::= x \"c\") true false))))"
     sf-ladder  (:wat::source::File :path "t.wat" :source src-ladder)
     fixed      (:wat::lint::lint-fix-file sf-ladder)
     src-clean  "(:wat::core::defn :t::add [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ a b))"
     sf-clean   (:wat::source::File :path "clean.wat" :source src-clean)
     fixed-clean (:wat::lint::lint-fix-file sf-clean)]
    (:wat::core::do
      ;; Part A: fixed source must contain "contains?" and "HashSet"
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed "contains?"))
      (:wat::test::assert-true
        (:wat::core::string::contains? fixed "HashSet"))
      ;; Part A: the original ladder must be gone
      (:wat::test::assert-false
        (:wat::core::string::contains? fixed "(:wat::core::if (:wat::core::= x"))
      ;; Part B: clean file must round-trip byte-identical
      (:wat::test::assert-eq fixed-clean src-clean))))
