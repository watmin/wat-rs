;; s2s-midlife-vec-probe.wat — arc 278 grant: mid-life + vec proof.
;; The circuit builder:
;;   1. starts echo' on a PROCESS;
;;   2. starts caller1' on a PROCESS whose post-spawn grants caller1's pid → drives it (echo:hi);
;;   3. MID-LIFE explicit grant: calls (echo'/grant eh <2-elem vec>) directly from main AFTER echo
;;      has already booted + served caller1 — proves the grant verb is callable post-boot AND folds
;;      a multi-element pid vec (the ack returns before the call completes);
;;   4. starts a SECOND caller (caller2') whose post-spawn grants ITS pid to the already-running
;;      echo (post-boot grant) → drives it (echo:hi).
;; Prints:
;;   echo:hi
;;   echo:hi

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])

(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo
  :durable   []
  :ephemeral []
  :impls
  [(echo [s req]
     (:wat::service::Outcome::Reply s
       (:probe::Echo::EchoResponse :reply
         (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

(:wat::core::defsurface :probe::Caller :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Caller::RunRequest  [])
   (:wat::core::defrecord :probe::Caller::RunResponse [out <- :wat::core::String])]
  :features
  [(run [self <- :probe::Caller  req <- :probe::Caller::RunRequest] -> :probe::Caller::RunResponse)])

(:wat::service::defservice :probe::caller'
  :satisfies :probe::Caller
  :durable   []
  :ephemeral [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]
  :peers     [:probe::Echo]
  :init (:wat::core::fn
          [record    <- :probe::caller'::Record
           echo-addr <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>]
          -> :probe::caller'::State
          (:probe::caller'::State :durable record :echo (:wat::kernel::connect' echo-addr)))
  :impls
  [(run [s req]
     (:wat::core::let
       [echo (:probe::caller'::State/echo s)
        er   (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg "hi"))
        out  (:probe::Echo::EchoResponse/reply er)]
       (:wat::service::Outcome::Reply s (:probe::Caller::RunResponse :out out))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     ea  (:probe::echo'::Handle/addr eh)
     ;; caller1 — granted at boot via its post-spawn hook.
     ch1 (:probe::caller'/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:probe::echo'/grant eh
                        (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
           :record (:probe::caller'::Record) :echo-addr ea)
     cc1 (:wat::kernel::connect' (:probe::caller'::Handle/addr ch1))
     rr1 (:probe::Caller/run cc1 (:probe::Caller::RunRequest))
     _   (:wat::kernel::println (:probe::Caller::RunResponse/out rr1))
     ;; MID-LIFE explicit grant, direct from main, echo already serving — a 2-element vec.
     ;; (dummy pids; the fold + ack must complete and return nil.)
     _   (:probe::echo'/grant eh (:wat::core::Vector :wat::core::i64 900001 900002))
     ;; caller2 — granted post-boot (echo is mid-life) via the same grant verb in its post-spawn.
     ch2 (:probe::caller'/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:probe::echo'/grant eh
                        (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
           :record (:probe::caller'::Record) :echo-addr ea)
     cc2 (:wat::kernel::connect' (:probe::caller'::Handle/addr ch2))
     rr2 (:probe::Caller/run cc2 (:probe::Caller::RunRequest))]
    (:wat::kernel::println (:probe::Caller::RunResponse/out rr2))))
