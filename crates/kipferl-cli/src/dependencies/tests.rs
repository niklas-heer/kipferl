use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{self, Cursor, Write};

use super::{Artifact, Stage, invalid, registry, resolver, sha256, wheel};

fn check(condition: bool, message: &str) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(invalid(message))
    }
}

fn artifact(name: &str, version: &str) -> Artifact {
    Artifact {
        name: name.to_owned(),
        version: version.to_owned(),
        filename: format!("{name}-{version}-py3-none-any.whl"),
        url: format!("https://files.pythonhosted.org/packages/{name}-{version}.whl"),
        sha256: String::new(),
        requires_dist: Vec::new(),
    }
}

fn archive(entries: &[(&str, &[u8])]) -> io::Result<Vec<u8>> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, bytes) in entries {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .map_err(invalid)?;
        zip.write_all(bytes)?;
    }
    Ok(zip.finish().map_err(invalid)?.into_inner())
}

fn package(name: &str, version: &str, requires: &[&str]) -> io::Result<(Artifact, Vec<u8>)> {
    let mut artifact = artifact(name, version);
    let mut metadata = format!("Metadata-Version: 2.1\nName: {name}\nVersion: {version}\n");
    for required in requires {
        writeln!(metadata, "Requires-Dist: {required}").map_err(invalid)?;
    }
    metadata.push('\n');
    let bytes = archive(&[
        (&format!("{name}/__init__.py"), b"VALUE = 42\n"),
        (
            &format!("{name}-{version}.dist-info/METADATA"),
            metadata.as_bytes(),
        ),
        (
            &format!("{name}-{version}.dist-info/WHEEL"),
            b"Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n",
        ),
    ])?;
    artifact.sha256 = sha256(&bytes);
    Ok((artifact, bytes))
}

fn fixture(packages: Vec<(Artifact, Vec<u8>)>) -> io::Result<(Stage, registry::Registry)> {
    let stage = Stage::new(&std::env::temp_dir())?;
    let mut projects: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for (artifact, bytes) in packages {
        std::fs::write(stage.0.join(format!("{}.whl", artifact.sha256)), bytes)?;
        let project = projects
            .entry(artifact.name.clone())
            .or_insert_with(|| serde_json::json!({"releases": {}}));
        let releases = project
            .get_mut("releases")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| invalid("fixture releases"))?;
        releases.insert(artifact.version.clone(), serde_json::json!([{
            "filename": artifact.filename, "url": artifact.url, "digests": {"sha256": artifact.sha256}, "yanked": false
        }]));
    }
    let registry = registry::Registry::fixture(stage.0.clone(), projects);
    Ok((stage, registry))
}

#[test]
fn resolver_backtracks_across_transitive_constraints() -> io::Result<()> {
    let (_stage, mut registry) = fixture(vec![
        package("app", "2.0", &["shared>=2"])?,
        package("app", "1.0", &["shared<2"])?,
        package("shared", "2.0", &[])?,
        package("shared", "1.0", &[])?,
    ])?;
    let selected = resolver::resolve(&["app".to_owned(), "shared<2".to_owned()], &mut registry)?;
    check(
        selected
            .iter()
            .any(|item| item.name == "app" && item.version == "1.0"),
        "resolver must backtrack app to satisfy shared<2",
    )?;
    check(
        selected
            .iter()
            .any(|item| item.name == "shared" && item.version == "1.0"),
        "transitive selection wrong",
    )
}

#[test]
fn pep440_ordering_compatible_releases_and_prereleases() -> io::Result<()> {
    let (_stage, mut registry) = fixture(vec![
        package("demo", "1.9", &[])?,
        package("demo", "1.10", &[])?,
        package("demo", "2.0rc1", &[])?,
    ])?;
    let selected = resolver::resolve(&["demo~=1.0".to_owned()], &mut registry)?;
    check(
        selected.first().is_some_and(|item| item.version == "1.10"),
        "PEP440 numeric order or compatible-release constraint failed",
    )?;
    let selected = resolver::resolve(&["demo>=2.0rc1".to_owned()], &mut registry)?;
    check(
        selected
            .first()
            .is_some_and(|item| item.version == "2.0rc1"),
        "explicit prerelease requirement failed",
    )
}

#[test]
fn resolution_rejects_cycles_with_conflicting_constraints() -> io::Result<()> {
    let (_stage, mut registry) = fixture(vec![
        package("alpha", "1", &["beta"])?,
        package("beta", "1", &["alpha>=2"])?,
    ])?;
    check(
        resolver::resolve(&["alpha".to_owned()], &mut registry).is_err(),
        "conflicting dependency cycle accepted",
    )
}

