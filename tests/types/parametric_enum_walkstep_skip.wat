;; parametric_enum_walkstep_skip.wat — WalkStep::Skip parametric inference.
;; Arc 255 STONE-the-eval-surface-faces-watast: `WalkStep::Skip`'s "terminal"
;; field is now `:wat::WatAST` (was `:wat::holon::HolonAST`) — a bare
;; `(:wat::holon::leaf 999)` no longer type-checks here; `to-wat` wraps it
;; (STOP-3: no new verb minted).
(:wat::core::defn :my::test::halt [n <- :wat::core::i64] -> (:wat::eval::WalkStep :- [:wat::core::i64])
  (:wat::eval::WalkStep::Skip
    (:wat::holon::to-wat (:wat::holon::leaf 999))
    n))
(:wat::core::defn :my::compute [] -> :wat::core::i64
  (:wat::core::let
    [halted (:my::test::halt 3)]
    3))
