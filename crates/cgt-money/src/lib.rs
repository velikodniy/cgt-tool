mod amount;
mod cache;
mod loader;
mod parser;
mod types;

use include_dir::{Dir, include_dir};

pub use amount::CurrencyAmount;
pub use cache::FxCache;
pub use iso_currency::Currency;
pub use loader::{FxLoaderError, RateFile, load_bundled_cache, load_cache_with_folder_files};
pub use parser::{FxParseError, parse_monthly_rates};
pub use types::{RateEntry, RateKey, RateSource};

/// Bundled FX rates directory (embedded at compile time).
/// Contains monthly XML files from HMRC for January 2015 through August 2025 (latest available).
pub static BUNDLED_RATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/resources/rates");

/// Load FX cache using only bundled rates.
pub fn load_default_cache() -> Result<FxCache, FxLoaderError> {
    load_bundled_cache(&BUNDLED_RATES_DIR)
}

/// Load FX cache using bundled rates plus folder XMLs provided by the caller.
pub fn load_cache_with_overrides(
    folder_files: impl IntoIterator<Item = RateFile>,
) -> Result<FxCache, FxLoaderError> {
    load_cache_with_folder_files(&BUNDLED_RATES_DIR, folder_files)
}
