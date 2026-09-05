
;; Does :wat::core::defn have a registry row, and does it carry a grammar?
;; The 36-row at-syntax census does not list it. This asks whether it is ABSENT from the registry
;; or PRESENT with an empty syntax -- a different answer, and a different fix.

(:wat::core::defn :q::interesting? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::let [n (:wat::core::str (:wat::intrinsic::Row/name r))]
    (:wat::core::if (:wat::core::= n ":wat.core/defn") true
      (:wat::core::if (:wat::core::= n ":wat.core/defrecord") true
        (:wat::core::if (:wat::core::= n ":wat.core/defstruct") true
          (:wat::core::if (:wat::core::= n ":wat.test/deftest") true (:wat::core::= n ":wat.core/fn")))))))

(:wat::core::defn :q::show [r <- :wat::intrinsic::Row] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::interpolate "{n}  kind={k}  syntax=[{s}]"
    :n (:wat::core::str (:wat::intrinsic::Row/name r))
    :k (:wat::core::str (:wat::intrinsic::Row/kind r))
    :s (:wat::intrinsic::Row/syntax r))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [hits (:wat::core::into (:wat::core::Vector :- [:wat::intrinsic::Row])
            (:wat::core::filter :q::interesting? (:wat::intrinsic::rows)))]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::concat "matched rows: "
        (:wat::i64::to-string (:wat::core::length hits))))
      (:wat::core::run! :q::show hits))))
