use rectiq_cli::scan;
use rectiq_core::{types::fixplanner::FixContext, Oracle};
use rectiq_test_support::{fixtures::sample_invalid_json, util::pretty_if_json};
use rectiq_types::SketchNode;

fn apply_actions(input: String, actions: &rectiq_core::types::FixPlan) -> String {
    let mut bytes = input.into_bytes();
    let mut combined: Vec<_> = actions
        .actions
        .iter()
        .map(|a| (a.span, a.replacement.clone()))
        .collect();
    combined.sort_by_key(|(s, _)| s.start);
    for (span, repl) in combined.into_iter().rev() {
        if span.start <= span.end && span.end <= bytes.len() {
            bytes.splice(span.start..span.end, repl.as_bytes().iter().copied());
        }
    }
    String::from_utf8(bytes).unwrap()
}

fn run_full(input: &str) -> (String, rectiq_core::types::FixPlan) {
    let sketches: Vec<SketchNode<'static>> = scan(input);
    let sketches = SketchNode::to_sketches_all(sketches).unwrap();
    let oracle = Oracle::default();
    let mut ctx = FixContext::default();
    let issues = oracle.detect(&sketches);
    let plan = oracle.plan(&issues, &mut ctx);
    let output = apply_actions(input.to_string(), &plan);
    (output, plan)
}

#[test]
fn idempotent_plan() {
    let input = sample_invalid_json();
    let (out, plan) = run_full(input);
    let out2 = apply_actions(out.clone(), &plan);
    assert_eq!(pretty_if_json(&out), pretty_if_json(&out2));
}

#[test]
fn non_overlapping_actions() {
    let input = sample_invalid_json();
    let (_out, plan) = run_full(input);
    let mut spans = plan.actions.iter().map(|a| a.span).collect::<Vec<_>>();
    spans.sort_by_key(|s| s.start);
    for w in spans.windows(2) {
        assert!(
            w[0].end <= w[1].start,
            "spans overlap: {:?} vs {:?}",
            w[0],
            w[1]
        );
    }
}

#[test]
fn deterministic_planning() {
    let input = sample_invalid_json();
    let (_out1, plan1) = run_full(input);
    let (_out2, plan2) = run_full(input);
    let s1 = serde_json::to_string(&plan1.actions).unwrap();
    let s2 = serde_json::to_string(&plan2.actions).unwrap();
    assert_eq!(s1, s2);
}
