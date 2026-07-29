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
//!       "params":{"name":"eval","arguments":{"edn":"(:wat::core::+ 2 2)"}}}
//! out: {"jsonrpc":"2.0","id":1,
//!       "result":{"content":[{"type":"text","text":"4"}],"isError":false}}
//! ```
//!
//! The envelope is a constant with two holes — the echoed `id`, and that `text`.
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
struct Session {
    defs: Vec<WatAST>,
    env: Environment,
    sym: SymbolTable,
}

pub(super) fn serve() -> ExitCode {
    // A baseline world, for its symbol table: `eval_form_against_defs` reads the session's
    // config off it and decodes freeze errors against it. The definitions themselves live in
    // `defs` and are re-frozen per turn — that is the oracle's deliberate slowness, not an
    // oversight (R1/R9); a fast incremental plane gets built behind a differential later.
    let loader: Arc<dyn crate::load::SourceLoader> = Arc::new(crate::load::InMemoryLoader::new());
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
        sym: base.symbols,
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
        Err(e) => return Some(error_frame(OwnedValue::Nil, -32700, &format!("parse error: {e}"))),
    };

    let id = get(&req, "id").cloned().unwrap_or(OwnedValue::Nil);
    let method = match get(&req, "method").and_then(as_str) {
        Some(m) => m.to_string(),
        None => return Some(error_frame(id, -32600, "invalid request: no method")),
    };

    // Notifications carry no id and expect no reply.
    let is_notification = matches!(id, OwnedValue::Nil);

    match method.as_str() {
        "initialize" => Some(result_frame(id, initialize_result())),
        "tools/list" => Some(result_frame(id, tools_list_result())),
        "tools/call" => Some(handle_tools_call(id, &req, session)),
        _ if is_notification => None,
        other => Some(error_frame(id, -32601, &format!("unknown method: {other}"))),
    }
}

fn handle_tools_call(id: OwnedValue, req: &OwnedValue, session: &mut Session) -> String {
    let params = match get(req, "params") {
        Some(p) => p,
        None => return error_frame(id, -32602, "tools/call: no params"),
    };
    let name = match get(params, "name").and_then(as_str) {
        Some(n) => n.to_string(),
        None => return error_frame(id, -32602, "tools/call: no tool name"),
    };
    let arguments = get(params, "arguments");

    match name.as_str() {
        "eval" => {
            // THE PAYLOAD. Already a string; it stays one.
            let src = match arguments.and_then(|a| get(a, "edn")).and_then(as_str) {
                Some(s) => s.to_string(),
                None => return error_frame(id, -32602, "eval: no `edn` argument"),
            };
            let (text, is_error) = eval_turn(&src, session);
            result_frame(id, tool_result(&text, is_error))
        }
        // `reset` is the definition set inverted — the same field the `Declared` arm grows,
        // emptied. Nothing else in the session is touched.
        "reset" => {
            session.defs.clear();
            result_frame(id, tool_result("nil", false))
        }
        other => error_frame(id, -32602, &format!("unknown tool: {other}")),
    }
}

/// The turn: parse the EDN, evaluate it against the session, render the answer as EDN.
/// Returns `(edn_text, is_error)`.
fn eval_turn(src: &str, session: &mut Session) -> (String, bool) {
    let forms = match crate::parser::parse_all_with_file(src, "<mcp>") {
        Ok(f) => f,
        Err(e) => return (format!("{e}"), true),
    };
    let form = match forms.into_iter().next() {
        Some(f) => f,
        // An empty payload is not a failure; it is a turn that said nothing.
        None => return ("nil".to_string(), false),
    };

    let outcome = match crate::runtime::eval_form_against_defs(
        &form,
        session.defs.clone(),
        &session.env,
        &session.sym,
    ) {
        Ok(v) => v,
        // A signal (a process-wide stop) is not a turn outcome — it unwinds.
        Err(e) => return (format!("{e:?}"), true),
    };

    let types = session.sym.types().map(|a| a.as_ref());
    match &outcome {
        Value::Enum(ev) => match ev.variant_name.as_str() {
            // The ONLY arm that grows the session. Kill this line and a definition made in
            // one call stops being visible in the next — which is what the gate must catch.
            "Declared" => {
                session.defs.push(form);
                ("nil".to_string(), false)
            }
            "Evaluated" => {
                let v = ev.fields.first().cloned().unwrap_or(Value::Unit);
                (render_edn(&v, types), false)
            }
            // CheckFailed / Raised are SUCCESSFUL tool calls reporting a failed evaluation:
            // the session survives them, which is the whole reason a REPL's failures have to
            // be values. `isError` marks the evaluation, not the transport.
            _ => {
                let v = ev.fields.first().cloned().unwrap_or(Value::Unit);
                (render_edn(&v, types), true)
            }
        },
        other => (render_edn(other, types), false),
    }
}

fn render_edn(v: &Value, types: Option<&crate::types::TypeEnv>) -> String {
    wat_edn::write(&crate::edn_shim::value_to_edn_with(v, types))
}

// ─── the envelope ────────────────────────────────────────────────────────────────────────
// Built as `OwnedValue` and serialized once, rather than spliced as text: the EDN payload
// routinely contains double quotes (`#some.ns/Rec {:field "val"}`), so hand-interpolating it
// into a JSON skeleton would be one escaping mistake away from a corrupt frame.

fn result_frame(id: OwnedValue, result: OwnedValue) -> String {
    wat_edn::to_json_string(&map(vec![
        ("jsonrpc", OwnedValue::String("2.0".into())),
        ("id", id),
        ("result", result),
    ]))
}

fn error_frame(id: OwnedValue, code: i64, message: &str) -> String {
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

fn tool_result(text: &str, is_error: bool) -> OwnedValue {
    map(vec![
        (
            "content",
            OwnedValue::Vector(vec![map(vec![
                ("type", OwnedValue::String("text".into())),
                ("text", OwnedValue::String(text.to_string().into())),
            ])]),
        ),
        ("isError", OwnedValue::Bool(is_error)),
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
                 returned as EDN text."
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
                        "edn",
                        map(vec![
                            ("type", OwnedValue::String("string".into())),
                            (
                                "description",
                                OwnedValue::String("The form to evaluate, as EDN.".into()),
                            ),
                        ]),
                    )]),
                ),
                (
                    "required",
                    OwnedValue::Vector(vec![OwnedValue::String("edn".into())]),
                ),
            ]),
        ),
    ]);

    let reset_tool = map(vec![
        ("name", OwnedValue::String("reset".into())),
        (
            "description",
            OwnedValue::String(
                "Discard every definition made in this session. The process keeps running."
                    .into(),
            ),
        ),
        (
            "inputSchema",
            map(vec![
                ("type", OwnedValue::String("object".into())),
                ("properties", map(vec![])),
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
