
;; 277-does-the-registry-know-slots.wat — CAN THE DEFAULT RULE LEARN SLOTS FROM THE REGISTRY?
;;
;; The builder ruled the ret-spec is ONE LINE, non-negotiable. R11 split it because a form like
;; :wat::core::fn has a GRAMMAR and no RULE. The fork is: give every grammared form its own rule,
;; or teach the DEFAULT to read the grammar the registry already stores -- Row/syntax, the
;; at-syntax string, parsed at src/intrinsic/mod.rs:3002 through the substrate own reader.
;;
;; This asks the registry directly. NOT a grep. R9: QVOD NON ROGATVR, NVMERATVR.

(:wat::core::defn :q::has-syntax? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::not (:wat::core::= (:wat::intrinsic::Row/syntax r) "")))

(:wat::core::defn :q::show [r <- :wat::intrinsic::Row] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::interpolate "{n}   {s}"
    :n (:wat::core::str (:wat::intrinsic::Row/name r))
    :s (:wat::intrinsic::Row/syntax r))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rows (:wat::intrinsic::rows)
     with (:wat::core::into (:wat::core::Vector :- [:wat::intrinsic::Row])
            (:wat::core::filter :q::has-syntax? rows))]
    (:wat::core::do
      (:wat::kernel::println (:wat::string::interpolate
        "registry rows={t}   rows WITH a non-empty syntax={w}"
        :t (:wat::i64::to-string (:wat::core::length rows))
        :w (:wat::i64::to-string (:wat::core::length with))))
      (:wat::core::run! :q::show with))))
