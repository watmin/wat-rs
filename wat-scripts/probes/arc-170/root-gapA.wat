;; ROOT Gap A — fn-forms directly on the kwargs $impl vs a hand-written same-shape fn.
;; CLAIM (measured): fn-forms on the hand-written fn (hf) and on the kwargs $impl (kf) both
;; ship exactly 5 forms (the macro + structtype registrations needed are identical in count —
;; fn-forms ships the full currently-registered macro/struct set, not a precise per-fn
;; dependency closure), and each one's LAST form is the correct target def — its name NODE is
;; the keyword `:test::hand` for hf and `:test::work` for kf. No gap: both paths name the right
;; function; "ROOT Gap A" resolves negative (no divergence between the two shippers).
;; Asserted as AST-NODE equality (`wat.type/AST` extends Equatable; `WatAST`'s PartialEq
;; compares structure, skipping span) against a node built by `:wat::core::keyword-node`, never
;; by extracting the node's name via `ast-name` and comparing that STRING (coordinator
;; correction, 255.76 weigh: "measuring strings is anti-wat").
(:wat::core::defstruct :probe::Bag [n <- wat.type/String])
(:wat::core::defn :probe::hand
  [item <- wat.type/String  bag <- :probe::Bag] -> wat.type/String
  (:wat::core::let [x (:probe::Bag/n bag)] (:wat::string::concat x item)))
(:wat::core::defn :probe::work
  [item <- wat.type/String  & [n <- wat.type/String]] -> wat.type/String
  (:wat::string::concat n item))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [_  (:wat::kernel::println "--- fn-forms HAND-WRITTEN ---")
     hf (:wat::kernel::fn-forms :probe::hand :test::hand)
     _  (:wat::kernel::println "hand: ok")
     _  (:wat::kernel::println "--- fn-forms KWARGS impl ---")
     kf (:wat::kernel::fn-forms :probe::work$impl :test::work)
     _  (:wat::kernel::println "work impl: ok")
     hf-last    (:wat::core::Option/expect (:wat::core::last hf) "no last")
     kf-last    (:wat::core::Option/expect (:wat::core::last kf) "no last")
     hf-name-node (:wat::core::nth (:wat::core::ast->children hf-last) 1)
     kf-name-node (:wat::core::nth (:wat::core::ast->children kf-last) 1)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::length hf) 5)
      (:wat::test::assert-eq (:wat::core::length kf) 5)
      (:wat::test::assert-eq hf-name-node (:wat::core::keyword-node ":test::hand"))
      (:wat::test::assert-eq kf-name-node (:wat::core::keyword-node ":test::work"))
      (:wat::kernel::println "both ok"))))
