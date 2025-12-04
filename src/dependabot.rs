use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct DependabotConfig {
    /// The configuration version (always 2)
    pub version: u32,
    /// A list of update configuration blocks for each package ecosystem.
    pub updates: Vec<Update>,
    /// Optional top-level private registries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registries: Option<HashMap<String, Registry>>,
}

/// Same as Update just wiht optional Schedule
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct UpdateOverride {
    /// Defines the package ecosystem (e.g. "npm", "docker", etc.)
    pub package_ecosystem: String,
    /// A single directory path where the dependency manifests reside.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    /// Alternatively, a list of directories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directories: Option<Vec<String>>,
    /// How often to check for updates.
    pub schedule: Option<Schedule>,
    /// Optional rules to allow specific dependencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<DependencyRule>>,
    /// Optional rules to ignore certain dependencies or versions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<DependencyRule>>,
    /// Optional assignees for pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    /// Optional commit message configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<CommitMessage>,
    /// Optional labels for pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    /// Optionally associate a milestone (by numeric ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u32>,
    /// Limit on the maximum number of open pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_pull_requests_limit: Option<u32>,
    /// Optionally override registries to use for this update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registries: Option<Vec<String>>,
    /// Optional reviewers for pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<String>>,
    /// Target branch for version updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_branch: Option<String>,
    /// Whether vendored dependencies should be maintained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<bool>,
    /// Strategy for updating version constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versioning_strategy: Option<String>,
    /// Allow execution of external code during updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure_external_code_execution: Option<bool>,
    /// Optional configuration for the generated pull request branch names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_branch_name: Option<PullRequestBranchName>,
    /// Optionally disable automatic rebasing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rebase_strategy: Option<String>,
    /// Optional grouping rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<IndexMap<String, Group>>,
    /// Optional cooldown configuration for dependency updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cooldown: Option<Cooldown>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Update {
    /// Defines the package ecosystem (e.g. "npm", "docker", etc.)
    pub package_ecosystem: String,
    /// A single directory path where the dependency manifests reside.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    /// Alternatively, a list of directories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directories: Option<Vec<String>>,
    /// How often to check for updates.
    pub schedule: Schedule,
    /// Optional rules to allow specific dependencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<DependencyRule>>,
    /// Optional rules to ignore certain dependencies or versions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<DependencyRule>>,
    /// Optional assignees for pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<String>>,
    /// Optional commit message configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_message: Option<CommitMessage>,
    /// Optional labels for pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    /// Optionally associate a milestone (by numeric ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<u32>,
    /// Limit on the maximum number of open pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_pull_requests_limit: Option<u32>,
    /// Optionally override registries to use for this update.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registries: Option<Vec<String>>,
    /// Optional reviewers for pull requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<String>>,
    /// Target branch for version updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_branch: Option<String>,
    /// Whether vendored dependencies should be maintained.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<bool>,
    /// Strategy for updating version constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versioning_strategy: Option<String>,
    /// Allow execution of external code during updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure_external_code_execution: Option<bool>,
    /// Optional configuration for the generated pull request branch names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_branch_name: Option<PullRequestBranchName>,
    /// Optionally disable automatic rebasing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rebase_strategy: Option<String>,
    /// Optional grouping rules.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<IndexMap<String, Group>>,
    /// Optional cooldown configuration for dependency updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cooldown: Option<Cooldown>,
}

impl Update {
    pub fn override_config(self, other: &UpdateOverride) -> Update {
        Update {
            package_ecosystem: self.package_ecosystem,
            directory: other.directory.clone().or(self.directory.clone()),
            directories: other.directories.clone().or(self.directories.clone()),
            schedule: other.schedule.clone().unwrap_or(self.schedule.clone()),
            allow: other.allow.clone().or(self.allow.clone()),
            ignore: other.ignore.clone().or(self.ignore.clone()),
            assignees: other.assignees.clone().or(self.assignees.clone()),
            commit_message: other.commit_message.clone().or(self.commit_message.clone()),
            labels: other.labels.clone().or(self.labels.clone()),
            milestone: other.milestone.or(self.milestone),
            open_pull_requests_limit: other
                .open_pull_requests_limit
                .or(self.open_pull_requests_limit),
            registries: other.registries.clone().or(self.registries.clone()),
            reviewers: other.reviewers.clone().or(self.reviewers.clone()),
            target_branch: other.target_branch.clone().or(self.target_branch.clone()),
            vendor: other.vendor.or(self.vendor),
            versioning_strategy: other
                .versioning_strategy
                .clone()
                .or(self.versioning_strategy.clone()),
            insecure_external_code_execution: other
                .insecure_external_code_execution
                .or(self.insecure_external_code_execution),
            pull_request_branch_name: other
                .pull_request_branch_name
                .clone()
                .or(self.pull_request_branch_name.clone()),
            rebase_strategy: other
                .rebase_strategy
                .clone()
                .or(self.rebase_strategy.clone()),
            groups: other.groups.clone().or(self.groups.clone()),
            cooldown: other.cooldown.clone().or(self.cooldown.clone()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Schedule {
    /// The frequency for checking updates: "daily", "weekly", or "monthly".
    pub interval: String,
    /// Optional day for weekly updates (e.g. "monday").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day: Option<String>,
    /// Optional time of day to run the update (format "hh:mm").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// Optional timezone for the scheduled time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Optional cronjob expression for custom scheduling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cronjob: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct CommitMessage {
    /// Prefix for all commit messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// Different prefix for development dependency updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix_development: Option<String>,
    /// Additional text to include after the prefix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct PullRequestBranchName {
    /// Separator character to use in branch names.
    pub separator: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyRule {
    /// The dependency name pattern (supports wildcards).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_name: Option<String>,
    /// The type of dependency (e.g. "direct", "indirect", "development", etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_type: Option<String>,
    /// (For ignore rules) specific versions or version ranges to ignore.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versions: Option<Vec<String>>,
    /// (For ignore rules) update types (like "minor", "patch", etc.) to ignore.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Registry {
    /// The registry type (e.g. "docker-registry", "npm-registry", etc.).
    pub r#type: String,
    /// URL to access the registry.
    pub url: String,
    /// Optional username for authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Optional password (often referenced from secrets).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Alternatively, an authentication token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// When true, use the given URL instead of the ecosystem’s default base URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replaces_base: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Group {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applies_to: Option<String>,
    /// Optionally limit the group to a dependency type ("development" or "production").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_type: Option<String>,
    /// Patterns of dependency names to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patterns: Option<Vec<String>>,
    /// Patterns of dependency names to exclude.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_patterns: Option<Vec<String>>,
    /// Limit the group to certain update types (e.g. "minor", "patch", "major").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Cooldown {
    /// Default cooldown period for dependencies without specific rules (in days).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_days: Option<u32>,
    /// Cooldown period for major version updates (in days).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semver_major_days: Option<u32>,
    /// Cooldown period for minor version updates (in days).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semver_minor_days: Option<u32>,
    /// Cooldown period for patch version updates (in days).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semver_patch_days: Option<u32>,
    /// List of dependencies to apply cooldown (supports wildcards, up to 150 items).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// List of dependencies excluded from cooldown (supports wildcards, up to 150 items).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}
