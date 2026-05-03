;; wat/core.wat — :wat::core::* dispatches.
;;
;; Substrate dispatches that route polymorphic-name primitives to
;; per-Type impls. Per arc 146 DESIGN: one entity-kind (dispatch) for
;; genuinely-polymorphic primitives; per-Type impls live in Rust as
;; clean rank-1 schemes registered in `register_builtins`.
;;
;; Each declaration uses arc 146's `:wat::core::define-dispatch`
;; (slice 1). Loads BEFORE `wat/runtime.wat` so the dispatches are
;; visible to any reflection-driven macro that might reference them.

(:wat::core::define-dispatch :wat::core::length
  ((:wat::core::Vector<T>)    :wat::core::Vector/length)
  ((:wat::core::HashMap<K,V>) :wat::core::HashMap/length)
  ((:wat::core::HashSet<T>)   :wat::core::HashSet/length))

;; Arc 146 slice 3 — bundled migration: empty? / contains? / get / conj.
;; Same shape as length above. contains? uses MIXED IMPL VERBS:
;; HashMap tests KEY membership (`contains-key?`); Vector + HashSet
;; test ELEMENT membership (`contains?`). Caller writes
;; `(:contains? c x)` regardless; dispatch picks the arm by container
;; shape and the impl's verb is internal.
;;
;; get's per-arm return type varies (Vec returns :Option<T>; HashMap
;; returns :Option<V>); infer_dispatch_call returns the matched arm's
;; specific Option<_> type per arc 146 DESIGN.
;;
;; conj is 2-arm only (Vector / HashSet); HashMap doesn't conj —
;; HashMap requires key+value pairing, so :wat::core::assoc is the
;; right verb there (DESIGN audit table).

(:wat::core::define-dispatch :wat::core::empty?
  ((:wat::core::Vector<T>)    :wat::core::Vector/empty?)
  ((:wat::core::HashMap<K,V>) :wat::core::HashMap/empty?)
  ((:wat::core::HashSet<T>)   :wat::core::HashSet/empty?))

(:wat::core::define-dispatch :wat::core::contains?
  ((:wat::core::Vector<T>    :T) :wat::core::Vector/contains?)
  ((:wat::core::HashMap<K,V> :K) :wat::core::HashMap/contains-key?)
  ((:wat::core::HashSet<T>   :T) :wat::core::HashSet/contains?))

(:wat::core::define-dispatch :wat::core::get
  ((:wat::core::Vector<T>    :wat::core::i64) :wat::core::Vector/get)
  ((:wat::core::HashMap<K,V> :K)               :wat::core::HashMap/get))

(:wat::core::define-dispatch :wat::core::conj
  ((:wat::core::Vector<T>  :T) :wat::core::Vector/conj)
  ((:wat::core::HashSet<T> :T) :wat::core::HashSet/conj))

;; Arc 146 slice 4 — :wat::core::* short-name aliases for single-impl
;; ops. Each alias maps a short ergonomic name to its explicit per-Type
;; impl. Per arc 146 DESIGN: single-impl ops are aliases (not
;; dispatches; dispatch is for genuine polymorphism). Both short + long
;; names work; both are honest. The alias machinery (arc 143's
;; :wat::runtime::define-alias) expands at registration time into a
;; delegating user-define whose head copies the target's signature
;; with the alias name substituted.

(:wat::runtime::define-alias :wat::core::assoc   :wat::core::HashMap/assoc)
(:wat::runtime::define-alias :wat::core::dissoc  :wat::core::HashMap/dissoc)
(:wat::runtime::define-alias :wat::core::keys    :wat::core::HashMap/keys)
(:wat::runtime::define-alias :wat::core::values  :wat::core::HashMap/values)
(:wat::runtime::define-alias :wat::core::concat  :wat::core::Vector/concat)
