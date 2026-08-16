//! `wat --mcp` — the MCP server mode.
//!
//! Drives the REAL binary over a pipe (`env!("CARGO_BIN_EXE_wat")`, the `wat_cli.rs` /
//! `wat_repl.rs` pattern), because the thing under test is a mode of the CLI.
//!
//! ## What each test would have to break to go red
//!
//! An MCP server is trivially easy to gate VACUOUSLY — assert the process exits 0 and you
//! have proven a binary starts (R59 `NISI FRANGAS, NIHIL PROBAS`: a suite passed 4105/4105
//! for weeks over a protocol that had never once executed, because nothing in it DEPENDED on
//! the mechanism). So each of these depends on the mechanism it names:
//!
//! 1. `definitions_persist_across_turns` — the load-bearing property, and the whole point of
//!    holding a session at all. Cut the `Declared` arm's `session.defs.push(form)` in
//!    `distribution/mcp.rs` and the second call answers with an unresolved reference instead
//!    of `42`. Nothing else in this file can go red from that edit; this test can only pass
//!    if the definition genuinely crossed a call boundary.
//! 2. `reset_empties_the_session` — the same mechanism inverted, which is why a no-op `reset`
//!    cannot sneak through: it asserts the call SUCCEEDS before the reset and a domain Fault
//!    after. Delete `session.defs.clear()` and the post-reset call still answers `42` → red.
//! 3. `a_failed_evaluation_is_not_fatal` — failures are values, so the session survives them.
//!    If a bad form ended the process the following good call would never be answered. The
//!    Fault is `isError: false`: Grok prefixes `Failed to call eval:` when that flag is true.
//! 4. `the_payload_is_edn_not_json` — the ruling this mode exists to honour: the result rides
//!    as EDN text in a string slot, never converted to JSON. A record answer must come back
//!    as `#ns/Rec {…}`; if anything ever "helpfully" JSON-ified the payload, this goes red.
//! 5. `mcp_rejects_a_positional` — the mode's arity contract, as `--repl` has.
//! 6. `every_reply_carries_the_same_gen` / `a_new_process_mints_a_new_gen` —
//!    a *replaced* process (new child, new pipe) must change `gen`, or a
//!    virgin world is unfalsifiable from the model's side.
//! 7. `a_reused_ticket_is_a_protocol_fault` — two evals presenting the same
//!    ticket: the second is `#wat.mcp/Fault {:kind :stale-ticket}` and is
//!    not evaluated. Prose in the tool description is not a gate. This is.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};

use wat_edn::OwnedValue;

/// The transcripts are JSON-RPC frames carrying wat forms, so they live in co-located
/// fixture files rather than Rust string literals (the `no_inlined_wat_in_tests` /
/// `no_inlined_edn` rubric — a `{`-opening literal full of wat forms is exactly what those
/// lints exist to keep out of `.rs`).
const PERSIST_IN: &str = include_str!("wat_mcp__persist.jsonl");
const MULTIFORM_IN: &str = include_str!("wat_mcp__multiform.jsonl");
const TOPLEVEL_EXPR_IN: &str = include_str!("wat_mcp__toplevel_expr.jsonl");
const RESET_IN: &str = include_str!("wat_mcp__reset.jsonl");
const BAD_THEN_GOOD_IN: &str = include_str!("wat_mcp__bad_then_good.jsonl");
const STALE_TICKET_IN: &str = include_str!("wat_mcp__stale_ticket.jsonl");

/// One live `--mcp` child. Turns are request/reply: the next frame is
/// written only after the previous Turn is read, because that Turn
/// carries the ticket the next frame must present.
struct Live {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    ticket: i64,
}

struct Reply {
    value: String,
    is_error: bool,
    turn: TurnView,
    line: String,
    /// `content[0].text` — the string the host forwards to the model.
    text: String,
}

struct TurnView {
    gen: i64,
    defs: i64,
    ticket: i64,
    value: String,
}

impl Live {
    fn start() -> Self {
        let bin = env!("CARGO_BIN_EXE_wat");
        let mut child = Command::new(bin)
            .arg("--mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn wat --mcp");
        let stdin = child.stdin.take().expect("mcp stdin");
        let stdout = BufReader::new(child.stdout.take().expect("mcp stdout"));
        Self {
            child,
            stdin,
            stdout,
            ticket: 0,
        }
    }

