;; ─── Arc 255 — do any alias rows DISAGREE with their target's arity? ─────────────────────────
;;
;; The fold-time resolution pass (src/intrinsic/mod.rs:726) copies exactly FIVE fields onto an
;; alias row — purity, determinism, totality, expand-time, category. It does NOT copy args, ret
;; or arity. So every alias's signature is its own hand-restated declaration, and nothing
;; compares it to the target's. That is the shape rete_alias.rs's own header forbids for axes
;; ("not restated here where they could disagree") applied to axes but never to the signature.
;;
;; This prints `name|arity|target` for every alias row so the pairs can be joined against
;; `name|arity` for all rows and any drift NAMED rather than assumed absent.
;;
;; ⛔ MEASUREMENT, never a ratchet — same standing rule as `255-registry-census.wat`.

(:wat::core::defn :drift::alias? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::match (:wat::intrinsic::Row/alias-of r)
    ((:wat::core::Some _) true) (:wat::core::None false)))

(:wat::core::defn :drift::render [r <- :wat::intrinsic::Row] -> :wat::core::String
  (:wat::string::concat
    (:wat::string::concat
      (:wat::string::concat (:wat::keyword::to-string (:wat::intrinsic::Row/name r)) "|")
      (:wat::string::concat (:wat::i64::to-string (:wat::intrinsic::Row/arity r)) "|"))
    (:wat::core::match (:wat::intrinsic::Row/alias-of r)
      ((:wat::core::Some t) t) (:wat::core::None ""))))

(:wat::core::defn :drift::plain [r <- :wat::intrinsic::Row] -> :wat::core::String
  (:wat::string::concat
    (:wat::string::concat (:wat::keyword::to-string (:wat::intrinsic::Row/name r)) "@")
    (:wat::i64::to-string (:wat::intrinsic::Row/arity r))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rows    (:wat::core::into [] (:wat::intrinsic::rows))
                    aliases (:wat::core::into [] (:wat::core::filter :drift::alias? rows))]
    (:wat::core::do
      (:wat::core::mapv
        (:wat::core::fn [r <- :wat::intrinsic::Row] -> :wat::core::nil
          (:wat::kernel::println (:wat::string::concat "ALIAS " (:drift::render r))))
        aliases)
      (:wat::core::mapv
        (:wat::core::fn [r <- :wat::intrinsic::Row] -> :wat::core::nil
          (:wat::kernel::println (:wat::string::concat "ROW " (:drift::plain r))))
        rows)
      (:wat::core::nil))))
