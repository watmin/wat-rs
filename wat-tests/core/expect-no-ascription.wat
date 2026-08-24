;; wat-tests/core/expect-no-ascription.wat — -> :T annihilation sub-strike 1 probe.
;;
;; The clean kills: Option/expect + Result/expect no longer carry a `-> :T`
;; return ascription — the unwrapped type is INFERRED from the (Option :- [T]) /
;; (Result :- [T E]) argument (exactly the recv'/select' kill, arc 258.5b).
;;
;; RED at HEAD: both forms REQUIRE the `-> :T` arrow (layout
;; `(Option/expect -> :T <opt> <msg>)`, items.len() >= 5). The bare 2-arg form
;; `(Option/expect <opt> <msg>)` is malformed → these tests fail to type-check.
;;
;; GREEN after sub-strike 1: the arrow is annihilated; `T` is inferred from the
;; argument; the bare form is the only form.

(:wat::test::deftest :wat-tests::core::option-expect-no-ascription
  
  (:wat::test::assert-eq
    (:wat::core::Option/expect (:wat::core::Some 5) "should be present")
    5))

(:wat::test::deftest :wat-tests::core::result-expect-no-ascription
  
  (:wat::test::assert-eq
    (:wat::core::Result/expect (:wat::core::Ok 7) "should be ok")
    7))
