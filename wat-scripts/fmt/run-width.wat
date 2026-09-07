;; Width control: for a single-line form, derived Width MUST equal span width,
;; excluding reader-synthesized nodes (Named and not Written). Prints CHECKED
;; beside MISMATCH so a vacuous 0/0 cannot hide.

(:wat::core::defrecord :user::WC
  [checked  <- :wat::core::i64
   mismatch <- :wat::core::i64])

(:wat::core::defn :user::ids-of
  [named <- (:wat::core::PersistentVector :- [:wat::grep::Named])]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     n <- :wat::grep::Named]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::hashmap::assoc m (:wat::grep::Named/id n) true))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    named))

(:wat::core::defn :user::written-ids
  [ws <- (:wat::core::PersistentVector :- [:wat::grep::Written])]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                     w <- :wat::grep::Written]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      (:wat::hashmap::assoc m (:wat::grep::Written/id w) true))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
    ws))

(:wat::core::defn :user::synthesized?
  [id     <- :wat::core::i64
   named  <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   written <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])]
  -> :wat::core::bool
  (:wat::core::match (:wat::core::get named id)
    [:wat::core::None {} false]
    [:wat::core::Some {:value _}
      (:wat::core::match (:wat::core::get written id)
        [:wat::core::Some {:value _} false]
        [:wat::core::None {} true])]))

(:wat::core::defn :user::parents-of
  [nodes <- (:wat::core::PersistentVector :- [:wat::grep::Node])]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::foldl
    (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                     n <- :wat::grep::Node]
      -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
      (:wat::hashmap::assoc m (:wat::grep::Node/id n) (:wat::grep::Node/parent n)))
    (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
    nodes))

(:wat::core::defn :user::mark-up
  [tainted <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   parents <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   id      <- :wat::core::i64]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::if (:wat::i64::<= id 0)
    tainted
    (:wat::core::match (:wat::core::get tainted id)
      [:wat::core::Some {:value _} tainted]
      [:wat::core::None {}
        (:wat::core::let [t2 (:wat::hashmap::assoc tainted id true)
                          p  (:wat::core::match (:wat::core::get parents id)
                                [:wat::core::None {} 0]
                                [:wat::core::Some {:value x} x])]
          (:user::mark-up t2 parents p))])))

(:wat::core::defn :user::taint-from
  [nodes   <- (:wat::core::PersistentVector :- [:wat::grep::Node])
   named   <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   written <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
  (:wat::core::let [parents (:user::parents-of nodes)]
    (:wat::core::foldl
      (:wat::core::fn [t <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
                       n <- :wat::grep::Node]
        -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
        (:wat::core::if (:user::synthesized? (:wat::grep::Node/id n) named written)
          (:user::mark-up t parents (:wat::grep::Node/id n))
          t))
      (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
      nodes)))

(:wat::core::defn :user::check-span
  [acc     <- :user::WC
   sp      <- :wat::grep::Span
   tainted <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::bool])
   widths  <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   path    <- :wat::core::String]
  -> :user::WC
  (:wat::core::if (:wat::core::not (:wat::core::= (:wat::grep::Span/line sp) (:wat::grep::Span/end-line sp)))
    acc
    (:wat::core::match (:wat::core::get tainted (:wat::grep::Span/id sp))
      [:wat::core::Some {:value _} acc]
      [:wat::core::None {}
      (:wat::core::let
        [id     (:wat::grep::Span/id sp)
         actual (:wat::i64::- (:wat::grep::Span/end-col sp) (:wat::grep::Span/col sp))]
        (:wat::core::match (:wat::core::get widths id)
          [:wat::core::None {}
            (:wat::core::do
              (:wat::kernel::println
                (:wat::string::interpolate
                  "MISMATCH {p}:{l} id={i} derived=MISSING actual={a}"
                  :p path
                  :l (:wat::i64::to-string (:wat::grep::Span/line sp))
                  :i (:wat::i64::to-string id)
                  :a (:wat::i64::to-string actual)))
              (:user::WC :checked (:wat::i64::+ (:user::WC/checked acc) 1)
                          :mismatch (:wat::i64::+ (:user::WC/mismatch acc) 1)))]
          [:wat::core::Some {:value derived}
            (:wat::core::if (:wat::core::= derived actual)
              (:user::WC :checked (:wat::i64::+ (:user::WC/checked acc) 1) :mismatch (:user::WC/mismatch acc))
              (:wat::core::do
                (:wat::kernel::println
                  (:wat::string::interpolate
                    "MISMATCH {p}:{l} id={i} derived={d} actual={a}"
                    :p path
                    :l (:wat::i64::to-string (:wat::grep::Span/line sp))
                    :i (:wat::i64::to-string id)
                    :d (:wat::i64::to-string derived)
                    :a (:wat::i64::to-string actual)))
                (:user::WC :checked (:wat::i64::+ (:user::WC/checked acc) 1)
                            :mismatch (:wat::i64::+ (:user::WC/mismatch acc) 1))))]))])))

(:wat::core::defn :user::report [path <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let
    [src   (:wat::io::read-file path)
     facts (:wat::grep::facts-of path src)]
    (:wat::core::match (:wat::core::read-string src)
      [:wat::core::ReadOutcome::Forms {:forms forms}
        (:wat::core::let
          [named   (:user::ids-of (:wat::grep::Facts/named facts))
           written (:user::written-ids (:wat::grep::Facts/written facts))
           tainted (:user::taint-from (:wat::grep::Facts/nodes facts) named written)
           widths  (:wat::fmt::widths-map (:wat::fmt::widths-of forms))
           wc      (:wat::core::foldl
                     (:wat::core::fn [acc <- :user::WC  sp <- :wat::grep::Span] -> :user::WC
                       (:user::check-span acc sp tainted widths path))
                     (:user::WC :checked 0 :mismatch 0)
                     (:wat::grep::Facts/spans facts))]
          (:wat::kernel::println
            (:wat::string::interpolate
              "{p}  single-line forms CHECKED={c}  MISMATCH={b}"
              :p path
              :c (:wat::i64::to-string (:user::WC/checked wc))
              :b (:wat::i64::to-string (:user::WC/mismatch wc)))))]
      [:wat::core::ReadOutcome::Malformed {:cause c}
        (:wat::kernel::assertion-failed! (:wat::core::Error/message c) :wat::core::None :wat::core::None)])))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [argv (:wat::runtime::argv)]
    (:wat::core::match (:wat::core::get argv 2)
      [:wat::core::Some {:value path} (:user::report path)]
      [:wat::core::None {}
        (:wat::core::do
          (:user::report "wat/io.wat")
          (:user::report "wat/grep.wat")
          (:user::report "wat/fix.wat")
          (:user::report "wat/core.wat")
          (:user::report "wat/service.wat")
          (:user::report "wat/rete.wat"))])))
