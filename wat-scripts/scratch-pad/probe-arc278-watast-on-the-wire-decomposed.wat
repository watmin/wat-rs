;; PROBE — DECOMPOSITION. `probe-arc278-rules-cross-the-wire.wat` came back
;; `LOST disconnected` on BOTH arms, which is the mechanism failing rather than the
;; differential firing. Two variables were braided in that probe:
;;   (a) the service shape itself (start/connect/op round-trip on a process locus), and
;;   (b) a `(Vector :- [WatAST])` as a request field crossing the pipe.
;;
;; This file separates them in ONE service with TWO ops that differ ONLY in the request
;; field's type. Both replies are a plain i64, so the RESPONSE side is identical and cannot
;; be the variable.
;;
;;   echo    [n <- i64]                  -> Ok n            ← the CONTROL: is my shape right?
;;   count   [defs <- (Vector :- [WatAST])]    -> Ok (length defs) ← the SUBJECT: does WatAST cross?
;;
;; READ:
;;   echo Ok  AND count Ok      => the wire carries WatAST; the earlier failure is in the
;;                                 service BODY (eval-with-defs!/rete), not the boundary.
;;   echo Ok  AND count LOST    => WatAST cannot cross a service boundary. That is a substrate
;;                                 finding and it blocks the declared-payload wire shape.
;;   echo LOST                  => my service shape is wrong and this file has proven NOTHING
;;                                 about WatAST. Fix the shape before reading the subject.
;;
;; No `eval-with-defs!`, no rete, no payload semantics — deliberately. The only question here
;; is whether the VALUE survives encode → pipe → decode.
;;
;; ⛔ MEASURED 2026-08-12 — verbatim:
;;   "CONTROL echo(i64)        => Ok n=7"
;;   "SUBJECT count(Vec<WatAST>) => LOST disconnected"
;;
;; ⛔⛔ THE ABOVE READING WAS WRONG AND IS SUPERSEDED BY THE NEXT MEASUREMENT. Kept visible
;; rather than deleted, because the wrong turn is the lesson: from `LOST disconnected` alone I
;; concluded "WatAST does not cross" and was one step from routing around it by shipping TEXT.
;; The builder refused the premise — *"why is (Vector WatAST) a problem?"* — and he was right.
;;
;; ★ MEASURED 2026-08-12, adding a THREAD-locus arm (same op, same value; a thread hands values
;; in-process, a process EDN-encodes through a pipe) AND printing the RequestMalformed fields
;; this probe had been discarding as `_p _e _g`:
;;
;;   "CONTROL echo(i64)        => Ok n=7"
;;   "SUBJECT count(Vec<WatAST>) => LOST disconnected"                    (process locus)
;;   "ISOLATOR count THREAD      => REQUEST-MALFORMED expected=:wat::WatAST got=List"
;;   ["defs" "[0]"]
;;
;; ★ FINDING 1 — IT IS A DECODE-VALIDATION BUG, NOT A TRANSPORT LIMIT. The frame ARRIVED and the
;; validator walked into element 0 of `defs`, so the value crossed. It then compared the decoded
;; shape's kind name "List" against the declared type name ":wat::WatAST" and refused. But a
;; List IS a WatAST — `WatAST::List` is one of its variants. This is CLAUDE.md's own recorded
;; corollary, fourth instance in arc 278: *"when a generic form misbehaves, suspect a string
;; comparison with one side normalized and the other not before suspecting the type system."*
;; Forms ARE EDN and DO ship (wat/repl.wat:20-22). Nothing here justifies a text workaround.
;;
;; ★ FINDING 2, separate and arguably larger — THE SAME CONDITION IS FACED ON ONE LOCUS AND LOST
;; ON THE OTHER. Thread: a named `RequestMalformed` carrying path/expected/got. Process: a bare
;; `LOST disconnected` with the cause gone. One locus faces the failure as a value; the other
;; destroys it. That is the no-hidden-failures class (R53/R57) surviving at a LOCUS boundary, and
;; it is why this took three runs to diagnose instead of one.
;;
;; ⚠ STILL NOT DETERMINED: where exactly the name comparison lives, and whether a BARE `WatAST`
;; field fails the same way as `(Vector :- [WatAST])`. Do not guess either.
;;
;; ★ MEASURED 2026-08-12, after the identity arm (`edn_shim::edn_to_typed_value_inner`,
;; `TypeExpr::Path` arm, `:wat::WatAST` case — DESIGN-STONE-watast-is-the-wire.md):
;;
;;   "CONTROL echo(i64)        => Ok n=7"
;;   "SUBJECT count(Vec<WatAST>) => LOST disconnected"      (process locus — UNCHANGED)
;;   "ISOLATOR count THREAD      => Ok n=3"                 (was REQUEST-MALFORMED — NOW FIXED)
;;
;; FINDING 1 (predicted) CONFIRMED: the identity arm closes the walker for both loci equally —
;; `count THREAD` now decodes/validates a `(Vector :- [WatAST])` correctly. Companion probe
;; `probe-arc278-watast-identity-arm.wat` additionally confirms the BARE case (not only
;; parametric) and the negative row (a wrong field type is still refused).
;;
;; FINDING 3, NEW and NOT predicted by the design stone — the process arm is STILL red, and
;; it is NOT a decode/validate defect: `:wat::edn::validate` (the walker this stone fixes)
;; never gets the chance to run. `strace -f` on the child service process shows it replying
;; with a `Reply::Failed` carrying:
;;
;;   "poll (process tier): client message decode failed: src/edn_shim.rs:1773:52:
;;    EDN Symbol — wat has no symbol value type"
;;
;; The GENERIC, untyped message dispatch decode (`edn_to_value`/`edn_to_value_caps`,
;; `runtime.rs:28719`'s `decode_trusted_wire` call, used to figure out WHICH op a frame is for
;; BEFORE any type-directed walk is possible) unconditionally refuses `Edn::Symbol`
;; (`edn_shim.rs:1773`). A real WatAST form legitimately CONTAINS symbols (`<-`, bare
;; identifiers) as part of its structure — so ANY non-trivial form crossing the process wire
;; trips this refusal, upstream of and independent from the walker this stone's identity arm
;; fixes. Confirmed by isolation: a bare `:wat::WatAST` field crashes the SAME way even when
;; the op handler never touches the field (so it's not the handler; it's message decode), and
;; a nested non-WatAST record field round-trips fine over the same locus (so it's not "any
;; non-primitive field" — specifically Symbol-bearing content). service.wat's own comment
;; (~line 727) says the generated client method should surface `Reply::Failed` as "an
;; unignorable raise" — but the client here observes a plain `RecvOutcome::Lost disconnected`
;; instead, which looks like the SEPARATE client-reply-handling gap the design stone's own
;; "LOCUS ASYMMETRY" finding already flags (STOP-4) as real-and-separately-tracked, not this
;; stone's scope. Reported, not fixed, per STOP-4 and the blast radius ("no defservice change,
;; no wat/ change") — a real fix here means loosening `edn_to_value_caps`'s GENERAL untyped
;; reader, used far beyond service dispatch, which is a materially larger change than "one arm
;; in one walker."

(:wat::core::defsurface :probe::WireKind :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::WireKind::EchoRequest [n <- :wat::core::i64])
   (:wat::core::defenum :probe::WireKind::EchoResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::WireKind::CountRequest [defs <- (:wat::core::Vector :- [:wat::WatAST])])
   (:wat::core::defenum :probe::WireKind::CountResponse :wat::enum::Pure
     :Ok               [n <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo  [self <- :probe::WireKind  req <- :probe::WireKind::EchoRequest]  -> :probe::WireKind::EchoResponse  :max-request-bytes 524288)
   (count [self <- :probe::WireKind  req <- :probe::WireKind::CountRequest] -> :probe::WireKind::CountResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::wirekindsvc
  :satisfies :probe::WireKind
  :durable   [calls <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :probe::wirekindsvc::Record] -> :probe::wirekindsvc::State
          (:probe::wirekindsvc::State :durable record))
  :impls
  [(echo [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::WireKind::EchoResponse::Ok (:probe::WireKind::EchoRequest/n req))))
   (count [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::WireKind::CountResponse::Ok
         (:wat::core::length (:probe::WireKind::CountRequest/defs req)))))])

