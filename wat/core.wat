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

;; ─── Arc 148 slice 4 — Numeric arithmetic ────────────────────────────
;;
;; Each of `+`, `-`, `*`, `/` is now a polymorphic surface backed by
;; a binary Dispatch entity routing to per-Type Rust leaves. Three
;; layers per the locked DESIGN § "Arithmetic — three layers":
;;
;;   1. Polymorphic variadic at `:wat::core::<v>` (bare name) — STAYS
;;      as a substrate primitive (Path C of slice 4's BRIEF). Custom
;;      inference (`infer_arithmetic`) is honest substrate; no wat
;;      type expresses "Vector of mixed numerics with f64-promoting
;;      fold." Variadic arity per Lisp/Clojure tradition.
;;
;;   2. Binary Dispatch entity at `:wat::core::<v>,2` — declared
;;      below. 4 arms covering (i64,i64), (f64,f64), (i64,f64),
;;      (f64,i64). Routes to per-Type leaves and mixed leaves.
;;
;;   3. Per-Type Rust binary primitives at `:wat::core::<Type>::<v>,2`
;;      and mixed-type leaves at `:wat::core::<v>,<type1>-<type2>` —
;;      registered in `register_builtins` (src/runtime.rs +
;;      src/check.rs). Reachable per the no-privacy doctrine.
;;
;; Same-type variadic wat fns at `:wat::core::<Type>::<v>` (the bare
;; per-Type name) wrap the per-Type binary leaf via arc 150's variadic
;; define + `:wat::core::foldl` — declared after the dispatches below.

(:wat::core::define-dispatch :wat::core::+,2
  ((:wat::core::i64 :wat::core::i64)  :wat::core::i64::+,2)
  ((:wat::core::f64 :wat::core::f64)  :wat::core::f64::+,2)
  ((:wat::core::i64 :wat::core::f64)  :wat::core::+,i64-f64)
  ((:wat::core::f64 :wat::core::i64)  :wat::core::+,f64-i64))

(:wat::core::define-dispatch :wat::core::-,2
  ((:wat::core::i64 :wat::core::i64)  :wat::core::i64::-,2)
  ((:wat::core::f64 :wat::core::f64)  :wat::core::f64::-,2)
  ((:wat::core::i64 :wat::core::f64)  :wat::core::-,i64-f64)
  ((:wat::core::f64 :wat::core::i64)  :wat::core::-,f64-i64))

(:wat::core::define-dispatch :wat::core::*,2
  ((:wat::core::i64 :wat::core::i64)  :wat::core::i64::*,2)
  ((:wat::core::f64 :wat::core::f64)  :wat::core::f64::*,2)
  ((:wat::core::i64 :wat::core::f64)  :wat::core::*,i64-f64)
  ((:wat::core::f64 :wat::core::i64)  :wat::core::*,f64-i64))

(:wat::core::define-dispatch :wat::core::/,2
  ((:wat::core::i64 :wat::core::i64)  :wat::core::i64::/,2)
  ((:wat::core::f64 :wat::core::f64)  :wat::core::f64::/,2)
  ((:wat::core::i64 :wat::core::f64)  :wat::core::/,i64-f64)
  ((:wat::core::f64 :wat::core::i64)  :wat::core::/,f64-i64))

;; ─── Same-type variadic wat fns (8 total) ─────────────────────────────
;;
;; Per-Type variadic wrappers using arc 150's variadic define syntax.
;; Each folds left over the per-Type binary leaf.
;;
;; Lisp/Clojure arity rules per DESIGN § "Arity rules":
;;   `+`/`*` — 0-ary returns identity; 1-ary returns arg unchanged
;;   `-`/`/` — 0-ary errors via 1-arity-min substrate enforcement;
;;             1-ary inserts identity-on-left (negation/reciprocal)
;;
;; The 0-ary case for `:i64::+`/`:i64::*` is expressed as the foldl
;; seed when the variadic surface receives zero rest args. For
;; `-`/`/`, the 0-ary case is enforced by requiring at least one
;; fixed parameter (the variadic accepts >= 1 arg via the (first
;; rest) convention — see DESIGN § "Variadic semantics").

;; i64 same-type variadic — :+/:*/:- / :/  fold over per-Type binary leaf.

(:wat::core::define
  (:wat::core::i64::+ & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
  (:wat::core::foldl xs 0
    (:wat::core::lambda ((acc :wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
      (:wat::core::i64::+,2 acc x))))

(:wat::core::define
  (:wat::core::i64::* & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
  (:wat::core::foldl xs 1
    (:wat::core::lambda ((acc :wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
      (:wat::core::i64::*,2 acc x))))

;; `:-` and `:/` require >= 1 arg. Express via fixed first param +
;; rest. 1-ary inserts identity-on-left; 2+-ary folds. The arity
;; checker rejects 0-ary via the fixed-param requirement.

(:wat::core::define
  (:wat::core::i64::- (first :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
  (:wat::core::if (:wat::core::Vector/empty? xs) -> :wat::core::i64
    (:wat::core::i64::-,2 0 first)
    (:wat::core::foldl xs first
      (:wat::core::lambda ((acc :wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
        (:wat::core::i64::-,2 acc x)))))

(:wat::core::define
  (:wat::core::i64::/ (first :wat::core::i64) & (xs :wat::core::Vector<wat::core::i64>) -> :wat::core::i64)
  (:wat::core::if (:wat::core::Vector/empty? xs) -> :wat::core::i64
    (:wat::core::i64::/,2 1 first)
    (:wat::core::foldl xs first
      (:wat::core::lambda ((acc :wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
        (:wat::core::i64::/,2 acc x)))))

;; f64 same-type variadic — :+/:*/:- / :/

(:wat::core::define
  (:wat::core::f64::+ & (xs :wat::core::Vector<wat::core::f64>) -> :wat::core::f64)
  (:wat::core::foldl xs 0.0
    (:wat::core::lambda ((acc :wat::core::f64) (x :wat::core::f64) -> :wat::core::f64)
      (:wat::core::f64::+,2 acc x))))

(:wat::core::define
  (:wat::core::f64::* & (xs :wat::core::Vector<wat::core::f64>) -> :wat::core::f64)
  (:wat::core::foldl xs 1.0
    (:wat::core::lambda ((acc :wat::core::f64) (x :wat::core::f64) -> :wat::core::f64)
      (:wat::core::f64::*,2 acc x))))

(:wat::core::define
  (:wat::core::f64::- (first :wat::core::f64) & (xs :wat::core::Vector<wat::core::f64>) -> :wat::core::f64)
  (:wat::core::if (:wat::core::Vector/empty? xs) -> :wat::core::f64
    (:wat::core::f64::-,2 0.0 first)
    (:wat::core::foldl xs first
      (:wat::core::lambda ((acc :wat::core::f64) (x :wat::core::f64) -> :wat::core::f64)
        (:wat::core::f64::-,2 acc x)))))

(:wat::core::define
  (:wat::core::f64::/ (first :wat::core::f64) & (xs :wat::core::Vector<wat::core::f64>) -> :wat::core::f64)
  (:wat::core::if (:wat::core::Vector/empty? xs) -> :wat::core::f64
    (:wat::core::f64::/,2 1.0 first)
    (:wat::core::foldl xs first
      (:wat::core::lambda ((acc :wat::core::f64) (x :wat::core::f64) -> :wat::core::f64)
        (:wat::core::f64::/,2 acc x)))))
