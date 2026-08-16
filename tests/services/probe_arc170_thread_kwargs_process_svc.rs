//! Thread `bracket/map` + kwargs tail against a process service.
//!
//! The cell the floor never gated: `(map (thread) items :work :echo eh)`.
//! cargo nextest run -p wat -E 'test(/probe_arc170_thread_kwargs_process_svc/)' --test-threads=1

use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

#[test]
fn thread_map_kwargs_reaches_process_service() {
    let world = startup_from_file("tests/services/probe_arc170_thread_kwargs_process_svc.wat")
        .expect("startup should succeed (thread map + kwargs + process service)");
    let call = WatAST::List(
        vec![WatAST::Keyword(":probe::run".into(), wat::rust_caller_span!())],
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
                "thread bracket kwargs must Setup-dial a process service and hold the peer"
            );
        }
        other => panic!("expected Vector<String>, got {other:?}"),
    }
}
