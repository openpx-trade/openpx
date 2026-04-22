//! Shared utilities for the openpx benchmark harness.
//!
//! Methodology: 20 timed iterations per target, 100 ms spacing, 5 warmup
//! requests discarded, all targets in a single process so machine / network /
//! time are held constant.

use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

pub mod fixtures {
    use std::path::PathBuf;

    pub fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
    }

    pub fn load_bytes(name: &str) -> Vec<u8> {
        let path = fixtures_dir().join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("load fixture {}: {e}", path.display()))
    }

    pub fn load_string(name: &str) -> String {
        let path = fixtures_dir().join(name);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("load fixture {}: {e}", path.display()))
    }
}

pub mod stats {
    use super::Duration;

    #[derive(Debug, Clone, serde::Serialize)]
    pub struct Summary {
        pub n: usize,
        pub mean_ms: f64,
        pub stddev_ms: f64,
        pub median_ms: f64,
        pub min_ms: f64,
        pub max_ms: f64,
        pub samples_ms: Vec<f64>,
    }

    pub fn summarize(samples: &[Duration]) -> Summary {
        if samples.is_empty() {
            return Summary {
                n: 0,
                mean_ms: 0.0,
                stddev_ms: 0.0,
                median_ms: 0.0,
                min_ms: 0.0,
                max_ms: 0.0,
                samples_ms: vec![],
            };
        }
        let ms: Vec<f64> = samples
            .iter()
            .map(|d| d.as_micros() as f64 / 1000.0)
            .collect();
        let mean = ms.iter().sum::<f64>() / ms.len() as f64;
        let variance = ms.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / ms.len() as f64;
        let stddev = variance.sqrt();
        let mut sorted = ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Summary {
            n: ms.len(),
            mean_ms: mean,
            stddev_ms: stddev,
            median_ms: sorted[sorted.len() / 2],
            min_ms: sorted[0],
            max_ms: sorted[sorted.len() - 1],
            samples_ms: ms,
        }
    }
}

pub fn fmt_ms(v: f64) -> String {
    if v >= 1000.0 {
        format!("{:7.2} s", v / 1000.0)
    } else {
        format!("{:6.1} ms", v)
    }
}

pub fn print_row(target: &str, s: &stats::Summary) {
    println!(
        "  {:<18} {} ± {}   {}   {}   {}   n={}",
        target,
        fmt_ms(s.mean_ms),
        fmt_ms(s.stddev_ms),
        fmt_ms(s.median_ms),
        fmt_ms(s.min_ms),
        fmt_ms(s.max_ms),
        s.n,
    );
}

pub fn print_table(results: &[(String, stats::Summary)]) {
    println!("\n═══════════════════════════════════════════════════════════════════════════");
    println!(
        "  {:<18} {:>9}   {:>9}   {:>9}   {:>9}   {:>9}   n",
        "Target", "Mean", "±StdDev", "Median", "Min", "Max"
    );
    println!("───────────────────────────────────────────────────────────────────────────");
    for (name, s) in results {
        print_row(name, s);
    }
    println!("═══════════════════════════════════════════════════════════════════════════");
    let Some((_, openpx)) = results.iter().find(|(n, _)| n == "openpx") else {
        return;
    };
    if openpx.mean_ms <= 0.0 {
        return;
    }
    for (name, other) in results.iter().filter(|(n, _)| n != "openpx") {
        if other.mean_ms <= 0.0 {
            continue;
        }
        let diff = other.mean_ms - openpx.mean_ms;
        let pct = (diff.abs() / other.mean_ms) * 100.0;
        if diff > 0.0 {
            println!(
                "  openpx is {:.1}% faster than {name}  ({:+.1} ms mean)",
                pct, -diff
            );
        } else {
            println!(
                "  openpx is {:.1}% slower than {name}  ({:+.1} ms mean)",
                pct, -diff
            );
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RunMetadata {
    pub timestamp: String,
    pub host: String,
    pub os: String,
    pub rustc: String,
    pub git_sha: String,
    pub endpoint: String,
    pub iterations: usize,
    pub delay_ms: u64,
    pub warmup: usize,
}

pub fn gather_metadata(
    endpoint: &str,
    iterations: usize,
    delay_ms: u64,
    warmup: usize,
) -> RunMetadata {
    let run_cmd = |cmd: &str, args: &[&str]| -> String {
        std::process::Command::new(cmd)
            .args(args)
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };
    RunMetadata {
        timestamp: chrono::Utc::now().to_rfc3339(),
        host: run_cmd("hostname", &[]),
        os: run_cmd("uname", &["-sr"]),
        rustc: run_cmd("rustc", &["--version"]),
        git_sha: run_cmd("git", &["rev-parse", "--short", "HEAD"]),
        endpoint: endpoint.to_string(),
        iterations,
        delay_ms,
        warmup,
    }
}

#[derive(Debug, Serialize)]
pub struct ReportFile<'a> {
    pub metadata: &'a RunMetadata,
    pub results: std::collections::BTreeMap<&'a str, &'a stats::Summary>,
}

pub fn write_report(
    out_dir: &PathBuf,
    metadata: &RunMetadata,
    results: &[(String, stats::Summary)],
) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(out_dir)?;
    let stamp = metadata.timestamp.replace([':', '-'], "");
    let filename = out_dir.join(format!("network-{stamp}.json"));
    let map: std::collections::BTreeMap<&str, &stats::Summary> =
        results.iter().map(|(k, v)| (k.as_str(), v)).collect();
    let report = ReportFile {
        metadata,
        results: map,
    };
    let body = serde_json::to_string_pretty(&report).expect("serialize report");
    std::fs::write(&filename, body)?;
    Ok(filename)
}
