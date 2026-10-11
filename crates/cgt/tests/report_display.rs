use cgt::report_display::{NoteKind, ResultKind, prepare};
use cgt::{Config, Holding, TaxReport, calculate, dsl};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn report(input: &str, year: Option<i32>) -> Result<TaxReport, Box<dyn std::error::Error>> {
    Ok(calculate(
        &dsl::parse(input)?,
        year,
        None,
        &Config::embedded()?,
    )?)
}

#[test]
fn prepared_disposals_preserve_precision_and_allocate_display_pennies()
-> Result<(), Box<dyn std::error::Error>> {
    let report = report(
        include_str!("../../../tests/inputs/DisposalRounding.cgt"),
        None,
    )?;
    let before = report.clone();
    let display = prepare(&report)?;
    let disposal = &display.data().tax_years[0].disposals[0];
    assert_eq!(disposal.allowable_cost, "£0.05");
    assert_eq!(disposal.result.signed, "£0.02");
    assert_eq!(disposal.rounding_adjustment.as_deref(), Some("£0.01"));
    let money = |text: &str| -> Result<Decimal, rust_decimal::Error> {
        text.replace(['£', ','], "").parse()
    };
    let mut cost = Decimal::ZERO;
    let mut gain = Decimal::ZERO;
    for leg in &disposal.matches {
        cost += money(&leg.allowable_cost)?;
        gain += money(&leg.gain_or_loss)?;
    }
    assert_eq!(cost, dec!(0.05));
    assert_eq!(gain, dec!(0.02));
    assert_eq!(report, before);

    let precise = prepare(&self::report(
        "2024-01-01 BUY UNIT 100 @ 1.001\n2024-06-01 SELL UNIT 100 @ 1.005\n\
         2024-01-01 BUY FEE 1 @ 0.5\n2024-06-01 SELL FEE 1 @ 1.004 FEES 0.005\n",
        None,
    )?)?;
    let disposals = &precise.data().tax_years[0].disposals;
    let unit = disposals
        .iter()
        .find(|row| row.ticker == "UNIT")
        .ok_or("missing UNIT")?;
    assert_eq!(unit.sale_unit_price, "£1.005");
    let fee = disposals
        .iter()
        .find(|row| row.ticker == "FEE")
        .ok_or("missing FEE")?;
    assert_eq!(fee.fees.as_deref(), Some("£0.00"));
    assert_eq!(fee.gross_proceeds, fee.proceeds);
    Ok(())
}

#[test]
fn prepared_holdings_use_exact_costs_and_core_ordering() -> Result<(), Box<dyn std::error::Error>> {
    let mut report = report(
        include_str!("../../../tests/renderers/holding_precision.cgt"),
        None,
    )?;
    report.holdings.extend([
        Holding {
            ticker: "a".into(),
            quantity: dec!(1),
            total_cost: dec!(1234),
        },
        Holding {
            ticker: "B".into(),
            quantity: dec!(1),
            total_cost: dec!(2),
        },
        Holding {
            ticker: "ZERO".into(),
            quantity: dec!(0),
            total_cost: Decimal::MAX,
        },
        Holding {
            ticker: "NEGATIVE".into(),
            quantity: dec!(-1),
            total_cost: dec!(1),
        },
    ]);
    let before = serde_json::to_value(&report)?;
    let display = prepare(&report)?;
    let holdings = &display.data().holdings;
    let tickers: Vec<_> = holdings.iter().map(|row| row.ticker.as_str()).collect();
    assert_eq!(tickers, ["B", "FRACTION", "a"]);
    assert_eq!(holdings[1].total_cost, "£10.00");
    assert_eq!(holdings[1].unit_cost.money, "£100.01");
    assert_eq!(holdings[2].unit_cost.money, "£1,234.00");
    assert_eq!(holdings[2].unit_cost.compact, "£1234");
    assert_eq!(serde_json::to_value(&report)?, before);
    Ok(())
}

