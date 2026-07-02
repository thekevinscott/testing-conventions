//! Rust-SDK warm latency: direct core call, no binding boundary.
fn main() {
    let tree = std::env::args().nth(1).expect("usage: warm_bench <tree>");
    let globs = vec!["**/*.md".to_string()];
    let mut times: Vec<f64> = (0..1000)
        .map(|_| {
            let t0 = std::time::Instant::now();
            agent_context_core::print(&globs, Some(&tree)).unwrap();
            t0.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("rust direct warm median: {:.3} ms", times[500]);
}
