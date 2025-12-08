//! End-to-end tests for the rabbitbench CLI
//!
//! These tests run the actual CLI binary with mock Criterion output
//! and verify correct parsing and behavior.

use std::io::Write;
use std::process::Command;

/// Sample Criterion benchmark output for testing
const CRITERION_OUTPUT: &str = r#"
Benchmarking fibonacci/10
Gnuplot not found, using plotters backend
fibonacci/10            time:   [1.2345 µs 1.2456 µs 1.2567 µs]
                        change: [-0.5765% +0.8623% +2.3829%] (p = 0.27 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking fibonacci/20
fibonacci/20            time:   [123.45 ns 124.56 ns 125.67 ns]
                        change: [-1.2345% +0.0000% +1.2345%] (p = 0.50 > 0.05)
                        No change in performance detected.

Benchmarking hash_function
hash_function           time:   [45.678 ms 46.789 ms 47.890 ms]
                        change: [+2.1234% +3.4567% +4.7890%] (p = 0.01 < 0.05)
                        Performance has regressed.
"#;

/// Get the path to the CLI binary
fn cli_binary() -> std::path::PathBuf {
    // cli is in workspace, so target is at parent level
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up from cli/ to workspace root
    path.push("target");
    path.push("release");
    path.push("rabbitbench");
    path
}

/// Build the release binary once
fn build_cli() {
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("Failed to build CLI");
    assert!(status.success(), "Failed to build CLI binary");
}

/// Create a temporary script that outputs Criterion-like results
fn create_mock_benchmark_script(output: &str) -> std::path::PathBuf {
    use std::fs;

    // Create a unique temp file path
    let temp_dir = std::env::temp_dir();
    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let script_path = temp_dir.join(format!("rabbitbench_test_{}.sh", unique_id));

    // Write the script content
    let mut content = String::new();
    content.push_str("#!/bin/sh\n");
    content.push_str("cat << 'CRITERION_EOF'\n");
    content.push_str(output);
    content.push_str("CRITERION_EOF\n");

    fs::write(&script_path, &content).expect("Failed to write script");

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }

    // Small delay to ensure file is fully written and closed
    std::thread::sleep(std::time::Duration::from_millis(50));

    script_path
}

#[test]
fn test_run_dry_run_parses_criterion_output() {
    build_cli();

    let script = create_mock_benchmark_script(CRITERION_OUTPUT);

    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "main",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    // Verify the CLI parsed the benchmarks correctly
    assert!(
        stdout.contains("Found 3 benchmark results"),
        "Should find 3 benchmarks, got stdout: {}\nstderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("fibonacci/10"),
        "Should contain fibonacci/10"
    );
    assert!(
        stdout.contains("fibonacci/20"),
        "Should contain fibonacci/20"
    );
    assert!(
        stdout.contains("hash_function"),
        "Should contain hash_function"
    );
    assert!(stdout.contains("Dry run"), "Should indicate dry run mode");
}

#[test]
fn test_run_dry_run_with_pr_flag() {
    build_cli();

    let script = create_mock_benchmark_script(CRITERION_OUTPUT);

    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "feature-branch",
            "--pr",
            "42",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout: {}", stdout);

    // Verify PR number is displayed
    assert!(stdout.contains("PR: #42"), "Should display PR number");
    assert!(
        stdout.contains("Branch: feature-branch"),
        "Should display branch"
    );
}

#[test]
fn test_run_dry_run_with_github_ref_env() {
    build_cli();

    let script = create_mock_benchmark_script(CRITERION_OUTPUT);

    // Run with GITHUB_REF environment variable
    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "pr-branch",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .env("GITHUB_REF", "refs/pull/123/merge")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout: {}", stdout);

    // Verify PR number is auto-detected from GITHUB_REF
    assert!(
        stdout.contains("PR: #123"),
        "Should auto-detect PR number from GITHUB_REF, got: {}",
        stdout
    );
}

#[test]
fn test_run_dry_run_explicit_pr_overrides_env() {
    build_cli();

    let script = create_mock_benchmark_script(CRITERION_OUTPUT);

    // Explicit --pr should override GITHUB_REF
    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "test",
            "--pr",
            "42",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .env("GITHUB_REF", "refs/pull/999/merge")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout: {}", stdout);

    // Explicit --pr 42 should win over GITHUB_REF=refs/pull/999/merge
    assert!(
        stdout.contains("PR: #42"),
        "Explicit --pr should override GITHUB_REF, got: {}",
        stdout
    );
    assert!(
        !stdout.contains("PR: #999"),
        "Should NOT use GITHUB_REF value"
    );
}

#[test]
fn test_run_no_benchmarks_found() {
    build_cli();

    let script = create_mock_benchmark_script("Hello, this is not benchmark output\n");

    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "main",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout: {}", stdout);

    assert!(
        stdout.contains("No benchmark results found"),
        "Should report no benchmarks found"
    );
}

#[test]
fn test_run_various_time_units() {
    build_cli();

    let mixed_units_output = r#"
bench_ns                time:   [100.00 ns 110.00 ns 120.00 ns]
bench_us                time:   [1.5000 µs 1.6000 µs 1.7000 µs]
bench_us_ascii          time:   [2.5000 us 2.6000 us 2.7000 us]
bench_ms                time:   [10.000 ms 11.000 ms 12.000 ms]
bench_s                 time:   [1.0000 s 1.1000 s 1.2000 s]
"#;

    let script = create_mock_benchmark_script(mixed_units_output);

    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "main",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout: {}", stdout);

    // Should find all 5 benchmarks with different time units
    assert!(
        stdout.contains("Found 5 benchmark results"),
        "Should find 5 benchmarks with various units, got: {}",
        stdout
    );
    assert!(stdout.contains("bench_ns"), "Should contain bench_ns");
    assert!(stdout.contains("bench_us"), "Should contain bench_us");
    assert!(stdout.contains("bench_ms"), "Should contain bench_ms");
    assert!(stdout.contains("bench_s"), "Should contain bench_s");
}

#[test]
fn test_run_displays_testbed_defaulting_to_os() {
    build_cli();

    let script = create_mock_benchmark_script(CRITERION_OUTPUT);

    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "main",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout: {}", stdout);

    // Should show testbed (defaults to OS)
    assert!(
        stdout.contains("Testbed:"),
        "Should display testbed info, got: {}",
        stdout
    );
}

#[test]
fn test_run_custom_testbed() {
    build_cli();

    let script = create_mock_benchmark_script(CRITERION_OUTPUT);

    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "main",
            "-t",
            "ci-runner-ubuntu-22.04",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout: {}", stdout);

    assert!(
        stdout.contains("Testbed: ci-runner-ubuntu-22.04"),
        "Should display custom testbed, got: {}",
        stdout
    );
}

#[test]
fn test_run_with_git_hash() {
    build_cli();

    let script = create_mock_benchmark_script(CRITERION_OUTPUT);

    let output = Command::new(cli_binary())
        .args([
            "run",
            "-p",
            "test-project",
            "-b",
            "main",
            "--hash",
            "abc123def456",
            "--dry-run",
            script.to_str().unwrap(),
        ])
        .env("RABBITBENCH_TOKEN", "fake-token-for-testing")
        .output()
        .expect("Failed to execute CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    println!("stdout: {}", stdout);

    assert!(
        stdout.contains("Git hash: abc123def456"),
        "Should display git hash, got: {}",
        stdout
    );
}
