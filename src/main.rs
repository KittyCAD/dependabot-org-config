mod dependabot;
mod github;

use crate::dependabot::Registry;
use anyhow::Context;
use argh::FromArgs;
use dependabot::{Cooldown, DependabotConfig, Group, Schedule, Update, UpdateOverride};
use github::{AssetLevel, CustomPropertyExt, get_all, get_all_repos};
use indexmap::IndexMap;
use indicatif::ProgressIterator;
use octocrab::Octocrab;
use octocrab::models::repos::{Content, Object};
use octocrab::models::{Code, Repository};
use octocrab::params::State;
use octocrab::params::repos::Reference;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::time::Duration;
use std::{env, fs};
use tokio::time::sleep;

#[derive(FromArgs)]
/// Check Dependabot status for all repositories in an organization
struct Args {
    // GitHub organization name
    #[argh(positional, description = "organization name")]
    org: String,
    #[argh(option, description = "optional cache to use for ecosystems")]
    ecosystems_cache: Option<String>,
    #[argh(option, description = "optional dependabot_overrides file path")]
    dependabot_overrides: Option<String>,

    #[argh(
        switch,
        description = "whether to create PRs for the dependabot config"
    )]
    create_pr: bool,

    #[argh(switch, description = "force creation of new dependabot config")]
    force_new: bool,

    #[argh(option, description = "limit to repos")]
    repo: Vec<String>,

    #[argh(switch, description = "whether to print verbose output")]
    verbose: bool,

    #[argh(switch, description = "only process repos with existing PRs")]
    only_existing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DependabotOverrides {
    registries: HashMap<String, Registry>,
    updates: HashMap<String, Vec<UpdateOverride>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Args = argh::from_env();
    let gh_token = env::var("GH_TOKEN").context("GitHub token not set")?;

    let octocrab = Octocrab::builder()
        .user_access_token(gh_token)
        .build()
        .expect("Failed to create GitHub client");

    let ecosystems = if let Some(ecosystem_cache) = &args.ecosystems_cache {
        if fs::exists(ecosystem_cache)? {
            let file = File::open(ecosystem_cache).context("failed to open file")?;
            serde_json::from_reader(&file).context("failed to read JSON file")?
        } else {
            let ecosystems = find_ecosystems(&octocrab).await?;
            let file = File::create(ecosystem_cache).context("failed to create file")?;
            serde_json::to_writer(&file, &ecosystems).context("failed to write JSON to file")?;
            ecosystems
        }
    } else {
        find_ecosystems(&octocrab).await?
    };

    let dependabot_overrides = if let Some(dependabot_overrides_file) = &args.dependabot_overrides {
        let mut file = File::open(dependabot_overrides_file).context("failed to open file")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let dependabot_overrides: DependabotOverrides =
            toml::from_str(&contents).context("failed to read overrides TOML from file")?;
        dependabot_overrides
    } else {
        DependabotOverrides {
            registries: Default::default(),
            updates: Default::default(),
        }
    };

    let repos = get_all_repos(&octocrab, &args.org)
        .await
        .context("failed to fetch repos")?;

    if repos.is_empty() {
        log::warn!("No repositories found.");
        return Ok(());
    }

    let default_schedule = Schedule {
        interval: "weekly".to_string(),
        day: Some("saturday".to_string()),
        time: None, // Some("03:00".to_string()),
        timezone: Some("America/Los_Angeles".to_string()),
        ..Schedule::default()
    };
    let open_pull_requests_limit = Some(5);
    let default_groups = IndexMap::from([
        (
            "security".to_string(),
            Group {
                applies_to: Some("security-updates".to_string()),
                update_types: Some(vec!["minor".to_string(), "patch".to_string()]),
                exclude_patterns: Some(vec!["kittycad*".to_string()]),
                ..Group::default()
            },
        ),
        (
            "security-major".to_string(),
            Group {
                applies_to: Some("security-updates".to_string()),
                update_types: Some(vec!["major".to_string()]),
                exclude_patterns: Some(vec!["kittycad*".to_string()]),
                ..Group::default()
            },
        ),
        (
            "patch".to_string(),
            Group {
                applies_to: Some("version-updates".to_string()),
                update_types: Some(vec!["patch".to_string()]),
                exclude_patterns: Some(vec!["kittycad*".to_string()]),
                ..Group::default()
            },
        ),
        (
            "major".to_string(),
            Group {
                applies_to: Some("version-updates".to_string()),
                update_types: Some(vec!["major".to_string()]),
                exclude_patterns: Some(vec!["kittycad*".to_string()]),
                ..Group::default()
            },
        ),
        (
            "minor".to_string(),
            Group {
                applies_to: Some("version-updates".to_string()),
                update_types: Some(vec!["minor".to_string(), "patch".to_string()]),
                exclude_patterns: Some(vec!["kittycad*".to_string()]),
                ..Group::default()
            },
        ),
    ]);

    let default_cooldown = Cooldown {
        default_days: Some(7),
        exclude: Some(vec![
            "*kcl*".to_string(),
            "*zoo*".to_string(),
            "*kittycad*".to_string(),
        ]),
        ..Cooldown::default()
    };

    for repo in repos.iter().progress() {
        // Filter out archived repos
        // Filter out repos that are not enabled via CLI
        if repo.archived.unwrap_or(false)
            || (!args.repo.is_empty() && !args.repo.contains(&repo.name))
        {
            continue;
        }

        let props = octocrab
            .list_custom_properties("KittyCAD", &repo.name)
            .await?;

        let repo_level = AssetLevel::get_from_props(&props);

        if repo_level.is_none() || repo_level == Some(AssetLevel::Playground) {
            log::debug!("Skipping repo {} as it is a playground repo", repo.name);
            continue;
        }

        // Get existing dependabot file
        let existing_dependabot = get_dependabot_yml(&octocrab, repo, "main").await?;

        if existing_dependabot.is_none() && !args.force_new {
            println!(
                "No existing dependabot config for repo {}, not creating a PR without --force-new",
                repo.name
            );
            continue;
        }

        if args.only_existing {
            let prs = octocrab
                .pulls("KittyCAD", &repo.name)
                .list()
                .state(State::Open)
                .base("main")
                .head("KittyCAD:ciso/update-dependabot")
                .send()
                .await?
                .items;
            if prs.is_empty() {
                log::info!("Skipping repo {} as it has no open PR", repo.name);
                continue;
            }
        }

        // Find updates
        let has_gha_config = has_gha_config(&octocrab, repo).await?;

        let mut updates = if has_gha_config {
            let gha_update = Update {
                package_ecosystem: "github-actions".to_string(),
                directory: Some("/".to_string()),
                schedule: default_schedule.clone(),
                open_pull_requests_limit,
                groups: Some(default_groups.clone()),
                cooldown: Some(default_cooldown.clone()),
                ..Update::default()
            };
            vec![apply_override(
                gha_update,
                &dependabot_overrides.updates,
                repo,
                &Ecosystem::GitHubActions,
            )]
        } else {
            vec![]
        };

        if let Some(ecosystems) =
            ecosystems.get(repo.full_name.as_ref().expect("full name must exist"))
        {
            for (path, ecosystem) in ecosystems {
                // Remove /repositories/848456627/contents/
                let path = path.split("/").skip(4).collect::<Vec<_>>();
                // Remove last filename
                let path = "/".to_string() + &path[..path.len() - 1].join("/");

                if updates.iter().any(|update| {
                    update.directory.as_ref() == Some(&path)
                        && update.package_ecosystem == ecosystem.to_string()
                }) {
                    log::warn!(
                        "Tried to generate an update config that would conflict with existing one for repo {} and ecosystem {}. Skipping...",
                        repo.name,
                        ecosystem
                    );
                    // TODO: If we configure target-branch, then we have to take this into consideration here aswell
                    continue;
                }

                let cooldown = match ecosystem {
                    Ecosystem::Submodule => None,
                    _ => Some(default_cooldown.clone()),
                };

                let update = Update {
                    package_ecosystem: ecosystem.to_string(),
                    directory: Some(path),
                    schedule: default_schedule.clone(),
                    groups: Some(default_groups.clone()),
                    reviewers: None,
                    open_pull_requests_limit,
                    cooldown,
                    ..Update::default()
                };

                // Apply overrides
                let update = apply_override(update, &dependabot_overrides.updates, repo, ecosystem);

                updates.push(update);

                log::debug!("Found ecosystem {:?} in repo {}", ecosystem, repo.name);
            }
        }

        // We don't generate registries right now so we can just take the overrides if they exist.
        let registries = if dependabot_overrides.registries.is_empty() {
            None
        } else {
            Some(dependabot_overrides.registries.clone())
        };

        // Apply updates if necessary
        if !updates.is_empty() {
            let config = DependabotConfig {
                version: 2,
                updates,
                registries,
            };

            if args.verbose {
                let content = serde_yaml_ng::to_string(&config)?;

                println!("{}", content);
            }

            create_pr(&octocrab, repo, &config, !args.create_pr).await?;
        } else {
            log::warn!("No potential dependabot config found for {}", repo.name);
            // TODO: Potentially make a PR to remove the file?
        }
    }
    Ok(())
}

