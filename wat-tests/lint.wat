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
  ()
  ;; A SourceFile whose top-level form is a 3-deep nested-if-=-ladder over
  ;; one var `x` — each branch compares (= x "a"), (= x "b"), (= x "c"),
  ;; all returning true, with `false` as the terminator.
  (:wat::core::let
    [src "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool (:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= x \"b\") true (:wat::core::if (:wat::core::= x \"c\") true false))))"
     sf  (:wat::deporder::SourceFile "t.wat" src)
     files (:wat::core::Vector :wat::deporder::SourceFile sf)
     findings (:wat::lint::lint-source files)]
    (:wat::core::do
      ;; must find at least 1 finding
      (:wat::test::assert-true
        (:wat::core::i64::>= (:wat::core::length findings) 1))
      ;; the first finding must be the ladder rule
      (:wat::test::assert-eq
        (:wat::lint::Finding/rule
          (:wat::core::Option/expect -> :wat::lint::Finding
            (:wat::core::first findings)
            "case-1: first finding"))
        "nested-if-=-ladder"))))

;; ─── Case 2: no false positive ───────────────────────────────────────

(:wat::test::deftest :wat-tests::lint::no-false-positive-on-clean-forms
  ()
  ;; Three clean files that must NOT trip the ladder rule:
  ;;   a) a single `if` (not a chain at all)
  ;;   b) two `if`s over DIFFERENT vars (mixed vars — not a ladder)
  ;;   c) a chain that is only 2 deep (below the >=3 threshold)
  (:wat::core::let
    [;; a: single if — one branch, no chain
     src-a "(:wat::core::if (:wat::core::= x \"a\") true false)"
     sf-a  (:wat::deporder::SourceFile "a.wat" src-a)
     ;; b: two ifs over different vars — var changes, so not a single-var ladder
     src-b "(:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= y \"b\") true false))"
     sf-b  (:wat::deporder::SourceFile "b.wat" src-b)
     ;; c: 2-deep chain over same var — below >=3 threshold
     src-c "(:wat::core::if (:wat::core::= z \"a\") true (:wat::core::if (:wat::core::= z \"b\") true false))"
     sf-c  (:wat::deporder::SourceFile "c.wat" src-c)
     files (:wat::core::Vector :wat::deporder::SourceFile sf-a sf-b sf-c)
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

(:wat::test::deftest :wat-tests::lint::lint-stdlib-runs
  ()
  ;; (:wat::lint::lint-stdlib) must evaluate without error and return a Vector.
  ;; Currently 0 rule-zero violations (arc 275 fixed them all); length >= 0.
  (:wat::core::let
    [findings (:wat::lint::lint-stdlib)]
    (:wat::test::assert-true
      (:wat::core::i64::>= (:wat::core::length findings) 0))))

(:wat::test::deftest :wat-tests::lint::rule-zero-finding-on-out-of-order-input
  ()
  ;; A fabricated out-of-order file pair: file "a" eval-depends on :t::f which
  ;; is defined in file "b" (a defn, not a defmacro), but "a" loads before "b".
  ;; deporder/verify must produce a violation; violations->findings must map it
  ;; to a Finding with rule == "load-order".
  (:wat::core::let
    [a     (:wat::deporder::SourceFile "a.wat" "(:t::caller (:t::f))")
     b     (:wat::deporder::SourceFile "b.wat" "(:wat::core::defn :t::f [] -> :wat::core::i64 1)")
     files (:wat::core::Vector :wat::deporder::SourceFile a b)
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
          (:wat::core::Option/expect -> :wat::lint::Finding
            (:wat::core::first rule-zero-findings)
            "case-4: first rule-zero finding"))
        "load-order"))))

;; ─── Case 5: detects concat-abuse ────────────────────────────────────

(:wat::test::deftest :wat-tests::lint::detects-concat-abuse
  ()
  ;; A SourceFile whose body contains a defn with a concat call that mixes
  ;; string literals ("x: ", " of ") with non-literal args (a, b) — the
  ;; textbook hand-rolled template that `format` cures.
  (:wat::core::let
    [src "(:wat::core::defn :t::g [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::String (:wat::core::string::concat \"x: \" a \" of \" b))"
     sf  (:wat::deporder::SourceFile "t.wat" src)
     files (:wat::core::Vector :wat::deporder::SourceFile sf)
     findings (:wat::lint::lint-source files)]
    (:wat::core::do
      ;; must find at least 1 finding
      (:wat::test::assert-true
        (:wat::core::i64::>= (:wat::core::length findings) 1))
      ;; there must be a finding with rule == "concat-abuse"
      (:wat::test::assert-true
        (:wat::core::i64::>=
          (:wat::core::length
            (:wat::core::filter
              (:wat::core::fn [f <- :wat::lint::Finding] -> :wat::core::bool
                (:wat::core::= (:wat::lint::Finding/rule f) "concat-abuse"))
              findings))
          1)))))

;; ─── Case 6: no false positive for concat ────────────────────────────

(:wat::test::deftest :wat-tests::lint::no-false-positive-concat
  ()
  ;; Two clean concat calls that must NOT trip the concat-abuse rule:
  ;;   a) all-literal  (concat "a" "b") — nothing to interpolate
  ;;   b) all-value    (concat a b)     — no literal scaffolding
  (:wat::core::let
    [;; a: all-literal — both args are string literals
     src-a "(:wat::core::string::concat \"a\" \"b\")"
     sf-a  (:wat::deporder::SourceFile "a.wat" src-a)
     ;; b: all-value — both args are symbols (non-literals)
     src-b "(:wat::core::string::concat a b)"
     sf-b  (:wat::deporder::SourceFile "b.wat" src-b)
     files (:wat::core::Vector :wat::deporder::SourceFile sf-a sf-b)
     findings (:wat::lint::lint-source files)]
    (:wat::test::assert-eq (:wat::core::length findings) 0)))

;; ─── Case 7: ladder-autofix rewrites to contains? + clean file round-trips ────

(:wat::test::deftest :wat-tests::lint::ladder-autofix-rewrites-and-round-trips
  ()
  ;; Part A: a 3-deep nested-if-=-ladder over `x` — lint-fix-file must rewrite it
  ;; to a (contains? (HashSet …) x) call. The fixed source must contain "contains?"
  ;; and no longer contain the nested "(:wat::core::if (:wat::core::= x".
  ;; Part B: a clean file (no findings) must round-trip byte-identical through
  ;; lint-fix-file (no edits applied means source is returned unchanged).
  (:wat::core::let
    [src-ladder "(:wat::core::defn :t::f [x <- :wat::core::String] -> :wat::core::bool (:wat::core::if (:wat::core::= x \"a\") true (:wat::core::if (:wat::core::= x \"b\") true (:wat::core::if (:wat::core::= x \"c\") true false))))"
     sf-ladder  (:wat::deporder::SourceFile "t.wat" src-ladder)
     fixed      (:wat::lint::lint-fix-file sf-ladder)
     src-clean  "(:wat::core::defn :t::add [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ a b))"
     sf-clean   (:wat::deporder::SourceFile "clean.wat" src-clean)
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
