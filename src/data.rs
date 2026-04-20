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
    Ok(ds)
}
