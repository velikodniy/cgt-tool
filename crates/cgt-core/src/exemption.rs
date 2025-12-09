//! UK Capital Gains Tax annual exemption amounts.
//!
//! This module provides the annual CGT exemption (Annual Exempt Amount) for each tax year.
//! The exemption is the amount of gains you can make before paying CGT.

use crate::CgtError;
use rust_decimal::Decimal;

/// Get the UK annual CGT exemption for a given tax year start.
///
/// # Arguments
/// * `year` - The calendar year when the tax year starts (e.g., 2023 for 2023/24)
///
/// # Returns
/// * `Ok(Decimal)` - The exemption amount for that tax year
/// * `Err(CgtError::UnsupportedExemptionYear)` - If the year is not supported
///
/// # Examples
/// ```
/// use cgt_core::get_exemption;
///
/// let exemption = get_exemption(2023).unwrap();
/// assert_eq!(exemption, rust_decimal::Decimal::from(6000));
/// ```
pub fn get_exemption(year: u16) -> Result<Decimal, CgtError> {
    match year {
        2014 => Ok(Decimal::from(11000)),
        2015 => Ok(Decimal::from(11100)),
        2016 => Ok(Decimal::from(11100)),
        2017 => Ok(Decimal::from(11300)),
        2018 => Ok(Decimal::from(11700)),
        2019 => Ok(Decimal::from(12000)),
        2020 => Ok(Decimal::from(12300)),
        2021 => Ok(Decimal::from(12300)),
        2022 => Ok(Decimal::from(12300)),
        2023 => Ok(Decimal::from(6000)),
        2024 => Ok(Decimal::from(3000)),
        _ => Err(CgtError::UnsupportedExemptionYear(year)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exemption_2014() {
        assert_eq!(get_exemption(2014).expect("2014"), Decimal::from(11000));
    }

    #[test]
    fn test_exemption_2015() {
        assert_eq!(get_exemption(2015).expect("2015"), Decimal::from(11100));
    }

    #[test]
    fn test_exemption_2016() {
        assert_eq!(get_exemption(2016).expect("2016"), Decimal::from(11100));
    }

    #[test]
    fn test_exemption_2017() {
        assert_eq!(get_exemption(2017).expect("2017"), Decimal::from(11300));
    }

    #[test]
    fn test_exemption_2018() {
        assert_eq!(get_exemption(2018).expect("2018"), Decimal::from(11700));
    }

    #[test]
    fn test_exemption_2019() {
        assert_eq!(get_exemption(2019).expect("2019"), Decimal::from(12000));
    }

    #[test]
    fn test_exemption_2020() {
        assert_eq!(get_exemption(2020).expect("2020"), Decimal::from(12300));
    }

    #[test]
    fn test_exemption_2021() {
        assert_eq!(get_exemption(2021).expect("2021"), Decimal::from(12300));
    }

    #[test]
    fn test_exemption_2022() {
        assert_eq!(get_exemption(2022).expect("2022"), Decimal::from(12300));
    }

    #[test]
    fn test_exemption_2023() {
        assert_eq!(get_exemption(2023).expect("2023"), Decimal::from(6000));
    }

    #[test]
    fn test_exemption_2024() {
        assert_eq!(get_exemption(2024).expect("2024"), Decimal::from(3000));
    }

    #[test]
    fn test_exemption_unsupported_past() {
        let result = get_exemption(2010);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CgtError::UnsupportedExemptionYear(2010))
        ));
    }

    #[test]
    fn test_exemption_unsupported_future() {
        let result = get_exemption(2030);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(CgtError::UnsupportedExemptionYear(2030))
        ));
    }
}
