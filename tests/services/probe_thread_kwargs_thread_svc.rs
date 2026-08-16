//! Thread `bracket/map` + kwargs tail against a THREAD service.
//!
//! The live MCP panic: a thread-locus handle in the kwargs tail killed the
//! service (allow' on a CrossbeamListener was a hard error) and the owner
//! panicked. GREEN is ["echo:a" "echo:b" "echo:c"].
//!
//! cargo nextest run -p wat -E 'test(/probe_thread_kwargs_thread_svc/)' --test-threads=1

use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

#[test]
fn thread_map_kwargs_reaches_thread_service() {
    let world = startup_from_file("tests/services/probe_thread_kwargs_thread_svc.wat")
        .expect("startup should succeed (thread map + kwargs + thread service)");
    let call = WatAST::List(
        vec![WatAST::Keyword(
            ":probe::run".into(),
            wat::rust_caller_span!(),
        )],
        wat::rust_caller_span!(),
    );
    let got = eval_in_frozen(&call, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("run raised: {e:?}"))
        .value_owned();
    match got {
        Value::Vec(ref v) => {
            let strs: Vec<String> = v
                .iter()
                .map(|tv| match tv {
                    Value::String(s) => (**s).clone(),
                    other => panic!("expected String elements, got {other:?}"),
                })
                .collect();
            assert_eq!(
                strs,
                vec![
                    "echo:a".to_string(),
                    "echo:b".to_string(),
                    "echo:c".to_string(),
                ],
                "thread bracket kwargs must grant+dial a thread service and hold the peer"
            );
        }
        other => panic!("expected Vector<String>, got {other:?}"),
    }
}
