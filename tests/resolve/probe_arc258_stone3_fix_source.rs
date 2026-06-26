//! FM 2-bis probe — arc 258 Stone 258.3: `fix-source`'s if-annotation strip rule.
//!
//! THE PROVING POINT: wat writes wat. `fix-source` is a recursive AST transform written
//! ENTIRELY IN WAT over the homoiconic bridge (read-string · ast->children · with-children ·
//! ast-kind · ast-name). This stone ships ONE rule — strip the now-redundant `-> :T` return
//! annotation from every `if` — and proves the recursive walk end-to-end on real forms.
//!
//! The lair (studied empirically here, never assumed):
//!   an ANNOTATED if `(:wat::core::if <cond> -> :T <then> <else>)` parses to a List whose
//!   children are [ kw:wat::core::if , <cond> , sym"->" , kw:T , <then> , <else> ] — 6 nodes,
//!   child[2] the bare Symbol `->`. A BARE if `(:wat::core::if c t e)` has child[2] = the
//!   then-branch. `Option/expect -> :T` ALSO carries an `arg -> :T` shape, so the strip rule
//!   keys on the EXACT head `:wat::core::if` (Option/expect's head differs) AND child[2] = sym"->".
//!
//! C01: annotated-if? is TRUE on an annotated if.
//! C02: annotated-if? is FALSE on a bare if (child[2] is the then-branch, not "->").
//! C03: annotated-if? is FALSE on `Option/expect -> :T …` (different head) — the guard.
//! C04: fix-source STRIPS — the result is no longer annotated, and its child[2] is now the
//!      then-branch (an int).
//! C05: fix-source RECURSES — an annotated if nested inside `(do …)` is stripped.
//! C06: fix-source PRESERVES `Option/expect -> :T` under recursion (the guard holds in the walk).
//! C07: end-to-end via write-forms — the cleaned source carries no `->` and still reads as an if.
//! C08 (maturity probe, may RED): quasiquote inside a plain `defn` — does ``(if ~a ~b ~c)`` build
//!      a node in a non-macro function? If this REDs, functions can't quasiquote — a flagged gap.
//!
//! Run: `cargo test --release --test probe_arc258_stone3_fix_source`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// fix-source + helpers, written in wat over the homoiconic bridge. Defined inline (the
/// home `wat/fix.wat` is pinned when 258.3b drives the corpus). Built with BARE ifs.
const FIX: &str = r#"
(:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST
  (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))))

(:wat::core::defn :user::structural? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::let [k (:wat::core::ast-kind node)]
    (:wat::core::if (:wat::core::= k "list") true
      (:wat::core::if (:wat::core::= k "vector") true
        (:wat::core::if (:wat::core::= k "map") true
          (:wat::core::if (:wat::core::= k "set") true false))))))

(:wat::core::defn :user::annotated-if? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? (:wat::core::drop ch 2))
        false
        (:wat::core::let [head (:wat::core::first ch)
                          c2   (:wat::core::first (:wat::core::drop ch 2))]
          (:wat::core::if (:wat::core::= (:wat::core::ast-name head) ":wat::core::if")
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind c2) "symbol")
              (:wat::core::= (:wat::core::ast-name c2) "->")
              false)
            false))))
    false))

(:wat::core::defn :user::strip-if [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::with-children node
    (:wat::core::concat (:wat::core::take (:wat::core::ast->children node) 2)
                        (:wat::core::drop (:wat::core::ast->children node) 4))))

(:wat::core::defn :user::fix-source [node <- :wat::WatAST] -> :wat::WatAST
  (:wat::core::if (:user::structural? node)
    (:wat::core::let [rebuilt (:wat::core::with-children node
                                (:wat::core::map
                                  (:wat::core::fn [c <- :wat::WatAST] -> :wat::WatAST (:user::fix-source c))
                                  (:wat::core::ast->children node)))]
      (:wat::core::if (:user::annotated-if? rebuilt)
        (:user::strip-if rebuilt)
        rebuilt))
    node))
"#;

fn eval_bool(body: &str) -> Result<bool, String> {
    let src = format!(
        "{FIX}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::bool {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::bool(b) => Ok(b),
        other => Err(format!("non-bool: {other:?}")),
    }
}

fn eval_string(body: &str) -> Result<String, String> {
    let src = format!(
        "{FIX}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::String {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::String(s) => Ok((*s).clone()),
        other => Err(format!("non-string: {other:?}")),
    }
}

const ANNOTATED_IF: &str = "(:wat::core::if true -> :wat::core::i64 1 2)";
const BARE_IF: &str = "(:wat::core::if true 1 2)";
const OPTION_EXPECT: &str = "(:wat::core::Option/expect -> :wat::core::i64 x \"m\")";

/// Escape a wat snippet so it can be embedded inside a wat double-quoted string literal
/// (the snippet is fed to `read-string`). Inner quotes/backslashes must survive verbatim.
fn embed(payload: &str) -> String {
    payload.replace('\\', "\\\\").replace('"', "\\\"")
}

#[test]
fn contract_01_annotated_if_recognized() {
    assert_eq!(
        eval_bool(&format!("(:user::annotated-if? (:user::topform \"{}\"))", embed(ANNOTATED_IF))),
        Ok(true),
        "an annotated if (head :wat::core::if, child[2] = sym \"->\") is recognized"
    );
}

