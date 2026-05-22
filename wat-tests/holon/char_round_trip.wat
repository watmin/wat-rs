;; wat-tests/holon/char_round_trip.wat — Arc 220 slice 2: :wat::core::Char.
;;
;; Exercises the `\c` literal + `(:wat::core::Char/of "x")` constructor.
;; All cases pass if the Char primitive is correctly wired (lexer, eval,
;; edn_shim bridge, equality, EDN round-trip).

;; ─── 1: Char/of constructor and equality ──────────────────────────────────

(:wat::test::deftest :wat-tests::holon::char_round_trip::char-of-constructor
  ()
  (:wat::core::let
    [a (:wat::core::Char/of "a")
     b (:wat::core::Char/of "a")]
    (:wat::test::assert-eq a b)))

;; ─── 2: `\c` literal equals Char/of ──────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::char_round_trip::char-literal
  ()
  (:wat::core::let
    [lit \a
     con (:wat::core::Char/of "a")]
    (:wat::test::assert-eq lit con)))

;; ─── 3: Named char `\newline` ─────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::char_round_trip::char-literal-newline
  ()
  (:wat::core::let
    [nl      \newline
     nl-con  (:wat::core::Char/of "\n")]
    (:wat::test::assert-eq nl nl-con)))

;; ─── 4: Named char `\space` ───────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::char_round_trip::char-literal-space
  ()
  (:wat::core::let
    [sp     \space
     sp-con (:wat::core::Char/of " ")]
    (:wat::test::assert-eq sp sp-con)))

;; ─── 5: Different chars are not equal ────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::char_round_trip::char-neq
  ()
  (:wat::core::let
    [a \a
     b \b
     eq (:wat::core::= a b)]
    (:wat::test::assert-eq eq false)))
