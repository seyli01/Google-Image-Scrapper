use reqwest;
use regex::Regex;
use std::{error::Error, time::{Duration, Instant}};
use url::Url;
use serde::{Deserialize, Serialize};
use rand::{rng, Rng};
use urlencoding;
use chrono::{Utc, DateTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageResult {
    pub url: String,
    pub source_site: String,
}

#[derive(Serialize)]
struct RechercheMetadonnees {
    identifiant: String,
    statut: &'static str,
    google_url: String,
    created_at: DateTime<Utc>,
    processed_at: DateTime<Utc>,
    total_time_secs: f64,
}

#[derive(Serialize)]
struct ParametresRecherche {
    moteur: &'static str,
    q: String,
    hl: String,
    safe: String,
}

#[derive(Serialize)]
struct RechercheInformations {
    brut_images_trouvees: usize,
    apres_filtre: usize,
    temps_parse_secs: f64,
}

#[derive(Serialize)]
struct Output {
    recherche_métadonnées: RechercheMetadonnees,
    paramètres_de_recherche: ParametresRecherche,
    recherche_informations: RechercheInformations,
    résultats_images: Vec<ImageResult>,
}

pub struct GoogleImageScraper {
    client: reqwest::Client,
    user_agents: Vec<String>,
}

impl GoogleImageScraper {
    pub fn new() -> Self {
        let user_agents = vec![
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
            "Mozilla/5.0 (iPhone; CPU iPhone OS 14_7_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.2 Mobile/15E148 Safari/604.1".to_string(),
        ];

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(5))
            .gzip(true)
            .cookie_store(true)
            .build()
            .unwrap();

        println!("✅ Google Images Scraper prêt !");
        Self { client, user_agents }
    }

    fn get_random_user_agent(&self) -> &str {
        let mut rng = rng();
        let idx = rng.random_range(0..self.user_agents.len());
        &self.user_agents[idx]
    }

    pub async fn search_google(&self, query: &str, max_images: usize) -> Result<Output, Box<dyn Error>> {
        let start = Instant::now();
        let created_at = Utc::now();

        let google_url = format!(
            "https://www.google.com/search?q={}&tbm=isch&hl=en&safe=off",
            urlencoding::encode(query)
        );

        let resp = self.client
            .get(&google_url)
            .header("User-Agent", self.get_random_user_agent())
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9,fr;q=0.8")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(format!("HTTP Error: {}", resp.status()).into());
        }

        let html = resp.text().await?;
        let parse_start = Instant::now();
        let mut raw = self.parse_google_html(&html);
        let parse_time = parse_start.elapsed().as_secs_f64();

        raw.truncate(raw.len().min(max_images));
        let filtered_count = raw.len();

        let processed_at = Utc::now();
        let total_time = start.elapsed().as_secs_f64();

        let identifiant = format!("{:x}", processed_at.timestamp_nanos_opt().unwrap_or_default());

        Ok(Output {
            recherche_métadonnées: RechercheMetadonnees {
                identifiant,
                statut: "Succès",
                google_url: google_url.clone(),
                created_at,
                processed_at,
                total_time_secs: total_time,
            },
            paramètres_de_recherche: ParametresRecherche {
                moteur: "Google",
                q: query.to_string(),
                hl: "en".into(),
                safe: "off".into(),
            },
            recherche_informations: RechercheInformations {
                brut_images_trouvees: raw.len(),
                apres_filtre: filtered_count,
                temps_parse_secs: parse_time,
            },
            résultats_images: raw,
        })
    }

    fn parse_google_html(&self, html: &str) -> Vec<ImageResult> {
        let mut results = Vec::new();
        let re = Regex::new(r#"\["(https://[^"]*\.(?:jpg|jpeg|png|gif|webp)(?:\?[^"]*)?)",\d+,\d+\]"#).unwrap();
        for cap in re.captures_iter(html) {
            if let Some(m) = cap.get(1) {
                let url = self.clean_url(m.as_str());
                if self.is_valid_image_url(&url)
                    && !results.iter().any(|r: &ImageResult| r.url == url)
                {
                    results.push(ImageResult {
                        url: url.clone(),
                        source_site: self.extract_domain(&url),
                    });
                }
            }
        }
        results
    }

    fn is_valid_image_url(&self, url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.starts_with("http")
            && [".jpg", ".jpeg", ".png", ".gif", ".webp"].iter().any(|&e| lower.contains(e))
            && !["encrypted-tbn", "gstatic.com", "logo", "icon", "base64"].iter().any(|b| lower.contains(b))
    }

    fn clean_url(&self, url: &str) -> String {
        url.replace("\\u003d", "=")
           .replace("\\u0026", "&")
           .replace("\\u002F", "/")
           .replace("\\u003F", "?")
           .replace("\\u003A", ":")
           .replace("\\/", "/")
           .replace("&amp;", "&")
           .trim()
           .to_string()
    }

    fn extract_domain(&self, url: &str) -> String {
        Url::parse(url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.replace("www.", "")))
            .unwrap_or_else(|| "unknown".into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let query = std::env::args().nth(1).unwrap_or_else(|| "cute cats ".into());
    let max_images = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(10);

    let scraper = GoogleImageScraper::new();
    let output = scraper.search_google(&query, max_images).await?;

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
