use std::path::{Path, PathBuf};

use pleiades_agent_core::error::Error;

use crate::common::git_diff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffReview {
    pub raw: String,
    pub staged: bool,
    pub files: Vec<DiffFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffFile {
    pub old_path: Option<PathBuf>,
    pub new_path: Option<PathBuf>,
    pub hunks: Vec<DiffHunk>,
    pub binary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: usize,
    pub old_len: usize,
    pub new_start: usize,
    pub new_len: usize,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    Context,
    Added,
    Removed,
    NoNewline,
}

impl DiffReview {
    pub fn summary(&self) -> String {
        let hunks = self
            .files
            .iter()
            .map(|file| file.hunks.len())
            .sum::<usize>();
        format!(
            "{} file(s), {hunks} hunk(s), {}",
            self.files.len(),
            if self.staged { "staged" } else { "unstaged" }
        )
    }
}

pub async fn working_diff_review(repository: &Path, staged: bool) -> Result<DiffReview, Error> {
    let raw = if staged {
        git_diff(repository, &["diff", "--staged", "--no-ext-diff"]).await?
    } else {
        git_diff(repository, &["diff", "--no-ext-diff"]).await?
    };
    parse_unified_diff(&raw, staged)
}

pub fn parse_unified_diff(raw: &str, staged: bool) -> Result<DiffReview, Error> {
    let mut files = Vec::<DiffFile>::new();
    let mut current_file: Option<DiffFile> = None;
    let mut current_hunk: Option<DiffHunk> = None;

    for line in raw.lines() {
        if line.starts_with("diff --git ") {
            flush_hunk(&mut current_file, &mut current_hunk);
            if let Some(file) = current_file.take() {
                files.push(file);
            }
            current_file = Some(DiffFile {
                old_path: None,
                new_path: None,
                hunks: Vec::new(),
                binary: false,
            });
            continue;
        }

        let Some(file) = current_file.as_mut() else {
            continue;
        };

        if let Some(rest) = line.strip_prefix("--- ") {
            file.old_path = parse_diff_path(rest);
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            file.new_path = parse_diff_path(rest);
        } else if line.starts_with("Binary files ") {
            file.binary = true;
        } else if line.starts_with("@@ ") {
            flush_hunk(&mut current_file, &mut current_hunk);
            current_hunk = Some(parse_hunk_header(line)?);
        } else if let Some(hunk) = current_hunk.as_mut() {
            if let Some(content) = line.strip_prefix(' ') {
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Context,
                    content: with_newline(content),
                });
            } else if let Some(content) = line.strip_prefix('+') {
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Added,
                    content: with_newline(content),
                });
            } else if let Some(content) = line.strip_prefix('-') {
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::Removed,
                    content: with_newline(content),
                });
            } else if line.starts_with("\\ No newline at end of file") {
                hunk.lines.push(DiffLine {
                    kind: DiffLineKind::NoNewline,
                    content: line.to_string(),
                });
            }
        }
    }

    flush_hunk(&mut current_file, &mut current_hunk);
    if let Some(file) = current_file.take() {
        files.push(file);
    }

    Ok(DiffReview {
        raw: raw.to_string(),
        staged,
        files,
    })
}

pub fn revert_hunk_in_worktree(
    repository: &Path,
    review: &DiffReview,
    file_index: usize,
    hunk_index: usize,
) -> Result<(), Error> {
    if review.staged {
        return Err(Error::invalid_input(
            "cannot revert staged hunks in this slice",
        ));
    }
    let file = review
        .files
        .get(file_index)
        .ok_or_else(|| Error::invalid_input("diff file index out of range"))?;
    if file.binary {
        return Err(Error::invalid_input("cannot revert binary hunks"));
    }
    let hunk = file
        .hunks
        .get(hunk_index)
        .ok_or_else(|| Error::invalid_input("diff hunk index out of range"))?;
    let path = file
        .new_path
        .as_ref()
        .or(file.old_path.as_ref())
        .ok_or_else(|| Error::invalid_input("diff file has no path"))?;
    let absolute = repository.join(path);
    let content = std::fs::read_to_string(&absolute).map_err(Error::from)?;
    let mut lines = split_lines_lossless(&content);
    let replacement = hunk
        .lines
        .iter()
        .filter_map(|line| match line.kind {
            DiffLineKind::Context | DiffLineKind::Removed => Some(line.content.clone()),
            DiffLineKind::Added | DiffLineKind::NoNewline => None,
        })
        .collect::<Vec<_>>();
    let start = hunk.new_start.saturating_sub(1);
    let end = start.saturating_add(hunk.new_len).min(lines.len());
    if start > lines.len() {
        return Err(Error::invalid_input("hunk start is outside the file"));
    }
    lines.splice(start..end, replacement);
    std::fs::write(absolute, lines.concat()).map_err(Error::from)
}

