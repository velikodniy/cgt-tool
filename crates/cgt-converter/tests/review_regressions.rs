use cgt_converter::BrokerConverter;
use cgt_converter::schwab::{SchwabConverter, SchwabInput};
use serde_json::json;

#[test]
fn overlapping_vests_keep_deposit_identity() -> Result<(), Box<dyn std::error::Error>> {
    let transactions = json!({"BrokerageTransactions": [
        {"Date":"01/17/2024","Action":"Stock Plan Activity","Symbol":"FOO","Quantity":"100"},
        {"Date":"01/19/2024","Action":"Stock Plan Activity","Symbol":"FOO","Quantity":"200"}
    ]});
    let awards = json!({"Transactions": [
        {"Date":"01/17/2024","Action":"Deposit","Symbol":"FOO","Quantity":"100",
         "TransactionDetails":[{"Details":{"VestDate":"01/15/2024","VestFairMarketValue":"$10.00"}}]},
        {"Date":"01/19/2024","Action":"Deposit","Symbol":"FOO","Quantity":"200",
         "TransactionDetails":[{"Details":{"VestDate":"01/17/2024","VestFairMarketValue":"$20.00"}}]}
    ]});
    let output = SchwabConverter.convert(&SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    })?;
    assert!(
        output
            .cgt_content
            .contains("2024-01-15 BUY FOO 100 @ 10.00 USD")
    );
    assert!(
        output
            .cgt_content
            .contains("2024-01-17 BUY FOO 200 @ 20.00 USD")
    );
    Ok(())
}

#[test]
fn ambiguous_vest_lookback_is_rejected() {
    let transactions = json!({"BrokerageTransactions": [
        {"Date":"01/19/2024","Action":"Stock Plan Activity","Symbol":"FOO","Quantity":"100"}
    ]});
    let awards = json!({"Transactions": [
        {"Date":"01/15/2024","Action":"Lapse","Symbol":"FOO",
         "TransactionDetails":[{"Details":{"FairMarketValuePrice":"$10.00"}}]},
        {"Date":"01/17/2024","Action":"Lapse","Symbol":"FOO",
         "TransactionDetails":[{"Details":{"FairMarketValuePrice":"$20.00"}}]}
    ]});
    let result = SchwabConverter.convert(&SchwabInput {
        transactions_json: transactions.to_string(),
        awards_json: Some(awards.to_string()),
    });
    assert!(
        matches!(result, Err(cgt_converter::ConvertError::InvalidTransaction(message)) if message.contains("Ambiguous awards"))
    );
}
