;; tests/wat_lang/wat_arc144_lookup_form.wat — co-located fixture.
;; Arc 144 slice 1 — uniform lookup_form reflection across five form-kinds.
;; Named :t:: functions return bool (Some→true / None→false) or String.

;; Declarations needed for reflection tests.
(:wat::core::defmacro :my::ident [x <- :wat::WatAST] -> :wat::WatAST `~x)

(:wat::core::defstruct :my::Bar
  [open  <- :wat::core::f64
   close <- :wat::core::f64])

(:wat::core::defstruct :my::Point
  [x <- :wat::core::f64
   y <- :wat::core::f64])

(:wat::core::defstruct :my::Tick
  [price <- :wat::core::f64])

(:wat::core::defn :t::my-add [x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::+ x y))

;; ─── Macro lookup ──────────────────────────────────────────────────────────

(:wat::core::defn :t::test1-lookup-macro-render [] -> :wat::core::String
  (:wat::core::let [def-opt  (:wat::runtime::lookup-define :my::ident)
                   rendered (:wat::edn::write def-opt)]
    rendered))

(:wat::core::defn :t::test2-sig-macro [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :my::ident)
    -> :wat::core::bool
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::test3-body-macro [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::body-of :my::ident)
    -> :wat::core::bool
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

;; ─── Type lookup ───────────────────────────────────────────────────────────

(:wat::core::defn :t::test4-lookup-struct-render [] -> :wat::core::String
  (:wat::core::let [def-opt  (:wat::runtime::lookup-define :my::Bar)
                   rendered (:wat::edn::write def-opt)]
    rendered))

(:wat::core::defn :t::test5-sig-struct [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :my::Point)
    -> :wat::core::bool
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::test6-body-struct-none [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::body-of :my::Tick)
    -> :wat::core::bool
    ((:wat::core::Some _) false)
    (:wat::core::None    true)))

;; ─── Regression guards: UserFunction + Primitive ────────────────────────────

(:wat::core::defn :t::test7-lookup-user-fn [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::lookup-define :t::my-add)
    -> :wat::core::bool
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

(:wat::core::defn :t::test8-sig-foldl [] -> :wat::core::bool
  (:wat::core::match
    (:wat::runtime::signature-of-defn :wat::core::foldl)
    -> :wat::core::bool
    ((:wat::core::Some _) true)
    (:wat::core::None    false)))

;; ─── Unknown name returns None across all three ──────────────────────────────

(:wat::core::defn :t::test9-all-none [] -> :wat::core::bool
  (:wat::core::let
    [d-opt (:wat::runtime::lookup-define :no::such::thing)
     s-opt (:wat::runtime::signature-of-defn :no::such::thing)
     b-opt (:wat::runtime::body-of    :no::such::thing)]
    (:wat::core::match d-opt
      -> :wat::core::bool
      ((:wat::core::Some _) false)
      (:wat::core::None
        (:wat::core::match s-opt
          -> :wat::core::bool
          ((:wat::core::Some _) false)
          (:wat::core::None
            (:wat::core::match b-opt
              -> :wat::core::bool
              ((:wat::core::Some _) false)
              (:wat::core::None    true))))))))