fn apply_override(
    update: Update,
    dependabot_overrides: &HashMap<String, Vec<UpdateOverride>>,
    repo: &Repository,
    ecosystem: &Ecosystem,
) -> Update {
    if let Some(override_updates) = dependabot_overrides.get(&repo.name) {
        let matching_overrides = override_updates
            .iter()
            .filter(|update| update.package_ecosystem == ecosystem.to_string())
            .collect::<Vec<_>>();

        if matching_overrides.len() > 1 {
            panic!("found more than one override");
        }

        log::debug!("found override for repo {}", repo.name);

        if let Some(override_update) = matching_overrides.first() {
            update.override_config(override_update)
        } else {
            update
        }
    } else {
        update
    }
}

async fn create_pr(
    octocrab: &Octocrab,
    repo: &Repository,
    config: &DependabotConfig,
    dry: bool,
) -> anyhow::Result<()> {
    let octocrab_repo = octocrab.repos("KittyCAD", &repo.name);

    let main_ref = octocrab_repo
        .get_ref(&Reference::Branch("main".to_string()))
        .await
        .context("failed to fetch ref to main branch")?;

    let existing_config = if octocrab_repo
        .get_ref(&Reference::Branch("ciso/update-dependabot".to_string()))
        .await
        .is_err()
    {
        // Create branch
        if !dry {
            octocrab_repo
                .create_ref(
                    &Reference::Branch("ciso/update-dependabot".to_string()),
                    match main_ref.object {
                        Object::Commit { sha, .. } => sha,
                        Object::Tag { sha, .. } => sha,
                        _ => panic!("unexpected object type"),
                    },
                )
                .await?;
        }

        // get current config from main
        get_dependabot_yml_content(octocrab, repo, "main").await?
    } else {
        // get current config from branch
        get_dependabot_yml_content(octocrab, repo, "ciso/update-dependabot").await?
    };

    let content = serde_yaml_ng::to_string(&config)?;
    let content = "# DO NOT EDIT THIS FILE. This dependabot file was generated \n\
                # by https://github.com/KittyCAD/ciso Changes to this file should be addressed in \n\
                # the ciso repository.\n\n".to_string() + &content;

    if let Some(existing_content) = existing_config {
        if let Some(decoded_content) = existing_content.decoded_content()
            && decoded_content == content
        {
            log::info!("No changes for {}", repo.name);
            return Ok(());
        }

        if !dry {
            log::info!("Updating dependabot file for {}", repo.name);
            octocrab_repo
                .update_file(
                    ".github/dependabot.yml",
                    "Update dependabot config from KittyCAD/ciso",
                    &content,
                    existing_content.sha,
                )
                .branch("ciso/update-dependabot")
                .send()
                .await?;
        }
    } else if !dry {
        log::info!("Creating dependabot file for {}", repo.name);
        octocrab_repo
            .create_file(
                ".github/dependabot.yml",
                "Update dependabot config from KittyCAD/ciso",
                &content,
            )
            .branch("ciso/update-dependabot")
            .send()
            .await?;
    }

    if !dry {
        match octocrab
            .pulls("KittyCAD", &repo.name)
            .create("Update dependabot config", "ciso/update-dependabot", "main")
            .body("This PR was automatically generated from KittyCAD/ciso. Let @maxammann know if you want changes applied to the PR. Please merge this soon.")
            .send()
            .await {
            Ok(r) => {
                log::info!("Created PR for {}: {}", repo.name, r.html_url.map(|url| url.to_string()).unwrap_or("no url".to_string()));

                // TODO octocrab.pulls("KittyCAD", &repo.name).request_reviews(r.number, vec!["maxammann".to_string()], vec![]).await?;
            }
            Err(e) => log::warn!("Did not create a (new) PR for {}. Likely it already exists. origin: {}", repo.name, e)
        }
    } else {
        log::info!(
            "Would create or update PR for {}. Pass --create-pr to perform the changes.",
            repo.name
        );
    }

    Ok(())
}

