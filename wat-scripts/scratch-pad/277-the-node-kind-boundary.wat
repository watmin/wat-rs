;; Scratch — the String -> enum BOUNDARY that Break.kind did not have.
;;
;; :wat::core::ast-kind is a RUST intrinsic returning :wat::core::String (14 arms, one
;; per WatAST variant, src/edn/render.rs). So the map at wat/grep.wat:194 cannot be
;; exhaustive on its INPUT: this measures what that costs, and proves the inverse that
;; a drift gate would walk.
;;
;; MEASURED HERE, and it decides the naming: a variant named :String is REFUSED —
;;   "bare primitive type ':String' is retired (arc 109 slice 1c)"
;; — the only one of the 14 short names that fails. Mirroring WatAST's variant names
;; sidesteps it AND makes the Rust<->wat correspondence readable by inspection.

(:wat::core::defenum :user::NodeKind :wat::enum::Pure
  :IntLit []
  :FloatLit []
  :RationalLit []
  :BigIntLit []
  :CharLit []
  :BoolLit []
  :StringLit []
  :NilLit []
  :Keyword []
  :Symbol []
  :List []
  :Vector []
  :Set []
  :Map [])

(:wat::core::defn :user::kind-of
  [s <- :wat::core::String]
  -> :user::NodeKind
  (:wat::core::cond
    ((:wat::core::= s "int") (:user::NodeKind::IntLit))
    ((:wat::core::= s "float") (:user::NodeKind::FloatLit))
    ((:wat::core::= s "rational") (:user::NodeKind::RationalLit))
    ((:wat::core::= s "bigint") (:user::NodeKind::BigIntLit))
    ((:wat::core::= s "char") (:user::NodeKind::CharLit))
    ((:wat::core::= s "bool") (:user::NodeKind::BoolLit))
    ((:wat::core::= s "string") (:user::NodeKind::StringLit))
    ((:wat::core::= s "nil") (:user::NodeKind::NilLit))
    ((:wat::core::= s "keyword") (:user::NodeKind::Keyword))
    ((:wat::core::= s "symbol") (:user::NodeKind::Symbol))
    ((:wat::core::= s "list") (:user::NodeKind::List))
    ((:wat::core::= s "vector") (:user::NodeKind::Vector))
    ((:wat::core::= s "set") (:user::NodeKind::Set))
    ((:wat::core::= s "map") (:user::NodeKind::Map))
    (:else (:wat::kernel::assertion-failed! :message (:wat::string::concat "grep: unknown ast-kind " s)))))

(:wat::core::defn :user::kind-name
  [k <- :user::NodeKind]
  -> :wat::core::String
  (:wat::core::match k
    [:user::NodeKind::IntLit {} "int"]
    [:user::NodeKind::FloatLit {} "float"]
    [:user::NodeKind::RationalLit {} "rational"]
    [:user::NodeKind::BigIntLit {} "bigint"]
    [:user::NodeKind::CharLit {} "char"]
    [:user::NodeKind::BoolLit {} "bool"]
    [:user::NodeKind::StringLit {} "string"]
    [:user::NodeKind::NilLit {} "nil"]
    [:user::NodeKind::Keyword {} "keyword"]
    [:user::NodeKind::Symbol {} "symbol"]
    [:user::NodeKind::List {} "list"]
    [:user::NodeKind::Vector {} "vector"]
    [:user::NodeKind::Set {} "set"]
    [:user::NodeKind::Map {} "map"]
))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::interpolate "roundtrip={r} same={s} cross={c}"
      :r (:user::kind-name (:user::kind-of "list"))
      :s (:wat::core::if (:wat::core::= (:user::kind-of "map") (:user::NodeKind::Map)) "true" "false")
      :c (:wat::core::if (:wat::core::= (:user::kind-of "map") (:user::NodeKind::Set)) "true" "false"))))
