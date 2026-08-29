;; PROBE — arc 285's crux, answered. Lives here (loader-gated by
;; `every_wat_scripts_file_loads`) so it cannot rot into a graveyard that reads
;; like live code.
;;
;; THE QUESTION the 285 STUB asks: "can a built-in `Value` type satisfy a
;; wat-defined defprotocol whose methods route to those Rust intrinsics?"
;;
;; THE QUESTION IT DOES NOT ASK, and which was the real unknown: `(Seqable :- [T])`
;; (wat/seq.wat:75) proves ONE type param over built-ins. `(Dialable :- [S R])`
;; (wat/capability.wat:44) proves TWO type params over a user Struct. A `(Map :- [K V])`
;; is TWO params over a BUILT-IN — a combination neither precedent covers.
;;
;; ANSWER, measured 2026-08-20 at HEAD 9b360374f: it works, with no new substrate.
;; Both map families satisfy one surface; a surface-typed fn param accepts either;
;; runtime dispatch reaches the right Rust intrinsic.
;;     (:user::lookup <HashMap>)        -> #wat.core.Option/Some [1]
;;     (:user::lookup <PersistentMap>)  -> #wat.core.Option/Some [2]
;;
;; ⚠ NOTE THE TWO CONSTRUCTORS BELOW. `HashMap` REQUIRES leading type keywords
;; (K V) and rejects their absence; `PersistentMap` REJECTS them and infers K/V
;; from the pairs. Both directions tested. That asymmetry is a live "a map is a
;; map" violation and is arc 285's own business — see its DESIGN.

(:wat::core::defsurface :user::Mapping :- [K V] :nature :wat::core::Struct
  :features [(mget [self <- (:user::Mapping :- [K V]) k <- K] -> (:wat::core::Option :- [V]))])

(:wat::core::extend-type :wat::core::HashMap (:user::Mapping :- [K V])
  (mget [self k] -> (:wat::core::Option :- [V]) (:wat::hashmap::get self k)))

(:wat::core::extend-type :wat::core::PersistentMap (:user::Mapping :- [K V])
  (mget [self k] -> (:wat::core::Option :- [V]) (:wat::map::get self k)))

;; The payoff: ONE fn, typed against the surface, taking EITHER family.
(:wat::core::defn :user::lookup [m <- (:user::Mapping :- [:wat::core::String :wat::core::i64])]
                  -> (:wat::core::Option :- [:wat::core::i64])
  (:user::Mapping/mget m "a"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println (:user::lookup (:wat::core::HashMap :- [:wat::core::String :wat::core::i64] "a" 1)))
    (:wat::kernel::println (:user::lookup (:wat::core::PersistentMap "a" 2)))))
