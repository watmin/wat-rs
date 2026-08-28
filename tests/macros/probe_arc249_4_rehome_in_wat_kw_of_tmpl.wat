;; STONE-defservice-emits-the-binder (arc 109) — `:wat::core::keyword/of` retired (its whole
;; purpose was minting `Head<a,b>`, the spelling this stone kills). This fixture's actual
;; subject was never keyword/of's own semantics — it is macro-in-TEMPLATE-position firing (a
;; macro whose quasiquote body calls ANOTHER macro, `:my::mk` calling `:test::mk-kw` here) —
;; so the vehicle swaps to a local test-only macro that mints a plain keyword with no angle
;; spelling anywhere, keeping the exact same test topology.
(:wat::core::defmacro :test::mk-kw
  [head <- :wat::WatAST arg <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::let [head-text (:wat::keyword::to-string head)
                    arg-text  (:wat::keyword::to-string arg)
                    full (:wat::string::concat head-text
                           (:wat::string::concat "-" arg-text))]
    `~(:wat::keyword::from-string full)))
(:wat::core::defmacro :my::mk
  [e <- :wat::WatAST] -> :wat::WatAST
  `(:test::mk-kw :foo ~e))
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::keyword::to-string (:my::mk :bar)))
