//! `wat --mcp` — an MCP server exposing the wat REPL as one tool.
//!
//! # The shape, stated first because everything else is detail
//!
//! EDN string in, EDN string out. An LLM emits EDN; the harness carries it here inside a
//! JSON-RPC frame; wat evaluates it against the session's accumulated definitions; the
//! result goes back as EDN text in the reply's one string slot, and the model reads it as
//! tokens. **The payload is never converted to JSON.** It rides inside a JSON string as
//! characters, exactly as it was written:
//!
//! ```text
//! in:  {"jsonrpc":"2.0","id":1,"method":"tools/call",
//!       "params":{"name":"eval","arguments":{"edn":"(:wat::core::+ 2 2)","ticket":0}}}
//! out: {"jsonrpc":"2.0","id":1,
//!       "result":{"content":[{"type":"text",
//!         "text":"#wat.mcp/Turn {:gen 1 :defs 0 :ticket 881 :value 4}"}],"isError":false}}
//! ```
//!
//! The JSON envelope is a constant. The `text` is always `#wat.mcp/Turn` —
//! gen, def-count, ticket, and the EDN value. Grok (and friends) forward
//! only that string to the model; an epoch that lives only on the JSON
//! object is invisible, and a replaced process looks like a working
//! server. The Turn makes a silent respawn unrepresentable.
//!
//! `isError` is always false on a Turn. A failed evaluation is a value
//! (`:value` is `#wat.core/Fault`); Grok prefixes `Failed to call eval:`
//! when `isError` is true, which would turn a navigable Fault into a
//! transport failure. Envelope faults (no `edn`, unknown tool) stay
//! JSON-RPC errors. They never become a Turn.
//!
//! `ticket` is the rendezvous that makes "two evals before reading" a
//! value instead of a hope. The next `eval`/`reset` must present the
//! last Turn's `:ticket` (or `0` if none has been read). A second call
//! with the same ticket is `#wat.mcp/Fault {:kind :stale-ticket}` and
//! is not evaluated. The server cannot see a "model turn"; it can see
//! a ticket that was already consumed. That is the violation.
//!
//! # Why the loop is HERE and not in a `wat/mcp.wat`
//!
//! Because JSON is not EDN, and wat's stdin/stdout are strict-EDN data channels by
//! construction (arc 278 R51, typed-Unix). A wat `println` EDN-*encodes* what it is handed:
//! printing a JSON frame delivers `"{\"jsonrpc\":\"2.0\"…}"` — an escaped EDN string
//! literal, not a JSON object (measured). That is the channel correctly refusing to carry a
//! foreign format, so the bridge belongs at the transport, beside argv and the frame reader.
//!
//! # What is NOT duplicated
//!
//! The turn. `runtime::eval_form_against_defs` is the same function `:wat::eval-with-defs!`
//! calls, so `--mcp` and `--repl` cannot drift on classification, on which arm grows the
//! definition set, or on what a failure looks like. This module owns the codec and the
//! protocol; it owns none of the semantics.
//!
//! # Honest limits
//!
//! `PROTOCOL_VERSION` and the `result` envelope shape are taken from the MCP specification
//! and have **not** been measured against a live harness in this repo. The end-to-end gate
//! against a real client is what settles them; until then they are the one part of this file
//! standing on a claim rather than a run.

use std::io::{BufRead, Write};
use std::process::ExitCode;
use std::sync::Arc;

use wat_edn::OwnedValue;

use crate::runtime::{Environment, SymbolTable, Value};
use crate::WatAST;

/// Reported to the client at `initialize`. From the MCP spec — see the honest-limits note
/// in the module header; this is the one constant here not proven by a run.
const PROTOCOL_VERSION: &str = "2024-11-05";

