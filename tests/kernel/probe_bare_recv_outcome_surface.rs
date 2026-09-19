//! ⭐ THE HONEST ROW-4 CONTROL — why `bracket.wat`'s `TimedOut` / `Malformed` arms cannot fire.
//!
//! DESIGN: `docs/excursus/2026/08/001-sns-sqs/the-bracket-runs-on-the-queue/DESIGN.md`
//! SCORE:  the sibling `SCORE.md` (and its orchestrator regrade)
//!
//! ⛔ WHAT THIS REPLACES, AND WHY THE PREDECESSOR PROVED NOTHING.
//! `probe_bracket_runs_on_the_queue_peer_dies.wat` was offered as the peer half of a
//! mutation control: "the peer path dies where the queue path survives". It never ran a
//! bracket. It started a queue at `drop-recv-bp=10000`, called `call-by-deadline` directly,
//! and **hand-wrote the two panic strings into its own match arms** — so the RED came from an
//! `assertion-failed!` the fixture author typed in, not from `bracket.wat`. Delete all ten arms
//! from `bracket.wat` and that test stayed green. A control that cannot redden on the change it
//! controls is not a control.
//!
//! ⛔ AND THE DEMANDED CONTROL WAS NEVER CONSTRUCTIBLE, because the premise was wrong.
//! The step-3 DESIGN (mine) claimed those ten arms were "the crusade's exact target — a momentary
//! failure, named as momentary, killing the process". **They cannot fire at all.** Measured here:
//!
//!   `:wat::kernel::recv` (bare) can return exactly FOUR variants — Message, Closed, Lost,
//!   Shutdown. `RecvOutcome::TimedOut` / `::Malformed` are constructed ONLY in
//!   `recv_outcome_from_peer_select` and `eval_peer_recv_by_deadline`, and bracket's five runner
//!   loops feed their `RecvOutcome` matches from bare `recv`.
//!
//! So those ten arms are **exhaustiveness filler** the checker forces (`check.rs` refuses a match
//! omitting a `RecvOutcome` arm), not live crash paths. That is a *better* state than the DESIGN
//! assumed — and a fragile one, because nothing recorded it. Rows 1-3 record it.
//!
//! ⚠ `RecvOutcome::Malformed` being unemittable is not an accident: `a-momentary-failure-is-not-fatal`
//! stone 1a added the variant and says so in its own words — *"Purely additive. Nothing emits it
//! yet."* Stone **1b**, which makes the transport emit it, is still UNSTRUCK. When 1b lands these
//! pins go RED and the five placeholders must be migrated to REPORT-FINAL first. **This file is
//! the tripwire those placeholders are waiting for.**
//!
//! ⛔⛔ AND THE DESIGN MISSED THE ARM THAT IS ACTUALLY LIVE — corrected here rather than left to
//! be rediscovered. `bracket.wat` DOES call `:wat::kernel::select` once (collect-loop), but that
//! yields a `:wat::spawn::ServiceEvent`, a different enum. `ServiceEvent::Malformed` IS emitted
//! (`src/runtime.rs`, on a real decode failure: "poll (process tier): client message decode
//! failed"), and bracket's arm for it calls `assertion-failed!` — "runner {idx} sent an undecodable
//! result". THAT is the live ungraceful site: a deterministic decode failure killing the whole
//! bracket run, REPORT-FINAL in the governing taxonomy, a raise today. Row 4 counts it.
//!
//! ⭐ The honest summary: the ten arms the step-3 DESIGN named are DEAD, and the one live arm of
//! this class is one the DESIGN never mentioned. Both halves of that were found by a test failing
//! on its author, not by reading.

use std::path::Path;

/// The body of a top-level `fn NAME(` in `src`, from its signature to the next top-level `fn`.
/// Scoped to the function on purpose — a byte-window pin (the shape used for the 8c sweep control)
/// reddens on unrelated reformatting; a function-scoped slice does not.
fn fn_body<'a>(src: &'a str, name: &str) -> &'a str {
    let needle_variants = [
        format!("\npub fn {name}("),
        format!("\npub(crate) fn {name}("),
        format!("\nfn {name}("),
    ];
    let start = needle_variants
        .iter()
        .find_map(|n| src.find(n.as_str()))
        .unwrap_or_else(|| panic!("no top-level `fn {name}(` in the source — the pin's subject is gone"));
    let rest = &src[start + 1..];
    // Next top-level item: a line that begins a new `fn`/`pub fn` at column 0.
    let end = ["\npub fn ", "\npub(crate) fn ", "\nfn "]
        .iter()
        .filter_map(|m| rest[1..].find(m).map(|i| i + 1))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

