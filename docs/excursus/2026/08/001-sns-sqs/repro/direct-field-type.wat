;; MINIMAL REPRO — a USERLAND peer surface whose message carries a USERLAND type
;; declared OUTSIDE :messages. The forked child cannot resolve it.

(:wat::core::defrecord :p::Item [id <- :wat::core::String])   ;; ← OUTSIDE the surface

(:wat::core::defsurface :p::Src :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :p::Src::GetRequest [])
   (:wat::core::defenum :p::Src::GetResponse :wat::enum::Pure
     :Ok [item <- :p::Item]                                    ;; ← carries the outside type
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get [self <- :p::Src  req <- :p::Src::GetRequest] -> :p::Src::GetResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :p::src
  :satisfies :p::Src  :durable [] :ephemeral []
  :impls [(get [s ctx req] (:wat::service::Outcome::Reply s
                             (:p::Src::GetResponse::Ok (:p::Item :id "x"))))])

;; the CONSUMER — holds a :p::Src peer, reads Item/id in its impl
(:wat::core::defsurface :p::Use :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :p::Use::RunRequest [])
   (:wat::core::defenum :p::Use::RunResponse :wat::enum::Pure
     :Ok [out <- :wat::core::String]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(run [self <- :p::Use  req <- :p::Use::RunRequest] -> :p::Use::RunResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :p::use
  :satisfies :p::Use  :durable []
  :ephemeral [src <- (:wat::kernel::Peer :- [:p::Src::Op :p::Src::Reply])]
  :peers     [:p::Src]
  :init (:wat::core::fn [record <- :p::use::Record
                         src-addr <- (:wat::kernel::Address :- [:p::Src::Op :p::Src::Reply])]
          -> :p::use::State
          (:p::use::State :durable record
            :src (:wat::core::match (:wat::kernel::connect src-addr)
                   ((:wat::kernel::ConnectOutcome::Connected p) p)
                   (_ (:wat::kernel::assertion-failed! "dial" :wat::core::None :wat::core::None)))))
  :impls
  [(run [s ctx req]
     (:wat::core::let
       [r (:p::Src/get (:p::use::State/src s) (:p::Src::GetRequest))
        out (:wat::core::match r
              ((:wat::kernel::RecvOutcome::Message m)
                (:wat::core::match m
                  ((:p::Src::GetResponse::Ok item) (:p::Item/id item))   ;; ★ THE CALL
                  (_ "other")))
              (_ "recv"))]
       (:wat::service::Outcome::Reply s (:p::Use::RunResponse::Ok out))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [sh (:p::src/start :locus (:wat::spawn::process) :record (:p::src::Record))
     sa (:p::src::Handle/addr sh)
     uh (:p::use/start
          :locus (:wat::spawn::process/post-spawn
                   (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                     (:p::src/grant sh (:wat::core::Vector :- [:wat::core::i64]
                                         (:wat::spawn::ProcessLaunch/pid pl)))))
          :record (:p::use::Record) :src-addr sa)]
    (:wat::kernel::println (:wat::core::str (:p::use::Handle/addr uh)))))