/// The session. `defs` is the accumulated definition set — the exact `Vector<WatAST>`
/// `wat --repl` threads through its tail call, and it grows on precisely one outcome
/// (`Declared`), which is the property the gate has to prove.
///
/// `gen` is this process's session epoch. Minted once at `serve` start, never
/// persisted, never bumped on `reset`. One stdio, one process, one gen. A
/// harness that *replaces* a dead child starts a new process on a new pipe and
/// mints a new number — that is the only way gen changes. The model can then
/// see the world flipped instead of treating a virgin session as the old one.
///
/// `sym` is TCO'd into the next turn: `runtime_def_values` (a `def` of
/// a handle, a minted uuid, a bound peer) are not rebuilt. `reset`
/// restores `baseline`. `ticket` is the next-turn rendezvous (0 if no
/// Turn has been issued). `ticket_seq` is only entropy for the mint.
struct Session {
    defs: Vec<WatAST>,
    env: Environment,
    /// Live world. TCO'd into the next turn so `runtime_def_values`
    /// (handles, minted uuids, bound peers) exist until `reset`.
    sym: SymbolTable,
    /// Virgin table. `reset` restores this; a new process mints a new gen.
    baseline: SymbolTable,
    gen: i64,
    ticket: i64,
    ticket_seq: u64,
}

/// One epoch per process lifetime. Not a shared counter — there is no second
/// process on this stdin. Hash(pid, nanos) so two boots in the same
/// millisecond cannot collide.
fn mint_gen() -> i64 {
    use std::hash::{BuildHasher, Hasher, RandomState};
    use std::time::{SystemTime, UNIX_EPOCH};
    // Hash, not millis+pid: two children spawned in the same millisecond
    // must not agree. Stay inside JSON's 2^53 exact integers.
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u32(std::process::id());
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(1);
    hasher.write_u128(nanos);
    ((hasher.finish() & JSON_SAFE_INT) as i64).max(1)
}

/// JSON numbers are only exact integers inside 2^53 (the same wall that
/// forced `mint_gen` off nanos). A ticket outside that range arrives as
/// a float or as `nil`, and every subsequent call looks stale.
const JSON_SAFE_INT: u64 = (1u64 << 53) - 1;

/// A fresh ticket. Never 0 (0 is only the unread-session bootstrap) and
/// never the ticket just consumed. RandomState keys are process-secret,
/// so the next ticket cannot be computed from the Turn the model already
/// holds — predicting it is how a client would dual-fire without reading.
fn next_ticket(session: &mut Session) -> i64 {
    use std::hash::{BuildHasher, Hasher, RandomState};
    session.ticket_seq = session.ticket_seq.saturating_add(1);
    let mut hasher = RandomState::new().build_hasher();
    hasher.write_u64(session.ticket_seq);
    hasher.write_i64(session.gen);
    hasher.write_u32(std::process::id());
    let t = ((hasher.finish() & JSON_SAFE_INT) as i64).max(1);
    if t == session.ticket {
        (t % (JSON_SAFE_INT as i64 - 1)).saturating_add(1).max(1)
    } else {
        t
    }
}

fn consume_ticket(session: &mut Session) {
    session.ticket = next_ticket(session);
}

pub(super) fn serve() -> ExitCode {
    // A baseline world, for its symbol table: `eval_form_against_defs` reads the session's
    // config off it and decodes freeze errors against it. The definitions themselves live in
    // `defs` and are re-frozen per turn — that is the oracle's deliberate slowness, not an
    // oversight (R1/R9); a fast incremental plane gets built behind a differential later.
    let loader: Arc<dyn crate::load::loader::SourceLoader> = Arc::new(crate::load::loader::InMemoryLoader::new());
    let base = match crate::freeze::startup_from_forms(Vec::new(), None, loader) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("wat --mcp: could not establish a baseline world: {e}");
            return ExitCode::from(70); // EX_SOFTWARE
        }
    };

    let mut session = Session {
        defs: Vec::new(),
        env: Environment::new(),
        sym: base.symbols.clone(),
        baseline: base.symbols,
        gen: mint_gen(),
        ticket: 0,
        ticket_seq: 0,
    };

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            // stdin died under us. The harness is gone; there is nobody to tell.
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        // A notification (no `id`) gets no reply — that is the JSON-RPC contract, not a
        // silent drop.
        if let Some(reply) = handle_line(&line, &mut session) {
            if writeln!(stdout, "{reply}").is_err() || stdout.flush().is_err() {
                break;
            }
        }
    }
    ExitCode::from(0)
}