fn runtime_src() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/runtime.rs");
    std::fs::read_to_string(p).expect("src/runtime.rs must be readable from the crate root")
}

/// ⭐ ROW 1 — bare `recv` constructs FOUR outcomes, and TimedOut / Malformed are not among them.
///
/// This is the fact that makes `bracket.wat`'s arms dead. If bare `recv` ever gains a deadline (or
/// starts routing through the select helper), this reddens — and the five placeholder arms become
/// live crash paths that must be migrated first.
#[test]
fn bare_recv_constructs_only_message_closed_lost_shutdown() {
    let src = runtime_src();
    let body = fn_body(&src, "eval_peer_recv_prime");

    // NON-VACUITY: the slice really is the recv implementation, not an empty find.
    assert!(
        body.len() > 400,
        "eval_peer_recv_prime's body sliced to {} bytes — the pin is reading the wrong thing",
        body.len()
    );
    assert!(
        // rune:lint(loose-assert) — a targeted PRESENCE over a large output: the subject is one
        // function body sliced out of a ~1MB src/runtime.rs, and the claim is "this constructor
        // is referenced here". An exact assert_eq! would pin the body byte-for-byte, i.e. pin
        // FORMATTING — the precise brittleness this file's doc rejects (see the 8c sweep
        // control's false-RED on a rustfmt run). Presence is the whole claim.
        body.contains("recv_outcome_message("),
        "eval_peer_recv_prime no longer constructs a Message outcome — the slice is wrong or the \
         recv path was rewritten; re-derive this pin before trusting it"
    );

    for present in [
        "recv_outcome_message(",
        "recv_outcome_closed(",
        "recv_outcome_lost(",
        "recv_outcome_shutdown(",
    ] {
        assert!(
            body.contains(present),
            "bare recv no longer constructs {present} — its outcome surface moved, and \
             bracket.wat's exhaustive matches were written against the old surface"
        );
    }

    for absent in ["recv_outcome_timedout(", "recv_outcome_malformed("] {
        assert!(
            !body.contains(absent),
            "⛔ BARE `recv` NOW CONSTRUCTS {absent} — bracket.wat's five `RecvOutcome::{}` arms \
             are no longer exhaustiveness filler, they are LIVE CRASH PATHS. Migrate them to the \
             three dispositions (RETRY / REPORT-FINAL / REPORT-GONE) per \
             docs/excursus/2026/08/001-sns-sqs/a-momentary-failure-is-not-fatal/DESIGN.md BEFORE \
             making this green again.",
            // rune:lint(loose-assert) — NOT an assertion. This picks a human label for the
            // panic text above; the assertion is the `!body.contains(absent)` on the line before.
            if absent.contains("timedout") { "TimedOut" } else { "Malformed" }
        );
    }
}

/// ⭐ ROW 2 — the two unemittable-by-`recv` outcomes are reachable only from `select` and the
/// deadline recv. Pins WHERE they come from, so "bare recv cannot" stays a statement about the
/// call graph rather than about one function read in isolation.
#[test]
fn timedout_and_malformed_come_only_from_select_and_deadline() {
    let src = runtime_src();
    let owners = ["recv_outcome_from_peer_select", "eval_peer_recv_by_deadline"];
    let bodies: Vec<&str> = owners.iter().map(|o| fn_body(&src, o)).collect();

    for (ctor, label) in [
        ("recv_outcome_timedout()", "TimedOut"),
        ("recv_outcome_malformed(", "Malformed"),
    ] {
        // Every call site in the file...
        let total = src.matches(ctor).count();
        // ...minus the definition itself.
        let defined = src.matches(&format!("fn {}", ctor.trim_end_matches(&['(', ')'][..]))).count();
        let calls = total - defined;
        assert!(
            calls > 0,
            "{label} has no constructor call at all — either it was deleted or this pin's \
             spelling drifted; a zero here would make row 1 vacuously true"
        );
        let accounted: usize = bodies.iter().map(|b| b.matches(ctor).count()).sum();
        assert_eq!(
            accounted, calls,
            "⛔ {label} is now constructed OUTSIDE {owners:?} ({calls} call(s), {accounted} \
             accounted for). A new emitter may make bracket.wat's placeholder arms live — check \
             whether bracket reaches it before making this green."
        );
    }
}

