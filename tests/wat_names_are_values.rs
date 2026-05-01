//! Integration coverage for arc 009 — names are values.
//!
//! A registered user/stdlib define's keyword-path evaluates to a
//! `Value::wat__core__lambda` in expression position; the type
//! checker infers a `:fn(params)->ret` scheme for the same position.
//! Callers pass named defines to `:fn(...)`-typed parameters without
//! a pass-through lambda wrapper — the asymmetry with
//! `:wat::kernel::spawn-thread`'s long-standing accept-by-name
//! convention dissolves.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::io::{StringIoReader, StringIoWriter, WatReader, WatWriter};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run_main_stdout(src: &str) -> Vec<String> {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let stdin: Arc<dyn WatReader> = Arc::new(StringIoReader::from_string(String::new()));
    let stdout = Arc::new(StringIoWriter::new());
    let stderr = Arc::new(StringIoWriter::new());
    let stdout_dyn: Arc<dyn WatWriter> = stdout.clone();
    let stderr_dyn: Arc<dyn WatWriter> = stderr.clone();
    let args = vec![
        Value::io__IOReader(stdin),
        Value::io__IOWriter(stdout_dyn),
        Value::io__IOWriter(stderr_dyn),
    ];
    invoke_user_main(&world, args).expect("main");
    let bytes = stdout.snapshot_bytes().expect("snapshot");
    let s = String::from_utf8(bytes).expect("utf8");
    if s.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<String> = s.split('\n').map(String::from).collect();
    if s.ends_with('\n') {
        lines.pop();
    }
    lines
}

// ─── named define lifts to a callable value ────────────────────────────

#[test]
fn named_define_is_a_function_value() {
    // `:my::double` is registered as a define. Referencing it in
    // expression position (not call-head) produces a lambda that can
    // be called by the user via a symbol binding.
    let src = r##"

        (:wat::core::define (:my::double (x :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::* x 2))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((f :fn(wat::core::i64)->wat::core::i64) :my::double)
             ((result :wat::core::i64) (f 21)))
            (:wat::io::IOWriter/println stdout
              (:wat::core::string::join ""
                (:wat::core::conj (:wat::core::Vector :wat::core::String) "result-is-")))))
    "##;
    // We can't stringify i64 without a fmt primitive. Check the call
    // worked by threading through a known branch.
    let src_check_result = r##"

        (:wat::core::define (:my::double (x :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::* x 2))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((f :fn(wat::core::i64)->wat::core::i64) :my::double)
             ((result :wat::core::i64) (f 21)))
            (:wat::core::if (:wat::core::= result 42) -> :wat::core::unit
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    let _ = src;
    assert_eq!(run_main_stdout(src_check_result), vec!["pass".to_string()]);
}

// ─── named define as higher-order argument ─────────────────────────────

#[test]
fn named_define_passes_to_higher_order_fn() {
    // A user-defined higher-order function `:my::apply-twice` takes
    // `:fn(wat::core::i64)->wat::core::i64` and an `:i64`; calling it with `:my::inc` and
    // `5` via the bare keyword path — no lambda wrapper — yields 7.
    let src = r##"

        (:wat::core::define (:my::inc (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+ n 1))

        (:wat::core::define (:my::apply-twice (f :fn(wat::core::i64)->wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
          (f (f x)))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((result :wat::core::i64) (:my::apply-twice :my::inc 5)))
            (:wat::core::if (:wat::core::= result 7) -> :wat::core::unit
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run_main_stdout(src), vec!["pass".to_string()]);
}

// ─── polymorphic named define — instantiation at call site ─────────────

#[test]
fn polymorphic_named_define_instantiates_at_use_site() {
    // Polymorphic `:my::identity<T>`. Passed to a monomorphic
    // `:fn(wat::core::i64)->wat::core::i64` slot; the scheme's `T` instantiates to `i64`.
    let src = r##"

        (:wat::core::define (:my::identity<T> (x :T) -> :T) x)

        (:wat::core::define (:my::apply (f :fn(wat::core::i64)->wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
          (f x))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((result :wat::core::i64) (:my::apply :my::identity 99)))
            (:wat::core::if (:wat::core::= result 99) -> :wat::core::unit
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run_main_stdout(src), vec!["pass".to_string()]);
}

// ─── unregistered keyword stays a literal ──────────────────────────────

#[test]
fn unregistered_keyword_still_a_literal() {
    // A keyword that is NOT a registered define remains a
    // `:wat::core::keyword` value. The lift is only when a define
    // exists at that path.
    let src = r##"

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((tag :wat::core::keyword) :my-app::tag::user-event)
             ((same? :wat::core::bool) (:wat::core::= tag :my-app::tag::user-event)))
            (:wat::core::if same? -> :wat::core::unit
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run_main_stdout(src), vec!["pass".to_string()]);
}

// ─── named define as stream map argument ───────────────────────────────

#[test]
fn named_define_as_stream_map_fn() {
    // The canonical target: pass `:my::double` to `:wat::stream::map`
    // without wrapping in a pass-through lambda.
    let src = r##"

        (:wat::core::define (:my::double (n :i64) -> :i64)
          (:wat::core::i64::* n 2))

        (:wat::core::define
          (:user::main
            (stdin  :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :wat::core::unit)
          (:wat::core::let*
            (((source :wat::stream::Stream<wat::core::i64>)
              (:wat::stream::spawn-producer
                (:wat::core::lambda ((tx :rust::crossbeam_channel::Sender<wat::core::i64>) -> :wat::core::unit)
                  (:wat::core::let*
                    (((_ :wat::core::unit)
                      (:wat::core::Result/expect -> :wat::core::unit
                        (:wat::kernel::send tx 1)
                        "producer: tx disconnected on send 1"))
                     ((_ :wat::core::unit)
                      (:wat::core::Result/expect -> :wat::core::unit
                        (:wat::kernel::send tx 2)
                        "producer: tx disconnected on send 2"))
                     ((_ :wat::core::unit)
                      (:wat::core::Result/expect -> :wat::core::unit
                        (:wat::kernel::send tx 3)
                        "producer: tx disconnected on send 3")))
                    ()))))
             ((doubled :wat::stream::Stream<wat::core::i64>)
              (:wat::stream::map source :my::double))
             ((collected :wat::core::Vector<wat::core::i64>) (:wat::stream::collect doubled))
             ((first :wat::core::i64)
              (:wat::core::match (:wat::core::first collected) -> :wat::core::i64
                ((:wat::core::Some n) n)
                (:wat::core::None -1)))
             ((len :wat::core::i64) (:wat::core::length collected)))
            (:wat::core::if (:wat::core::and (:wat::core::= first 2) (:wat::core::= len 3))
              -> :wat::core::unit
              (:wat::io::IOWriter/println stdout "pass")
              (:wat::io::IOWriter/println stdout "fail"))))
    "##;
    assert_eq!(run_main_stdout(src), vec!["pass".to_string()]);
}
