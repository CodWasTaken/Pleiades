//! Git integration for Pleiades.
//!
//! Provides commit message generation, PR summaries,
//! code review, and diff explanations.

pub mod commit;
pub mod review;

pub use commit::CommitGenerator;
pub use review::ReviewGenerator;
