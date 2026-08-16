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

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::{apply_function, Value};

#[test]
fn process_post_spawn_hook_receives_child_pid() {
    // Proof 1: process hook receives child pid. Wat source: probe_arc209_c0b3bc_post_spawn.wat
    let got = call_beside_value(file!(), ":user::compute")
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

#[test]
fn thread_post_spawn_hook_fires_with_empty_launch() {
    // Proof 2: thread hook fires with empty ThreadLaunch. Wat source: probe_arc209_c0b3bc_post_spawn_thread.wat
    let world = startup_from_file("tests/services/probe_arc209_c0b3bc_post_spawn_thread.wat")
        .expect("startup should succeed (C0b.3b-c: thread post-spawn hook)");
    let func = world
        .symbols()
        .get(":user::compute")
        .expect("no :user::compute in probe_arc209_c0b3bc_post_spawn_thread.wat")
        .clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(777)),
        "expected the thread post-spawn hook to fire owner-side with the empty ThreadLaunch and \
         forward the sentinel 777; got {got:?}"
    );
}

#[test]
fn accessor_typechecks_at_parse_time() {
    // GREEN after 3b-c: the ctor is known, so the checker reaches the hook body and rejects the
    // nonexistent field — the error names `bogus-field`. RED at HEAD: the ctor `process/post-spawn`
    // is unknown, so the error is about the ctor and does NOT mention the field.
    // Wat source: probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat (NEGATIVE — must fail startup)
    match startup_from_file("tests/services/probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat") {
        Ok(_) => panic!(
            "expected a check error: ProcessLaunch has no field `bogus-field`, so the hook fn must \
             fail to type-check at parse time"
        ),
        Err(e) => {
            let msg = format!("{e:?}");
            wat::assert_edn_matches_file!(
                msg,
                "probe_arc209_c0b3bc_post_spawn__accessor_typechecks_at_parse_time.edn",
                "parse-time error: ProcessLaunch has no field `bogus-field` (exactly ONE error — \
                 the fixture's SendOutcome match was missing its `Stopped` arm until 296 B6, \
                 which added a second, unrelated MalformedForm to this negative fixture)"
            );
        }
    }
}
