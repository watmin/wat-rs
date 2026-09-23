;; Arc 251.8d-ii (EIGHTH DRAW) — the macro member join, POSITIVE rows.
;;
;; Row CURE: a `defmacro` whose name carries a `Type/member` join, declared in the
;; FAITHFUL-CLOJURE surface (what `to-faithful-clojure.wat` emits) and called in the
;; KEYWORD surface. The faithful name `user.Box/of` registers through `ns_to_wat_path`
;; as `:user::Box::of`; the caller asks for `:user::Box/of`. A `defmacro` is registered
;; and expanded before the TypeEnv exists, so `rekey_type_member_functions` cannot
;; restore the join — the keyword arm of the macro-call dispatch has to ask the other
;; join itself.
;;
;; The CONTROL row lives in `probe_arc251_8d_macro_member_join_control.wat`: this file
;; does not FREEZE pre-cure, so a control sharing it would be unreadable on the binary
;; it exists to measure.
(:wat::core::defrecord :user::Box [n <- :wat::core::i64])

(wat.core/defmacro user.Box/of [n :- wat/WatAST] :- wat/WatAST
  `(:user::Box :n ~n))

(:wat::core::defn :user::cure [] -> :wat::core::i64
  (:user::Box/n (:user::Box/of 7)))