/// One frame in, at most one frame out.
fn handle_line(line: &str, session: &mut Session) -> Option<String> {
    let req = match wat_edn::from_json_string(line) {
        Ok(v) => v,
        // Malformed JSON has no `id` to answer against, so the error is id-less. It must not
        // be fatal: one bad byte from a remote harness ending the session is the exact
        // failure `read-string` and `read-json` were both made total to prevent.
        Err(e) => {
            return Some(error_frame(
                OwnedValue::Nil,
                -32700,
                &format!("parse error: {e}"),
                session.gen,
            ));
        }
    };

    let id = get(&req, "id").cloned().unwrap_or(OwnedValue::Nil);
    let method = match get(&req, "method").and_then(as_str) {
        Some(m) => m.to_string(),
        None => {
            return Some(error_frame(
                id,
                -32600,
                "invalid request: no method",
                session.gen,
            ))
        }
    };

    // Notifications carry no id and expect no reply.
    let is_notification = matches!(id, OwnedValue::Nil);

    match method.as_str() {
        "initialize" => Some(result_frame(id, initialize_result(), session.gen)),
        "tools/list" => Some(result_frame(id, tools_list_result(), session.gen)),
        "tools/call" => Some(handle_tools_call(id, &req, session)),
        _ if is_notification => None,
        other => Some(error_frame(
            id,
            -32601,
            &format!("unknown method: {other}"),
            session.gen,
        )),
    }
}

fn handle_tools_call(id: OwnedValue, req: &OwnedValue, session: &mut Session) -> String {
    let gen = session.gen;
    let params = match get(req, "params") {
        Some(p) => p,
        None => return error_frame(id, -32602, "tools/call: no params", gen),
    };
    let name = match get(params, "name").and_then(as_str) {
        Some(n) => n.to_string(),
        None => return error_frame(id, -32602, "tools/call: no tool name", gen),
    };
    let arguments = get(params, "arguments");

    match name.as_str() {
        "eval" => {
            // THE PAYLOAD. Already a string; it stays one.
            let src = match arguments.and_then(|a| get(a, "edn")).and_then(as_str) {
                Some(s) => s.to_string(),
                None => return error_frame(id, -32602, "eval: no `edn` argument", gen),
            };
            if let Some(fault) = reject_stale_ticket(arguments, session) {
                return result_frame(id, tool_result(&fault), gen);
            }
            // A panic in the turn must become a reply, not a dead process. The
            // REPL already claims survive-a-bad-line; `--mcp` was exiting 0 on
            // unwind and the next client saw a virgin world. `AssertUnwindSafe`
            // is the honest cost: we do not persist a half-mutated `defs` across
            // the catch (eval_turn only pushes on Declared after the eval returns).
            let text = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                eval_turn(&src, session)
            })) {
                Ok(text) => text,
                Err(payload) => {
                    let msg = panic_text(&payload);
                    format!("#wat.core/Fault {{:message \"session survived a panic: {msg}\"}}")
                }
            };
            consume_ticket(session);
            result_frame(
                id,
                tool_result(&render_turn(gen, session.defs.len(), session.ticket, &text)),
                gen,
            )
        }
        // `reset` is the definition set inverted — the same field the `Declared` arm grows,
        // emptied. `gen` is NOT bumped: the process is the same, the model asked for this.
        "reset" => {
            if let Some(fault) = reject_stale_ticket(arguments, session) {
                return result_frame(id, tool_result(&fault), gen);
            }
            session.defs.clear();
            session.sym = session.baseline.clone();
            consume_ticket(session);
            result_frame(
                id,
                tool_result(&render_turn(gen, 0, session.ticket, "nil")),
                gen,
            )
        }
        other => error_frame(id, -32602, &format!("unknown tool: {other}"), gen),
    }
}

