//! Performance benchmarks for Prompt Tracking System

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

use prompt_tracking::{
    analysis::{EfficiencyAnalyzer, QualityAnalyzer},
    capture::CaptureService,
    database::Database,
    models::Prompt,
    utils::{calculate_hash, normalize_whitespace},
};

/// Benchmark hash calculation
fn bench_hash(c: &mut Criterion) {
    let contents = vec![
        ("small", "Short prompt content"),
        ("medium", &"Medium length prompt content. ".repeat(10)),
        ("large", &"Large prompt with lots of content for testing performance. ".repeat(100)),
    ];

    let mut group = c.benchmark_group("hash_calculation");

    for (name, content) in contents {
        group.bench_with_input(
            BenchmarkId::new("calculate_hash", name),
            content,
            |b, content| {
                b.iter(|| calculate_hash(black_box(content)));
            },
        );
    }

    group.finish();
}

/// Benchmark whitespace normalization
fn bench_normalize(c: &mut Criterion) {
    let content = "  Multiple   spaces   and\n\nnewlines   here  ".repeat(100);

    c.bench_function("normalize_whitespace", |b| {
        b.iter(|| normalize_whitespace(black_box(&content)));
    });
}

/// Benchmark prompt capture
fn bench_capture(c: &mut Criterion) {
    let service = CaptureService::default();
    let contents = vec![
        ("simple", "Write a function"),
        ("detailed", "Write a Rust function that implements a binary search tree with insert, delete, and search operations. Include proper error handling and documentation."),
        ("complex", &"Context: Building a REST API. Create a Python function that validates user input. Must handle null values. Return JSON format with error messages. Include tests.".to_string()),
    ];

    let mut group = c.benchmark_group("capture_service");

    for (name, content) in contents {
        group.bench_with_input(
            BenchmarkId::new("process_content", name),
            content,
            |b, content| {
                b.iter(|| service.process_content(black_box(content)));
            },
        );
    }

    group.finish();
}

/// Benchmark quality analysis
fn bench_quality_analysis(c: &mut Criterion) {
    let analyzer = QualityAnalyzer::default();

    let prompts: Vec<(&str, Prompt)> = vec![
        ("low_quality", create_prompt("do thing")),
        ("medium_quality", create_prompt("Write a function to sort an array")),
        ("high_quality", create_prompt(
            "Context: Building a REST API. Create a Python function that validates user input. \
             Must handle null values. Return JSON format with error messages."
        )),
    ];

    let mut group = c.benchmark_group("quality_analysis");

    for (name, prompt) in &prompts {
        group.bench_with_input(
            BenchmarkId::new("analyze", name),
            prompt,
            |b, prompt| {
                b.iter(|| analyzer.analyze(black_box(prompt)));
            },
        );
    }

    group.finish();
}

/// Benchmark efficiency analysis
fn bench_efficiency_analysis(c: &mut Criterion) {
    let analyzer = EfficiencyAnalyzer::default();

    let mut prompt = create_prompt("Test prompt for efficiency analysis");
    prompt.metadata.input_tokens = Some(100);
    prompt.metadata.output_tokens = Some(200);
    prompt.metadata.execution_time_ms = Some(1500);
    prompt.metadata.estimated_cost = Some(0.005);

    c.bench_function("efficiency_analyze", |b| {
        b.iter(|| analyzer.analyze(black_box(&prompt)));
    });
}

/// Benchmark database operations
fn bench_database(c: &mut Criterion) {
    let mut group = c.benchmark_group("database");

    // Create prompt benchmark
    group.bench_function("create_prompt", |b| {
        let db = Database::in_memory().unwrap();
        let mut counter = 0;

        b.iter(|| {
            let mut prompt = create_prompt(&format!("Test prompt {}", counter));
            prompt.content_hash = format!("hash_{}", counter);
            counter += 1;
            db.create_prompt(black_box(&prompt))
        });
    });

    // Get prompt benchmark
    group.bench_function("get_prompt", |b| {
        let db = Database::in_memory().unwrap();
        let mut prompt = create_prompt("Test prompt");
        prompt.content_hash = "test_hash".to_string();
        db.create_prompt(&prompt).unwrap();
        let id = prompt.id.clone();

        b.iter(|| db.get_prompt(black_box(&id)));
    });

    // List prompts benchmark
    group.bench_function("list_prompts", |b| {
        let db = Database::in_memory().unwrap();

        // Insert 100 prompts
        for i in 0..100 {
            let mut prompt = create_prompt(&format!("Prompt {}", i));
            prompt.content_hash = format!("hash_{}", i);
            db.create_prompt(&prompt).unwrap();
        }

        b.iter(|| {
            db.list_prompts(black_box(&prompt_tracking::database::PromptFilter::default()))
        });
    });

    // Search prompts benchmark
    group.bench_function("search_prompts", |b| {
        let db = Database::in_memory().unwrap();

        // Insert prompts with searchable content
        for i in 0..100 {
            let mut prompt = create_prompt(&format!("Test prompt {} with searchable content", i));
            prompt.content_hash = format!("hash_{}", i);
            db.create_prompt(&prompt).unwrap();
        }

        b.iter(|| db.search_prompts(black_box("searchable")));
    });

    group.finish();
}

/// Benchmark similarity calculation
fn bench_similarity(c: &mut Criterion) {
    let service = CaptureService::default();

    let pairs = vec![
        ("identical", "Write a function to sort an array", "Write a function to sort an array"),
        ("similar", "Write a function to sort an array in ascending order", "Write a function to sort an array in descending order"),
        ("different", "Write a function to sort", "Create a web server"),
    ];

    let mut group = c.benchmark_group("similarity");

    for (name, s1, s2) in pairs {
        group.bench_with_input(
            BenchmarkId::new("calculate", name),
            &(s1, s2),
            |b, (s1, s2)| {
                b.iter(|| service.calculate_similarity(black_box(s1), black_box(s2)));
            },
        );
    }

    group.finish();
}

/// Helper function to create test prompts
fn create_prompt(content: &str) -> Prompt {
    let mut prompt = Prompt::new(content.to_string());
    prompt.content_hash = calculate_hash(content);
    prompt
}

criterion_group!(
    benches,
    bench_hash,
    bench_normalize,
    bench_capture,
    bench_quality_analysis,
    bench_efficiency_analysis,
    bench_database,
    bench_similarity,
);

criterion_main!(benches);
