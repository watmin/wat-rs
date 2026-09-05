;; ─── Arc 255 (b0) — how many registry rows carry NO native handler? ──────────────────────────
;;
;; `cond` is a stdlib defmacro and `reduce` is a wat-side defalias; neither has a Rust fn to hang
;; a `#[wat_intrinsic]` on. The question this answers is whether a row can exist WITHOUT one —
;; i.e. whether the registry can already hold a pure declaration whose implementation lives
;; elsewhere. `:wat::intrinsic::Row/has-handler` is the field that knows.
;;
;; ⛔ MEASUREMENT, never a ratchet — same standing rule as `255-registry-census.wat`.

(:wat::core::defn :hh::handlerless? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::not (:wat::intrinsic::Row/has-handler r)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rows (:wat::core::into [] (:wat::intrinsic::rows))
                    hl   (:wat::core::into [] (:wat::core::filter :hh::handlerless? rows))]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat
        "rows with NO native handler: " (:wat::i64::to-string (:wat::core::length hl))))
      (:wat::core::mapv
        (:wat::core::fn [r <- :wat::intrinsic::Row] -> :wat::core::nil
          (:wat::kernel::println (:wat::keyword::to-string (:wat::intrinsic::Row/name r))))
        hl)
      (:wat::kernel::println ""))))
