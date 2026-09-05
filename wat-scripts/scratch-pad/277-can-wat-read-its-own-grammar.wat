
;; 277-can-wat-read-its-own-grammar.wat — CAN read-string EAT THE at-syntax STRINGS?
;;
;; Builder ruled (b): the DEFAULT layout rule learns slots from the registry Row/syntax rather
;; than 36 hand-written rule files restating a grammar the registry already holds. That rests on
;; one unmeasured assumption -- that a grammar string is itself readable wat source, so wat can
;; read wat own declared grammar the way wat-fix and wat-grep read wat source.
;;
;; The strings carry PLACEHOLDERS the language never sees in real code:
;;   <param>   :T   ...   <body>+   <exprs>+   :V1
;; If read-string refuses any of them, (b) needs a tolerant reader and gets more expensive.
;;
;; This tries ALL 36. Not one. R9: QVOD NON ROGATVR, NVMERATVR.

(:wat::core::defn :g::has-syntax? [r <- :wat::intrinsic::Row] -> :wat::core::bool
  (:wat::core::not (:wat::core::= (:wat::intrinsic::Row/syntax r) "")))

(:wat::core::defn :g::try [r <- :wat::intrinsic::Row] -> :wat::core::i64
  (:wat::core::match (:wat::core::read-string (:wat::intrinsic::Row/syntax r))
    ((:wat::core::ReadOutcome::Forms forms)
      (:wat::core::let [kids (:wat::core::ast->children forms)]
        (:wat::core::do
          (:wat::kernel::println (:wat::string::interpolate "  OK   {n}  top-forms={k}"
            :n (:wat::core::str (:wat::intrinsic::Row/name r))
            :k (:wat::i64::to-string (:wat::core::length kids))))
          0)))
    ((:wat::core::ReadOutcome::Malformed cause)
      (:wat::core::do
        (:wat::kernel::println (:wat::string::interpolate "  FAIL {n}  {m}"
          :n (:wat::core::str (:wat::intrinsic::Row/name r))
          :m (:wat::core::Error/message cause)))
        1))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [with (:wat::core::into (:wat::core::Vector :- [:wat::intrinsic::Row])
            (:wat::core::filter :g::has-syntax? (:wat::intrinsic::rows)))
     bad  (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::i64 r <- :wat::intrinsic::Row] -> :wat::core::i64
              (:wat::core::+ acc (:g::try r)))
            0 with)]
    (:wat::kernel::println (:wat::string::interpolate
      "GRAMMARS={t}   UNREADABLE={b}" 
      :t (:wat::i64::to-string (:wat::core::length with))
      :b (:wat::i64::to-string bad)))))