#[test]
fn requirements_fail_explicitly_for_unsupported_semantics() -> io::Result<()> {
    for text in [
        "demo[extra]",
        "demo; extra == 'test'",
        "demo[extra]; extra == 'test'",
        "demo @ https://example.com/a.whl ; extra == 'test'",
        "demo; python_version>'3'",
        "demo @ https://example.com/a.whl",
        "../demo",
    ] {
        check(
            resolver::requirement(text).is_err(),
            "unsupported requirement accepted",
        )?;
    }
    check(
        resolver::requirement("Demo_Name ~= 1.2, != 1.2.3").is_ok(),
        "valid PEP508 constraint rejected",
    )
}

#[test]
fn inactive_optional_dependencies_preserve_metadata_and_offline_graph() -> io::Result<()> {
    let requires = [
        "shared>=1",
        "test-runner; extra == 'test'",
        "documentation; extra == 'docs' and sys_platform == 'win32'",
        "optional-tool; extra == 'test' or extra == 'docs'",
    ];
    let (_stage, mut registry) = fixture(vec![
        package("app", "1.0", &requires)?,
        package("shared", "1.0", &[])?,
    ])?;
    let roots = ["app==1.0".to_owned()];
    let selected = resolver::resolve(&roots, &mut registry)?;
    check(
        selected.len() == 2,
        "inactive extras entered the resolved graph",
    )?;
    let app = selected
        .iter()
        .find(|item| item.name == "app")
        .ok_or_else(|| invalid("resolved app missing"))?;
    check(
        app.requires_dist.iter().map(String::as_str).eq(requires),
        "resolution discarded original optional dependency metadata",
    )?;
    resolver::validate_graph(&roots, &selected)?;
    check(
        resolver::validate_graph(&roots, std::slice::from_ref(app)).is_err(),
        "offline validation dropped a required ordinary dependency",
    )?;
    let mut extra = selected.clone();
    extra.push(artifact("test-runner", "1.0"));
    check(
        resolver::validate_graph(&roots, &extra).is_err(),
        "offline graph accepted an unreachable optional package",
    )?;
    check(
        resolver::validate_graph(&["app; extra == 'test'".to_owned()], &selected).is_err(),
        "offline graph weakened explicit root requirements",
    )
}

#[test]
fn potentially_active_markers_are_rejected_during_resolution_and_offline_validation()
-> io::Result<()> {
    for required in [
        "other; extra == 'test' or sys_platform == 'win32'",
        "other; extra != 'test'",
        "other; extra != 'docs' and python_version >= '3.11'",
        "other; sys_platform == 'linux'",
    ] {
        let (mut app, bytes) = package("app", "1.0", &[required])?;
        let (_stage, mut registry) = fixture(vec![(app.clone(), bytes)])?;
        let roots = ["app".to_owned()];
        let error = resolver::resolve(&roots, &mut registry)
            .err()
            .ok_or_else(|| invalid("potentially active dependency was silently discarded"))?;
        check(
            error.to_string().contains("markers"),
            "lost active-marker diagnostic",
        )?;
        app.requires_dist = vec![required.to_owned()];
        let error = resolver::validate_graph(&roots, &[app])
            .err()
            .ok_or_else(|| invalid("offline graph silently discarded an active dependency"))?;
        check(
            error.to_string().contains("markers"),
            "offline graph lost active-marker diagnostic",
        )?;
    }
    Ok(())
}

#[test]
fn optional_edges_do_not_enable_explicit_extras_or_direct_urls() -> io::Result<()> {
    for (required, diagnostic) in [
        ("other[extra]", "extras"),
        ("other[extra]; extra == 'test'", "extras"),
        ("other @ https://example.com/other.whl", "direct URLs"),
        (
            "other @ https://example.com/other.whl ; extra == 'test'",
            "direct URLs",
        ),
    ] {
        let (mut app, bytes) = package("app", "1.0", &[required])?;
        let (_stage, mut registry) = fixture(vec![(app.clone(), bytes)])?;
        let roots = ["app".to_owned()];
        let error = resolver::resolve(&roots, &mut registry)
            .err()
            .ok_or_else(|| invalid("unsupported dependency form was resolved"))?;
        check(
            error.to_string().contains(diagnostic),
            "resolution lost unsupported-form diagnostic",
        )?;
        app.requires_dist = vec![required.to_owned()];
        let error = resolver::validate_graph(&roots, &[app])
            .err()
            .ok_or_else(|| invalid("offline graph accepted unsupported dependency form"))?;
        check(
            error.to_string().contains(diagnostic),
            "offline graph lost unsupported-form diagnostic",
        )?;
    }
    Ok(())
}

