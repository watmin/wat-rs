//! ★ strike-import-accounting (arc 278, class A7) — **a door that opens a session must charge it,
//! and must not build without a bound.**
//!
//! `grep 'check_session_ceiling\|mark_session_origin' src/rete/export.rs` returned NOTHING at
//! HEAD. Two findings behind that one grep:
//!
//! 1. **The import was charged to nothing, and that is worse than uncounted.**
//!    `alloc_counter::session_bytes` does `entry(key).or_insert(now)`, so for a session whose
//!    origin was never marked, THE FIRST CEILING CHECK BECOMES ITS ORIGIN — every byte the import
//!    allocated is retroactively free and the session's ceiling begins after its network already
//!    exists. Driven on the same 2 MB of allocation: marked-at-birth reads `2097268`,
//!    never-marked reads `0`. This is A4's defect at a door A4 did not cover.
//! 2. **The build is quadratic with no cap on N.** `PMap::from_pairs`'s accumulator scans
//!    everything already accumulated once per pair, measured at 1.05 µs/pair over 500 pairs and
//!    4.87 µs/pair over 4 000 — per-pair cost doubling as N doubles. Nothing bounded N, so what
//!    `import` accepted was whatever the caller was willing to WAIT for.
//!
//! ## Three arms, three mechanisms, and why each probe here is the only one that can see its own
//!
//! - **The origin capture** — `import_refuses_a_build_that_outgrows_the_session_ceiling`, driven
//!   through the two `.wat` twins. The trap this arc named in advance is that the natural place to
//!   file the origin is AFTER the build (that is where the key exists), which reads a
//!   `thread_bytes()` already containing the build and charges the session zero. An origin would
//!   still be visibly filed. Nothing that asks *"is an origin filed?"* can tell the two apart; only
//!   asking what the origin is WORTH can, which is why this arm is driven through a real ceiling.
//! - **The cap** — `import_refuses_a_node_count_past_the_cap`, with a control one node UNDER the
//!   cap that must refuse for a different reason. Without that control the probe would pass on a
//!   door that refuses every tampered `nodes` field for any reason at all.
//! - **Non-clobber** — `an_origin_already_filed_is_never_re_based`. `mark_session_origin_at` files
//!   an origin captured EARLIER than now, so a clobbering write here would move a live session's
//!   zero point BACKWARDS and hand it free bytes. The `.wat` arms above cannot see this: they file
//!   each key exactly once.

use std::sync::Arc;
use std::path::Path;
use std::process::{Command, Stdio};

use wat::freeze::{call_beside_value, startup_beside};
use wat::runtime::{apply_function, Value};
use wat::AggregateValue;

/// The declared cap, mirrored here because the constant is private to `src/rete/export.rs` —
/// the same reason `probe_arc278_export.rs` mirrors `MAX_IMPORT_DEPTH`.
const MAX_IMPORT_NODES: usize = 10_000;

