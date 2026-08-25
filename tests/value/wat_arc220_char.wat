
;; tests/value/wat_arc220_char.wat — co-located fixture for the sibling probe (.rs).
;; Slurped via startup_beside(file!()). Each function covers one test case.
;; Tests 1-3, 5, 9, 10 return bool. Tests 6-8 call char/of with invalid args → runtime error.
;; Test 4 uses a separate negative fixture (supplementary-plane char literal fails at lex time).

;; ─── Test 1: Lexer accepts \a single-char literal ────────────────────────────

(:wat::core::defn :t::test1-char-literal-single-letter [] -> :wat::core::bool
  (:wat::core::let
    [c        \a
     expected (:wat::core::char "a")]
    (:wat::core::= c expected)))

;; ─── Test 2: Lexer accepts named chars ───────────────────────────────────────

(:wat::core::defn :t::test2-char-literal-named-chars [] -> :wat::core::bool
  (:wat::core::let
    [nl      \newline
     sp      \space
     tab     \tab
     ret     \return
     nl-exp  (:wat::core::char "\n")
     sp-exp  (:wat::core::char " ")
     tab-exp (:wat::core::char "\t")
     ret-exp (:wat::core::char "\r")]
    (:wat::core::and
      (:wat::core::= nl nl-exp)
      (:wat::core::and
        (:wat::core::= sp sp-exp)
        (:wat::core::and
          (:wat::core::= tab tab-exp)
          (:wat::core::= ret ret-exp))))))

;; ─── Test 3: Lexer accepts A Unicode BMP escape (= 'A') ───────────────

(:wat::core::defn :t::test3-char-literal-unicode-escape [] -> :wat::core::bool
  (:wat::core::let
    [c        \u0041
     expected (:wat::core::char "A")]
    (:wat::core::= c expected)))

;; ─── Test 5: char/of valid single char ───────────────────────────────────────

(:wat::core::defn :t::test5-char-of-valid-single-char [] -> :wat::core::bool
  (:wat::core::let
    [c1  (:wat::core::char "x")
     c2  (:wat::core::char "x")]
    (:wat::core::= c1 c2)))

;; ─── Test 6: char/of "" errors with length diagnostic ────────────────────────

(:wat::core::defn :t::test6-char-of-empty [] -> :wat::core::nil
  (:wat::core::let
    [_c (:wat::core::char "")]
    nil))

;; ─── Test 7: char/of "ab" errors with length diagnostic ──────────────────────

(:wat::core::defn :t::test7-char-of-multi [] -> :wat::core::nil
  (:wat::core::let
    [_c (:wat::core::char "ab")]
    nil))

;; ─── Test 8: char/of with supplementary-plane char rejected ──────────────────

(:wat::core::defn :t::test8-char-of-supplementary [] -> :wat::core::nil
  (:wat::core::let
    [_c (:wat::core::char "😀")]
    nil))

;; ─── Test 9: Round-trip: char/of → EDN write → edn read → identical ─────────

(:wat::core::defn :t::test9-char-edn-round-trip [] -> :wat::core::bool
  (:wat::core::let
    [orig  (:wat::core::char "x")
     edn   (:wat::edn::write orig)
     back  (:wat::edn::read edn)
     ok    (:wat::core::= orig back)]
    ok))

;; ─── Test 10: Equality ───────────────────────────────────────────────────────

(:wat::core::defn :t::test10-char-equality [] -> :wat::core::bool
  (:wat::core::let
    [a1  \a
     a2  \a
     b   \b
     eq1 (:wat::core::= a1 a2)
     eq2 (:wat::core::= a1 b)]
    (:wat::core::and eq1 (:wat::core::not eq2))))
