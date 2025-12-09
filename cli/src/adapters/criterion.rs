use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct CriterionResult {
    pub name: String,
    pub value: f64,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
}

// Matches both single-line and multi-line Criterion output formats:
// Single line: `benchmark_name            time:   [...]`
// Multi-line:  `benchmark_name\n                        time:   [...]`
static CRITERION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?m)^(\S+)\s*\n?\s*time:\s+\[([0-9.]+)\s+(ns|µs|us|ms|s)\s+([0-9.]+)\s+(ns|µs|us|ms|s)\s+([0-9.]+)\s+(ns|µs|us|ms|s)\]"
    ).unwrap()
});

pub fn parse_criterion_output(output: &str) -> Vec<CriterionResult> {
    CRITERION_REGEX
        .captures_iter(output)
        .filter_map(|cap| {
            let name = cap.get(1)?.as_str().to_string();
            let lower = parse_time(cap.get(2)?.as_str(), cap.get(3)?.as_str())?;
            let mean = parse_time(cap.get(4)?.as_str(), cap.get(5)?.as_str())?;
            let upper = parse_time(cap.get(6)?.as_str(), cap.get(7)?.as_str())?;

            Some(CriterionResult {
                name,
                value: mean,
                lower: Some(lower),
                upper: Some(upper),
            })
        })
        .collect()
}

fn parse_time(value: &str, unit: &str) -> Option<f64> {
    let v: f64 = value.parse().ok()?;
    let multiplier = match unit {
        "ns" => 1.0,
        "µs" | "us" => 1_000.0,
        "ms" => 1_000_000.0,
        "s" => 1_000_000_000.0,
        _ => return None,
    };
    Some(v * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_criterion_output() {
        let output = r#"
Benchmarking fibonacci/10
fibonacci/10            time:   [1.2345 µs 1.2456 µs 1.2567 µs]

Benchmarking fibonacci/20
fibonacci/20            time:   [123.45 ns 124.56 ns 125.67 ns]
        "#;

        let results = parse_criterion_output(output);
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].name, "fibonacci/10");
        assert!((results[0].value - 1245.6).abs() < 0.1);

        assert_eq!(results[1].name, "fibonacci/20");
        assert!((results[1].value - 124.56).abs() < 0.01);
    }

    #[test]
    fn test_parse_criterion_output_multiline() {
        // Some Criterion versions output the benchmark name and time on separate lines
        let output = r#"
ch_opt_projection_pushdown/10
                        time:   [83.524 µs 83.754 µs 83.998 µs]
                        change: [-1.2345% +0.0000% +1.2345%] (p = 0.50 > 0.05)

ch_opt_projection_pushdown/100
                        time:   [845.67 µs 850.12 µs 854.89 µs]
        "#;

        let results = parse_criterion_output(output);
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].name, "ch_opt_projection_pushdown/10");
        assert!((results[0].value - 83754.0).abs() < 1.0); // 83.754 µs = 83754 ns

        assert_eq!(results[1].name, "ch_opt_projection_pushdown/100");
        assert!((results[1].value - 850120.0).abs() < 1.0); // 850.12 µs = 850120 ns
    }

    #[test]
    fn test_parse_criterion_output_mixed_formats() {
        // Test a mix of single-line and multi-line formats
        let output = r#"
bench_inline            time:   [100.00 ns 110.00 ns 120.00 ns]

bench_multiline
                        time:   [200.00 ns 210.00 ns 220.00 ns]
        "#;

        let results = parse_criterion_output(output);
        assert_eq!(results.len(), 2);

        assert_eq!(results[0].name, "bench_inline");
        assert!((results[0].value - 110.0).abs() < 0.01);

        assert_eq!(results[1].name, "bench_multiline");
        assert!((results[1].value - 210.0).abs() < 0.01);
    }
}