/// ⭐ ROW 3 — bracket's `RecvOutcome` matches are fed by BARE `recv`, so their TimedOut /
/// Malformed arms are dead. `recv-by-deadline` would make them live; forbid it here.
///
/// ⚠ `select` is NOT forbidden, and the reason is the finding: `(:wat::kernel::select peers)`
/// yields a `:wat::spawn::ServiceEvent`, a DIFFERENT enum with its own arms — not a `RecvOutcome`.
/// Row 4 covers that surface. `:wat::service::call-by-deadline` (the queue path) is also expected.
#[test]
fn bracket_recvoutcome_matches_are_fed_by_bare_recv() {
    let code = bracket_code();
    let bare = code.matches(":wat::kernel::recv").count()
        - code.matches(":wat::kernel::recv-by-deadline").count()
        - code.matches(":wat::kernel::recv-all").count();
    assert!(
        bare >= 5,
        "expected the five peer runner loops to call bare `:wat::kernel::recv`; found {bare}"
    );
    let deadline = code.matches(":wat::kernel::recv-by-deadline").count();
    assert_eq!(
        deadline, 0,
        "⛔ bracket.wat now calls recv-by-deadline ({deadline} site(s)), which CAN return \
         RecvOutcome::TimedOut. The five placeholder arms are no longer unreachable — migrate them \
         to RETRY / REPORT-FINAL / REPORT-GONE first."
    );
}

/// ⭐ ROW 4 — THE ARM THAT IS ACTUALLY LIVE, and which the step-3 DESIGN never named.
///
/// `(:wat::kernel::select peers)` yields a `ServiceEvent`, and `ServiceEvent::Malformed` IS emitted
/// — `src/runtime.rs` builds it on a real decode failure ("poll (process tier): client message
/// decode failed"). `wat/bracket.wat`'s collect-loop matches it and calls `assertion-failed!`
/// ("runner {idx} sent an undecodable result"). **That is a deterministic decode failure killing
/// the whole bracket run** — REPORT-FINAL in the governing taxonomy, a raise today.
///
/// This pin does NOT bless it. It records that the live arm is known and counted, so the number
/// cannot grow silently and so a future migration has a starting point that is measured rather
/// than assumed. ⛔ The ten `RecvOutcome` arms are NOT the crusade target here; this one is.
#[test]
fn the_live_ungraceful_arm_is_serviceevent_malformed_and_it_is_counted() {
    let src = runtime_src();
    assert!(
        // rune:lint(loose-assert) — a targeted PRESENCE over a large output, same reason as row 1:
        // two needles located anywhere in src/runtime.rs prove the ServiceEvent::Malformed emitter
        // still exists. What is being asserted is EXISTENCE, not a value, so there is no
        // structured value to capture in an .edn golden.
        src.contains("variant_name: \"Malformed\".into(),")
            && src.contains("client message decode failed"),
        "the ServiceEvent::Malformed emitter moved — re-derive which of bracket's arms are live \
         before trusting this file's reachability claims"
    );

    let code = bracket_code();
    let raising: usize = code
        .split(":wat::spawn::ServiceEvent::")
        .skip(1)
        .filter(|seg| {
            let head = &seg[..seg.len().min(400)];
            head.starts_with("Malformed") && head.contains("assertion-failed!")
        })
        .count();
    assert_eq!(
        raising, 1,
        "expected exactly ONE raising ServiceEvent::Malformed arm in wat/bracket.wat (collect-loop, \
         \"sent an undecodable result\"); found {raising}. A second one is a new ungraceful site of \
         the same class — place it in RETRY / REPORT-FINAL / REPORT-GONE rather than copying the raise."
    );
}

fn bracket_code() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("wat/bracket.wat");
    let src = std::fs::read_to_string(p).expect("wat/bracket.wat must be readable");
    src.lines()
        .map(|l| match l.find(";;") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}