#[test]
fn wheel_rejects_traversal_native_hooks_and_case_collisions() -> io::Result<()> {
    let artifact = artifact("demo", "1.0");
    for name in [
        "../escape",
        "/absolute",
        "a\\b",
        "C:bad",
        "demo/NATIVE.SO",
        "site.PTH",
        "x.data/purelib/mod.py",
        "a//b",
        "a/./b",
    ] {
        check(
            wheel::inspect(&archive(&[(name, b"x")])?, &artifact).is_err(),
            "unsafe wheel path accepted",
        )?;
    }
    let bytes = archive(&[("demo/A.py", b"x"), ("demo/a.py", b"x")])?;
    check(
        wheel::inspect(&bytes, &artifact).is_err(),
        "case collision accepted",
    )
}

#[test]
fn wheel_rejects_symlinks_and_oversized_sources() -> io::Result<()> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    zip.add_symlink(
        "demo/link",
        "../../outside",
        zip::write::SimpleFileOptions::default(),
    )
    .map_err(invalid)?;
    let bytes = zip.finish().map_err(invalid)?.into_inner();
    check(
        wheel::inspect(&bytes, &artifact("demo", "1.0")).is_err(),
        "symlink accepted",
    )?;
    let bytes = archive(&[("demo/large.py", &vec![b' '; 1_048_577])])?;
    check(
        wheel::inspect(&bytes, &artifact("demo", "1.0")).is_err(),
        "oversized Python source accepted",
    )
}

#[test]
fn metadata_and_filename_identity_must_match() -> io::Result<()> {
    let (mut artifact, bytes) = package("demo", "1.0", &[])?;
    check(
        wheel::inspect(&bytes, &artifact).is_ok(),
        "valid wheel rejected",
    )?;
    artifact.filename = "other-1.0-py3-none-any.whl".to_owned();
    check(
        wheel::inspect(&bytes, &artifact).is_err(),
        "mismatched filename accepted",
    )?;
    artifact.filename = "demo-2.0-py3-none-any.whl".to_owned();
    check(
        wheel::inspect(&bytes, &artifact).is_err(),
        "mismatched filename version accepted",
    )
}

#[test]
fn corrupted_cache_fails_closed_and_offline_never_downloads() -> io::Result<()> {
    let (artifact, bytes) = package("demo", "1.0", &[])?;
    let (stage, registry) = fixture(vec![(artifact.clone(), bytes)])?;
    std::fs::write(stage.0.join(format!("{}.whl", artifact.sha256)), b"corrupt")?;
    check(
        registry.wheel(&artifact).is_err(),
        "corrupt cached artifact accepted",
    )?;
    std::fs::remove_file(stage.0.join(format!("{}.whl", artifact.sha256)))?;
    let error = registry
        .wheel(&artifact)
        .err()
        .ok_or_else(|| invalid("missing offline artifact accepted"))?;
    check(
        error.to_string().contains("offline cache"),
        "missing wheel attempted online download",
    )
}

#[test]
fn locked_graph_rejects_missing_extra_and_unsatisfied_packages() -> io::Result<()> {
    let (mut demo, _) = package("demo", "1.0", &[])?;
    demo.requires_dist = vec!["missing".to_owned()];
    check(
        resolver::validate_graph(&["demo".to_owned()], &[demo.clone()]).is_err(),
        "missing dependency accepted",
    )?;
    demo.requires_dist.clear();
    check(
        resolver::validate_graph(&["demo>=2".to_owned()], &[demo.clone()]).is_err(),
        "unsatisfied lock accepted",
    )?;
    check(
        resolver::validate_graph(&[], &[demo]).is_err(),
        "unreachable locked package accepted",
    )
}

#[test]
fn compile_checks_never_execute_package_source() -> io::Result<()> {
    let stage = Stage::new(&std::env::temp_dir())?;
    let marker = stage.0.join("EXECUTED");
    let source = format!(
        "global marker\nmarker = 1\nopen({}, 'w').write('executed')\n",
        crate::run_command::python_string(&marker.to_string_lossy())
    );
    let wheel = wheel::Wheel {
        files: BTreeMap::from([("demo.py".to_owned(), source.into_bytes())]),
        requirements: Vec::new(),
    };
    wheel::extract(&wheel, &stage.0)?;
    super::compile_sources(&wheel, &stage.0)?;
    check(
        !marker.exists(),
        "compatibility check executed untrusted source",
    )
}