    fn roundtrip(&mut self, frame: &str) -> Reply {
        writeln!(self.stdin, "{frame}").expect("write mcp frame");
        self.stdin.flush().expect("flush mcp frame");
        let mut line = String::new();
        let n = self.stdout.read_line(&mut line).expect("read mcp reply");
        assert!(n > 0, "mcp server closed stdout; frame was {frame}");
        parse_reply(&line)
    }

    fn play(&mut self, jsonl: &str) -> Vec<Reply> {
        let mut out = Vec::new();
        for line in jsonl.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let req = wat_edn::from_json_string(line).expect("fixture line is JSON");
            let framed = inject_ticket(req, self.ticket);
            let reply = self.roundtrip(&wat_edn::to_json_string(&framed));
            self.ticket = reply.turn.ticket;
            out.push(reply);
        }
        out
    }

    fn eval_ticket(&mut self, edn: &str, ticket: i64) -> Reply {
        let frame = eval_frame(edn, ticket);
        let reply = self.roundtrip(&frame);
        self.ticket = reply.turn.ticket;
        reply
    }
}

impl Drop for Live {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn eval_frame(edn: &str, ticket: i64) -> String {
    wat_edn::to_json_string(&map(vec![
        ("jsonrpc", OwnedValue::String("2.0".into())),
        ("id", OwnedValue::Integer(1)),
        ("method", OwnedValue::String("tools/call".into())),
        (
            "params",
            map(vec![
                ("name", OwnedValue::String("eval".into())),
                (
                    "arguments",
                    map(vec![
                        ("edn", OwnedValue::String(edn.to_string().into())),
                        ("ticket", OwnedValue::Integer(ticket)),
                    ]),
                ),
            ]),
        ),
    ]))
}

fn inject_ticket(req: OwnedValue, ticket: i64) -> OwnedValue {
    let params = get(&req, "params").cloned().unwrap_or_else(|| map(vec![]));
    let args = get(&params, "arguments")
        .cloned()
        .unwrap_or_else(|| map(vec![]));
    let args = map_assoc(args, "ticket", OwnedValue::Integer(ticket));
    let params = map_assoc(params, "arguments", args);
    map_assoc(req, "params", params)
}

fn map(entries: Vec<(&str, OwnedValue)>) -> OwnedValue {
    OwnedValue::Map(
        entries
            .into_iter()
            .map(|(k, v)| (OwnedValue::String(k.to_string().into()), v))
            .collect(),
    )
}

fn map_assoc(v: OwnedValue, key: &str, val: OwnedValue) -> OwnedValue {
    match v {
        OwnedValue::Map(mut entries) => {
            if let Some(pos) = entries
                .iter()
                .position(|(k, _)| matches!(k, OwnedValue::String(s) if s.as_ref() == key))
            {
                entries[pos].1 = val;
            } else {
                entries.push((OwnedValue::String(key.to_string().into()), val));
            }
            OwnedValue::Map(entries)
        }
        other => other,
    }
}

fn get<'a>(v: &'a OwnedValue, key: &str) -> Option<&'a OwnedValue> {
    match v {
        OwnedValue::Map(entries) => entries.iter().find_map(|(k, val)| match k {
            OwnedValue::String(s) if s.as_ref() == key => Some(val),
            _ => None,
        }),
        _ => None,
    }
}

fn call_edn(line: &str) -> String {
    let v = wat_edn::from_json_string(line).expect("fixture line is JSON");
    let params = get(&v, "params").expect("params");
    let args = get(params, "arguments").expect("arguments");
    match get(args, "edn") {
        Some(OwnedValue::String(s)) => s.to_string(),
        other => panic!("arguments.edn must be a string; got {other:?}"),
    }
}

fn parse_reply(line: &str) -> Reply {
    let v = wat_edn::from_json_string(line).expect("a reply frame must be JSON");
    let result = get(&v, "result").expect("a tools/call reply carries a result");
    let content = match get(result, "content") {
        Some(OwnedValue::Vector(items)) => items.clone(),
        other => panic!("content must be a vector; got {other:?}"),
    };
    let text = match content.first().and_then(|c| get(c, "text")) {
        Some(OwnedValue::String(s)) => s.to_string(),
        other => panic!("content[0].text must be a string; got {other:?}"),
    };
    let is_error = matches!(get(result, "isError"), Some(OwnedValue::Bool(true)));
    let turn = parse_turn(&text);
    Reply {
        value: turn.value.clone(),
        is_error,
        turn,
        line: line.to_string(),
        text,
    }
}