/// `None` = the presented ticket matches; caller may run the turn.
/// `Some(turn_text)` = stale or missing; world and ticket unchanged.
fn reject_stale_ticket(arguments: Option<&OwnedValue>, session: &Session) -> Option<String> {
    let presented = arguments.and_then(|a| get(a, "ticket")).and_then(as_i64);
    if presented == Some(session.ticket) {
        return None;
    }
    Some(render_turn(
        session.gen,
        session.defs.len(),
        session.ticket,
        &stale_ticket_value(session.ticket, presented),
    ))
}

fn stale_ticket_value(expected: i64, got: Option<i64>) -> String {
    match got {
        Some(g) => format!("#wat.mcp/Fault {{:kind :stale-ticket :expected {expected} :got {g}}}"),
        None => format!("#wat.mcp/Fault {{:kind :stale-ticket :expected {expected} :got nil}}"),
    }
}

fn panic_text(payload: &Box<dyn std::any::Any + Send>) -> String {
    crate::runtime::format_panic_payload(payload)
}

/// The turn: parse the payload, evaluate EVERY form in it against the session, render the
/// last answer as EDN.
///
/// ⚠ THE BUG THIS SHAPE FIXES, kept visible because it is the class this arc exists to kill.
/// The first version took `forms.into_iter().next()` — the FIRST form — and silently discarded
/// the rest. A payload of two `defn`s answered `nil` while the second definition never
/// existed. A caller could not tell. Found by driving the live tool, not by the gate: all
/// five tests sent one form per payload, so nothing in the suite ever depended on the
/// mechanism — R59's third face, in the module that cites R59.
///
/// Forms run IN ORDER against a definition set that grows as they go, so a `defrecord` in the
/// first form is in scope for the second. The LAST form's value is the answer, mirroring an
/// implicit `do` — the same rule the turn already applies to a single form's residue.
///
/// A failure STOPS the sequence and is returned. Continuing past one would run later forms
/// against a world that is not the one they were written for, and report a success built on it.
/// The failure is still a Turn value, not an MCP `isError`.
fn eval_turn(src: &str, session: &mut Session) -> String {
    let forms = match crate::parser::parse_all_with_file(src, "<mcp>") {
        Ok(f) => f,
        Err(e) => return format!("{e}"),
    };
    // An empty payload is not a failure; it is a turn that said nothing.
    let mut answer = "nil".to_string();
    for form in forms {
        let (text, halt) = eval_one_form(form, session);
        answer = text;
        if halt {
            return answer;
        }
    }
    answer
}

/// One form against the session — the whole of a turn when the payload holds a single form.
/// The bool is "stop the remaining forms", not MCP `isError`.
fn eval_one_form(form: WatAST, session: &mut Session) -> (String, bool) {
    let (outcome, next_sym) = match crate::runtime::eval_form_against_defs(
        &form,
        session.defs.clone(),
        &session.env,
        &session.sym,
    ) {
        Ok(pair) => pair,
        // A signal (a process-wide stop) is not a turn outcome — it unwinds.
        Err(e) => return (format!("{e:?}"), true),
    };

    let (text, halt, keep) = {
        let types = session.sym.types().map(|a| a.as_ref());
        match &outcome {
            Value::Enum(ev) => match ev.variant_name.as_str() {
                // The ONLY arm that grows the AST set. The live table is TCO'd
                // separately so a `def` of a handle is not rebuilt next turn.
                "Declared" => {
                    session.defs.push(form);
                    ("nil".to_string(), false, true)
                }
                "Evaluated" => {
                    let v = ev.fields.first().cloned().unwrap_or(Value::Unit);
                    (render_edn(&v, types), false, true)
                }
                // CheckFailed / Raised are values. The session survives them
                // and is untouched (same as --repl).
                _ => {
                    let v = ev.fields.first().cloned().unwrap_or(Value::Unit);
                    (render_edn(&v, types), true, false)
                }
            },
            other => (render_edn(other, types), false, true),
        }
    };
    if keep {
        if let Some(sym) = next_sym {
            session.sym = sym;
        }
    }
    (text, halt)
}

fn render_edn(v: &Value, types: Option<&crate::types::TypeEnv>) -> String {
    wat_edn::write(&crate::edn::render::value_to_edn_with(v, types))
}

