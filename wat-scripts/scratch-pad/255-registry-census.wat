;; ─── Arc 255 — the campaign census, ASKED not counted ────────────────────────────────────────
;;
;; Every number here comes from `(:wat::intrinsic::rows)` — the registry answering about itself.
;; Before the stone "the registry can be enumerated", each of these was a grep against Rust
;; source, and this session published three of them wrong (30 for 35, 10/42 for 32/75, 46 for 37).
;; `[[R9 (k)Now F(orever)]]` — QVOD NON ROGATVR, NVMERATVR; QVOD NVMERATVR, MENTITVR.
;;
;; ⛔ This is a MEASUREMENT, never a ratchet. It must not be used to derive
;; REGISTRY_MEMBERSHIP_GAP_A/GAP_B, FROZEN_CHECKER_DEBT_LEDGER or FROZEN_TYPES_UNCHECKED: a gate
;; freezes NAMES so it can DISAGREE with the present, and one that computes both sides always
;; agrees with itself. This is what those frozen lists are compared AGAINST.
;;
;; ⚠ Only `Totality` and `ExpandTime` carry an `Unreviewed` pole. `Purity`, `Determinism` and
;; `Category` have none — they are complete BY CONSTRUCTION, every row graded. The backlog is
;; exactly two axes, not five. (Measured: the checker refuses `Purity::Unreviewed` as a variant.)

(:wat::core::defn :census::totality-unreviewed? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/totality r) :wat::runtime::Totality::Unreviewed))
(:wat::core::defn :census::totality-partial? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/totality r) :wat::runtime::Totality::Partial))
(:wat::core::defn :census::expand-unreviewed? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/expand-time r) :wat::runtime::ExpandTime::Unreviewed))
(:wat::core::defn :census::both-unreviewed? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::and (:census::totality-unreviewed? r) (:census::expand-unreviewed? r)))
(:wat::core::defn :census::alias? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::match (:wat::intrinsic::Row/alias-of r)
    ((:wat::core::Some _) true) (:wat::core::None false)))
(:wat::core::defn :census::variadic? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::i64::= (:wat::intrinsic::Row/arity r) -1))
(:wat::core::defn :census::no-syntax? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::string::empty? (:wat::intrinsic::Row/syntax r)))
(:wat::core::defn :census::special-form? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::= (:wat::intrinsic::Row/kind r) :wat::runtime::Kind::SpecialForm))

(:wat::core::defn :census::count
  [rows <- (:wat::core::Vector :- [:wat::intrinsic::Row])
   p    <- [:wat::intrinsic::Row :-> :wat::core::bool]] -> :wat::core::String
  (:wat::i64::to-string
    (:wat::core::length (:wat::core::into [] (:wat::core::filter p rows)))))

(:wat::core::defn :census::line
  [label <- :wat::core::String
   rows  <- (:wat::core::Vector :- [:wat::intrinsic::Row])
   p     <- [:wat::intrinsic::Row :-> :wat::core::bool]] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat label (:census::count rows p))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [rows (:wat::core::into [] (:wat::intrinsic::rows))]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat
        "registry rows ................ " (:wat::i64::to-string (:wat::core::length rows))))
      (:census::line "Kind::SpecialForm ............ " rows :census::special-form?)
      (:census::line "alias rows ................... " rows :census::alias?)
      (:census::line "arity Variadic (-1) .......... " rows :census::variadic?)
      (:census::line "no @syntax ................... " rows :census::no-syntax?)
      (:wat::kernel::println "")
      (:wat::kernel::println "── the grading backlog — TWO axes have an Unreviewed pole, three do not ──")
      (:census::line "totality  Partial (WORK LIST)  " rows :census::totality-partial?)
      (:census::line "totality  Unreviewed ......... " rows :census::totality-unreviewed?)
      (:census::line "expand    Unreviewed ......... " rows :census::expand-unreviewed?)
      (:census::line "BOTH unreviewed (one pass) ... " rows :census::both-unreviewed?))))