#[test]
fn contract_02_bare_if_not_annotated() {
    assert_eq!(
        eval_bool(&format!("(:user::annotated-if? (:user::topform \"{}\"))", embed(BARE_IF))),
        Ok(false),
        "a bare if is NOT annotated (child[2] is the then-branch, not the \"->\" symbol)"
    );
}

#[test]
fn contract_03_option_expect_not_annotated_if() {
    // THE GUARD: Option/expect carries `arg -> :T` too, but its head is not :wat::core::if.
    assert_eq!(
        eval_bool(&format!("(:user::annotated-if? (:user::topform \"{}\"))", embed(OPTION_EXPECT))),
        Ok(false),
        "Option/expect's `-> :T` must NOT be mistaken for an if annotation"
    );
}

#[test]
fn contract_04_fix_source_strips_if_annotation() {
    // After fix-source: not annotated anymore, and child[2] is the then-branch (int 1).
    assert_eq!(
        eval_bool(&format!(
            "(:user::annotated-if? (:user::fix-source (:user::topform \"{}\")))",
            embed(ANNOTATED_IF)
        )),
        Ok(false),
        "fix-source strips the annotation (result no longer recognized as annotated)"
    );
    assert_eq!(
        eval_bool(&format!(
            "(:wat::core::= (:wat::core::ast-kind \
               (:wat::core::first (:wat::core::drop \
                 (:wat::core::ast->children (:user::fix-source (:user::topform \"{}\"))) 2))) \
               \"int\")",
            embed(ANNOTATED_IF)
        )),
        Ok(true),
        "after strip, child[2] is the then-branch (int 1), proving -> :T was removed"
    );
}

#[test]
fn contract_05_fix_source_recurses() {
    // The annotated if is nested inside (do …); fix-source must reach and strip it.
    let nested = "(:wat::core::do (:wat::core::if true -> :wat::core::i64 1 2))";
    let out = eval_string(&format!(
        "(:wat::core::write-forms (:user::fix-source (:user::topform \"{}\")))",
        embed(nested)
    ))
    .expect("fix-source + write-forms");
    assert!(
        !out.contains("->"),
        "fix-source recurses into (do …) and strips the inner if's annotation; got: {out}"
    );
}

#[test]
fn contract_06_fix_source_preserves_option_expect() {
    // The guard holds under the recursive walk: Option/expect's -> survives.
    let nested = "(:wat::core::do (:wat::core::Option/expect -> :wat::core::i64 x \"m\"))";
    let out = eval_string(&format!(
        "(:wat::core::write-forms (:user::fix-source (:user::topform \"{}\")))",
        embed(nested)
    ))
    .expect("fix-source + write-forms");
    assert!(
        out.contains("->"),
        "Option/expect's `-> :T` must be preserved through the walk; got: {out}"
    );
}

#[test]
fn contract_07_end_to_end_clean_source() {
    // The strip is end-to-end: rendered output has no `->`; and structurally the head node
    // still names :wat::core::if (ast-name reads the stored token verbatim — write-forms'
    // dotted clean-EDN rendering is a SEPARATE concern, flagged for the corpus drive).
    let out = eval_string(&format!(
        "(:wat::core::write-forms (:user::fix-source (:user::topform \"{}\")))",
        embed(ANNOTATED_IF)
    ))
    .expect("fix-source + write-forms");
    assert!(!out.contains("->"), "cleaned if carries no `->`; got: {out}");
    assert_eq!(
        eval_bool(&format!(
            "(:wat::core::= (:wat::core::ast-name \
               (:wat::core::first (:wat::core::ast->children \
                 (:user::fix-source (:user::topform \"{}\"))))) \
               \":wat::core::if\")",
            embed(ANNOTATED_IF)
        )),
        Ok(true),
        "the cleaned form's head node is still the :wat::core::if keyword (verbatim token)"
    );
}

#[test]
fn contract_08_maturity_quasiquote_in_defn() {
    // FLAGGED maturity probe: can a PLAIN defn (not a macro) quasiquote a form? If this REDs,
    // functions cannot quasiquote — an asymmetry worth its own finding. fix-source routes
    // around it (with-children), so this is diagnostic, not load-bearing.
    let defs = r#"
(:wat::core::defn :user::qq [a <- :wat::WatAST b <- :wat::WatAST c <- :wat::WatAST] -> :wat::WatAST
  `(:wat::core::if ~a ~b ~c))
"#;
    let src = format!(
        "{defs}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::bool \
            (:wat::core::List? (:user::qq \
              (:user::topform \"true\") (:user::topform \"1\") (:user::topform \"2\")))) \
         (:wat::core::defn :user::topform [src <- :wat::core::String] -> :wat::WatAST \
            (:wat::core::first (:wat::core::ast->children (:wat::core::read-string src))))\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let result = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("{e:?}"))
        .and_then(|world| {
            let ast = wat::parse_one!("(:user::compute)").expect("parse");
            eval_in_frozen(&ast, &world, &Environment::new())
                .map(|tv| tv.value_owned())
                .map_err(|e| format!("{e:?}"))
        });
    // We ASSERT the finding either way: print the outcome so the gate records it.
    match result {
        Ok(Value::bool(true)) => { /* quasiquote-in-defn WORKS — no gap */ }
        other => panic!(
            "MATURITY FLAG: quasiquote inside a plain defn did not yield a List node — \
             functions may not be able to quasiquote (macros-only). Outcome: {other:?}"
        ),
    }
}
