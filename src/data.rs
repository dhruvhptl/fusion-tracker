use crate::models::Dataset;

#[derive(Debug)]
pub enum DataError {
    Parse(serde_json::Error),
    OrphanLocation { location_id: String, company_id: String },
    CompanyWithoutLocations { company_id: String },
    DuplicateId { collection: &'static str, id: String },
    OutOfRangeCoord { location_id: String, lat: f64, lng: f64 },
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataError::Parse(e) => write!(f, "JSON parse error: {e}"),
            DataError::OrphanLocation { location_id, company_id } => write!(
                f,
                "location '{location_id}' references unknown company_id '{company_id}'"
            ),
            DataError::CompanyWithoutLocations { company_id } => {
                write!(f, "company '{company_id}' has no locations")
            }
            DataError::DuplicateId { collection, id } => {
                write!(f, "duplicate id '{id}' in {collection}")
            }
            DataError::OutOfRangeCoord { location_id, lat, lng } => write!(
                f,
                "location '{location_id}' has out-of-range coords: lat={lat}, lng={lng}"
            ),
        }
    }
}

impl std::error::Error for DataError {}

pub fn load_dataset_from_str(raw: &str) -> Result<Dataset, DataError> {
    let ds: Dataset = serde_json::from_str(raw).map_err(DataError::Parse)?;
    validate(&ds)?;
    Ok(ds)
}

fn validate(ds: &Dataset) -> Result<(), DataError> {
    use std::collections::HashSet;

    let mut company_ids = HashSet::new();
    for c in &ds.companies {
        if !company_ids.insert(c.id.clone()) {
            return Err(DataError::DuplicateId {
                collection: "companies",
                id: c.id.clone(),
            });
        }
    }

    let mut location_ids = HashSet::new();
    for l in &ds.locations {
        if !location_ids.insert(l.id.clone()) {
            return Err(DataError::DuplicateId {
                collection: "locations",
                id: l.id.clone(),
            });
        }
        if !company_ids.contains(&l.company_id) {
            return Err(DataError::OrphanLocation {
                location_id: l.id.clone(),
                company_id: l.company_id.clone(),
            });
        }
        if !(-90.0..=90.0).contains(&l.lat) || !(-180.0..=180.0).contains(&l.lng) {
            return Err(DataError::OutOfRangeCoord {
                location_id: l.id.clone(),
                lat: l.lat,
                lng: l.lng,
            });
        }
    }

    let companies_with_location: HashSet<&String> =
        ds.locations.iter().map(|l| &l.company_id).collect();
    for c in &ds.companies {
        if !companies_with_location.contains(&c.id) {
            return Err(DataError::CompanyWithoutLocations {
                company_id: c.id.clone(),
            });
        }
    }

    Ok(())
}

use std::path::Path;

pub fn load_dataset_from_path<P: AsRef<Path>>(path: P) -> Dataset {
    let path_ref = path.as_ref();
    let raw = std::fs::read_to_string(path_ref)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path_ref.display()));
    match load_dataset_from_str(&raw) {
        Ok(ds) => ds,
        Err(e) => panic!("invalid dataset at {}: {e}", path_ref.display()),
    }
}