#[test]
fn prepared_sections_and_notes_follow_the_selected_report() -> Result<(), Box<dyn std::error::Error>>
{
    let input = "2023-01-01 BUY A 1 @ 1\n2023-06-01 SELL A 1 @ 1\n\
                 2024-06-01 DIVIDEND A TOTAL 0 TAX 1\n";
    let all = prepare(&report(input, None)?)?;
    assert_eq!(
        all.data()
            .notes
            .iter()
            .map(|note| note.kind)
            .collect::<Vec<_>>(),
        [
            NoteKind::DisposalCount,
            NoteKind::Gains,
            NoteKind::Proceeds,
            NoteKind::TaxableGain
        ]
    );
    let zero = &all.data().tax_years[0].disposals[0];
    assert_eq!(zero.result.kind, ResultKind::Gain);
    assert_eq!(zero.rounding_adjustment, None);
    assert_eq!(zero.fees, None);
    assert_eq!(all.data().tax_years[0].dividend, None);
    let selected = prepare(&report(input, Some(2024))?)?;
    assert_eq!(selected.data().notes.len(), 1);
    assert_eq!(selected.data().notes[0].kind, NoteKind::TaxableGain);
    let dividend = selected.data().tax_years[0]
        .dividend
        .as_ref()
        .ok_or("missing dividend")?;
    assert_eq!(dividend.income, "£0.00");
    assert_eq!(dividend.tax_paid.as_deref(), Some("£1.00"));
    let income = prepare(&report("2024-06-01 DIVIDEND A TOTAL 42.5\n", None)?)?;
    assert_eq!(
        income.data().tax_years[0]
            .dividend
            .as_ref()
            .ok_or("missing income")?
            .tax_paid,
        None
    );
    let empty = prepare(&report("2024-01-01 BUY A 1 @ 1\n", None)?)?;
    assert!(empty.data().notes.is_empty());
    Ok(())
}

#[test]
fn prepared_echoes_keep_original_currencies_and_sort_once() -> Result<(), Box<dyn std::error::Error>>
{
    let report = TaxReport {
        tax_years: Vec::new(),
        holdings: Vec::new(),
        warnings: vec!["example warning".into()],
        transactions: Some(dsl::parse(
            "2024-06-02 SELL Z 0.1 @ 1.005 USD FEES 0.0001 USD\n\
             2024-06-01 BUY B 2 @ 1 EUR\n2024-06-01 BUY A 3 @ 2\n\
             2024-06-02 DIVIDEND Z TOTAL 1.005 USD\n2024-06-01 SPLIT A RATIO 2\n",
        )?),
    };
    let display = prepare(&report)?;
    let data = display.data();
    assert_eq!(
        data.trades
            .iter()
            .map(|trade| trade.ticker.as_str())
            .collect::<Vec<_>>(),
        ["A", "B", "Z"]
    );
    assert_eq!(data.trades[2].price, "$1.005");
    assert_eq!(data.trades[2].fees, "$0.0001");
    assert_eq!(data.trades[2].date.iso, "2024-06-02");
    assert_eq!(data.trades[2].date.uk, "02/06/2024");
    assert_eq!(data.asset_events[0].amount.as_deref(), Some("2"));
    assert_eq!(data.asset_events[0].value, None);
    assert_eq!(data.asset_events[1].value.as_deref(), Some("1.01 USD"));
    assert_eq!(data.warnings, ["example warning"]);
    Ok(())
}