fn run(rel: &str) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_wat");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let out = Command::new(bin)
        .arg(&path)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {}: {e}", path.display()));
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// ★ ARM 1 — THE ORIGIN CAPTURE. The import is charged for the network it builds.
///
/// The ceiling fixture imports a network whose build allocates 15_172 live bytes under an 8_192
/// byte ceiling, so the door must refuse. **File the origin with a `thread_bytes()` read at the
/// filing rather than the one captured at the door and this goes green-to-red: the ceiling then
/// sees ~0 bytes, the import succeeds, and `IMPORTED` reaches stdout.** That is the whole reason
/// the reading and the filing are two separate operations in `import_export`.
#[test]
fn import_refuses_a_build_that_outgrows_the_session_ceiling() {
    let (ok, out, err) = run("tests/rete/probe_arc278_import_accounting_ceiling.wat");
    assert!(
        !ok,
        "an import whose build outgrows `max-session-bytes` must refuse at the door — this one \
         returned a Session. If stdout says IMPORTED, the import allocated 15 KB against an 8 KB \
         ceiling and the session it created was charged nothing for it.\nstdout: {out}\nstderr: {err}"
    );
    // Exact rather than a `!contains`: the refusal is a raise, so it goes to stderr and this
    // program's stdout must be EMPTY. Nothing weaker is needed and nothing weaker is honest.
    assert_eq!(
        out.trim(),
        "",
        "the import must not have completed — anything on stdout here means it returned a Session"
    );
    // The refusal names both halves of the judgement. `used` is not pinned exactly — it is a live
    // allocator reading — but it must EXCEED the limit it tripped, which is the property that
    // distinguishes a real measurement from a check that refuses unconditionally.
    assert!(
        err.contains("past max-session-bytes 8192"), // rune:lint(loose-assert) — the reason string is the contract; the located wrapper around it is not pinned
        "the refusal must name the CONFIGURED ceiling — a hardcoded default here would mean the \
         wat directive is decorative\nstderr: {err}"
    );
    let used = err
        .split("import allocated ")
        .nth(1)
        .and_then(|t| t.split(' ').next())
        .and_then(|n| n.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("the refusal must report the bytes it measured\nstderr: {err}"));
    assert!(
        used > 8192,
        "the reported usage must exceed the limit it tripped; got {used}\nstderr: {err}"
    );

    // NON-VACUITY. The byte-for-byte identical program at the DEFAULT 1 GiB ceiling must import,
    // FIRE, and derive its one Hit. Without this row a ceiling of zero, or a check placed before
    // any work, satisfies every assertion above.
    let (ok_d, out_d, err_d) = run("tests/rete/probe_arc278_import_accounting_default.wat");
    assert!(
        ok_d,
        "the same import at the default ceiling must not refuse — 15 KB is nowhere near 1 GiB\
         \nstdout: {out_d}\nstderr: {err_d}"
    );
    assert_eq!(
        out_d.trim(),
        "1",
        "and the imported session must actually DERIVE — a run that completes without doing the \
         work proves nothing about the ceiling's headroom\nstdout: {out_d}"
    );
}

/// ★ ARM 2 — THE NODE CAP. `import` will not build a network larger than it has measured.
///
/// The nodes here are deliberately JUNK (`0` is not a packed node). That is the point: the cap is
/// checked against the DECLARED length before a single node is unpacked, so a probe carrying valid
/// nodes would prove nothing about the ordering — and building 10_001 valid nodes would itself pay
/// the quadratic cost the cap exists to bound.
///
/// ⛔ THE CONTROL IS NOT DECORATION. The same junk one node UNDER the cap must ALSO refuse (it is
/// still not a network) but with a DIFFERENT message. Without that row this test passes on a door
/// that refuses every tampered `nodes` field for any reason whatsoever, which is the vacuous-gate
/// shape this arc keeps pulling out.
#[test]
fn import_refuses_a_node_count_past_the_cap() {
    let world = startup_beside(file!()).expect("freeze");
    let exp = call_beside_value(file!(), ":user::an-export").expect("export");

    let junk = |n: usize| Value::Vec(Arc::new(vec![Value::i64(0); n]));

    let over = poke_named(exp.clone(), "nodes", junk(MAX_IMPORT_NODES + 1));
    let err = import_one(&world, over).expect_err(
        "a node count past MAX_IMPORT_NODES must refuse — the build is quadratic in it",
    );
    let msg = format!("{err:?}");
    assert!(
        msg.contains("MAX_IMPORT_NODES"), // rune:lint(loose-assert) — refuse wraps rust_caller_span; the named cap is the contract
        "the refusal must name the cap it tripped, got {msg}"
    );
    assert!(
        msg.contains(&(MAX_IMPORT_NODES + 1).to_string()), // rune:lint(loose-assert) — as above; the COUNT is what a caller needs to act
        "the refusal must name the count it refused, got {msg}"
    );

    // CONTROL — one node under the cap. Still junk, so still refused, but NOT by the cap.
    let under = poke_named(exp, "nodes", junk(MAX_IMPORT_NODES));
    let ctl = import_one(&world, under).expect_err("junk nodes must still refuse");
    let ctl_msg = format!("{ctl:?}");
    assert!(
        // rune:lint(loose-assert) — a targeted ABSENCE over a located raise: the control must be
        // refused by something OTHER than the cap, and what that something says is not this
        // probe's contract. Pinning the unpack refusal's wording here would couple arm 2 to a
        // wall it is not about.
        !ctl_msg.contains("MAX_IMPORT_NODES"),
        "a count AT the cap must be refused by whatever the nodes actually are, not by the cap — \
         a cap that fires here is off by one and this probe could not tell, got {ctl_msg}"
    );
}

