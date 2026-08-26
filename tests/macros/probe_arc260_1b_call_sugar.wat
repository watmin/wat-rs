;; tests/macros/probe_arc260_1b_call_sugar.wat — co-located fixture for
;; probe_arc260_1b_call_sugar.rs, slurped via startup_beside(file!()).
;;
;; Merged from KWARGS_SUGAR (connect fn + call-sugar wrappers) and PASCAL_KWARGS
;; (pascal-fn + pascal-kebab wrappers), sharing one :user::main.

(:wat::core::defn :user::connect
  [host <- :wat::core::String
   & [port <- :wat::core::i64  tls <- :wat::core::bool]]
  -> :wat::core::i64
  (:wat::i64::+ port (:wat::core::if tls  1 0)))

;; inline :k v, in order
(:wat::core::defn :user::via-kv [] -> :wat::core::i64
  (:user::connect "h" :port 443 :tls true))

;; inline :k v, OUT OF ORDER — only a true reorder-by-field yields 444
(:wat::core::defn :user::via-kv-reorder [] -> :wat::core::i64
  (:user::connect "h" :tls true :port 443))

;; literal {map}
(:wat::core::defn :user::via-map [] -> :wat::core::i64
  (:user::connect "h" {:port 443 :tls true}))

;; explicit record (the escape hatch — 260.1a; must still work).
;; Arc 294 item 9a: the bundle is built with KWARGS (the bare name is the kwargs macro;
;; the positional prime is generated-code-only). The escape hatch is passing a PRE-BUILT
;; ::Kwargs record instead of using the call sugar — how the record is built is orthogonal.
(:wat::core::defn :user::via-record [] -> :wat::core::i64
  (:user::connect "h" (:user::connect::Kwargs :port 443 :tls true)))

(:wat::core::defn :user::pascal-fn
  [& [FooBar <- :wat::core::i64]]
  -> :wat::core::i64
  FooBar)

;; Wrapper functions that invoke the companion macro at startup (macro expansion time).
(:wat::core::defn :user::via-kv-pascal [] -> :wat::core::i64
  (:user::pascal-fn :foo-bar 42))

(:wat::core::defn :user::via-map-pascal [] -> :wat::core::i64
  (:user::pascal-fn {:foo-bar 99}))

