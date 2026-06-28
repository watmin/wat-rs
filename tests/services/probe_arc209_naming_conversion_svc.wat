(:wat::service::defservice :my::svc
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:GetObject [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::svc::GetObjectResponse (:my::svc::Record/count (:my::svc::State/durable s)))))])

(:wat::core::defn :user::req-id [] -> :wat::core::i64
  (:my::svc::GetObjectRequest/n (:my::svc/get-object-request 42)))
