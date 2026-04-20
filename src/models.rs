use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LocationType {
    Hq,
    Facility,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LocationStatus {
    Active,
    Planned,
    Announced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    pub description: String,
    pub reactor_type: String,
    pub funding_raised_usd: u64,
    pub funding_display: String,
    pub current_milestone: String,
    pub source_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub company_id: String,
    pub name: String,
    pub location_type: LocationType,
    pub city: String,
    pub country: String,
    pub lat: f64,
    pub lng: f64,
    pub status: LocationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub companies: Vec<Company>,
    pub locations: Vec<Location>,
}
