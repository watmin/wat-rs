;; wat-scripts/scratch-pad/arc109-gamma-i-anon-fn-is-rigid.wat — arc 109 γ-i, the DISCONFIRMING probe.
;;
;; γ-i's load-bearing claim is that an ANONYMOUS fn can bind its own type params. This file records
;; what the substrate does TODAY, so the rider mirrors measured behaviour instead of a description.
;;
;; ── RUNG 1 — a def'd generic works, WITHOUT any param list ────────────────────────────────────
;; Stone 251.7 (`src/runtime.rs:3499`) unions every free bare type-var in the fn SIGNATURE into
;; `Function.type_params`, so the `<T>` on the name is already nearly vestigial. Measured: this
;; checks clean AND instantiates at two distinct types.
(:wat::core::defn :user::id [x <- :T] -> :T x)

(:wat::core::defn :user::rung-1-two-instantiations [] -> :wat::core::nil
  (:wat::core::let [_  (:user::id 1)
                    __ (:user::id "s")]
    nil))

;; ── RUNG 2 — the SAME fn, anonymous, is RIGID ─────────────────────────────────────────────────
;; `src/function/eval.rs:66` hardcodes `type_params: Vec::new()` and performs NO union, and
;; `src/function/infer.rs` builds NO SCHEME at all — it binds the params into `body_locals` and
;; checks the body. So a bare-Uppercase `:T` in an anonymous fn is a CONCRETE Path, not a variable.
;;
;; UNCOMMENT to reproduce. Measured 2026-08-21, `target/release/wat --check`:
;;
;;   ⛔ "(value head): parameter #1 expects :T; got :wat::core::i64"
;;
;; (:wat::core::defn :user::rung-2-anon-rigid [] -> :wat::core::nil
;;   (:wat::core::let [f (:wat::core::fn [x <- :T] -> :T x)
;;                     _ (f 1)]
;;     nil))

;; ── RUNG 3 — the binder spelling is not accepted anywhere on a fn/defn yet ────────────────────
;; UNCOMMENT to reproduce. Measured 2026-08-21:
;;
;;   ⛔ "fn signature: expected a vector `[name <- :T ...]` as the args-vector; got keyword"
;;
;; (:wat::core::defn :user::rung-3-binder :- [T] [x <- :T] -> :T x)

;; ── RUNG 4 — the CONTROL that must keep passing ───────────────────────────────────────────────
;; An anonymous fn with CONCRETE types, handed to a generic HOF, already works. γ-i must not
;; disturb it — if this ever goes red, the change reached further than its own surface.
(:wat::core::defn :user::app :- [T] [f <- [T :-> T] x <- :T] -> :T (:wat::core::apply f [x]))

(:wat::core::defn :user::rung-4-control [] -> :wat::core::nil
  (:wat::core::let [_  (:user::app (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x) 1)
                    __ (:user::app (:wat::core::fn [s <- :wat::core::String] -> :wat::core::String s) "s")]
    nil))
