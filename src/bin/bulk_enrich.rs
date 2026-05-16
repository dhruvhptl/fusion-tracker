use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use dotenv;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CredibilityBucket {
    FrontRunner,
    Credible,
    Speculative,
    Conceptual,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProjectLayer {
    #[default]
    PrimaryProject,
    PrecursorFacility,
    ProgramRoadmap,
    Watchlist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    pub id: String,
    pub project_name: Option<String>,
    pub company_or_operator: Option<String>,
    pub country: Option<String>,
    pub city_or_region: Option<String>,
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
}

#[derive(Debug, Serialize, Deserialize)]
struct FusionData {
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

#[derive(Debug, Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    raw_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterChoice {
    message: OpenRouterMessage,
}

#[derive(Debug, Deserialize)]
struct OpenRouterMessage {
    content: String,
}

struct EnrichContext {
    tavily_api_key: String,
    openrouter_api_key: String,
}

fn display_name(project: &Project) -> String {
    project
        .company_or_operator
        .as_ref()
        .or(project.project_name.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("Unknown")
        .to_string()
}

fn null_fields(project: &Project) -> Vec<&'static str> {
    let mut fields = vec![];
    if project.current_milestone.is_none() {
        fields.push("current_milestone");
    }
    if project.next_milestone.is_none() {
        fields.push("next_milestone");
    }
    if project.offtaker.is_none() {
        fields.push("offtaker");
    }
    if project.utility_partner.is_none() {
        fields.push("utility_partner");
    }
    if project.source_url.is_none() {
        fields.push("source_url");
    }
    if project.funding_raised_usd.is_none() {
        fields.push("funding_raised_usd");
    }
    if project.target_online_year.is_none() {
        fields.push("target_online_year");
    }
    fields
}

async fn search_tavily(
    company_name: &str,
    api_key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let query = format!("{} fusion energy latest news funding milestone", company_name);

    let response = client
        .post("https://api.tavily.com/search")
        .json(&json!({
            "api_key": api_key,
            "query": query,
            "include_raw_content": true,
            "max_results": 3,
            "include_domains": ["fusionenergybase.com", "en.wikipedia.org", "fusionindustryassociation.org"]
        }))
        .send()
        .await?;

    let body: TavilyResponse = response.json().await?;

    let mut content = String::new();
    for result in body.results.iter().take(3) {
        if let Some(raw) = &result.raw_content {
            content.push_str(raw);
            content.push_str("\n\n");
        }
    }

    Ok(if content.trim().is_empty() {
        format!("Company: {}", company_name)
    } else {
        content
    })
}

async fn extract_with_llm(
    company: &Project,
    context: &str,
    null_fields: &[&str],
    api_key: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let known_fields = format!(
        "reactor_type={:?}, funding_raised_usd={:?}, country={:?}",
        company.reactor_type, company.funding_raised_usd, company.country
    );

    let field_list = null_fields.join(", ");

    let field_definitions = r#"
- current_milestone: most recent notable achievement, 1-2 sentences
- next_milestone: next stated public goal, 1-2 sentences
- offtaker: named power purchase agreement or offtake partner if any
- utility_partner: named utility or grid partner if any
- source_url: most authoritative URL from the sources provided
- funding_raised_usd: total funding raised in USD as integer
- target_online_year: first 4-digit year of pilot/commercial plant target"#;

    let system_prompt = format!(
        r#"You are a fusion energy research assistant. For the company "{}",
I already know the following: {}.

Using ONLY the provided source text, extract the following missing fields:
{}

Return a JSON object with ONLY those fields. Set a field to null if not confidently found.
Do not hallucinate. Do not re-extract fields I already have.

Field definitions:
{}

Return ONLY the JSON object, no markdown code blocks, no array wrapping."#,
        company
            .company_or_operator
            .as_deref()
            .unwrap_or("Unknown"),
        known_fields,
        field_list,
        field_definitions
    );

    let user_prompt = format!(
        "Extract the missing fields from this context about {}:\n\n{}",
        company
            .company_or_operator
            .as_deref()
            .unwrap_or("Unknown"),
        context
    );

    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "google/gemini-2.0-flash-001",
            "response_format": { "type": "json_object" },
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
        }))
        .send()
        .await?;

    let body: OpenRouterResponse = response.json().await?;

    if body.choices.is_empty() {
        return Err("No choices in OpenRouter response".into());
    }

    let ai_response = &body.choices[0].message.content;
    let parsed: serde_json::Value = serde_json::from_str(ai_response)?;

    Ok(parsed)
}

