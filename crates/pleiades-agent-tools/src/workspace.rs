use std::path::{Component, Path, PathBuf};

use pleiades_agent_core::error::Error;
use pleiades_agent_core::tool::ToolContext;

pub(crate) fn resolve_path(
    value: impl AsRef<Path>,
    context: &ToolContext,
    allow_missing: bool,
) -> Result<PathBuf, Error> {
    let workspace = context
        .working_directory
        .canonicalize()
        .map_err(|error| Error::io(format!("Could not resolve workspace root: {error}")))?;
    let value = value.as_ref();
    let candidate = if value.is_absolute() {
        value.to_path_buf()
    } else {
        workspace.join(value)
    };
    let candidate = normalize(&candidate)?;

    if candidate.exists() {
        let resolved = candidate.canonicalize().map_err(|error| {
            Error::io(format!("Could not resolve '{}': {error}", value.display()))
        })?;
        if !resolved.starts_with(&workspace) {
            return Err(boundary_error(value, &workspace));
        }
        return Ok(resolved);
    }
    if !allow_missing {
        return Err(Error::io(format!(
            "Path '{}' does not exist",
            value.display()
        )));
    }

    let mut ancestor = candidate.as_path();
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or_else(|| boundary_error(value, &workspace))?;
    }
    let resolved_ancestor = ancestor.canonicalize().map_err(|error| {
        Error::io(format!(
            "Could not resolve '{}': {error}",
            ancestor.display()
        ))
    })?;
    if !resolved_ancestor.starts_with(&workspace) {
        return Err(boundary_error(value, &workspace));
    }
    let suffix = candidate
        .strip_prefix(ancestor)
        .map_err(|_| boundary_error(value, &workspace))?;
    Ok(resolved_ancestor.join(suffix))
}

pub(crate) fn ensure_pattern_is_relative(pattern: &str) -> Result<(), Error> {
    let path = Path::new(pattern);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(Error::ToolPermissionDenied {
            name: "filesystem".to_string(),
            level: "workspace boundary".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn is_inside_workspace(path: &Path, context: &ToolContext) -> bool {
    let Ok(workspace) = context.working_directory.canonicalize() else {
        return false;
    };
    path.canonicalize()
        .map(|resolved| resolved.starts_with(workspace))
        .unwrap_or(false)
}

fn normalize(path: &Path) -> Result<PathBuf, Error> {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => result.push(prefix.as_os_str()),
            Component::RootDir => result.push(Path::new(std::path::MAIN_SEPARATOR_STR)),
            Component::CurDir => {}
            Component::Normal(part) => result.push(part),
            Component::ParentDir => {
                if !result.pop() {
                    return Err(Error::invalid_input("Path escapes its filesystem root"));
                }
            }
        }
    }
    Ok(result)
}

fn boundary_error(value: &Path, workspace: &Path) -> Error {
    Error::tool(format!(
        "Path '{}' is outside the selected workspace '{}'",
        value.display(),
        workspace.display()
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pleiades_agent_core::tool::{PermissionMode, ToolContext};

    use super::resolve_path;

    fn context(root: &std::path::Path) -> ToolContext {
        ToolContext {
            cwd: root.to_path_buf(),
            working_directory: root.to_path_buf(),
            permission_mode: PermissionMode::Ask,
            sandbox_mode: "workspace-write".to_string(),
            config: Arc::new(serde_json::Value::Null),
        }
    }

    #[test]
    fn rejects_parent_traversal() {
        let root = tempfile::tempdir().unwrap();
        let error = resolve_path("../secret", &context(root.path()), true).unwrap_err();
        assert!(error.to_string().contains("outside the selected workspace"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_that_leave_the_workspace() {
        let root = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink("/etc", root.path().join("external")).unwrap();
        assert!(resolve_path("external/passwd", &context(root.path()), false).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn accepts_an_absolute_path_through_a_workspace_alias() {
        let root = tempfile::tempdir().unwrap();
        let aliases = tempfile::tempdir().unwrap();
        let alias = aliases.path().join("workspace");
        std::os::unix::fs::symlink(root.path(), &alias).unwrap();
        let file = alias.join("sample.txt");
        std::fs::write(root.path().join("sample.txt"), "content").unwrap();

        assert_eq!(
            resolve_path(&file, &context(root.path()), false).unwrap(),
            root.path().canonicalize().unwrap().join("sample.txt")
        );
        assert_eq!(
            resolve_path(alias.join("new.txt"), &context(root.path()), true).unwrap(),
            root.path().canonicalize().unwrap().join("new.txt")
        );
    }
}
