//! Tail position — one list, two consumers.
//!
//! [`eval_tail`](crate::runtime) is the authority on what tail position means in wat.
//! Seven forms *carry* tail position into a sub-expression, each with an `eval_*_tail`
//! sibling. This module is the single source of that set: `eval_tail`'s dispatch matches
//! on [`TailForm`], and the checker walks [`TailForm::tail_children`] at a `let` that
//! creates a Handle.
//!
//! A duplicated string list in the checker would drift invisibly — a form gains a tail
//! variant, the wall misses real escapes and invents false ones, and the floor stays
//! green. Exhaustiveness of `match form` in `eval_tail` is the compile-time half of
//! that gate; [`tests::eval_tail_calls_exactly_the_seven_tail_evaluators`] is the
//! parse-time half.

use crate::ast::WatAST;
use crate::rete::vocabulary::{rete_op_for, OpClass, RETE_PREFIX};

pub const FORM_IF: &str = ":wat::core::if";
pub const FORM_MATCH: &str = ":wat::core::match";
pub const FORM_LET: &str = ":wat::core::let";
pub const FORM_DO: &str = ":wat::core::do";
pub const FORM_AND: &str = ":wat::core::and";
pub const FORM_OR: &str = ":wat::core::or";
pub const FORM_ANN_FORM: &str = ":wat::core::ann-form";

/// The closed set of forms that carry tail position. Order matches `eval_tail`'s
/// historical dispatch (if, match, let, do, and, or, ann-form).
///
/// Read by the drift gate; `tail_form` is the runtime/checker consumer.
#[allow(dead_code)] // consumed by the drift-gate test and as the published table
pub const TAIL_CARRYING_FORMS: &[&str] = &[
    FORM_IF,
    FORM_MATCH,
    FORM_LET,
    FORM_DO,
    FORM_AND,
    FORM_OR,
    FORM_ANN_FORM,
];

/// A form that carries tail position into one or more children.
///
/// The string names live only in [`tail_form`] / [`TAIL_CARRYING_FORMS`]. Adding a
/// form means adding a variant here, a `FORM_*` constant, a `TAIL_CARRYING_FORMS`
/// entry, a `tail_form` arm, an `eval_*_tail` sibling, and an `eval_tail` match
/// arm — the last is exhaustiveness-checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TailForm {
    If,
    Match,
    Let,
    Do,
    And,
    Or,
    AnnForm,
}

/// Remap a rete `Form` head to its core twin, mirroring `eval_tail`'s rete gate.
/// Core names and non-Form rete heads pass through unchanged.
pub fn remap_rete_form(head: &str) -> &str {
    if head.starts_with(RETE_PREFIX) {
        if let Some(op) = rete_op_for(head) {
            if op.class == OpClass::Form {
                return op.core_name;
            }
        }
    }
    head
}

/// Classify `head` as a tail-carrying form, after rete Form remapping.
pub fn tail_form(head: &str) -> Option<TailForm> {
    match remap_rete_form(head) {
        FORM_IF => Some(TailForm::If),
        FORM_MATCH => Some(TailForm::Match),
        FORM_LET => Some(TailForm::Let),
        FORM_DO => Some(TailForm::Do),
        FORM_AND => Some(TailForm::And),
        FORM_OR => Some(TailForm::Or),
        FORM_ANN_FORM => Some(TailForm::AnnForm),
        _ => None,
    }
}