async fn get_dependabot_yml(
    octocrab: &Octocrab,
    repository: &Repository,
    branch: &str,
) -> anyhow::Result<Option<(DependabotConfig, String)>> {
    let Some(content) = get_dependabot_yml_content(octocrab, repository, branch).await? else {
        return Ok(None);
    };

    let text = content
        .decoded_content()
        .context("failed to decode content")?;

    let config = serde_yaml_ng::from_str::<DependabotConfig>(&text)?;
    Ok(Some((config.clone(), content.sha.clone())))
}

async fn get_dependabot_yml_content(
    octocrab: &Octocrab,
    repository: &Repository,
    branch: &str,
) -> anyhow::Result<Option<Content>> {
    let mut result = octocrab
        .repos("KittyCAD", &repository.name)
        .get_content()
        .path(".github/dependabot.yml")
        .r#ref(branch)
        .send()
        .await
        .context("failed to fetch content")
        .map(|items| items.items)
        .unwrap_or_default();

    if result.is_empty() {
        return Ok(None);
    }

    if result.len() != 1 {
        panic!("found more than one dependabot config")
    }

    Ok(Some(result.remove(0)))
}

async fn has_gha_config(octocrab: &Octocrab, repository: &Repository) -> anyhow::Result<bool> {
    let result = octocrab
        .repos("KittyCAD", &repository.name)
        .get_content()
        .path(".github/workflows")
        .r#ref("main")
        .send()
        .await
        .context("failed to content for GHA check")
        .map(|items| items.items)
        .unwrap_or_default();

    if result.is_empty() {
        Ok(false)
    } else {
        Ok(true)
    }
}
async fn search_ecosystems(
    octocrab: &Octocrab,
    file: &str,
    content: Option<&str>,
) -> anyhow::Result<Vec<Code>> {
    log::info!("Searching for ecosystems using file: {}", file);

    let repos = get_all(octocrab, move |octocrab: &Octocrab, page| {
        Box::pin({
            async move {
                octocrab
                    .search()
                    .code(
                        format!(
                            "org:KittyCAD filename:{}{}",
                            file,
                            if let Some(content) = content {
                                format!(" \"{}\"", content)
                            } else {
                                String::new()
                            }
                        )
                        .as_str(),
                    )
                    .sort("indexed")
                    .order("asc")
                    .per_page(100)
                    .page(page)
                    .send()
                    .await
            }
        })
    })
    .await?;
    Ok(repos)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Ecosystem {
    Cargo,
    Npm,
    Go,
    Submodule,
    Terraform,
    Pip,
    Uv,
    Bundler,
    Docker,
    GitHubActions,
}

impl Display for Ecosystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Ecosystem::Cargo => write!(f, "cargo")?,
            Ecosystem::Npm => write!(f, "npm")?,
            Ecosystem::Go => write!(f, "gomod")?,
            Ecosystem::Submodule => write!(f, "gitsubmodule")?,
            Ecosystem::Terraform => write!(f, "terraform")?,
            Ecosystem::Pip => write!(f, "pip")?,
            Ecosystem::Uv => write!(f, "uv")?,
            Ecosystem::Bundler => write!(f, "bundler")?,
            Ecosystem::Docker => write!(f, "docker")?,
            Ecosystem::GitHubActions => write!(f, "github-actions")?,
        }

        Ok(())
    }
}

