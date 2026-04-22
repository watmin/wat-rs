//! End-to-end proofs for `wat-lru`'s external-crate contract.
//!
//! Each test composes `wat-lru`'s `stdlib_sources()` + `register()`
//! with a user program that exercises `:user::wat::std::lru::LocalCache`,
//! runs through `wat::Harness::from_source_with_deps`, and asserts
//! on the captured stdout.
//!
//! Together these are the load-bearing proof that arc 013's
//! external-wat-crate mechanism carries a real Rust-backed surface
//! through Cargo dep resolution, `#[wat_dispatch]` shim composition,
//! and `Harness` freeze-and-run. They replace the single-threaded
//! LocalCache tests that previously lived inside wat-rs's own
//! `src/runtime.rs` / `src/freeze.rs` / `src/check.rs` — retired
//! there when slice 4b emptied wat-rs's baked LRU surface.
//!
//! Driver-thread / service-tier integration (the former
//! `wat-tests/std/service/Cache.wat`) lives in slice 5's
//! `examples/with-lru/` binary — a real fork target that knows
//! about wat-lru. `wat::test::run-hermetic-ast` forks to the
//! `wat-rs` CLI, which correctly does NOT link wat-lru, so the
//! service-tier test's subprocess shape waits for slice 5.

use wat::harness::{Harness, Outcome};

/// Run a user program with wat-lru composed in and return its
/// captured Outcome. Shared setup across every test below.
fn run_with_lru(src: &str) -> Outcome {
    let h = Harness::from_source_with_deps(
        src,
        &[wat_lru::stdlib_sources()],
        &[wat_lru::register],
    )
    .expect("freeze with wat-lru deps");
    h.run(&[]).expect("run")
}

/// The common program prelude — dims + capacity mode. Keeps
/// every test's source terse.
const PRELUDE: &str = r#"
    (:wat::config::set-dims! 1024)
    (:wat::config::set-capacity-mode! :error)
    (:wat::core::use! :rust::lru::LruCache)
"#;

#[test]
fn local_cache_put_then_get_returns_some() {
    let src = format!(
        r#"
        {prelude}
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::core::let*
            (((cache :user::wat::std::lru::LocalCache<String,i64>)
              (:user::wat::std::lru::LocalCache::new 16))
             ((_ :()) (:user::wat::std::lru::LocalCache::put cache "answer" 42))
             ((got :Option<i64>)
              (:user::wat::std::lru::LocalCache::get cache "answer")))
            (:wat::core::match got -> :()
              ((Some v) (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string v)))
              (:None    (:wat::io::IOWriter/println stdout "miss")))))
        "#,
        prelude = PRELUDE,
    );
    let Outcome { stdout, stderr } = run_with_lru(&src);
    assert_eq!(stdout, vec!["42".to_string()]);
    assert!(stderr.is_empty(), "expected empty stderr; got {:?}", stderr);
}

#[test]
fn local_cache_miss_returns_none() {
    let src = format!(
        r#"
        {prelude}
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::core::let*
            (((cache :user::wat::std::lru::LocalCache<String,i64>)
              (:user::wat::std::lru::LocalCache::new 16))
             ((got :Option<i64>)
              (:user::wat::std::lru::LocalCache::get cache "missing")))
            (:wat::core::match got -> :()
              ((Some v) (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string v)))
              (:None    (:wat::io::IOWriter/println stdout "miss")))))
        "#,
        prelude = PRELUDE,
    );
    let Outcome { stdout, .. } = run_with_lru(&src);
    assert_eq!(stdout, vec!["miss".to_string()]);
}

#[test]
fn local_cache_evicts_at_capacity() {
    // Capacity 2: after putting 3 entries, key 1 is evicted.
    let src = format!(
        r#"
        {prelude}
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::core::let*
            (((cache :user::wat::std::lru::LocalCache<i64,i64>)
              (:user::wat::std::lru::LocalCache::new 2))
             ((_ :()) (:user::wat::std::lru::LocalCache::put cache 1 10))
             ((_ :()) (:user::wat::std::lru::LocalCache::put cache 2 20))
             ((_ :()) (:user::wat::std::lru::LocalCache::put cache 3 30))
             ((got :Option<i64>)
              (:user::wat::std::lru::LocalCache::get cache 1)))
            (:wat::core::match got -> :()
              ((Some v) (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string v)))
              (:None    (:wat::io::IOWriter/println stdout "evicted")))))
        "#,
        prelude = PRELUDE,
    );
    let Outcome { stdout, .. } = run_with_lru(&src);
    assert_eq!(stdout, vec!["evicted".to_string()]);
}

#[test]
fn local_cache_put_overwrites_existing_key() {
    let src = format!(
        r#"
        {prelude}
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::core::let*
            (((cache :user::wat::std::lru::LocalCache<String,i64>)
              (:user::wat::std::lru::LocalCache::new 16))
             ((_ :()) (:user::wat::std::lru::LocalCache::put cache "k" 1))
             ((_ :()) (:user::wat::std::lru::LocalCache::put cache "k" 99))
             ((got :Option<i64>)
              (:user::wat::std::lru::LocalCache::get cache "k")))
            (:wat::core::match got -> :()
              ((Some v) (:wat::io::IOWriter/println stdout (:wat::core::i64::to-string v)))
              (:None    (:wat::io::IOWriter/println stdout "miss")))))
        "#,
        prelude = PRELUDE,
    );
    let Outcome { stdout, .. } = run_with_lru(&src);
    assert_eq!(stdout, vec!["99".to_string()]);
}
