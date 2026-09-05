//! Bounded backtracking over PEP 440 versions and PEP 508 requirements.
use std::collections::BTreeMap;
use std::io;

use pep508_rs::{Requirement, VersionOrUrl, pep440_rs::Version};

use super::{Artifact, invalid, registry::Registry, wheel};

fn conflict(message: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, message.to_string())
}

const MAX_PACKAGES: usize = 128;
const MAX_ATTEMPTS: usize = 512;

fn parse_requirement(text: &str) -> io::Result<Requirement> {
    let parsed: Requirement = text
        .parse()
        .map_err(|error| invalid(format!("invalid requirement {text:?}: {error}")))?;
    if !parsed.extras.is_empty() {
        return Err(invalid(format!(
            "{text}: extras are not supported yet; specify ordinary distribution requirements"
        )));
    }
    if matches!(parsed.version_or_url, Some(VersionOrUrl::Url(_))) {
        return Err(invalid(format!(
            "{text}: direct URLs, VCS and local dependencies are not supported; use a PyPI package name"
        )));
    }
    Ok(parsed)
}

fn require_unconditional(text: &str, parsed: Requirement) -> io::Result<Requirement> {
    if !parsed.marker.is_true() {
        return Err(invalid(format!(
            "{text}: environment markers are not supported yet; dependency selection would be ambiguous"
        )));
    }
    Ok(parsed)
}

pub(super) fn requirement(text: &str) -> io::Result<Requirement> {
    require_unconditional(text, parse_requirement(text)?)
}

fn transitive_requirement(text: &str) -> io::Result<Option<Requirement>> {
    let parsed = parse_requirement(text)?;
    // No extras can be requested through our public requirement parser. The
    // marker API existentially evaluates all remaining environments: false
    // proves this edge is inactive without inventing host or target values.
    if !parsed.marker.evaluate_extras(&[]) {
        return Ok(None);
    }
    require_unconditional(text, parsed).map(Some)
}

fn matches(requirement: &Requirement, version: &Version) -> bool {
    match &requirement.version_or_url {
        Some(VersionOrUrl::VersionSpecifier(specifiers)) => specifiers.contains(version),
        None => true,
        Some(VersionOrUrl::Url(_)) => false,
    }
}

pub(super) fn resolve(
    requirements: &[String],
    registry: &mut Registry,
) -> io::Result<Vec<Artifact>> {
    let runtime = super::runtime_hash()?;
    resolve_with_lookup(requirements, registry, &|artifact| {
        crate::package_compat::lookup(
            &artifact.name,
            &artifact.version,
            &artifact.sha256,
            &runtime,
        )
    })
}

// Keep lookup ordering independently testable without requiring new live audit
// evidence whenever an embedded runtime binary changes. Production callers use
// resolve above, which always binds the real runtime hash and host target.
pub(super) fn resolve_with_lookup(
    requirements: &[String],
    registry: &mut Registry,
    lookup: &impl Fn(&Artifact) -> io::Result<crate::package_compat::Report>,
) -> io::Result<Vec<Artifact>> {
    let pending = requirements
        .iter()
        .map(|text| requirement(text))
        .collect::<io::Result<Vec<_>>>()?;
    let mut attempts = 0;
    let selected = search(&pending, BTreeMap::new(), registry, &mut attempts, lookup)?;
    Ok(selected.into_values().collect())
}

