;; ─── Arc 255 (b0) — what ACTUALLY gates the rete rows, ASKED not assumed ─────────────────────
;;
;; SEQUENCING-the-only-chain-that-gates-the-founding-target.md says of the orphan core_name
;; targets: "Three rows gate twenty-nine." That was written when THREE orphans existed
;; (`Vector`, `cond`, `reduce`); `Vector` was registered later the same day, so the claim is
;; already one row stale. This probe exists because the REST of it needed testing too, and the
;; registry can now be asked instead of grepped.
;;
;; What it prints: every alias row's TARGET. A rete row registered as `alias_of = core_name`
;; would be REFUSED by `no_dangling_or_chained_aliases` if that target is itself an alias
;; (a CHAINED alias). So the set below is what a candidate core_name must NOT be found in.
;;
;; ⛔ MEASUREMENT, never a ratchet — same standing rule as `255-registry-census.wat`: a gate
;; freezes NAMES so it can DISAGREE with the present; one computing both sides always agrees.

(:wat::core::defn :b0::alias? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::match (:wat::intrinsic::Row/alias-of r)
    ((:wat::core::Some _) true) (:wat::core::None false)))

(:wat::core::defn :b0::target [r <- :wat::intrinsic::Row] -> :wat::core::String
  (:wat::core::match (:wat::intrinsic::Row/alias-of r)
    ((:wat::core::Some t) t) (:wat::core::None "")))

(:wat::core::defn :b0::render [r <- :wat::intrinsic::Row] -> :wat::core::String
  (:wat::string::concat
    (:wat::string::concat (:wat::keyword::to-string (:wat::intrinsic::Row/name r)) "  ->  ")
    (:b0::target r)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rows    (:wat::core::into [] (:wat::intrinsic::rows))
                    aliases (:wat::core::into [] (:wat::core::filter :b0::alias? rows))]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat
        "alias rows: " (:wat::i64::to-string (:wat::core::length aliases))))
      (:wat::kernel::println "── every alias row, as `name -> target` ──")
      (:wat::core::mapv
        (:wat::core::fn [r <- :wat::intrinsic::Row] -> :wat::core::nil
          (:wat::kernel::println (:b0::render r)))
        aliases)
      (:wat::kernel::println ""))))
