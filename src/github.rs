use chrono::{DateTime, Utc};
use http_body_util::BodyExt;
use octocrab::models::{Collaborator, Repository};
use octocrab::{FromResponse, Octocrab, Page};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};


#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum AssetLevel {
    Production,
    /// Just testing the waters. Not even development breaks if this breaks.
    Playground,
    /// Used for development. If this is pwned, other security controls should stop spreading to production.
    #[serde(rename = "Research & Development")]
    ResearchNDevelopment,
    /// Only relevant for internal folks. No link to production.
    Corporate,
    /// Publicly accessible services, but not part of our core product like store.zoo.dev.
    #[serde(rename = "Non-essential Production")]
    NonEssentialProduction,
}

impl AssetLevel {
    pub fn get_from_props(props: &[CustomProperty]) -> Option<AssetLevel> {
        props
            .iter()
            .find(|prop| prop.property_name == "repository-level")
            .and_then(|prop| match &prop.value {
                None => None,
                Some(CustomPropertyValue::Array(_array)) => {
                    panic!("Array not supported for repository-level")
                }
                Some(CustomPropertyValue::String(str)) => match str.as_str() {
                    "Production" => Some(AssetLevel::Production),
                    "Playground" => Some(AssetLevel::Playground),
                    "Research & Development" => Some(AssetLevel::ResearchNDevelopment),
                    "Corporate" => Some(AssetLevel::Corporate),
                    "Non-essential Production" => Some(AssetLevel::NonEssentialProduction),
                    _ => None,
                },
            })
    }
}

impl Display for AssetLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetLevel::Production => write!(f, "Production"),
            AssetLevel::Playground => write!(f, "Playground"),
            AssetLevel::ResearchNDevelopment => write!(f, "Research & Development"),
            AssetLevel::Corporate => write!(f, "Corporate"),
            AssetLevel::NonEssentialProduction => write!(f, "Non-essential Production"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub repository: Option<Repository>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageVersion {
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomProperty {
    pub property_name: String,
    pub value: Option<CustomPropertyValue>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum CustomPropertyValue {
    String(String),
    Array(Vec<String>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchCustomProperties {
    pub properties: Vec<CustomProperty>,
}

pub trait CustomPropertyExt {
    fn list_custom_properties(
        &self,
        owner: &str,
        repo: &str,
    ) -> impl std::future::Future<Output = octocrab::Result<Vec<CustomProperty>>> + Send;
    fn set_custom_properties(
        &self,
        owner: &str,
        repo: &str,
        properties: Vec<CustomProperty>,
    ) -> impl std::future::Future<Output = octocrab::Result<(), octocrab::Error>> + Send;
}

pub trait PackagesExt {
    fn list_containers(
        &self,
        org: &str,
    ) -> impl std::future::Future<Output = octocrab::Result<Vec<Package>>> + Send;
    fn list_container_versions(
        &self,
        org: &str,
        pkg: &str,
    ) -> impl std::future::Future<Output = octocrab::Result<Vec<PackageVersion>>> + Send;
}

impl PackagesExt for Octocrab {
    async fn list_containers(&self, org: &str) -> Result<Vec<Package>, octocrab::Error> {
        self.get(
            format!("/orgs/{org}/packages?package_type=container&per_page=100&visibility=internal"),
            None::<&()>,
        )
        .await
    }

    async fn list_container_versions(
        &self,
        org: &str,
        pkg: &str,
    ) -> Result<Vec<PackageVersion>, octocrab::Error> {
        self.get(
            format!("/orgs/{org}/packages/container/{pkg}/versions?per_page=10"),
            None::<&()>,
        )
        .await
    }
}

struct Noop;

#[async_trait::async_trait]
impl FromResponse for Noop {
    async fn from_response<B>(response: http::Response<B>) -> octocrab::Result<Self>
    where
        B: http_body::Body<Data = bytes::Bytes, Error = octocrab::Error> + Send,
    {
        let (_, body) = response.into_parts();
        let _body = body.collect().await?.to_bytes();
        Ok(Noop)
    }
}

impl CustomPropertyExt for Octocrab {
    async fn list_custom_properties(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<CustomProperty>, octocrab::Error> {
        self.get(
            format!("/repos/{owner}/{repo}/properties/values"),
            None::<&()>,
        )
        .await
    }

    async fn set_custom_properties(
        &self,
        owner: &str,
        repo: &str,
        properties: Vec<CustomProperty>,
    ) -> Result<(), octocrab::Error> {
        let _: Noop = self
            .patch(
                format!("/repos/{owner}/{repo}/properties/values"),
                Some(&PatchCustomProperties { properties }),
            )
            .await?;
        Ok(())
    }
}

pub trait CollaboratorsExt {
    fn list_collaborators(
        &self,
        owner: &str,
        repo: &str,
        affiliation: &str,
        page: u32,
    ) -> impl std::future::Future<Output = octocrab::Result<Page<Collaborator>>> + Send;
}

impl CollaboratorsExt for Octocrab {
    async fn list_collaborators(
        &self,
        owner: &str,
        repo: &str,
        affiliation: &str,
        page: u32,
    ) -> Result<Page<Collaborator>, octocrab::Error> {
        self.get(
            format!("/repos/{owner}/{repo}/collaborators"),
            Some(&HashMap::from([
                ("affiliation", affiliation),
                ("page", &page.to_string()),
            ])),
        )
        .await
    }
}

pub async fn get_all<'a, T>(
    octocrab: &'a Octocrab,
    fetch_page: impl Fn(
        &'a Octocrab,
        u32,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = octocrab::Result<octocrab::Page<T>>> + Send + 'a>,
    >,
) -> Result<Vec<T>, octocrab::Error> {
    let mut items = Vec::new();
    let mut page = 1u32;
    loop {
        let response = fetch_page(octocrab, page).await?;

        if response.items.is_empty() {
            break;
        }

        items.extend(response.items);

        page += 1;

        if page > 5 {
            panic!(
                "We dont want to hit the rate limit of Github. Aborting after 1000 elements fetched."
            );
        }
    }
    Ok(items)
}

pub async fn get_all_repos(
    octocrab: &Octocrab,
    org: &str,
) -> Result<Vec<Repository>, octocrab::Error> {
    let org = org.to_string();
    get_all(octocrab, move |octocrab: &Octocrab, page| {
        Box::pin({
            let value = org.clone();
            async move {
                octocrab
                    .orgs(value)
                    .list_repos()
                    .per_page(100)
                    .page(page)
                    .send()
                    .await
            }
        })
    })
    .await
}
