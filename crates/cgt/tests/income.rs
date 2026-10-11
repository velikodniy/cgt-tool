use cgt::{Config, TaxReport, calculate, dsl};
use rust_decimal_macros::dec;

fn report(input: &str, year: Option<i32>) -> Result<TaxReport, Box<dyn std::error::Error>> {
    let transactions = dsl::parse(input)?;
    Ok(calculate(&transactions, year, None, &Config::embedded()?)?)
}

#[test]
fn plain_summary_includes_tax_without_dividend_income() -> Result<(), Box<dyn std::error::Error>> {
    let report = report("2024-06-01 DIVIDEND A TOTAL 0 TAX 1\n", None)?;
    let year = report.tax_years.first().ok_or("Missing income year")?;
    assert_eq!(year.dividend_income, dec!(0));
    assert_eq!(year.dividend_tax_paid, dec!(1));
    let text = cgt::format::plain::render(&cgt::report_display::prepare(&report)?);
    assert!(
        text.contains("Dividend income: £0.00 (tax paid: £1.00)"),
        "{text}"
    );
    Ok(())
}

#[test]
fn income_only_years_follow_the_april_six_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let report = report(
        "2024-04-05 DIVIDEND CASH TOTAL 20 TAX 2\n\
         2024-04-06 DIVIDEND CASH TOTAL 30 TAX 3\n",
        None,
    )?;
    assert_eq!(report.tax_years.len(), 2);
    let [first, second] = report.tax_years.as_slice() else {
        return Err("Expected two income years".into());
    };
    assert_eq!(first.period.start_year(), 2023);
    assert_eq!(first.dividend_income, dec!(20));
    assert_eq!(first.dividend_tax_paid, dec!(2));
    assert_eq!(second.period.start_year(), 2024);
    assert_eq!(second.dividend_income, dec!(30));
    assert_eq!(second.dividend_tax_paid, dec!(3));
    assert!(
        report
            .tax_years
            .iter()
            .all(|year| year.disposals.is_empty())
    );
    Ok(())
}

#[test]
fn accumulation_reports_income_and_tax_without_an_extra_basis_adjustment()
-> Result<(), Box<dyn std::error::Error>> {
    let report = report(
        "2023-01-01 BUY FUND 10 @ 10\n\
         2023-04-06 ACCUMULATION FUND 10 TOTAL 30 TAX 3\n\
         2024-04-05 DIVIDEND FUND TOTAL 20 TAX 2\n",
        Some(2023),
    )?;
    let year = report.tax_years.first().ok_or("Missing income year")?;
    assert_eq!(year.dividend_income, dec!(50));
    assert_eq!(year.dividend_tax_paid, dec!(5));
    assert_eq!(year.total_gain, dec!(0));
    let holding = report.holdings.first().ok_or("Missing fund holding")?;
    assert_eq!(holding.quantity, dec!(10));
    assert_eq!(holding.total_cost, dec!(130));
    Ok(())
}

#[test]
fn income_fixture_reports_all_income_and_ignores_zero_quantity_accumulation()
-> Result<(), Box<dyn std::error::Error>> {
    let report = report(include_str!("../../../tests/inputs/IncomeYears.cgt"), None)?;
    let years: Vec<_> = report
        .tax_years
        .iter()
        .map(|year| year.period.start_year())
        .collect();
    assert_eq!(years, vec![2022, 2023, 2024]);
    let income: Vec<_> = report
        .tax_years
        .iter()
        .map(|year| year.dividend_income)
        .collect();
    assert_eq!(income, vec![dec!(20), dec!(70), dec!(0)]);
    let tax: Vec<_> = report
        .tax_years
        .iter()
        .map(|year| year.dividend_tax_paid)
        .collect();
    assert_eq!(tax, vec![dec!(2), dec!(7), dec!(0)]);
    assert_eq!(report.tax_years[2].total_gain, dec!(20));
    Ok(())
}