fn flush_hunk(file: &mut Option<DiffFile>, hunk: &mut Option<DiffHunk>) {
    if let (Some(file), Some(hunk)) = (file.as_mut(), hunk.take()) {
        file.hunks.push(hunk);
    }
}

fn parse_diff_path(value: &str) -> Option<PathBuf> {
    if value == "/dev/null" {
        return None;
    }
    let normalized = value
        .strip_prefix("a/")
        .or_else(|| value.strip_prefix("b/"))
        .unwrap_or(value);
    Some(PathBuf::from(normalized))
}

fn parse_hunk_header(line: &str) -> Result<DiffHunk, Error> {
    let end = line[3..]
        .find(" @@")
        .map(|index| index + 3)
        .ok_or_else(|| Error::invalid_input(format!("invalid hunk header `{line}`")))?;
    let range = &line[3..end];
    let mut pieces = range.split_whitespace();
    let old = pieces
        .next()
        .ok_or_else(|| Error::invalid_input("missing old hunk range"))?;
    let new = pieces
        .next()
        .ok_or_else(|| Error::invalid_input("missing new hunk range"))?;
    let (old_start, old_len) = parse_range(old.trim_start_matches('-'))?;
    let (new_start, new_len) = parse_range(new.trim_start_matches('+'))?;
    Ok(DiffHunk {
        header: line.to_string(),
        old_start,
        old_len,
        new_start,
        new_len,
        lines: Vec::new(),
    })
}

fn parse_range(value: &str) -> Result<(usize, usize), Error> {
    let (start, len) = value.split_once(',').unwrap_or((value, "1"));
    Ok((
        start
            .parse()
            .map_err(|_| Error::invalid_input(format!("invalid hunk start `{start}`")))?,
        len.parse()
            .map_err(|_| Error::invalid_input(format!("invalid hunk length `{len}`")))?,
    ))
}

fn with_newline(value: &str) -> String {
    format!("{value}\n")
}

fn split_lines_lossless(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }
    value
        .split_inclusive('\n')
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unified_diff_files_and_hunks() {
        let review = parse_unified_diff(
            r#"diff --git a/src/main.rs b/src/main.rs
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,2 +1,3 @@
 fn main() {
-    println!("old");
+    println!("new");
+    println!("extra");
 }
"#,
            false,
        )
        .unwrap();
        assert_eq!(review.files.len(), 1);
        assert_eq!(review.files[0].new_path, Some(PathBuf::from("src/main.rs")));
        assert_eq!(review.files[0].hunks.len(), 1);
        assert_eq!(review.files[0].hunks[0].old_start, 1);
        assert_eq!(review.files[0].hunks[0].new_len, 3);
    }

    #[tokio::test]
    async fn revert_hunk_restores_file_content_exactly() {
        let directory = tempfile::tempdir().unwrap();
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            let status = tokio::process::Command::new("git")
                .args(args)
                .current_dir(directory.path())
                .status()
                .await
                .unwrap();
            assert!(status.success());
        }
        let path = directory.path().join("file.txt");
        let original = "alpha\nbeta\ngamma\n";
        std::fs::write(&path, original).unwrap();
        let status = tokio::process::Command::new("git")
            .args(["add", "file.txt"])
            .current_dir(directory.path())
            .status()
            .await
            .unwrap();
        assert!(status.success());
        let status = tokio::process::Command::new("git")
            .args(["commit", "-qm", "initial"])
            .current_dir(directory.path())
            .status()
            .await
            .unwrap();
        assert!(status.success());

        std::fs::write(&path, "alpha\nBETA\ngamma\nextra\n").unwrap();
        let review = working_diff_review(directory.path(), false).await.unwrap();
        revert_hunk_in_worktree(directory.path(), &review, 0, 0).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), original);
    }
}