impl TailForm {
    /// The `eval_*_tail` sibling `eval_tail` must call for this form. Drift-gate
    /// parses `eval_tail` for these names.
    #[allow(dead_code)] // consumed by the drift-gate test
    pub const fn eval_tail_fn_name(self) -> &'static str {
        match self {
            TailForm::If => "eval_if_tail",
            TailForm::Match => "eval_match_tail",
            TailForm::Let => "eval_let_tail",
            TailForm::Do => "eval_do_tail",
            TailForm::And => "eval_and_tail",
            TailForm::Or => "eval_or_tail",
            TailForm::AnnForm => "eval_ann_form_tail",
        }
    }

    /// Children that inherit tail position, matching each `eval_*_tail` sibling:
    /// `if` then/else, `match` arm bodies, `let`/`do`/`and`/`or` last body, `ann-form` expr.
    pub fn tail_children<'a>(self, args: &'a [WatAST]) -> Vec<&'a WatAST> {
        match self {
            TailForm::If => {
                if args.len() == 3 {
                    vec![&args[1], &args[2]]
                } else {
                    Vec::new()
                }
            }
            TailForm::Match => args
                .iter()
                .skip(1)
                .filter_map(|arm| match arm {
                    WatAST::List(items, _) if items.len() == 2 => Some(&items[1]),
                    _ => None,
                })
                .collect(),
            // Bindings are args[0]; eval_let_tail eval_tails only the last body form.
            TailForm::Let => args.get(1..).and_then(|b| b.last()).into_iter().collect(),
            TailForm::Do | TailForm::And | TailForm::Or => args.last().into_iter().collect(),
            TailForm::AnnForm => args.first().into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Row 7 — if `eval_tail` gains or loses an `eval_*_tail` call without this
    /// table learning, the wall goes quietly wrong in both directions and the
    /// floor stays green. Parse the dispatch, do not trust a comment.
    #[test]
    fn eval_tail_calls_exactly_the_seven_tail_evaluators() {
        let src = include_str!("runtime.rs");
        let start = src
            .find("fn eval_tail(")
            .expect("eval_tail must exist in runtime.rs");
        let rest = &src[start..];
        let end = rest
            .find("\nfn eval_if_tail(")
            .expect("eval_if_tail must follow eval_tail");
        let body = &rest[..end];

        let mut found: Vec<String> = Vec::new();
        for line in body.lines() {
            let line = line.trim();
            if line.starts_with("//") {
                continue;
            }
            let mut search = line;
            while let Some(i) = search.find("eval_") {
                let slice = &search[i..];
                if let Some(end_name) = slice.find('(') {
                    let name = &slice[..end_name];
                    if name.ends_with("_tail")
                        && name != "eval_tail"
                        && name
                            .bytes()
                            .all(|b| b.is_ascii_lowercase() || b == b'_')
                    {
                        found.push(name.to_string());
                    }
                    search = &slice[end_name + 1..];
                } else {
                    break;
                }
            }
        }

        let expected: Vec<&str> = [
            TailForm::If,
            TailForm::Match,
            TailForm::Let,
            TailForm::Do,
            TailForm::And,
            TailForm::Or,
            TailForm::AnnForm,
        ]
        .into_iter()
        .map(TailForm::eval_tail_fn_name)
        .collect();

        let mut found_sorted = found.clone();
        found_sorted.sort_unstable();
        let mut expected_sorted: Vec<String> = expected.iter().map(|s| (*s).to_string()).collect();
        expected_sorted.sort_unstable();
        assert_eq!(
            found_sorted, expected_sorted,
            "eval_tail dispatch drifted from TAIL_CARRYING_FORMS.\n  found: {found:?}\n  expected: {expected:?}\n  A duplicated list with no gate is a FAIL even with a green floor."
        );
        assert_eq!(TAIL_CARRYING_FORMS.len(), 7);
        for form in TAIL_CARRYING_FORMS {
            assert!(
                tail_form(form).is_some(),
                "{form} is in TAIL_CARRYING_FORMS but tail_form returns None"
            );
        }
    }

    #[test]
    fn tail_form_table_is_closed_and_named() {
        assert_eq!(
            TAIL_CARRYING_FORMS,
            &[
                FORM_IF, FORM_MATCH, FORM_LET, FORM_DO, FORM_AND, FORM_OR, FORM_ANN_FORM
            ]
        );
        assert!(tail_form(":wat::core::if").is_some());
        assert!(tail_form(":wat::i64::+").is_none());
        assert!(tail_form(":wat::rete::insert").is_none());
    }
}
