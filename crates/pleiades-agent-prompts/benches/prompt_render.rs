use std::collections::HashMap;

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use pleiades_agent_prompts::BuiltinPrompts;

fn render_builtin_prompt(c: &mut Criterion) {
    let template = BuiltinPrompts::code_reviewer();
    let variables = HashMap::from([(
        "diff".to_string(),
        "diff --git a/src/lib.rs b/src/lib.rs\n+fn example() {}".repeat(100),
    )]);
    c.bench_function("render_code_review_prompt", |b| {
        b.iter(|| template.render(black_box(&variables)).unwrap())
    });
}

criterion_group!(benches, render_builtin_prompt);
criterion_main!(benches);
