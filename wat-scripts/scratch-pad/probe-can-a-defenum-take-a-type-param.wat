;; probe-can-a-defenum-take-a-type-param.wat — is a PARAMETRIC defenum expressible in wat?
;;
;; Arc 278. `:wat::service::call-by-deadline` returns `(Tuple (Option :- [O]) i64)` with
;; 0=answer / 1=lost / 2=deadline. That carries one fact twice: `(None, 0)` and `(Some x, 2)`
;; are both writable, and `circuit.wat:401` already reads only the Option. The rung-3 form is
;; a three-arm enum where the reply cannot be obtained without learning why it arrived:
;;
;;   Answered [reply <- :O] | PeerLost | DeadlineFired
;;
;; ⛔ THE DISCONFIRMING QUESTION. Every parametric type the tree uses -- Option, RecvOutcome,
;; Peer, Vector -- is RUST-side. `grep "defenum :… :- ["` over wat/ and wat-scripts/ returns
;; NOTHING. So there is no worked example of a wat-declared parametric enum, and the whole
;; stone rests on one thing: can `defenum` take a type parameter at all?
;;
;; If YES  -> the enum lives in wat/service.wat beside the helper and the stone is small.
;; If NO   -> the tightening needs a per-surface enum (one per Reply type), or it is a
;;            substrate stone against `defenum`, and either way the shape of the work changes.
;;
;; Two cells, so a failure names WHICH half is missing:
;;   1. declare a parametric enum
;;   2. construct and match one at a concrete type

(:wat::config::set-redef! true)

(:wat::core::defenum :dp::CallOutcome :- [O] :wat::enum::Pure
  :Answered      [reply <- :O]
  :PeerLost      []
  :DeadlineFired [])

(:wat::core::defn :dp::describe [c <- (:dp::CallOutcome :- [:wat::core::String])]
  -> :wat::core::String
  (:wat::core::match c
    ((:dp::CallOutcome::Answered r) (:wat::core::format "answered={r}" :r r))
    ((:dp::CallOutcome::PeerLost) "peer-lost")
    ((:dp::CallOutcome::DeadlineFired) "deadline")))

(:wat::core::defn :dp::run [] -> :wat::core::String
  (:wat::core::let
    [a (:dp::describe (:dp::CallOutcome::Answered "hi"))
     b (:dp::describe (:dp::CallOutcome::PeerLost))
     c (:dp::describe (:dp::CallOutcome::DeadlineFired))]
    (:wat::core::format "a={a};b={b};c={c};parametric-defenum={ok}"
      :a a :b b :c c
      :ok (:wat::core::if
            (:wat::core::and (:wat::core::= a "answered=hi")
              (:wat::core::and (:wat::core::= b "peer-lost") (:wat::core::= c "deadline")))
            "yes" "NO"))))

(:wat::core::defn :user::compute [] -> :wat::core::String (:dp::run))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println (:dp::run)))