#[test]
fn direct_reports_return_arithmetic_errors_without_partial_display()
-> Result<(), Box<dyn std::error::Error>> {
    let original = report("2024-01-01 BUY A 1 @ 1\n2024-06-01 SELL A 1 @ 2\n", None)?;
    let mut ratio = original.clone();
    ratio.tax_years[0].disposals[0].quantity = dec!(0.1);
    ratio.tax_years[0].disposals[0].gross_proceeds = Decimal::MAX;
    let error = prepare(&ratio).err().ok_or("expected ratio error")?;
    assert!(
        error
            .to_string()
            .contains("sale unit price of A on 2024-06-01")
    );
    let mut sums = original.clone();
    sums.tax_years[0].disposals[0].legs[0].allowable_cost = Decimal::MAX;
    let extra = original.tax_years[0].disposals[0].legs[0].clone();
    sums.tax_years[0].disposals[0].legs.push(extra);
    assert!(prepare(&sums).is_err());
    let mut adjustment = original.clone();
    adjustment.tax_years[0].disposals[0].proceeds = -Decimal::MAX;
    assert!(prepare(&adjustment).is_err());
    let mut fees = original.clone();
    fees.tax_years[0].disposals[0].gross_proceeds = -Decimal::MAX;
    assert!(prepare(&fees).is_err());
    let mut zero = original;
    zero.tax_years[0].disposals[0].quantity = Decimal::ZERO;
    assert_eq!(
        prepare(&zero)?.data().tax_years[0].disposals[0].sale_unit_price,
        "£0"
    );
    Ok(())
}

#[test]
fn direct_reports_reject_precision_loss_in_match_totals() -> Result<(), Box<dyn std::error::Error>>
{
    let original = report("2024-01-01 BUY A 1 @ 1\n2024-06-01 SELL A 1 @ 2\n", None)?;
    for field in ["matched cost", "matched result"] {
        let mut report = original.clone();
        let disposal = &mut report.tax_years[0].disposals[0];
        let mut extra = disposal.legs[0].clone();
        if field == "matched cost" {
            disposal.legs[0].allowable_cost = dec!(0.49);
            extra.allowable_cost = Decimal::MAX;
        } else {
            disposal.legs[0].gain_or_loss = dec!(0.49);
            extra.gain_or_loss = Decimal::MAX;
        }
        disposal.legs.push(extra);
        let error = prepare(&report).err().ok_or("expected precision error")?;
        assert!(
            error
                .to_string()
                .contains(&format!("{field} of A on 2024-06-01"))
        );
    }
    Ok(())
}

#[test]
fn direct_reports_reject_precision_loss_in_sale_fees() -> Result<(), Box<dyn std::error::Error>> {
    let original = report("2024-01-01 BUY A 1 @ 1\n2024-06-01 SELL A 1 @ 2\n", None)?;
    for proceeds in [dec!(-0.49), dec!(0.49)] {
        let mut report = original.clone();
        let disposal = &mut report.tax_years[0].disposals[0];
        disposal.gross_proceeds = Decimal::MAX;
        disposal.proceeds = proceeds;
        disposal.legs[0].allowable_cost = Decimal::ZERO;
        disposal.legs[0].gain_or_loss = proceeds;
        let error = prepare(&report).err().ok_or("expected precision error")?;
        assert!(error.to_string().contains("sale fees of A on 2024-06-01"));
    }
    Ok(())
}

#[test]
fn direct_reports_preserve_representable_totals_at_decimal_limit()
-> Result<(), Box<dyn std::error::Error>> {
    let mut report = report("2024-01-01 BUY A 1 @ 1\n2024-06-01 SELL A 1 @ 2\n", None)?;
    let disposal = &mut report.tax_years[0].disposals[0];
    disposal.gross_proceeds = Decimal::MAX;
    disposal.proceeds = Decimal::MAX;
    disposal.legs[0].allowable_cost = Decimal::MAX;
    disposal.legs[0].gain_or_loss = Decimal::ZERO;
    let mut extra = disposal.legs[0].clone();
    extra.allowable_cost = dec!(0.00);
    disposal.legs.push(extra);
    let display = prepare(&report)?;
    let disposal = &display.data().tax_years[0].disposals[0];
    assert_eq!(disposal.matches[0].allowable_cost, disposal.allowable_cost);
    assert_eq!(disposal.matches[1].allowable_cost, "£0.00");
    assert_eq!(disposal.result.signed, "£0.00");
    assert_eq!(disposal.rounding_adjustment, None);
    assert_eq!(disposal.fees, None);
    Ok(())
}
