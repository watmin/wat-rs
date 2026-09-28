;; tests/function/fn_signature.wat — combined positive fixture for fn_signature tests.
;; Arc 167 — flat-shape fn signature tests.

;; T1: fn_with_flat_shape_compiles_and_runs
(:wat::core::defn :my::compute_t1 [] -> wat.type/i64
  ((:wat::core::fn [x <- wat.type/i64 y <- wat.type/i64]
               -> wat.type/i64
               (:wat::i64::+ x y))
             2 3))

;; T2: defn_with_flat_shape_compiles_and_runs
(:wat::core::defn :user::add_t2
  [x <- wat.type/i64 y <- wat.type/i64]
  -> wat.type/i64
  (:wat::i64::+ x y))

(:wat::core::defn :my::compute_t2 [] -> wat.type/i64 (:user::add_t2 2 3))

;; T3: recursive_defn_with_flat_shape
(:wat::core::defn :user::fact_t3
  [n <- wat.type/i64]
  -> wat.type/i64
  (:wat::core::if (:wat::core::= n 0) 
    1
    (:wat::i64::* n (:user::fact_t3 (:wat::i64::- n 1)))))

(:wat::core::defn :my::compute_t3 [] -> wat.type/i64 (:user::fact_t3 5))

;; T4: zero_arg_fn_with_empty_vector
(:wat::core::defn :my::compute_t4 [] -> wat.type/i64 ((:wat::core::fn [] -> wat.type/i64 42)))

;; T9: reflection_on_flat_defn_resolves — lookup-define :user::add_t2
(:wat::core::defn :my::compute_t9 [] -> wat.type/i64
  (:wat::core::match
              (:wat::runtime::lookup-define :user::add_t2)
              
              [:wat::core::Option.Some {:value _} 1]
              [:wat::core::Option.None {}    0]))
