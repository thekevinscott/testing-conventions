//! Integration test for #239: `unit mutation --language typescript` resolves the Stryker
//! engine *bundled with the tool*, not from the consumer's project tree.
//!
//! The bug was that the rule ran `npx --no-install stryker` with the working directory set
//! to the consumer's project, so it searched the *consumer's* `node_modules` for an engine
//! that actually shipped alongside the binary — and hard-errored telling the consumer to
//! install Stryker themselves. The fix resolves the engine from the tool's own install
//! tree (and honors `TESTING_CONVENTIONS_STRYKER_BIN` as an override / test seam).
//!
//! This test stands a project that has **no** Stryker in its own tree and points the rule
//! at an engine elsewhere via the override. The old `--no-install`-from-cwd code can't find
//! an engine and produces no report (it would fail); the fixed code runs the engine it was
//! pointed at and parses its report. A *fake* Stryker (a tiny script that writes a known
//! `mutation.json`) stands in for the real one, so the test needs no toolchain and is fast
//! and deterministic — it proves the *resolution + run + parse* path, which is the fix.
//!
//! It lives in its own test binary because it sets a process-wide env var; keeping it out
//! of `mutation_typescript.rs` stops that var from leaking into those parallel tests.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::mutation::measure_typescript;

/// A unique throwaway dir under the temp root, removed by the caller.
fn scratch(slug: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "tc-ts-provision-{}-{}-{}",
        slug,
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn resolves_the_bundled_engine_and_runs_it_against_a_project_with_none() {
    // A project that ships no engine in its own tree — only its source.
    let project = scratch("project");
    std::fs::write(
        project.join("index.ts"),
        "export const ok = (n: number): boolean => n > 0;\n",
    )
    .unwrap();
    assert!(
        !project.join("node_modules").exists(),
        "the project must have no engine of its own — resolving the bundled one is the point"
    );

    // A fake Stryker engine living *outside* the project: a script that writes the known
    // json report Stryker would, then exits. Pointed at via the override the wrapper uses
    // in production to hand the binary its bundled engine's location.
    let engine = scratch("engine");
    let fake = engine.join("stryker");
    std::fs::write(
        &fake,
        "#!/bin/sh\nmkdir -p reports/mutation\ncat > reports/mutation/mutation.json <<'JSON'\n\
         {\"files\":{\"index.ts\":{\"mutants\":[\
         {\"mutatorName\":\"ConditionalExpression\",\"replacement\":\"true\",\"status\":\"Survived\",\
         \"location\":{\"start\":{\"line\":1,\"column\":20}}}]}}}\nJSON\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    std::env::set_var("TESTING_CONVENTIONS_STRYKER_BIN", &fake);

    let survivors = measure_typescript(&project, &[], &BTreeMap::new(), None).expect(
        "the rule resolves the \
        engine it was pointed at and parses its report, even though the project ships none",
    );

    std::env::remove_var("TESTING_CONVENTIONS_STRYKER_BIN");
    let _ = std::fs::remove_dir_all(&project);
    let _ = std::fs::remove_dir_all(&engine);

    assert_eq!(
        survivors.len(),
        1,
        "the engine's report has one survivor; got {survivors:?}"
    );
    assert_eq!(survivors[0].file, "index.ts");
    assert_eq!(survivors[0].line, 1);
    assert!(survivors[0].description.contains("ConditionalExpression"));
}
