;; Scratch — CAN a rete `:where` compare an ENUM-typed field against a variant?
;;
;; The RHS side is already settled (277-can-a-rete-rhs-carry-an-enum): a :then may
;; construct a variant. This asks the OTHER side, and it is the one the migration
;; lives on — every fmt rule file dispatches with
;;   (:wat::rete::where (:wat::rete::string::= ?ak "list"))
;; and there are 62 such comparisons on `kind` across 12 rule files.
;;
;; The hazard is named in src/rete/validate.rs: resolve_operand_type ends at
;; rete_type_segment_of, which returns NotComparable for "a real declared type for
;; which rete has NO comparator (a record, a collection, an opaque)". Whether an
;; ENUM has a comparator is exactly what this probe asks.
;;
;; Three arms, so a partial answer is still an answer:
;;   A  equality against a variant CONSTRUCTOR in :where, via core::enum::= (arc 278 #57)
;;   B  pattern position — the variant used as a fact-field pattern
;;   C  the control: the SAME shape with a String, which must pass

(:wat::core::defenum :user::NodeKind :wat::enum::Pure
  :List []
  :Vector []
  :Keyword [])

(:wat::core::defrecord :user::EnumNode
  [id   <- :wat::core::i64
   kind <- :user::NodeKind])

(:wat::core::defrecord :user::StrNode
  [id   <- :wat::core::i64
   kind <- :wat::core::String])

(:wat::core::defrecord :user::HitA [id <- :wat::core::i64])
(:wat::core::defrecord :user::HitC [id <- :wat::core::i64])

;; ARM A — equality against a variant constructor in :where
(:wat::rete::defrule :user::enum-eq-in-where
  :when [(:user::EnumNode (?i <- :id) (?k <- :kind))
         (:wat::rete::where (:wat::rete::core::enum::= ?k (:user::NodeKind::List)))]
  :then [(:user::HitA :id ?i)])

;; ARM C — the CONTROL. Same shape, String. Must pass, or the probe proves nothing.
(:wat::rete::defrule :user::string-eq-in-where
  :when [(:user::StrNode (?i <- :id) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "list"))]
  :then [(:user::HitC :id ?i)])

(:wat::rete::defquery :user::q-HitA :params [] :when [(?fact <- :user::HitA)])
(:wat::rete::defquery :user::q-HitC :params [] :when [(?fact <- :user::HitC)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules    (:wat::rete::collect-rules :user)
     template (:wat::rete::compile-all rules
                (:wat::core::PersistentVector (:user::q-HitA) (:user::q-HitC)))
     fired    (:wat::rete::fire-rules
                (:wat::rete::insert
                  (:wat::rete::insert
                    (:wat::rete::insert
                      (:wat::rete::insert template
                        (:user::EnumNode :id 1 :kind (:user::NodeKind::List)))
                      ;; ★ THE DISCRIMINATOR — a NON-matching variant. If the :where is
                      ;; ignored rather than evaluated, ARM-A counts these too.
                      (:user::EnumNode :id 3 :kind (:user::NodeKind::Vector)))
                    (:user::EnumNode :id 4 :kind (:user::NodeKind::Keyword)))
                  (:user::StrNode :id 2 :kind "list")))]
    (:wat::kernel::println
      (:wat::string::interpolate "ARM-A-enum={a} (MUST be 1 of 3 inserted) ARM-C-string-control={c}"
        :a (:wat::i64::to-string (:wat::core::length (:wat::rete::query fired (:user::q-HitA))))
        :c (:wat::i64::to-string (:wat::core::length (:wat::rete::query fired (:user::q-HitC))))))))
