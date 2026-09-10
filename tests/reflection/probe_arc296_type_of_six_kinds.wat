;; Co-located fixture for probe_arc296_reflection_answers_for_every_type_kind.rs
;; — one declared type per TypeDef kind, plus a 2-field variant whose field
;; names are printed in declaration order (the fact the match-arm codemod
;; could not ask).
;;
;; stdout, one fact per line:
;;   Aggregate / Enum / Newtype / Alias / Union / Surface
;;   :left / :right          (Pair's declared fields, declaration order)
;;   Record                  (Rec's nature)
;;   :alpha                  (field-names-of Rec — not retired)
;;   T                       (Option's type-params)

(:wat::core::defrecord :probe::Rec [alpha <- :wat::core::i64])

(:wat::core::defenum :probe::Box :wat::enum::Pure
  :Full [payload <- :wat::core::i64]
  :Pair [left <- :wat::core::i64  right <- :wat::core::String]
  :Empty [])

(:wat::core::newtype :probe::Count :wat::core::i64)

(:wat::core::typealias :probe::Alias :wat::core::i64)

(:wat::core::typeunion :probe::Num [:wat::core::i64 :wat::core::f64])

(:wat::core::defsurface :probe::Surf
  :nature :wat::core::Record
  :features [message <- :wat::core::String])

(:wat::core::defn :user::print-kind [info <- :wat::runtime::TypeInfo] -> :wat::core::nil
  (:wat::core::match (:wat::runtime::TypeInfo/kind info)
    [:wat::runtime::TypeKind.Aggregate {} (:wat::kernel::println "Aggregate")]
    [:wat::runtime::TypeKind.Enum {} (:wat::kernel::println "Enum")]
    [:wat::runtime::TypeKind.Newtype {} (:wat::kernel::println "Newtype")]
    [:wat::runtime::TypeKind.Alias {} (:wat::kernel::println "Alias")]
    [:wat::runtime::TypeKind.Union {} (:wat::kernel::println "Union")]
    [:wat::runtime::TypeKind.Surface {} (:wat::kernel::println "Surface")]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::print-kind (:wat::runtime::type-of :probe::Rec))
  (:user::print-kind (:wat::runtime::type-of :probe::Box))
  (:user::print-kind (:wat::runtime::type-of :probe::Count))
  (:user::print-kind (:wat::runtime::type-of :probe::Alias))
  (:user::print-kind (:wat::runtime::type-of :probe::Num))
  (:user::print-kind (:wat::runtime::type-of :probe::Surf))
  (:wat::core::match (:wat::runtime::TypeInfo/body (:wat::runtime::type-of :probe::Box))
    [:wat::runtime::TypeBody.Enum {:purity _ :variants vs}
      (:wat::core::let
        [pair (:wat::core::nth vs 1)
         fs   (:wat::runtime::TypeVariant/fields pair)]
        (:wat::kernel::println
          (:wat::core::str (:wat::runtime::TypeField/name (:wat::core::first fs))))
        (:wat::kernel::println
          (:wat::core::str (:wat::runtime::TypeField/name (:wat::core::second fs)))))]
    [_ (:wat::kernel::println "not-enum")])
  (:wat::core::match (:wat::runtime::TypeInfo/body (:wat::runtime::type-of :probe::Rec))
    [:wat::runtime::TypeBody.Aggregate {:nature n :fields _}
      (:wat::core::match n
        [:wat::runtime::TypeNature.Struct {} (:wat::kernel::println "Struct")]
        [:wat::runtime::TypeNature.Record {} (:wat::kernel::println "Record")]
        [:wat::runtime::TypeNature.HolonRecord {} (:wat::kernel::println "HolonRecord")]
        [:wat::runtime::TypeNature.Peer {} (:wat::kernel::println "Peer")])]
    [_ (:wat::kernel::println "not-aggregate")])
  (:wat::kernel::println
    (:wat::core::str (:wat::core::first (:wat::runtime::field-names-of :probe::Rec))))
  (:wat::kernel::println
    (:wat::core::first (:wat::runtime::TypeInfo/type-params
                         (:wat::runtime::type-of :wat::core::Option)))))