/// The text the model actually reads. Harnesses (Grok measured 2026-08-16)
/// forward `content[0].text`; Grok also tolerated an envelope-level `gen`, but
/// Claude Code (measured 2026-08-20) rejects the frame outright — see `with_gen`. A Turn that omits
/// `:gen` or `:ticket` cannot be constructed from this function — the
/// epoch and the rendezvous are mandatory.
fn render_turn(gen: i64, defs: usize, ticket: i64, value_edn: &str) -> String {
    format!("#wat.mcp/Turn {{:gen {gen} :defs {defs} :ticket {ticket} :value {value_edn}}}")
}

// ─── the envelope ────────────────────────────────────────────────────────────────────────
// Built as `OwnedValue` and serialized once, rather than spliced as text: the EDN payload
// routinely contains double quotes (`#some.ns/Rec {:field "val"}`), so hand-interpolating it
// into a JSON skeleton would be one escaping mistake away from a corrupt frame.

fn result_frame(id: OwnedValue, result: OwnedValue, gen: i64) -> String {
    wat_edn::to_json_string(&map(vec![
        ("jsonrpc", OwnedValue::String("2.0".into())),
        ("id", id),
        ("result", with_gen(result, gen)),
    ]))
}

fn error_frame(id: OwnedValue, code: i64, message: &str, _gen: i64) -> String {
    wat_edn::to_json_string(&map(vec![
        ("jsonrpc", OwnedValue::String("2.0".into())),
        ("id", id),
        (
            "error",
            map(vec![
                ("code", OwnedValue::Integer(code)),
                ("message", OwnedValue::String(message.to_string().into())),
            ]),
        ),
    ]))
}

/// Stamp `gen` onto a result map so a client that only forwards `result` still
/// sees the epoch. This is the ONLY place `gen` rides the wire outside the Turn
/// text: the JSON-RPC *envelope* must carry nothing but `jsonrpc`/`id`/
/// `result`|`error`. Claude Code validates every inbound frame against the MCP
/// SDK's `JSONRPCResponseSchema`, which is `.strict()` — one extra top-level key
/// and the frame matches no arm of the message union, is dropped as a parse
/// error, and the request hangs until it times out ("Failed to reconnect to
/// wat: Request timed out"). `ResultSchema` is loose, so `gen` is legal HERE.
fn with_gen(result: OwnedValue, gen: i64) -> OwnedValue {
    match result {
        OwnedValue::Map(mut entries) => {
            entries.push((OwnedValue::String("gen".into()), OwnedValue::Integer(gen)));
            OwnedValue::Map(entries)
        }
        other => map(vec![("value", other), ("gen", OwnedValue::Integer(gen))]),
    }
}

/// A Turn is a successful tool result. `isError: true` is unrepresentable
/// here: Grok (measured 2026-08-16) prefixes `Failed to call eval:` and
/// the Fault stops being a value.
fn tool_result(text: &str) -> OwnedValue {
    map(vec![
        (
            "content",
            OwnedValue::Vector(vec![map(vec![
                ("type", OwnedValue::String("text".into())),
                ("text", OwnedValue::String(text.to_string().into())),
            ])]),
        ),
        ("isError", OwnedValue::Bool(false)),
    ])
}

fn initialize_result() -> OwnedValue {
    map(vec![
        (
            "protocolVersion",
            OwnedValue::String(PROTOCOL_VERSION.into()),
        ),
        ("capabilities", map(vec![("tools", map(vec![]))])),
        (
            "serverInfo",
            map(vec![
                ("name", OwnedValue::String("wat".into())),
                (
                    "version",
                    OwnedValue::String(env!("CARGO_PKG_VERSION").into()),
                ),
            ]),
        ),
    ])
}

