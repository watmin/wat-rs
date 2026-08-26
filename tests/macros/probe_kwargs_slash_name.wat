;; tests/macros/probe_kwargs_slash_name.wat — co-located fixture for
;; probe_kwargs_slash_name.rs, slurped via startup_beside(file!()).
;;
;; A /-named kwargs fn (mirrors a defservice worker/start shape) + wrapper fns.
(:wat::core::defn :t::worker/start
  [& [count <- :wat::core::i64  step <- :wat::core::i64]]
  -> :wat::core::i64
  (:wat::i64::+ count step))

;; inline :k v, in order
(:wat::core::defn :t::via-kv [] -> :wat::core::i64
  (:t::worker/start :count 40 :step 2))

;; inline :k v, OUT OF ORDER — only a true reorder-by-field yields 42
(:wat::core::defn :t::via-kv-reorder [] -> :wat::core::i64
  (:t::worker/start :step 2 :count 40))

;; literal {map}
(:wat::core::defn :t::via-map [] -> :wat::core::i64
  (:t::worker/start {:count 40 :step 2}))