async fn find_ecosystems(
    octocrab: &Octocrab,
) -> anyhow::Result<HashMap<String, Vec<(String, Ecosystem)>>> {
    // TODO Homebrew?
    // TODO: Handle workspaces (Cargo.toml but maybe also others)
    let cargo_roots = search_ecosystems(octocrab, "Cargo.toml", Some("[workspace]")).await?;
    let npm_roots = search_ecosystems(octocrab, "package.json", None).await?;
    let go_roots = search_ecosystems(octocrab, "go.mod", None).await?;
    let submodule_roots = search_ecosystems(octocrab, ".gitmodules", None).await?;

    // avoid rate limits, 9 searches seems max
    sleep(Duration::from_secs(65)).await;

    let python_roots = search_ecosystems(octocrab, "requirements.txt", None).await?;
    let pyprojects_roots = search_ecosystems(octocrab, "pyproject.toml", None).await?;
    let bundler_roots = search_ecosystems(octocrab, "Gemfile.lock", None).await?;
    let docker_roots = search_ecosystems(octocrab, "Dockerfile", None).await?;

    // avoid rate limits
    sleep(Duration::from_secs(65)).await;

    let terraform_roots = search_ecosystems(octocrab, ".terraform.lock.hcl", None).await?;
    let uv_roots_1 = search_ecosystems(octocrab, "uv.lock", None).await?;
    let uv_roots_2 = search_ecosystems(octocrab, "pyproject.toml", Some("tool.uv")).await?;
    let uv_roots = uv_roots_1
        .into_iter()
        .chain(uv_roots_2.into_iter())
        .collect::<Vec<_>>();

    let pyprojects_roots: Vec<_> = pyprojects_roots
        .into_iter()
        .filter(|root| {
            !uv_roots
                .iter()
                .any(|code| code.repository == root.repository)
        })
        .collect();

    let ecosystems: HashMap<String, Vec<(String, Ecosystem)>> = [
        (cargo_roots, Ecosystem::Cargo),
        (npm_roots, Ecosystem::Npm),
        (go_roots, Ecosystem::Go),
        (submodule_roots, Ecosystem::Submodule),
        (terraform_roots, Ecosystem::Terraform),
        (pyprojects_roots, Ecosystem::Pip),
        (python_roots, Ecosystem::Pip),
        (uv_roots, Ecosystem::Uv),
        (bundler_roots, Ecosystem::Bundler),
        (docker_roots, Ecosystem::Docker),
    ]
    .iter()
    .flat_map(|(roots, ecosystem)| {
        roots.iter().map(move |code| {
            (
                code.repository
                    .full_name
                    .clone()
                    .expect("full_name must be available"),
                (code.url.path().to_string(), *ecosystem),
            )
        })
    })
    .fold(HashMap::new(), |mut acc, (repo, entry)| {
        acc.entry(repo).or_default().push(entry);
        acc
    });

    Ok(ecosystems)
}
