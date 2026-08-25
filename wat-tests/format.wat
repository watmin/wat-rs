;; wat-tests/format.wat — arc 279 :wat::core::format deftests.
;;
;; Happy-path cases for the named-template printf macro.
;; Strict-error cases (macro-error on missing/unused kwarg) are in
;; tests/probe_arc279_format.rs (require startup_from_source failure testing).
;;
;; All cases compile at deftest-discovery time — the format macro is
;; expanded when startup_from_source loads this file.

;; ── 1. Named substitution, out-of-order kwargs, heterogeneous types ──────────
;;
;; "{a} {b}" with :b 5 :a "x" — kwargs out of template order.
;; String fills as itself (no EDN quotes); i64 fills as its decimal digits.
(:wat::test::deftest :wat-tests::format::named-out-of-order-heterogeneous
  
  (:wat::test::assert-eq
    (:wat::core::format "{a} {b}" :b 5 :a "x")
    "x 5"))

;; ── 2. Unquoted rendering — String fills as itself, i64 as digits ────────────
;;
;; A String value fills without EDN surrounding quotes; an i64 fills as digits.
;; Contrast with `show` which would render "ada" as `"ada"` and 42 as `42`.
(:wat::test::deftest :wat-tests::format::unquoted-string-and-i64
  
  (:wat::core::let [s (:wat::core::format "{name} is {age}" :name "ada" :age 42)]
    (:wat::core::do
      (:wat::test::assert-eq s "ada is 42")
      ;; Confirm string has no surrounding quotes (unquoted, not EDN-show).
      (:wat::test::assert-eq (:wat::string::contains? s "\"") false))))

;; ── 3. Full probe case — multi-placeholder with static text between ───────────
;;
;; The same call as probe_arc279_format.rs: three placeholders, out-of-order,
;; with static comma/space/! text between.
(:wat::test::deftest :wat-tests::format::full-probe-case
  
  (:wat::test::assert-eq
    (:wat::core::format "{greeting}, {name}! you have {count} messages"
      :name "ada" :greeting "hello" :count 3)
    "hello, ada! you have 3 messages"))

;; ── 4. Single placeholder — minimal case ────────────────────────────────────
(:wat::test::deftest :wat-tests::format::single-placeholder
  
  (:wat::test::assert-eq
    (:wat::core::format "{msg}" :msg "hello world")
    "hello world"))

;; ── 5. Bool placeholder — bool fills as true/false ──────────────────────────
(:wat::test::deftest :wat-tests::format::bool-placeholder
  
  (:wat::test::assert-eq
    (:wat::core::format "enabled: {flag}" :flag true)
    "enabled: true"))
