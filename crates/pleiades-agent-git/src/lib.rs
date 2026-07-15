//! Git integration for Pleiades.
//!
//! Provides commit message generation, PR summaries,
//! code review, and diff explanations.

pub mod commit;
mod common;
pub mod diff;
pub mod review;
pub mod summary;

pub use commit::{CommitGenerator, staged_diff, working_diff};
pub use diff::{
    DiffFile, DiffHunk, DiffLine, DiffLineKind, DiffReview, revert_hunk_in_worktree,
    working_diff_review,
};
pub use review::ReviewGenerator;
pub use summary::PrSummaryGenerator;
