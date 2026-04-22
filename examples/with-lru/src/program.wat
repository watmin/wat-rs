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
;; stdlib_sources() + register() into Harness composition so the
;; :user::wat::std::lru::LocalCache<K,V> path resolves, the Rust
;; shim dispatches, and the user code evaluates against a real
;; lru::LruCache.

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

;; wat-lru's real external dep. use! is load-bearing here —
;; :rust::lru::LruCache is a :#[wat_dispatch]'d third-party crate,
;; not substrate. wat-lru's register() adds the type to the
;; rust_deps registry at startup; the resolver validates this
;; declaration against it.
(:wat::core::use! :rust::lru::LruCache)

(:wat::core::define (:user::main
                     (stdin  :wat::io::IOReader)
                     (stdout :wat::io::IOWriter)
                     (stderr :wat::io::IOWriter)
                     -> :())
  (:wat::core::let*
    (((cache :user::wat::std::lru::LocalCache<String,i64>)
      (:user::wat::std::lru::LocalCache::new 16))
     ((_ :())
      (:user::wat::std::lru::LocalCache::put cache "answer" 42))
     ((got :Option<i64>)
      (:user::wat::std::lru::LocalCache::get cache "answer")))
    (:wat::core::match got -> :()
      ((Some v) (:wat::io::IOWriter/println stdout "hit"))
      (:None    (:wat::io::IOWriter/println stdout "miss")))))