/// Epoch fields (`:gen`, `:ticket`) vary per process. Zero them on the
/// parsed map so the rest of the Turn can sit in a golden. Missing keys
/// stay missing — a Turn that dropped `:ticket` cannot grow one here.
fn zero_epoch_fields(text: &str) -> String {
    let v = wat_edn::parse_owned(text)
        .unwrap_or_else(|e| panic!("model-visible text must be EDN: {e}; got {text}"));
    match v {
        OwnedValue::Tagged(tag, inner) => {
            let map = match *inner {
                OwnedValue::Map(entries) => entries,
                other => panic!("Turn body must be a map; got {other:?}"),
            };
            let rewritten = map
                .into_iter()
                .map(|(k, val)| {
                    let epoch = matches!(
                        &k,
                        OwnedValue::Keyword(kw)
                            if kw.namespace().is_none()
                                && (kw.name() == "gen" || kw.name() == "ticket")
                    );
                    if epoch {
                        (k, OwnedValue::Integer(0))
                    } else {
                        (k, val)
                    }
                })
                .collect();
            wat_edn::write(&OwnedValue::Tagged(
                tag,
                Box::new(OwnedValue::Map(rewritten)),
            ))
        }
        other => panic!("model-visible text must be a tagged Turn; got {other:?}"),
    }
}

fn parse_turn(text: &str) -> TurnView {
    let v = wat_edn::parse_owned(text)
        .unwrap_or_else(|e| panic!("turn text must be EDN: {e}; got {text}"));
    match v {
        OwnedValue::Tagged(tag, inner) if tag.namespace() == "wat.mcp" && tag.name() == "Turn" => {
            let map = match *inner {
                OwnedValue::Map(entries) => entries,
                other => panic!("Turn body must be a map; got {other:?}"),
            };
            let mut gen = None;
            let mut defs = None;
            let mut ticket = None;
            let mut value = None;
            for (k, val) in &map {
                let kw = match k {
                    OwnedValue::Keyword(kw) if kw.namespace().is_none() => kw.name(),
                    _ => continue,
                };
                match kw {
                    "gen" => {
                        gen = match val {
                            OwnedValue::Integer(n) => Some(*n),
                            other => panic!("Turn :gen must be an integer; got {other:?}"),
                        };
                    }
                    "defs" => {
                        defs = match val {
                            OwnedValue::Integer(n) => Some(*n),
                            other => panic!("Turn :defs must be an integer; got {other:?}"),
                        };
                    }
                    "ticket" => {
                        ticket = match val {
                            OwnedValue::Integer(n) => Some(*n),
                            other => panic!("Turn :ticket must be an integer; got {other:?}"),
                        };
                    }
                    "value" => value = Some(wat_edn::write(val)),
                    _ => {}
                }
            }
            TurnView {
                gen: gen.unwrap_or_else(|| panic!("Turn must carry :gen; got {text}")),
                defs: defs.unwrap_or_else(|| panic!("Turn must carry :defs; got {text}")),
                ticket: ticket.unwrap_or_else(|| panic!("Turn must carry :ticket; got {text}")),
                value: value.unwrap_or_else(|| panic!("Turn must carry :value; got {text}")),
            }
        }
        _ => panic!("reply text must be #wat.mcp/Turn so the model sees :gen; got {text}"),
    }
}

/// Session epoch out of one reply frame — top-level `gen`, falling back to
/// `result.gen`. Parsed STRUCTURALLY. A missing `gen` is a red: the death-as-
/// empty-world hole is exactly "the reply does not name the process."
fn frame_gen(line: &str) -> i64 {
    let v = wat_edn::from_json_string(line).expect("a reply frame must be JSON");
    if let Some(OwnedValue::Integer(n)) = get(&v, "gen") {
        return *n;
    }
    if let Some(result) = get(&v, "result") {
        if let Some(OwnedValue::Integer(n)) = get(result, "gen") {
            return *n;
        }
    }
    panic!("reply must carry gen; got {line}");
}