/// ★ ARM 3 — NON-CLOBBER. An origin already filed for a key wins over a later filing.
///
/// A4's rule, kept by the explicit-origin sibling. It matters MORE here than at
/// `mark_session_origin`, because the origin this door files was captured in the past: a
/// clobbering write would move a live session's zero point BACKWARDS and hand it free bytes.
///
/// The two `.wat` arms above cannot see this — each files its key exactly once — so this arm has
/// its own probe or it has none. Replace `or_insert` with `insert` in
/// `alloc_counter::mark_session_origin_at` and this goes red.
#[test]
fn an_origin_already_filed_is_never_re_based() {
    // A key no `PMap` will ever mint: `next_intern()` is a monotonic counter from 0 and this test
    // process mints nothing near it. Colliding with a live session would make the reading below
    // measure that session instead.
    let key: wat::alloc_counter::SessionOriginKey = Some(u64::MAX - 1);

    // A megabyte held LIVE, so the two candidate origins are separated by a figure no allocator
    // noise can close. `thread_bytes` is a live-bytes reading, so this must stay in scope.
    //
    // ⛔ `black_box` IS LOAD-BEARING AND WAS MEASURED SO. The floor is weighed in RELEASE, and
    // there LLVM deleted this allocation outright — a `Vec` that is only dropped is dead, and
    // eliding it is legal even under a custom global allocator. `thread_bytes()` read **121**, the
    // assert below fired, and the probe failed for a reason that had nothing to do with what it
    // measures. A probe whose subject is an allocation must make that allocation observable.
    let ballast: Vec<u8> = vec![7u8; 1 << 20];
    std::hint::black_box(ballast.as_ptr());
    let late = wat::alloc_counter::thread_bytes();
    assert!(
        late >= (1 << 20),
        "the ballast must be visible to the counter for the two origins to be distinguishable; \
         thread_bytes() = {late}"
    );

    // FIRST WRITE WINS: origin 0, so `session_bytes` must report the whole thread reading.
    wat::alloc_counter::mark_session_origin_at(key, 0);
    // A LATER origin for the same key. Under `or_insert` it is ignored; under `insert` it re-bases
    // the session to ~now and everything it had already spent stops being charged to it.
    wat::alloc_counter::mark_session_origin_at(key, late);

    let used = wat::alloc_counter::session_bytes(key);
    assert!(
        used >= (1 << 20),
        "an origin already filed must never be re-based: with the first write standing, this \
         session is charged for the whole {late}-byte thread; got {used}, which is what a \
         clobbering write leaves behind"
    );
    assert_eq!(ballast[0], 7, "the ballast must survive to here — see the `black_box` note above");
    std::hint::black_box(&ballast);
}

fn import_one(
    world: &wat::freeze::FrozenWorld,
    exp: Value,
) -> Result<Value, wat::runtime::RuntimeError> {
    let import = world
        .symbols()
        .get(":user::import-one")
        .expect("import-one")
        .clone();
    apply_function(import, vec![exp], world.symbols(), wat::rust_caller_span!())
}

fn poke_named(exp: Value, field: &str, v: Value) -> Value {
    match exp {
        Value::Aggregate(a) => {
            let mut fields = a.fields.as_ref().clone();
            let i = a.names.iter().position(|n| n == field).expect(field);
            fields[i] = v;
            Value::Aggregate(Arc::new(AggregateValue::record(
                a.class.to_string(),
                a.names.clone(),
                Arc::new(fields),
            )))
        }
        other => panic!("expected Export, got {other:?}"),
    }
}