fn tools_list_result() -> OwnedValue {
    let eval_tool = map(vec![
        ("name", OwnedValue::String("eval".into())),
        (
            "description",
            OwnedValue::String(
                "Evaluate one wat/EDN form against this session. Definitions accumulate: a \
                 form declared in one call is in scope for every later one. The result is \
                 always `#wat.mcp/Turn {:gen N :defs N :ticket T :value <edn>}` with \
                 isError false. `:gen` is this process's epoch — if it changes, the \
                 process was replaced and every prior definition is gone; re-declare; \
                 do not diagnose unresolved names as rete bugs. `:defs` is how many \
                 declarations the session holds. `:ticket` is the rendezvous for the \
                 next eval/reset: pass it as `ticket` (0 if you have read no Turn). \
                 A second call with the same ticket is not evaluated — `:value` is \
                 `#wat.mcp/Fault {:kind :stale-ticket :expected T :got G}`. `:value` \
                 is otherwise the form's EDN (nil for a declaration, #wat.core/Fault \
                 for a failed evaluation). That Fault is a value, not a failed tool call."
                    .into(),
            ),
        ),
        (
            "inputSchema",
            map(vec![
                ("type", OwnedValue::String("object".into())),
                (
                    "properties",
                    map(vec![
                        (
                            "edn",
                            map(vec![
                                ("type", OwnedValue::String("string".into())),
                                (
                                    "description",
                                    OwnedValue::String("The form to evaluate, as EDN.".into()),
                                ),
                            ]),
                        ),
                        (
                            "ticket",
                            map(vec![
                                ("type", OwnedValue::String("integer".into())),
                                (
                                    "description",
                                    OwnedValue::String(
                                        "The :ticket of the last Turn you read, or 0 \
                                         if you have read none. Reusing a ticket is a \
                                         protocol Fault; the form is not evaluated."
                                            .into(),
                                    ),
                                ),
                            ]),
                        ),
                    ]),
                ),
                (
                    "required",
                    OwnedValue::Vector(vec![
                        OwnedValue::String("edn".into()),
                        OwnedValue::String("ticket".into()),
                    ]),
                ),
            ]),
        ),
    ]);

    let reset_tool = map(vec![
        ("name", OwnedValue::String("reset".into())),
        (
            "description",
            OwnedValue::String(
                "Discard every definition made in this session. The process keeps running. \
                 Requires the last Turn's `:ticket` (or 0 if none). Replies \
                 `#wat.mcp/Turn {:gen N :defs 0 :ticket T :value nil}` — same gen, \
                 new ticket, empty defs. A stale ticket is `#wat.mcp/Fault` and \
                 does not clear the session."
                    .into(),
            ),
        ),
        (
            "inputSchema",
            map(vec![
                ("type", OwnedValue::String("object".into())),
                (
                    "properties",
                    map(vec![(
                        "ticket",
                        map(vec![
                            ("type", OwnedValue::String("integer".into())),
                            (
                                "description",
                                OwnedValue::String(
                                    "The :ticket of the last Turn you read, or 0 \
                                     if you have read none."
                                        .into(),
                                ),
                            ),
                        ]),
                    )]),
                ),
                (
                    "required",
                    OwnedValue::Vector(vec![OwnedValue::String("ticket".into())]),
                ),
            ]),
        ),
    ]);

    map(vec![(
        "tools",
        OwnedValue::Vector(vec![eval_tool, reset_tool]),
    )])
}

// ─── OwnedValue helpers ──────────────────────────────────────────────────────────────────

fn map(entries: Vec<(&str, OwnedValue)>) -> OwnedValue {
    OwnedValue::Map(
        entries
            .into_iter()
            .map(|(k, v)| (OwnedValue::String(k.to_string().into()), v))
            .collect(),
    )
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

fn as_str(v: &OwnedValue) -> Option<&str> {
    match v {
        OwnedValue::String(s) => Some(s.as_ref()),
        _ => None,
    }
}

fn as_i64(v: &OwnedValue) -> Option<i64> {
    match v {
        OwnedValue::Integer(n) => Some(*n),
        // JSON hosts sometimes deliver whole numbers as floats.
        OwnedValue::Float(f) if f.fract() == 0.0 && *f >= 0.0 && *f <= JSON_SAFE_INT as f64 => {
            Some(*f as i64)
        }
        _ => None,
    }
}