fn tagged_ns_name(text: &str, ns: &str, name: &str) -> bool {
    match wat_edn::parse_owned(text) {
        Ok(OwnedValue::Tagged(tag, _)) => tag.namespace() == ns && tag.name() == name,
        _ => false,
    }
}

fn is_core_fault(text: &str) -> bool {
    tagged_ns_name(text, "wat.core", "Fault")
}

fn is_stale_ticket(text: &str) -> bool {
    let v = match wat_edn::parse_owned(text) {
        Ok(v) => v,
        Err(_) => return false,
    };
    match v {
        OwnedValue::Tagged(tag, inner) if tag.namespace() == "wat.mcp" && tag.name() == "Fault" => {
            let map = match *inner {
                OwnedValue::Map(entries) => entries,
                _ => return false,
            };
            map.iter().any(|(k, val)| match (k, val) {
                (OwnedValue::Keyword(kw), OwnedValue::Keyword(kind))
                    if kw.namespace().is_none()
                        && kw.name() == "kind"
                        && kind.namespace().is_none()
                        && kind.name() == "stale-ticket" =>
                {
                    true
                }
                _ => false,
            })
        }
        _ => false,
    }
}

fn pair(r: &Reply) -> (&str, bool) {
    (r.value.as_str(), r.is_error)
}

#[test]
fn definitions_persist_across_turns() {
    // THE load-bearing property. Call 1 declares; call 2 uses it. Only a session that
    // actually grew its definition set can answer 42 here.
    let mut live = Live::start();
    let r = live.play(PERSIST_IN);
    assert_eq!(r.len(), 2, "one reply per request");
    assert_eq!(pair(&r[0]), ("nil", false), "a declaration answers nil");
    assert_eq!(
        pair(&r[1]),
        ("42", false),
        "a definition from an earlier call must be live in a later one"
    );
}

#[test]
fn reset_empties_the_session() {
    // The same mechanism inverted — asserting BOTH sides so a `reset` that does nothing
    // cannot pass: it must work before, and it must stop working after.
    let mut live = Live::start();
    let r = live.play(RESET_IN);
    assert_eq!(r.len(), 4);
    assert_eq!(pair(&r[1]), ("42", false), "live before the reset");
    assert_eq!(pair(&r[2]), ("nil", false), "reset answers nil");
    assert!(
        !r[3].is_error,
        "a missing definition is a Turn value, not a failed tool call: {:?}",
        r[3].value
    );
    assert!(
        is_core_fault(&r[3].value),
        "after reset the definition is gone: got {}",
        r[3].value
    );
}

#[test]
fn a_failed_evaluation_is_not_fatal() {
    // A failed evaluation is a SUCCESSFUL tool call reporting a failure — the session
    // survives it. If a bad form killed the process, the second reply would never arrive.
    // If isError were true, Grok would prefix `Failed to call eval:` and the Fault
    // would stop being a value (measured 2026-08-16).
    let mut live = Live::start();
    let r = live.play(BAD_THEN_GOOD_IN);
    assert_eq!(r.len(), 2, "the session must answer BOTH calls");
    assert!(
        !r[0].is_error,
        "a domain Fault must not set isError (Grok prefixes the call as failed): {}",
        r[0].value
    );
    assert!(
        is_core_fault(&r[0].value),
        "an unresolved reference is a #wat.core/Fault: got {}",
        r[0].value
    );
    assert_eq!(
        pair(&r[1]),
        ("7", false),
        "the session must survive and answer the next call"
    );
}

