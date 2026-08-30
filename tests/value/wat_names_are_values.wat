;; tests/value/wat_names_are_values.wat — co-located fixture for the sibling probe (.rs).
;; Slurped via startup_beside(file!()). Each function pair covers one test case.
;; No :user::main needed — startup_beside loads defns; tests call each fn via eval_in_frozen.

;; ─── Test 1: named define is a function value ─────────────────────────────────

(:wat::core::defn :t::test1-double [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::* x 2))

(:wat::core::defn :t::test1 [] -> :wat::core::i64
  (:wat::core::let
    [f      :t::test1-double
     result (f 21)]
    result))

;; ─── Test 2: named define as higher-order argument ────────────────────────────

(:wat::core::defn :t::test2-inc [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ n 1))

(:wat::core::defn :t::test2-apply-twice
  [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64
   x <- :wat::core::i64]
  -> :wat::core::i64
  (f (f x)))

(:wat::core::defn :t::test2 [] -> :wat::core::i64
  (:t::test2-apply-twice :t::test2-inc 5))

;; ─── Test 3: polymorphic named define instantiates at use site ───────────────

(:wat::core::defn :t::test3-identity :- [T] [x <- :T] -> :T x)

(:wat::core::defn :t::test3-apply
  [f <- :wat::core::Fn(wat::core::i64)->wat::core::i64
   x <- :wat::core::i64]
  -> :wat::core::i64
  (f x))

(:wat::core::defn :t::test3 [] -> :wat::core::i64
  (:t::test3-apply :t::test3-identity 99))

;; ─── Test 4: unregistered keyword stays a literal ────────────────────────────

(:wat::core::defn :t::test4 [] -> :wat::core::i64
  (:wat::core::let
    [tag    :my-app::tag::user-event
     same?  (:wat::core::= tag :my-app::tag::user-event)]
    (:wat::core::if same? 
      1
      0)))

;; ─── Test 5: named define as map argument ────────────────────────────────────
;; (Migrated off the annihilated :wat::stream::* — arc 118, 2026-06-27;
;;  the intent is named-defn-as-HOF-arg, the collection vehicle is incidental.)

(:wat::core::defn :t::test5-double [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::* n 2))

;; Arc 118.2a — `map` flipped LAZY; `doubled` is consumed TWICE (`first` and `length`) and
;; `length` needs a concrete container regardless, so `mapv`.
(:wat::core::defn :t::test5 [] -> :wat::core::i64
  (:wat::core::let
    [source  (:wat::core::Vector :- [:wat::core::i64] 1 2 3)
     doubled (:wat::core::mapv :t::test5-double source)
     first   (:wat::core::first doubled)
     len     (:wat::core::length doubled)]
    (:wat::core::if (:wat::core::and (:wat::core::= first 2) (:wat::core::= len 3))
      
      1
      0)))