fn merge_enrichment(project: &mut Project, enrichment: serde_json::Value) {
    if let Some(obj) = enrichment.as_object() {
        if let Some(val) = obj.get("current_milestone") {
            if !val.is_null() && project.current_milestone.is_none() {
                if let Some(s) = val.as_str() {
                    project.current_milestone = Some(s.to_string());
                }
            }
        }
        if let Some(val) = obj.get("next_milestone") {
            if !val.is_null() && project.next_milestone.is_none() {
                if let Some(s) = val.as_str() {
                    project.next_milestone = Some(s.to_string());
                }
            }
        }
        if let Some(val) = obj.get("offtaker") {
            if !val.is_null() && project.offtaker.is_none() {
                if let Some(s) = val.as_str() {
                    project.offtaker = Some(s.to_string());
                }
            }
        }
        if let Some(val) = obj.get("utility_partner") {
            if !val.is_null() && project.utility_partner.is_none() {
                if let Some(s) = val.as_str() {
                    project.utility_partner = Some(s.to_string());
                }
            }
        }
        if let Some(val) = obj.get("source_url") {
            if !val.is_null() && project.source_url.is_none() {
                if let Some(s) = val.as_str() {
                    project.source_url = Some(s.to_string());
                }
            }
        }
        if let Some(val) = obj.get("funding_raised_usd") {
            if !val.is_null() && project.funding_raised_usd.is_none() {
                if let Some(n) = val.as_u64() {
                    project.funding_raised_usd = Some(n);
                }
            }
        }
        if let Some(val) = obj.get("target_online_year") {
            if !val.is_null() && project.target_online_year.is_none() {
                if let Some(n) = val.as_u64() {
                    project.target_online_year = Some(n as u32);
                }
            }
        }
    }
}

async fn enrich_project(
    project: &mut Project,
    ctx: &EnrichContext,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let null_field_list = null_fields(project);

    if null_field_list.is_empty() {
        return Ok(vec![]);
    }

    let context = search_tavily(
        project
            .company_or_operator
            .as_deref()
            .unwrap_or("Unknown"),
        &ctx.tavily_api_key,
    )
    .await?;

    let enrichment =
        extract_with_llm(project, &context, &null_field_list, &ctx.openrouter_api_key).await?;

    merge_enrichment(project, enrichment);

    let filled_fields = null_field_list
        .iter()
        .filter_map(|&field| {
            match field {
                "current_milestone" => project.current_milestone.as_ref().map(|_| field),
                "next_milestone" => project.next_milestone.as_ref().map(|_| field),
                "offtaker" => project.offtaker.as_ref().map(|_| field),
                "utility_partner" => project.utility_partner.as_ref().map(|_| field),
                "source_url" => project.source_url.as_ref().map(|_| field),
                "funding_raised_usd" => project.funding_raised_usd.as_ref().map(|_| field),
                "target_online_year" => project.target_online_year.as_ref().map(|_| field),
                _ => None,
            }
        })
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    Ok(filled_fields)
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();

    let tavily_api_key = std::env::var("TAVILY_API_KEY")
        .unwrap_or_else(|_| panic!("TAVILY_API_KEY not set"));
    let openrouter_api_key = std::env::var("OPENROUTER_API_KEY")
        .unwrap_or_else(|_| panic!("OPENROUTER_API_KEY not set"));

    let ctx = EnrichContext {
        tavily_api_key,
        openrouter_api_key,
    };

    let data_path = PathBuf::from("data/projects.json");

    println!("Reading {}", data_path.display());
    let data_json = fs::read_to_string(&data_path)
        .unwrap_or_else(|e| panic!("Failed to read data file: {}", e));

    let mut fusion_data: FusionData =
        serde_json::from_str(&data_json).expect("Failed to parse projects.json");

    let mut all_projects = vec![];

    for project in fusion_data.primary_projects.iter() {
        all_projects.push((project.clone(), "primary"));
    }
    for project in fusion_data.precursor_facilities.iter() {
        all_projects.push((project.clone(), "precursor"));
    }
    for project in fusion_data.program_roadmaps.iter() {
        all_projects.push((project.clone(), "roadmap"));
    }
    for project in fusion_data.watchlist.iter() {
        all_projects.push((project.clone(), "watchlist"));
    }

    println!("Found {} total projects\n", all_projects.len());

    let mut enriched = 0;
    let mut skipped = 0;
    let mut failed = 0;

    let mut primary = vec![];
    let mut precursor = vec![];
    let mut roadmap = vec![];
    let mut watchlist = vec![];

    for (mut project, layer) in all_projects {
        let name = display_name(&project);

        if null_fields(&project).is_empty() {
            println!("⏭ {} — already complete", name);
            skipped += 1;
        } else {
            match enrich_project(&mut project, &ctx).await {
                Ok(filled) => {
                    if filled.is_empty() {
                        println!("✗ {} — no new data found", name);
                        failed += 1;
                    } else {
                        println!("✓ {} — filled: {}", name, filled.join(", "));
                        enriched += 1;
                    }
                }
                Err(e) => {
                    println!("✗ {} — error: {}", name, e);
                    failed += 1;
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(1000)).await;

        match layer {
            "primary" => primary.push(project),
            "precursor" => precursor.push(project),
            "roadmap" => roadmap.push(project),
            "watchlist" => watchlist.push(project),
            _ => {}
        }
    }

    fusion_data.primary_projects = primary;
    fusion_data.precursor_facilities = precursor;
    fusion_data.program_roadmaps = roadmap;
    fusion_data.watchlist = watchlist;

    println!("\n=== Bulk Enrich Summary ===");
    println!("Enriched: {} projects", enriched);
    println!("Skipped: {} projects (already complete)", skipped);
    println!("Failed: {} projects", failed);

    let output_json = serde_json::to_string_pretty(&fusion_data)
        .expect("Failed to serialize data");

    let temp_path = data_path.with_extension("json.tmp");
    fs::write(&temp_path, &output_json)
        .unwrap_or_else(|e| panic!("Failed to write temp file: {}", e));

    fs::rename(&temp_path, &data_path)
        .unwrap_or_else(|e| panic!("Failed to rename temp file: {}", e));

    println!("Wrote to: {}", data_path.display());
}
