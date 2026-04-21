use crate::models::{FusionData, Project, ProjectLayer};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug)]
pub enum DataError {
    Parse(serde_json::Error),
    DuplicateId {
        id: String,
    },
    OutOfRangeCoord {
        project_id: String,
        lat: f64,
        lng: f64,
    },
    NoVisibleProjects,
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::Parse(e) => write!(f, "JSON parse error: {e}"),
            DataError::DuplicateId { id } => write!(f, "duplicate project id '{id}'"),
            DataError::OutOfRangeCoord {
                project_id,
                lat,
                lng,
            } => write!(
                f,
                "project '{project_id}' has out-of-range coords: lat={lat}, lng={lng}"
            ),
            DataError::NoVisibleProjects => {
                write!(f, "dataset has zero projects with a non-empty globe_modes")
            }
        }
    }
}

impl std::error::Error for DataError {}

pub fn load_projects_from_str(raw: &str) -> Result<Vec<Project>, DataError> {
    let mut data: FusionData = serde_json::from_str(raw).map_err(DataError::Parse)?;
    for p in &mut data.primary_projects {
        p.layer = ProjectLayer::PrimaryProject;
    }
    for p in &mut data.precursor_facilities {
        p.layer = ProjectLayer::PrecursorFacility;
    }
    for p in &mut data.program_roadmaps {
        p.layer = ProjectLayer::ProgramRoadmap;
    }
    for p in &mut data.watchlist {
        p.layer = ProjectLayer::Watchlist;
    }

    let visible: Vec<Project> = data
        .primary_projects
        .into_iter()
        .chain(data.precursor_facilities)
        .chain(data.program_roadmaps)
        .chain(data.watchlist)
        .filter(|p| !p.globe_modes.is_empty())
        .filter(|p| p.latitude.is_some() && p.longitude.is_some())
        .collect();

    validate(&visible)?;
    Ok(visible)
}

fn validate(projects: &[Project]) -> Result<(), DataError> {
    if projects.is_empty() {
        return Err(DataError::NoVisibleProjects);
    }

    let mut ids = HashSet::new();
    for p in projects {
        if !ids.insert(p.id.clone()) {
            return Err(DataError::DuplicateId { id: p.id.clone() });
        }
        let lat = p.latitude.expect("filtered for Some");
        let lng = p.longitude.expect("filtered for Some");
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
            return Err(DataError::OutOfRangeCoord {
                project_id: p.id.clone(),
                lat,
                lng,
            });
        }
    }
    Ok(())
}

pub fn load_projects_from_path<P: AsRef<Path>>(path: P) -> Vec<Project> {
    let path_ref = path.as_ref();
    let raw = std::fs::read_to_string(path_ref)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path_ref.display()));
    match load_projects_from_str(&raw) {
        Ok(projects) => projects,
        Err(e) => panic!("invalid dataset at {}: {e}", path_ref.display()),
    }
}
