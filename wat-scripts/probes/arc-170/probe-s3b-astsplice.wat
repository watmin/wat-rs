;; probe-s3b-astsplice.wat — PROVE the derive-and-splice mechanism (259 S3b Blocker A).
;;
;; Take a concrete work-fn (:my::double, i64->i64), extract its two concrete type keywords
;; from the fn-forms output (AST-walk), splice them into a shipped process-runner's
;; self-peer/Peer tuple types via keyword-node + quasiquote, spawn + drain.
;;
;; EXPECT "6 10".

(:wat::core::defn :my::double [n <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* n 2))

;; typed drain: the param pins the Process I/O (parent sends (idx,I), recvs (idx,O)); I=O=i64.
(:wat::core::defn :probe::drain
  [w <- (:wat::kernel::Process :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]
  -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::core::match (:wat::kernel::send w (:wat::core::Tuple 0 3)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     _ (:wat::core::match (:wat::kernel::send w (:wat::core::Tuple 1 5)) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) (:wat::kernel::SendOutcome::Stopped nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     ra (:wat::kernel::recv w)
     a  (:wat::core::match ra
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))
     rb (:wat::kernel::recv w)
     b  (:wat::core::match rb
          ((:wat::kernel::RecvOutcome::Message m) m)
          ((:wat::kernel::RecvOutcome::Lost cause)
            (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Stopped
            (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
          (:wat::kernel::RecvOutcome::Closed
            (:wat::kernel::assertion-failed! "recv': w closed unexpectedly" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))]
    (:wat::kernel::println
      (:wat::string::concat
        (:wat::i64::to-string (:wat::core::second a))
        (:wat::string::concat " " (:wat::i64::to-string (:wat::core::second b)))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [work-fn  (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* x 2))
     forms    (:wat::kernel::fn-forms work-fn :probe::__work)
     ;; ── extract the concrete arg/return type keywords off the reified work-fn ──
     def-node (:wat::core::Option/expect (:wat::core::last forms) "no def")
     def-ch   (:wat::core::ast->children def-node)
     fn-form  (:wat::core::nth def-ch 2)
     fn-ch    (:wat::core::ast->children fn-form)
     argspec  (:wat::core::nth fn-ch 1)
     arg-ty   (:wat::core::Option/expect (:wat::core::last (:wat::core::ast->children argspec)) "no argty")
     ret-ty   (:wat::core::nth fn-ch 3)
     ;; ast-name → ":wat::core::i64"; strip leading colon → "wat::core::i64" for tuple bodies
     arg-nm   (:wat::core::ast-name arg-ty)
     ret-nm   (:wat::core::ast-name ret-ty)
     arg-t    (:wat::string::subs arg-nm 1 (:wat::string::length arg-nm))
     ret-t    (:wat::string::subs ret-nm 1 (:wat::string::length ret-nm))
     ;; ── build the concrete tuple-type keyword nodes ──
     peer-node (:wat::core::keyword-node
                 (:wat::string::concat ":wat::kernel::Peer<(wat::core::i64,"
                   (:wat::string::concat arg-t
                     (:wat::string::concat "),(wat::core::i64,"
                       (:wat::string::concat ret-t ")>")))))
     sp1-node  (:wat::core::keyword-node
                 (:wat::string::concat ":(wat::core::i64,"
                   (:wat::string::concat ret-t ")")))
     sp2-node  (:wat::core::keyword-node
                 (:wat::string::concat ":(wat::core::i64,"
                   (:wat::string::concat arg-t ")")))
     ;; ── build the shipped runner via quasiquote, splicing the concrete types ──
     runner-def `(:wat::core::defn :probe::__runner
                   [prn <- ~peer-node] -> :wat::core::nil
                   (:wat::core::let
                     [pair (:wat::kernel::recv prn)
                      out  (:wat::core::Tuple (:wat::core::first pair)
                                              (:probe::__work (:wat::core::second pair)))
                      _    (:wat::core::match (:wat::kernel::send prn out) (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))]
                     (:probe::__runner prn)))
     main-def   `(:wat::core::defn :user::main [] -> :wat::core::nil
                   (:probe::__runner
                     (:wat::program::self-peer ~sp1-node ~sp2-node)))
     runner-forms (:wat::core::Vector :- [:wat::WatAST] runner-def main-def)
     w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::concat forms runner-forms))]
    (:probe::drain w)))
