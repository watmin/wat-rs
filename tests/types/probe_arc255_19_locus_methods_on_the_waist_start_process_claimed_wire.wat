;; Stone 255.19 — defservice's abstract `start$impl`/`resume$impl` take `(Locus :- [T])`, T being the
;; Handle's own transport letter, so the locus's binding names the Handle's transport.
;; POSITIVE twin of _start_process_claimed_shared.wat.bad: the same abstract start, claimed Wire.
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Kv::GetRequest [k <- :wat::core::String])
   (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure
     :Ok              [v <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String]) expected <- :wat::core::String got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Kv req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::kv
  :satisfies :probe::Kv :durable [] :ephemeral []
  :impls [(get [s ctx req] (:wat::service::Outcome.Reply {:state s :reply (:probe::Kv::GetResponse.Ok {:v "v"})}))])
(:wat::core::defn :probe::go [l <- :wat::spawn::ProcessOpts] -> (:probe::kv::Handle :- [:wat::kernel::Transport.Wire])
  (:probe::kv/start :locus l :record (:probe::kv::Record)))