#[test]
fn the_payload_is_edn_not_json() {
    // The ruling this mode exists to honour: EDN in, EDN out. A record answer comes back as
    // EDN TEXT in the string slot — `#ns/Rec {…}` — never converted into a JSON object.
    let mut live = Live::start();
    let r = live.play(include_str!("wat_mcp__record.jsonl"));
    let last = r.last().expect("a reply");
    assert!(
        !last.is_error,
        "the record form must evaluate: {}",
        last.value
    );
    // Compared STRUCTURALLY against a captured golden, not by prefix: the claim is that the
    // payload is an EDN VALUE, and only parsing both sides proves that. A `starts_with` here
    // would pass on a truncated or malformed tail.
    wat::assert_edn_matches_file!(last.value.clone(), "wat_mcp__record.edn");
    // Arc 296 G-2 — the golden USED TO record a defect, deliberately and visibly: the field
    // names came back as `:field-0`/`:field-1`, not the declared `:x`/`:y`, because the
    // renderer recovered names via a registry lookup that this session's symbol table (never
    // having seen the `defrecord`) could not satisfy. `AggregateValue` now carries its own
    // `names` at construction, so the value no longer depends on that lookup — the golden is
    // updated to the real declared names (`:x`/`:y`), and this comparison now checks what the
    // test's docstring always claimed: EDN in, EDN out, faithfully.
}

#[test]
fn every_form_in_a_payload_takes_effect() {
    // REGRESSION GATE for a shipped hidden failure: the first version evaluated only the
    // FIRST form of a payload and silently dropped the rest, answering `nil` + `isError:
    // false` — success — while the later definitions never existed. Nothing in the original
    // suite could see it, because every fixture sent exactly one form (R59's third face: a
    // gate whose success criteria never touch the mechanism).
    let mut live = Live::start();
    let r = live.play(MULTIFORM_IN);
    assert_eq!(r.len(), 3);
    assert_eq!(pair(&r[0]), ("nil", false), "two declarations answer nil");
    // Only a session where BOTH defns landed can answer 3. Restore
    // `forms.into_iter().next()` in `eval_turn` and this goes red.
    assert_eq!(
        pair(&r[1]),
        ("3", false),
        "form 2 of the payload must take effect, not be dropped"
    );
    // …and a failure mid-payload is REPORTED, not swallowed behind an earlier success.
    assert!(
        !r[2].is_error,
        "a mid-payload Fault is a Turn value, not a failed tool call: {}",
        r[2].value
    );
    assert!(
        is_core_fault(&r[2].value),
        "a failing form later in the payload must surface: got {}",
        r[2].value
    );
}

#[test]
fn a_toplevel_let_or_do_answers_its_value() {
    // `let` and `do` are EXPRESSIONS. A top-level one used to be classified
    // `FormOutcome::Declared` — because one list was answering both "might this carry a def?"
    // (yes, they splice) and "is this a declaration?" (no) — so its value was discarded:
    // `--mcp` answered `nil`, `--repl` printed nothing. Reported by a zero-prior model driving
    // this tool live, then reproduced in both modes.
    //
    // Restore `is_runtime_declaration_head` at either site in `eval_form_against_defs` and the
    // first two of these go red. The third is the control that kept the bug hidden: a NESTED
    // `let` always worked, which is why "let is broken" looked wrong on inspection.
    let mut live = Live::start();
    let r = live.play(TOPLEVEL_EXPR_IN);
    assert_eq!(r.len(), 3);
    assert_eq!(
        pair(&r[0]),
        ("1", false),
        "a top-level let answers its body"
    );
    assert_eq!(
        pair(&r[1]),
        ("3", false),
        "a top-level do answers its last form"
    );
    assert_eq!(
        pair(&r[2]),
        ("\"wat::core::i64\"", false),
        "the nested-let control must stay green"
    );
}

#[test]
fn every_reply_carries_the_same_gen() {
    let mut live = Live::start();
    let r = live.play(PERSIST_IN);
    assert_eq!(r.len(), 2);
    assert_ne!(
        r[0].turn.gen, 0,
        "gen is a real epoch, not a zero placeholder"
    );
    assert_eq!(
        r[0].turn.gen, r[1].turn.gen,
        "same process must not flip gen between turns"
    );
    assert_eq!(frame_gen(&r[0].line), r[0].turn.gen);
    // The model never sees the JSON envelope. The TEXT must be a Turn or
    // a respawn is invisible again (measured: Grok forwards only content[0].text).
    for reply in &r {
        assert!(
            !reply.is_error,
            "a Turn is never isError (Grok would prefix Failed to call): {}",
            reply.value
        );
        assert_ne!(
            reply.turn.ticket, 0,
            "a completed turn mints a non-bootstrap ticket"
        );
        assert!(
            reply.turn.ticket < (1i64 << 53),
            "ticket must stay inside JSON's exact-integer range; got {}",
            reply.turn.ticket
        );
    }
    wat::assert_edn_matches_file!(zero_epoch_fields(&r[0].text), "wat_mcp__turn_decl.edn");
    wat::assert_edn_matches_file!(zero_epoch_fields(&r[1].text), "wat_mcp__turn_call.edn");
    assert_ne!(
        r[0].turn.ticket, r[1].turn.ticket,
        "each accepted turn must mint a new ticket"
    );
}

