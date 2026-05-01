;; examples/with-lru/src/program.wat — arc 013 slice 5's walkable
;; proof that a consumer binary can COMPOSE wat-rs's runtime +
;; wat-lru's external wat crate + its own wat program into a
;; single runnable binary.
;;
;; Run: `cargo run -p with-lru-example`. Expected output: `hit`.
;;
;; This is the minimal shape: put one entry into a LocalCache,
;; read it back, print hit/miss. The interesting bit is what the
;; wat::main! macro does BEHIND this — it wires wat-lru's
;; wat_sources() + register() into Harness composition so the
;; :wat::lru::LocalCache<K,V> path resolves, the Rust
;; shim dispatches, and the user code evaluates against a real
;; lru::LruCache.


;; No `(:wat::core::use! :rust::lru::LruCache)` is needed here: this
;; consumer only uses the wat-level wrapper `:wat::lru::LocalCache`.
;; wat-lru's own `wat/LocalCache.wat` declares the `use!` internally,
;; covering the Rust-side dispatch. `use!` belongs in whichever wat
;; file directly references `:rust::<crate>::*` — which, for consumers
;; of wrapped Rust types, is usually the wrapping crate, not the
;; consumer.

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :wat::core::unit)
  (:wat::core::let*
    (((cache :wat::lru::LocalCache<wat::core::String,wat::core::i64>)
      (:wat::lru::LocalCache::new 16))
     ((_ :wat::core::Option<(wat::core::String,wat::core::i64)>)
      (:wat::lru::LocalCache::put cache "answer" 42))
     ((got :wat::core::Option<wat::core::i64>)
      (:wat::lru::LocalCache::get cache "answer")))
    (:wat::core::match got -> :wat::core::unit
      ((:wat::core::Some v) (:wat::io::IOWriter/println stdout "hit"))
      (:wat::core::None    (:wat::io::IOWriter/println stdout "miss")))))
