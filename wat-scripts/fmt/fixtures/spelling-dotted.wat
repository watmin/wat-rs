;; Same form as spelling-fqdn.wat and spelling-clojure.wat. ≥2 defn args, ≥2 let binders.
;; 255.95: `:wat.core/defn` is the printer of `:wat::core::defn`. The checker
;; accepts that call head, so this program is legal. The formatter still rules it.
(:wat.core/defn :fix::two
  [a <- wat.type/i64
   b <- wat.type/i64]
  -> wat.type/i64
  (:wat.core/let
    [x a
     y b]
    x))