#[test]
fn a_new_process_mints_a_new_gen() {
    // A respawn that kept the same gen would make the empty world look like
    // the old session. Two `serve()`s must disagree.
    let mut a = Live::start();
    let mut b = Live::start();
    let ra = a.play(TOPLEVEL_EXPR_IN);
    let rb = b.play(TOPLEVEL_EXPR_IN);
    let ga = ra[0].turn.gen;
    let gb = rb[0].turn.gen;
    assert_ne!(ga, gb, "a new process must mint a new gen ({ga} vs {gb})");
}

#[test]
fn a_reused_ticket_is_a_protocol_fault() {
    // THE load-bearing protocol gate. Two evals presenting ticket 0: the first
    // consumes it, the second is `#wat.mcp/Fault {:kind :stale-ticket}` and
    // does not evaluate. Delete reject_stale_ticket and both forms land → red.
    // A tool-description sentence is not this test.
    let forms: Vec<&str> = STALE_TICKET_IN
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert_eq!(forms.len(), 2);
    let first = call_edn(forms[0]);
    let second = call_edn(forms[1]);

    let mut live = Live::start();
    let r1 = live.eval_ticket(&first, 0);
    assert_eq!(
        pair(&r1),
        ("1", false),
        "the first bootstrap ticket is accepted"
    );
    assert_ne!(r1.turn.ticket, 0, "accepted turn mints a new ticket");

    let minted = r1.turn.ticket;
    let r2 = live.eval_ticket(&second, 0);
    assert!(
        !r2.is_error,
        "a protocol Fault is still a successful tool result: {}",
        r2.value
    );
    assert!(
        is_stale_ticket(&r2.value),
        "reused ticket must be #wat.mcp/Fault :stale-ticket; got {}",
        r2.value
    );
    assert_eq!(
        r2.turn.ticket, minted,
        "a stale ticket must not consume the live rendezvous"
    );
    assert_eq!(
        r2.turn.defs, r1.turn.defs,
        "a stale ticket must not evaluate — defs stay put"
    );

    // Recovery: the still-valid ticket evaluates the second form.
    let r3 = live.eval_ticket(&second, minted);
    assert_eq!(
        pair(&r3),
        ("2", false),
        "the form must still be evaluable once the live ticket is presented"
    );
}

#[test]
fn a_missing_ticket_is_a_protocol_fault() {
    // The host that has not learned the new argument still fires. The reply
    // must name the violation; silently accepting a missing ticket would
    // make dual-fire-without-ticket unobservable again.
    let mut live = Live::start();
    let bare = PERSIST_IN.lines().next().expect("persist fixture");
    let r = live.roundtrip(bare);
    assert!(!r.is_error, "missing ticket is a Turn, not isError");
    assert!(
        is_stale_ticket(&r.value),
        "missing ticket must be #wat.mcp/Fault :stale-ticket; got {}",
        r.value
    );
    assert_eq!(r.turn.ticket, 0, "bootstrap ticket stays until consumed");
    assert_eq!(r.turn.defs, 0, "the undeclared form must not have landed");

    // The session is still at ticket 0, so a correct first turn can proceed.
    let r2 = live.play(PERSIST_IN);
    assert_eq!(pair(&r2[1]), ("42", false));
}

#[test]
fn mcp_rejects_a_positional() {
    // Same arity contract as `--repl`: the program is baked, so a path would be a silent lie
    // about what runs.
    let bin = env!("CARGO_BIN_EXE_wat");
    let out = Command::new(bin)
        .arg("--mcp")
        .arg("some_program.wat")
        .stdin(Stdio::null())
        .output()
        .expect("spawn wat --mcp with a positional");
    assert_eq!(out.status.code(), Some(64), "usage error is EX_USAGE (64)");
}
