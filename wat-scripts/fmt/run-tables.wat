;; Probe — print table group counts. A green over zero groups proves nothing.
(:wat::load-file! "rules/defn.wat")
(:wat::load-file! "rules/siblings.wat")
(:wat::load-file! "rules/match.wat")
(:wat::load-file! "rules/if.wat")
(:wat::load-file! "rules/cond.wat")
(:wat::load-file! "rules/let.wat")
(:wat::load-file! "rules/let-blank.wat")
(:wat::load-file! "rules/kwargs.wat")
(:wat::load-file! "rules/table.wat")
(:wat::load-file! "rules/atoms.wat")
(:wat::load-file! "rules/defrecord.wat")

(:wat::core::defn :user::inc-group
  [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
   g <- :wat::core::i64]
  -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::match (:wat::core::get m g)
    (:wat::core::None (:wat::hashmap::assoc m g 1))
    ((:wat::core::Some n) (:wat::hashmap::assoc m g (:wat::i64::+ n 1)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [argv (:wat::runtime::argv)
     path (:wat::core::Option/expect (:wat::core::get argv 2) "usage: wat run-tables.wat <file.wat>")
     src  (:wat::io::read-file path)
     rules (:wat::rete::collect-rules :fmt)]
    (:wat::core::match (:wat::core::read-string-with-comments src)
      ((:wat::core::ReadWithCommentsOutcome::Forms forms comments)
        (:wat::core::let
          [facts   (:wat::grep::facts-of path src)
           rec0    (:wat::grep::facts-as-records facts)
           records (:wat::core::foldl
                     (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])
                                      s   <- :wat::fmt::FormSig]
                       -> (:wat::core::PersistentVector :- [:wat::core::Record])
                       (:wat::vector::conj acc s))
                     rec0
                     (:wat::fmt::form-sigs-of forms))
           queries (:wat::core::PersistentVector :- [:wat::rete::Query]
                     (:wat::fmt::q-break)
                     (:wat::fmt::q-claim)
                     (:wat::fmt::q-fallback)
                     (:wat::fmt::q-blank)
                     (:wat::fmt::q-align)
                     (:wat::fmt::q-table)
                     (:wat::fmt::q-atoms)
                     (:wat::fmt::q-stride)
                     (:wat::fmt::q-empty-vec))]
          (:wat::rete::with-overlay rules queries
            (:wat::core::fn [overlay <- :wat::rete::Overlay]
              -> :wat::core::nil
              (:wat::core::let
                [fired (overlay records)
                 sizes (:wat::core::foldl
                         (:wat::core::fn [m <- (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                                          binding <- :wat::core::PersistentMap]
                           -> (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                           (:wat::core::let [tr (:wat::core::Option/expect
                                                  (:wat::map::get binding "?t")
                                                  "no ?t")]
                             (:user::inc-group m (:wat::fmt::TableRow/group tr))))
                         (:wat::core::HashMap :- [:wat::core::i64 :wat::core::i64])
                         (:wat::rete::query fired (:wat::fmt::q-table)))
                 n2 (:wat::core::foldl
                      (:wat::core::fn [n <- :wat::core::i64  g <- :wat::core::i64] -> :wat::core::i64
                        (:wat::core::let [sz (:wat::core::Option/expect (:wat::core::get sizes g) "sz")]
                          (:wat::core::if (:wat::i64::>= sz 2) (:wat::i64::+ n 1) n)))
                      0
                      (:wat::hashmap::keys sizes))
                 n3 (:wat::core::foldl
                      (:wat::core::fn [n <- :wat::core::i64  g <- :wat::core::i64] -> :wat::core::i64
                        (:wat::core::let [sz (:wat::core::Option/expect (:wat::core::get sizes g) "sz")]
                          (:wat::core::if (:wat::i64::>= sz 3) (:wat::i64::+ n 1) n)))
                      0
                      (:wat::hashmap::keys sizes))
                 rows (:wat::core::length
                         (:wat::rete::query fired (:wat::fmt::q-table)))
                 out (:wat::fmt::emit forms comments
                       (:wat::fmt::breaks-map fired)
                       (:wat::fmt::owned-set fired)
                       (:wat::fmt::blanks-set fired)
                       (:wat::fmt::aligns-set fired)
                       (:wat::fmt::tables-map fired)
                       (:wat::fmt::atoms-set fired)
                       (:wat::fmt::widths-map (:wat::fmt::widths-of forms))
                       (:wat::fmt::strides-map fired)
                       (:wat::fmt::empties-set fired))]
                (:wat::core::do
                  (:wat::kernel::println
                    (:wat::string::interpolate "GROUPS2={a} GROUPS3={b} ROWS={r}"
                      :a (:wat::i64::to-string n2)
                      :b (:wat::i64::to-string n3)
                      :r (:wat::i64::to-string rows)))
                  (:wat::kernel::println out)))))))
      ((:wat::core::ReadWithCommentsOutcome::Malformed cause)
        (:wat::kernel::assertion-failed! (:wat::core::Error/message cause) :wat::core::None :wat::core::None)))))