#[test]
fn resolver_filters_requires_python_before_selecting_a_release() -> io::Result<()> {
    let older = package("demo", "1.0", &[])?;
    let (newer, bytes) = package("demo", "2.0", &[])?;
    let (stage, _) = fixture(vec![older])?;
    let mut project = serde_json::json!({"releases": {}});
    let old_artifact = package("demo", "1.0", &[])?.0;
    let releases = project
        .get_mut("releases")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| invalid("releases"))?;
    for (candidate, required) in [(&old_artifact, ">=3.8"), (&newer, ">=3.12")] {
        releases.insert(candidate.version.clone(), serde_json::json!([{
            "filename": candidate.filename, "url": candidate.url, "digests": {"sha256": candidate.sha256}, "requires_python": required
        }]));
    }
    std::fs::write(stage.0.join(format!("{}.whl", newer.sha256)), bytes)?;
    let mut registry = registry::Registry::fixture(
        stage.0.clone(),
        BTreeMap::from([("demo".to_owned(), project)]),
    );
    let selected = resolver::resolve(&["demo".to_owned()], &mut registry)?;
    check(
        selected.first().is_some_and(|item| item.version == "1.0"),
        "newer Python-only release was selected",
    )
}

#[test]
fn resolver_never_downgrades_to_hide_unsupported_metadata() -> io::Result<()> {
    let (_stage, mut registry) = fixture(vec![
        package("demo", "2.0", &["other; python_version>'3'"])?,
        package("demo", "1.0", &[])?,
        package("other", "1.0", &[])?,
    ])?;
    let error = resolver::resolve(&["demo".to_owned()], &mut registry)
        .err()
        .ok_or_else(|| invalid("unsupported newest metadata caused silent downgrade"))?;
    check(
        error.to_string().contains("markers"),
        "missing marker diagnostic",
    )
}

#[test]
fn catalog_blocks_exact_artifacts_before_any_wheel_download() -> io::Result<()> {
    let digest = "a".repeat(64);
    let evidence = "Fixture reproduces an exact artifact compilation blocker.";
    let project = serde_json::json!({"releases": {"1.0": [{
        "filename": "demo-1.0-py3-none-any.whl",
        "url": "https://files.pythonhosted.org/packages/demo-1.0-py3-none-any.whl",
        "digests": {"sha256": digest}, "yanked": false
    }]}});
    let stage = Stage::new(&std::env::temp_dir())?;
    let mut registry = registry::Registry::fixture(
        stage.0.clone(),
        BTreeMap::from([("demo".to_owned(), project)]),
    );
    let lookup = |artifact: &Artifact| {
        let exact =
            artifact.name == "demo" && artifact.version == "1.0" && artifact.sha256 == digest;
        Ok(crate::package_compat::Report {
            status: if exact {
                crate::package_compat::Status::Incompatible
            } else {
                crate::package_compat::Status::Unverified
            },
            diagnostics: vec![evidence.to_owned()],
        })
    };
    let error = resolver::resolve_with_lookup(&["demo==1.0".to_owned()], &mut registry, &lookup)
        .err()
        .ok_or_else(|| invalid("known catalog blocker was resolved"))?;
    check(
        error.to_string().contains("no download needed"),
        "catalog blocker was not checked before missing cache/network",
    )?;
    check(
        error.to_string().contains(evidence),
        "catalog blocker diagnostic omitted its evidence",
    )?;
    check(
        std::fs::read_dir(&stage.0)?.next().is_none(),
        "catalog shortcut populated wheel cache",
    )
}

#[test]
fn compiler_checks_later_batches_without_executing_valid_sources() -> io::Result<()> {
    let stage = Stage::new(&std::env::temp_dir())?;
    let mut files = BTreeMap::new();
    for index in 0..33 {
        files.insert(
            format!("source_{index:02}.py"),
            b"raise RuntimeError('must not execute')\n".to_vec(),
        );
    }
    files.insert("source_33.py".to_owned(), b"def broken(:\n".to_vec());
    let wheel = wheel::Wheel {
        files,
        requirements: Vec::new(),
    };
    wheel::extract(&wheel, &stage.0)?;
    let error = super::compile_sources(&wheel, &stage.0)
        .err()
        .ok_or_else(|| invalid("later source batch was never compiled"))?;
    check(
        error.to_string().contains("source_33.py"),
        "later batch failure did not retain its filename",
    )
}
