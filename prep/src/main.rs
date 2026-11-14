use chrono::NaiveDate;
use lunar_core::{Horizon, Site, Track, parse_horizons};
use serde::Serialize;
use std::{error::Error, fs, path::Path, time::Duration};

#[derive(Serialize)]
struct SiteData {
    site: Site,
    sun: Track,
    earth: Track,
    horizon: Option<Horizon>,
}

fn download(
    client: &reqwest::blocking::Client,
    site: &Site,
    body: &str,
    start: &str,
    stop: &str,
    step: &str,
) -> Result<Track, Box<dyn Error>> {
    let coords = format!("'{},{},0'", site.lon_deg, site.lat_deg);
    let text = client
        .get("https://ssd.jpl.nasa.gov/api/horizons.api")
        .query(&[
            ("format", "text"),
            ("COMMAND", body),
            ("CENTER", "'coord@301'"),
            ("COORD_TYPE", "'GEODETIC'"),
            ("SITE_COORD", &coords),
            ("MAKE_EPHEM", "'YES'"),
            ("EPHEM_TYPE", "'OBSERVER'"),
            ("QUANTITIES", "'4'"),
            ("ANG_FORMAT", "'DEG'"),
            ("START_TIME", &format!("'{start}'")),
            ("STOP_TIME", &format!("'{stop}'")),
            ("STEP_SIZE", &format!("'{step}'")),
        ])
        .send()?
        .error_for_status()?
        .text()?;
    parse_horizons(&text)
        .map_err(|e| format!("Horizons for {} ({body}): {e}\n{text}", site.name).into())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(4..=5).contains(&args.len()) {
        return Err("Usage: prep <sites.json> <start YYYY-MM-DD> <stop YYYY-MM-DD> <step, e.g. 6h> [output-dir]".into());
    }
    let start = NaiveDate::parse_from_str(&args[1], "%Y-%m-%d")?;
    let stop = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d")?;
    if stop <= start {
        return Err("Stop date must be after start date".into());
    }
    let step = &args[3];
    if !step.ends_with(['m', 'h', 'd']) || step[..step.len() - 1].parse::<u32>().unwrap_or(0) == 0 {
        return Err("Step must be a positive integer followed by m, h, or d".into());
    }
    let sites: Vec<Site> = serde_json::from_str(&fs::read_to_string(&args[0])?)?;
    let output = Path::new(args.get(4).map(String::as_str).unwrap_or("app/assets/data"));
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    let mut slugs = std::collections::HashSet::new();
    for site in &sites {
        if !site.lat_deg.is_finite()
            || !(-90.0..=90.0).contains(&site.lat_deg)
            || !site.lon_deg.is_finite()
        {
            return Err(format!("Invalid coordinates for {}", site.name).into());
        }
        let slug = slug(&site.name);
        if slug.is_empty() || !slugs.insert(slug) {
            return Err("Site names need unique, nonempty ASCII slugs".into());
        }
    }
    fs::create_dir_all(output)?;
    for site in sites {
        let sun = download(&client, &site, "'10'", &args[1], &args[2], step)?;
        let earth = download(&client, &site, "'399'", &args[1], &args[2], step)?;
        if sun.t != earth.t {
            return Err("Sun and Earth timestamps differ".into());
        }
        let path = output.join(format!("{}.json", slug(&site.name)));
        // TODO: Load a LOLA DEM and call horizon_from_dem here when DEM input is supported.
        let data = SiteData {
            site,
            sun,
            earth,
            horizon: None,
        };
        fs::write(&path, serde_json::to_string_pretty(&data)?)?;
        println!(
            "Wrote {} ({} samples per track)",
            path.display(),
            data.sun.t.len()
        );
    }
    Ok(())
}

fn slug(name: &str) -> String {
    name.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}