(:wat::core::defn :probe::connect! [h <- :probe::wirekindsvc::Handle] -> :probe::WireKind
  (:wat::core::match (:wat::kernel::connect (:probe::wirekindsvc::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; three quoted declarations — a payload with a known length of 3
(:wat::core::defn :probe::three-forms [] -> (:wat::core::Vector :- [:wat::WatAST])
  (:wat::core::Vector :wat::WatAST
    (:wat::core::quote (:wat::core::defrecord :usr::A [c <- :wat::core::i64]))
    (:wat::core::quote (:wat::core::defrecord :usr::B [c <- :wat::core::i64]))
    (:wat::core::quote (:wat::core::defrecord :usr::C [c <- :wat::core::i64]))))

(:wat::core::defn :probe::run-echo [] -> :wat::core::nil
  (:wat::core::let
    [h (:probe::wirekindsvc/start :locus (:wat::spawn::process) :record (:probe::wirekindsvc::Record :calls 0))
     c (:probe::connect! h)]
    (:wat::core::match (:probe::WireKind/echo c (:probe::WireKind::EchoRequest :n 7))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::WireKind::EchoResponse::Ok n)
            (:wat::kernel::println (:wat::string::concat "CONTROL echo(i64)        => Ok n=" (:wat::core::i64::to-string n))))
          ((:probe::WireKind::EchoResponse::RequestTooLarge _b _c) (:wat::kernel::println "CONTROL echo(i64)        => REQUEST-TOO-LARGE"))
          ((:probe::WireKind::EchoResponse::RequestMalformed _p _e _g) (:wat::kernel::println "CONTROL echo(i64)        => REQUEST-MALFORMED"))))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::println (:wat::string::concat "CONTROL echo(i64)        => LOST " (:wat::kernel::LociDiedError/message cause))))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println "CONTROL echo(i64)        => STOPPED"))
      (:wat::kernel::RecvOutcome::Closed  (:wat::kernel::println "CONTROL echo(i64)        => CLOSED")))))