fn search(
    pending: &[Requirement],
    selected: BTreeMap<String, Artifact>,
    registry: &mut Registry,
    attempts: &mut usize,
    lookup: &impl Fn(&Artifact) -> io::Result<crate::package_compat::Report>,
) -> io::Result<BTreeMap<String, Artifact>> {
    if selected.len() > MAX_PACKAGES || *attempts >= MAX_ATTEMPTS {
        return Err(invalid(
            "dependency resolution exceeded 128 packages or 512 candidates; use tighter version constraints",
        ));
    }
    for required in pending {
        if let Some(existing) = selected.get(required.name.as_ref()) {
            let version = existing.version.parse::<Version>().map_err(invalid)?;
            if !matches(required, &version) {
                return Err(conflict(format!(
                    "dependency conflict: {required}, but {}=={} was selected",
                    existing.name, existing.version
                )));
            }
        }
    }
    let Some(next) = pending
        .iter()
        .find(|item| !selected.contains_key(item.name.as_ref()))
    else {
        return Ok(selected);
    };
    let name = next.name.to_string();
    let constraints: Vec<_> = pending
        .iter()
        .filter(|item| item.name == next.name)
        .collect();
    let candidates = registry.candidates(&name)?;
    let mut last = format!(
        "{name}: no non-yanked py3-none-any wheel satisfies all requirements (native extensions/source builds are unsupported)"
    );
    for mut artifact in candidates {
        let version = artifact.version.parse::<Version>().map_err(invalid)?;
        if !constraints.iter().all(|item| matches(item, &version)) {
            continue;
        }
        // PEP 440 excludes prereleases by default; explicit prerelease constraints opt in.
        if version.any_prerelease() && !constraints.iter().any(|item| {
            matches!(&item.version_or_url, Some(VersionOrUrl::VersionSpecifier(specifiers)) if specifiers.iter().any(|specifier| specifier.any_prerelease() && !matches!(specifier.operator(), pep508_rs::pep440_rs::Operator::NotEqual | pep508_rs::pep440_rs::Operator::NotEqualStar)))
        }) { continue; }
        *attempts = attempts.saturating_add(1);
        if *attempts > MAX_ATTEMPTS {
            return Err(invalid("dependency resolution exceeded 512 candidates"));
        }
        let report = lookup(&artifact)?;
        if report.status == crate::package_compat::Status::Incompatible {
            return Err(invalid(format!(
                "{}=={} is incompatible with this exact runtime and wheel (catalog; no download needed):\n{}",
                artifact.name,
                artifact.version,
                report.diagnostics.join("\n")
            )));
        }
        let bytes = registry.wheel(&artifact)?;
        let wheel = wheel::inspect(&bytes, &artifact)?;
        artifact.requires_dist = wheel.requirements;
        let mut remaining = pending.to_vec();
        for text in &artifact.requires_dist {
            if let Some(required) = transitive_requirement(text).map_err(|error| {
                invalid(format!(
                    "{}=={} -> {error}",
                    artifact.name, artifact.version
                ))
            })? {
                remaining.push(required);
            }
        }
        let mut branch = selected.clone();
        branch.insert(name.clone(), artifact.clone());
        let result = search(&remaining, branch, registry, attempts, lookup);
        match result {
            Ok(solution) => return Ok(solution),
            Err(error) if error.kind() == io::ErrorKind::NotFound => last = error.to_string(),
            Err(error) => return Err(error),
        }
    }
    Err(conflict(last))
}

pub(super) fn validate_graph(requirements: &[String], artifacts: &[Artifact]) -> io::Result<()> {
    let mut packages = BTreeMap::new();
    for artifact in artifacts {
        let name = artifact
            .name
            .parse::<pep508_rs::PackageName>()
            .map_err(invalid)?;
        if name.as_ref() != artifact.name
            || packages.insert(artifact.name.clone(), artifact).is_some()
        {
            return Err(invalid(
                "lock has duplicate or non-normalized package names",
            ));
        }
    }
    let mut pending = requirements
        .iter()
        .map(|text| requirement(text))
        .collect::<io::Result<Vec<_>>>()?;
    let mut visited = std::collections::BTreeSet::new();
    while let Some(needed) = pending.pop() {
        let artifact = packages
            .get(needed.name.as_ref())
            .ok_or_else(|| invalid(format!("lock is missing {needed}")))?;
        if !matches(&needed, &artifact.version.parse().map_err(invalid)?) {
            return Err(invalid(format!(
                "locked {}=={} does not satisfy {needed}",
                artifact.name, artifact.version
            )));
        }
        if visited.insert(artifact.name.clone()) {
            for text in &artifact.requires_dist {
                if let Some(required) = transitive_requirement(text)? {
                    pending.push(required);
                }
            }
        }
        if visited.len() > MAX_PACKAGES {
            return Err(invalid("lock has too many packages"));
        }
    }
    if visited.len() != artifacts.len() {
        return Err(invalid(
            "lock contains packages outside the dependency graph",
        ));
    }
    Ok(())
}
