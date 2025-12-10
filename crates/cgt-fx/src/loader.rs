use crate::cache::FxCache;
use crate::parser::{FxParseError, parse_monthly_rates};
use crate::types::RateSource;
use include_dir::Dir;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
pub enum FxLoaderError {
    #[error("Invalid UTF-8 in {name}")]
    InvalidUtf8 { name: String },
    #[error("Invalid file name for rate period: {name}")]
    InvalidFileName { name: String },
    #[error("Parse error in {name}: {source}")]
    Parse {
        name: String,
        #[source]
        source: FxParseError,
    },
}

/// Pre-parsed XML provided by the caller (e.g., CLI) for folder overrides.
#[derive(Debug, Clone)]
pub struct RateFile {
    pub name: PathBuf,
    pub modified: Option<SystemTime>,
    pub xml: String,
}

fn expected_year_month_from_path(path: &Path) -> Result<(i32, u32), FxLoaderError> {
    let stem = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
        FxLoaderError::InvalidFileName {
            name: path.to_string_lossy().into_owned(),
        }
    })?;
    // Allow optional prefixes like "monthly_xml_2024-12"
    let stem = stem.rsplit_once('_').map(|(_, tail)| tail).unwrap_or(stem);
    let mut parts = stem.split('-');
    let year = parts
        .next()
        .and_then(|y| y.parse::<i32>().ok())
        .ok_or_else(|| FxLoaderError::InvalidFileName {
            name: path.to_string_lossy().into_owned(),
        })?;
    let month = parts
        .next()
        .and_then(|m| m.parse::<u32>().ok())
        .ok_or_else(|| FxLoaderError::InvalidFileName {
            name: path.to_string_lossy().into_owned(),
        })?;
    if (1..=12).contains(&month) {
        Ok((year, month))
    } else {
        Err(FxLoaderError::InvalidFileName {
            name: path.to_string_lossy().into_owned(),
        })
    }
}

fn load_bundled_dir(dir: &Dir<'_>) -> Result<Vec<crate::types::RateEntry>, FxLoaderError> {
    let mut all_entries = Vec::new();

    for file in dir.files() {
        let path = file.path();
        if path.extension().and_then(|e| e.to_str()) != Some("xml") {
            continue;
        }

        let name = path.to_string_lossy().into_owned();
        let expected = expected_year_month_from_path(path)?;
        let content = file
            .contents_utf8()
            .ok_or_else(|| FxLoaderError::InvalidUtf8 { name: name.clone() })?;

        let source = RateSource::Bundled { period: None };
        let entries = parse_monthly_rates(content, source, Some(expected)).map_err(|source| {
            FxLoaderError::Parse {
                name: name.clone(),
                source,
            }
        })?;

        all_entries.extend(entries);
    }

    Ok(all_entries)
}

/// Load FX cache from bundled directory and optional folder-provided XML files.
pub fn load_cache_with_folder_files(
    bundled_dir: &Dir<'_>,
    folder_files: impl IntoIterator<Item = RateFile>,
) -> Result<FxCache, FxLoaderError> {
    let mut cache = FxCache::new();

    // Bundled first
    let bundled_entries = load_bundled_dir(bundled_dir)?;
    cache.extend(bundled_entries);

    // Then folder overrides in timestamp order
    let mut overrides: Vec<RateFile> = folder_files.into_iter().collect();
    overrides.sort_by_key(|f| f.modified.unwrap_or(UNIX_EPOCH));

    for file in overrides {
        let name = file.name.to_string_lossy().into_owned();
        let expected = expected_year_month_from_path(&file.name)?;
        let source = RateSource::Folder {
            path: file.name.clone(),
            period: None,
            modified: file.modified,
        };
        let entries = parse_monthly_rates(&file.xml, source, Some(expected)).map_err(|source| {
            FxLoaderError::Parse {
                name: name.clone(),
                source,
            }
        })?;
        cache.extend(entries);
    }

    Ok(cache)
}

/// Load FX cache using only bundled rates.
pub fn load_bundled_cache(bundled_dir: &Dir<'_>) -> Result<FxCache, FxLoaderError> {
    let entries = load_bundled_dir(bundled_dir)?;
    let mut cache = FxCache::new();
    cache.extend(entries);
    Ok(cache)
}
