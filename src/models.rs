use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredibilityBucket {
    FrontRunner,
    Credible,
    Speculative,
    Conceptual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub project_name: String,
    pub company_or_operator: String,
    pub country: String,
    pub city_or_region: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub project_stage: Option<String>,
    pub reactor_type: Option<String>,
    pub facility_type: Option<String>,
    pub intended_output_mwe: Option<f64>,
    pub target_online_year: Option<u32>,
    pub offtaker: Option<String>,
    pub utility_partner: Option<String>,
    pub served_region: Option<String>,
    pub site_status: Option<String>,
    pub funding_raised_usd: Option<u64>,
    pub current_milestone: Option<String>,
    pub next_milestone: Option<String>,
    pub source_url: Option<String>,
    pub credibility_bucket: Option<CredibilityBucket>,
    #[serde(default)]
    pub globe_modes: Vec<String>,
    #[serde(skip_deserializing, default)]
    pub layer: ProjectLayer,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLayer {
    #[default]
    PrimaryProject,
    PrecursorFacility,
    ProgramRoadmap,
    Watchlist,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FusionData {
    #[serde(default)]
    pub primary_projects: Vec<Project>,
    #[serde(default)]
    pub precursor_facilities: Vec<Project>,
    #[serde(default)]
    pub program_roadmaps: Vec<Project>,
    #[serde(default)]
    pub watchlist: Vec<Project>,
    #[serde(default)]
    pub relationships: Vec<Value>,
    #[serde(default)]
    pub missing_candidates: Vec<Value>,
    #[serde(default)]
    pub audit_notes: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectsResponse {
    pub projects: Vec<Project>,
}
