//! Arc 209 C0b.3b-c — the post-spawn hook (owner-side after-spawn effects, per-env record).
//!
//! Every hosting env supports a `post-spawn-fn`: a function the OWNER supplies that runs
//! owner-side, after the peer is spawned, before `spawn-program'` returns, for effects. What it
//! RECEIVES differs per env — a per-env launch record (`ThreadLaunch` empty / `ProcessLaunch`
//! carrying the child pid) — but the PATTERN is universal. Mirror of `init-fn` (child-side, value
//! producer); this is owner-side, effects. Because `spawn-program'` dispatches on the host opts
//! TYPE, the hook fn + its record accessors type-check at PARSE time.
//!
//! THREE proofs:
//! 1. `process_post_spawn_hook_receives_child_pid` — the owner mints an owner-side `peer-pair'`
//!    channel, spawns a `(process/post-spawn f)` where `f` forwards `(ProcessLaunch/pid launch)`
//!    onto the channel; the owner reads the pid and asserts it is a real child pid (> 0, ≠ owner).
//!    RED at HEAD: `process/post-spawn` is an unknown ctor. GREEN after: the hook fires owner-side
//!    with the child pid. (Also proves the `ProcessLaunch/pid` accessor type-checks.)
//! 2. `thread_post_spawn_hook_fires_with_empty_launch` — a `(thread/post-spawn f)` whose `f`
//!    receives the empty `ThreadLaunch` and forwards a sentinel; the owner reads it. RED at HEAD
//!    (`thread/post-spawn` unknown), GREEN after (the hook fires owner-side on the thread tier).
//! 3. `accessor_typechecks_at_parse_time` — a `process/post-spawn` hook reading a NONEXISTENT
//!    field off `ProcessLaunch` fails to type-check; the error names the bogus field. RED at HEAD
//!    (the error is about the unknown ctor, not the field), GREEN after (the accessor is checked
//!    against the record at parse time — the builder's headline payoff).
//!
//! Test 1 FORKS (spawn-program' (process)). Run:
//! cargo test --release -p wat --test probe_arc209_c0b3bc_post_spawn -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ── Proof 1: process post-spawn hook receives the child pid, owner-side. ─────────────────────
const PROCESS_PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::peer-pair' :wat::core::i64 :wat::core::i64)
     tx    (:wat::core::first pair)
     rx    (:wat::core::second pair)
     _proc (:wat::kernel::spawn-program'
             (:wat::spawn::process/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::kernel::send' tx (:wat::spawn::ProcessLaunch/pid launch))]
                   nil)))
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)))
     pid   (:wat::kernel::recv' rx)]
    pid))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn process_post_spawn_hook_receives_child_pid() {
    let world = startup_from_source(PROCESS_PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3b-c: process post-spawn hook)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    let owner_pid = std::process::id() as i64;
    match got {
        Value::i64(pid) => assert!(
            pid > 0 && pid != owner_pid,
            "expected the process post-spawn hook to receive the spawned CHILD's pid \
             (> 0, ≠ the owner's pid {owner_pid}); got {pid}"
        ),
        other => panic!("expected an i64 child pid forwarded by the post-spawn hook; got {other:?}"),
    }
}

// ── Proof 2: thread post-spawn hook fires owner-side with the empty ThreadLaunch. ────────────
const THREAD_PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::peer-pair' :wat::core::i64 :wat::core::i64)
     tx    (:wat::core::first pair)
     rx    (:wat::core::second pair)
     _thr  (:wat::kernel::spawn-program'
             (:wat::spawn::thread/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ThreadLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::kernel::send' tx 777)] nil)))
             (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
               nil))
     sentinel (:wat::kernel::recv' rx)]
    sentinel))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn thread_post_spawn_hook_fires_with_empty_launch() {
    let world = startup_from_source(THREAD_PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3b-c: thread post-spawn hook)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(777)),
        "expected the thread post-spawn hook to fire owner-side with the empty ThreadLaunch and \
         forward the sentinel 777; got {got:?}"
    );
}

// ── Proof 3: the hook's record accessors type-check at parse time. ───────────────────────────
const BOGUS_ACCESSOR_PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair  (:wat::kernel::peer-pair' :wat::core::i64 :wat::core::i64)
     tx    (:wat::core::first pair)
     _proc (:wat::kernel::spawn-program'
             (:wat::spawn::process/post-spawn
               (:wat::core::fn [launch <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                 (:wat::core::let [_ (:wat::kernel::send' tx (:wat::spawn::ProcessLaunch/bogus-field launch))]
                   nil)))
             (:wat::core::forms
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)))]
    0))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn accessor_typechecks_at_parse_time() {
    // GREEN after 3b-c: the ctor is known, so the checker reaches the hook body and rejects the
    // nonexistent field — the error names `bogus-field`. RED at HEAD: the ctor `process/post-spawn`
    // is unknown, so the error is about the ctor and does NOT mention the field.
    match startup_from_source(BOGUS_ACCESSOR_PROGRAM, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!(
            "expected a check error: ProcessLaunch has no field `bogus-field`, so the hook fn must \
             fail to type-check at parse time"
        ),
        Err(e) => {
            let msg = format!("{e:?}");
            assert!(
                msg.contains("bogus-field"),
                "expected the parse-time error to name the nonexistent record field `bogus-field` \
                 (the accessor type-checks against ProcessLaunch); got: {msg}"
            );
        }
    }
}