(:wat::core::defn :probe::run-count [] -> :wat::core::nil
  (:wat::core::let
    [h (:probe::wirekindsvc/start :locus (:wat::spawn::process) :record (:probe::wirekindsvc::Record :calls 0))
     c (:probe::connect! h)]
    (:wat::core::match (:probe::WireKind/count c (:probe::WireKind::CountRequest :defs (:probe::three-forms)))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::WireKind::CountResponse::Ok n)
            (:wat::kernel::println (:wat::string::concat "SUBJECT count(Vec<WatAST>) => Ok n=" (:wat::core::i64::to-string n))))
          ((:probe::WireKind::CountResponse::RequestTooLarge _b _c) (:wat::kernel::println "SUBJECT count(Vec<WatAST>) => REQUEST-TOO-LARGE"))
          ((:probe::WireKind::CountResponse::RequestMalformed _p _e _g) (:wat::kernel::println "SUBJECT count(Vec<WatAST>) => REQUEST-MALFORMED"))))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::println (:wat::string::concat "SUBJECT count(Vec<WatAST>) => LOST " (:wat::kernel::LociDiedError/message cause))))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println "SUBJECT count(Vec<WatAST>) => STOPPED"))
      (:wat::kernel::RecvOutcome::Closed  (:wat::kernel::println "SUBJECT count(Vec<WatAST>) => CLOSED")))))

;; ── THE ENCODE ISOLATOR — same op, same value, THREAD locus ───────────────────────────────
;; A thread peer hands values across in-process; a process peer EDN-encodes them through a pipe.
;; thread Ok + process LOST => the defect is in WatAST's EDN encode/decode, not in the value.
;; both LOST                => it is not the encoding; look at the serve loop / decode path.
(:wat::core::defn :probe::run-count-thread [] -> :wat::core::nil
  (:wat::core::let
    [h (:probe::wirekindsvc/start :locus (:wat::spawn::thread) :record (:probe::wirekindsvc::Record :calls 0))
     c (:probe::connect! h)]
    (:wat::core::match (:probe::WireKind/count c (:probe::WireKind::CountRequest :defs (:probe::three-forms)))
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::WireKind::CountResponse::Ok n)
            (:wat::kernel::println (:wat::string::concat "ISOLATOR count THREAD      => Ok n=" (:wat::core::i64::to-string n))))
          ((:probe::WireKind::CountResponse::RequestTooLarge _b _c) (:wat::kernel::println "ISOLATOR count THREAD      => REQUEST-TOO-LARGE"))
          ((:probe::WireKind::CountResponse::RequestMalformed p expected got)
            (:wat::core::do
              (:wat::kernel::println (:wat::string::concat "ISOLATOR count THREAD      => REQUEST-MALFORMED expected=" expected " got=" got))
              (:wat::kernel::println p)))))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::println (:wat::string::concat "ISOLATOR count THREAD      => LOST " (:wat::kernel::LociDiedError/message cause))))
      (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println "ISOLATOR count THREAD      => STOPPED"))
      (:wat::kernel::RecvOutcome::Closed  (:wat::kernel::println "ISOLATOR count THREAD      => CLOSED")))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "--- what a Vector<WatAST> LOOKS like to println ---")
    (:wat::kernel::println (:probe::three-forms))
    (:wat::kernel::println "--- and one bare WatAST ---")
    (:wat::kernel::println (:wat::core::quote (:wat::core::defrecord :usr::A [c <- :wat::core::i64])))
    (:probe::run-echo)
    (:probe::run-count)
    (:probe::run-count-thread)
    (:wat::kernel::println "READ: echo Ok + count Ok => the wire carries WatAST. echo Ok + count LOST => it does not. echo LOST => the shape is wrong and this proves nothing.")))
