use serde::{Deserialize, Serialize};

/// GitHub webhook event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GitHubEventType {
    Push,
    PullRequest,
    Issues,
    IssueComment,
    Release,
    Create,
    Delete,
    Fork,
    Watch,
    Other(String),
}

/// GitLab webhook event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GitLabEventType {
    PushHook,
    MergeRequestHook,
    IssueHook,
    NoteHook,
    TagPushHook,
    PipelineHook,
    Other(String),
}
