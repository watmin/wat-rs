;; probe-s3b-astsplice.wat — PROVE the derive-and-splice mechanism (259 S3b Blocker A).
;;
;; Take a concrete work-fn (:my::double, i64->i64), extract its two concrete type AST nodes
;; from the fn-forms output (AST-walk), splice them into a shipped process-runner's
;; self-peer/Peer tuple types via a `(Head :- [args])` type-form quasiquote splice (arc 109
;; "annihilate the angle bracket" retired the earlier keyword-node + string-concat spelling;
;; repaired at 255.73), spawn + drain.
;;
;; EXPECT "6 10".

(:wat::core::defn :my::double [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))

;; typed drain: the param pins the Process I/O (parent sends (idx,I), recvs (idx,O)); I=O=i64.
(:wat::core::defn :probe::drain
  [w <- (:wat::kernel::Process :- [(wat.type/Tuple :- [wat.type/i64 wat.type/i64]) (wat.type/Tuple :- [wat.type/i64 wat.type/i64])])]
  -> wat.type/nil
  (:wat::core::let
    [_ (:wat::core::match (:wat::kernel::send w (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 0 3)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
     _ (:wat::core::match (:wat::kernel::send w (wat.type/Tuple :- [wat.type/i64 wat.type/i64] 1 5)) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
     ra (:wat::kernel::recv w)
     a  (:wat::core::match ra
          [:wat::kernel::RecvOutcome.Message {:msg m} m]
          [:wat::kernel::RecvOutcome.Lost {:cause cause}
            (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
          [:wat::kernel::RecvOutcome.Stopped {}
            (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [:wat::kernel::RecvOutcome.Closed {}
            (:wat::kernel::assertion-failed! :message "recv': w closed unexpectedly")])
     rb (:wat::kernel::recv w)
     b  (:wat::core::match rb
          [:wat::kernel::RecvOutcome.Message {:msg m} m]
          [:wat::kernel::RecvOutcome.Lost {:cause cause}
            (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
          [:wat::kernel::RecvOutcome.Stopped {}
            (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
          [:wat::kernel::RecvOutcome.Closed {}
            (:wat::kernel::assertion-failed! :message "recv': w closed unexpectedly")])]
    (:wat::kernel::println
      (:wat::string::concat
        (:wat::i64::to-string (:wat::core::second a))
        (:wat::string::concat " " (:wat::i64::to-string (:wat::core::second b)))))))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [work-fn  (:wat::core::fn [x <- wat.type/i64] -> wat.type/i64 (:wat::i64::* x 2))
     forms    (:wat::kernel::fn-forms work-fn :probe::__work)
     ;; ── extract the concrete arg/return type keywords off the reified work-fn ──
     def-node (:wat::core::Option/expect (:wat::core::last forms) "no def")
     def-ch   (:wat::core::ast->children def-node)
     fn-form  (:wat::core::nth def-ch 2)
     fn-ch    (:wat::core::ast->children fn-form)
     argspec  (:wat::core::nth fn-ch 1)
     arg-ty   (:wat::core::Option/expect (:wat::core::last (:wat::core::ast->children argspec)) "no argty")
     ret-ty   (:wat::core::nth fn-ch 3)
     ;; arc 109 ③ "annihilate the angle bracket" — angle-bracket type parameters are illegal
     ;; in a NAME, including one built at expand time by string concatenation into a
     ;; keyword-node (the retired `Peer<(i64,I),(i64,O)>` spelling this site used to build).
     ;; `:-` is the one parameterization operator; mint the reference FORM `(Head :- [args])`
     ;; structurally off the raw type-position AST nodes directly — the SAME `~ret-ty`/`~arg-ty`
     ;; splice precedent as `wat/bracket.wat:422/538` (`sp-out`) and 255.72's typed site 1
     ;; above (no `ast-name` / string round-trip at all).
     ;; prn's SEND type = (i64, ret-ty) (the `out` tuple below); prn's RECV type = (i64, arg-ty)
     ;; (the `pair` recv'd below) — same send-first/recv-second slot order as
     ;; `wat/bracket.wat:425`'s `runner-self-kw = (Peer :- [~sp-out ~sp-in])`.
     sp1-node  `(wat.type/Tuple :- [wat.type/i64 ~ret-ty])   ;; S — send type
     sp2-node  `(wat.type/Tuple :- [wat.type/i64 ~arg-ty])   ;; R — recv type
     peer-node `(:wat::kernel::Peer :- [~sp1-node ~sp2-node])
     ;; ── build the shipped runner via quasiquote, splicing the concrete types ──
     ;; `recv` returns `(RecvOutcome :- [T])`, not T directly — this site was never
     ;; type-checked before (execution died inside `peer-node`'s own construction, above), so
     ;; the bare-bind form it originally shipped went unreached. Face the outcome the way
     ;; `:probe::drain` and `wat/bracket.wat`'s `dial-runner` already do: dispatch on the
     ;; RecvOutcome directly (Message processes + recurses, Lost raises, Stopped/Closed exit) —
     ;; not a let-bound unwrap, which would force every arm's result to the same type (the
     ;; Message arm's tuple vs. Stopped/Closed's `nil`).
     runner-def `(:wat::core::defn :probe::__runner
                   [prn <- ~peer-node] -> wat.type/nil
                   (:wat::core::match (:wat::kernel::recv prn)
                     [:wat::kernel::RecvOutcome.Message {:msg pair}
                       (:wat::core::let
                         ;; arc 255 Stone 255.72 (the wall has no exceptions) — splice the
                         ;; probe's own concrete types into the bracket, as
                         ;; wat/bracket.wat:491's `~ret-ty` precedent does. The index is i64 by
                         ;; this runner's own protocol (the Peer's inbound/outbound pairs are
                         ;; `(i64, T)`); `~ret-ty` is the raw type AST already extracted off the
                         ;; reified work-fn's declared return (`ret-ty` above), not re-derived
                         ;; by hand.
                         [out  (wat.type/Tuple :- [wat.type/i64 ~ret-ty] (:wat::core::first pair)
                                                 (:probe::__work (:wat::core::second pair)))
                          _    (:wat::core::match (:wat::kernel::send prn out) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])]
                         (:probe::__runner prn))]
                     [:wat::kernel::RecvOutcome.Lost {:cause cause}
                       (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                     [:wat::kernel::RecvOutcome.Stopped {} nil]
                     [:wat::kernel::RecvOutcome.Closed {} nil]))
     main-def   `(:wat::core::defn :user::main [] -> wat.type/nil
                   (:probe::__runner
                     (:wat::program::self-peer ~sp1-node ~sp2-node)))
     runner-forms (wat.type/Vector :- [wat.type/AST] runner-def main-def)
     w (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::concat forms runner-forms))]
    (:probe::drain w)))
