use cgt::TaxPeriod;
use cgt::money::CurrencyAmount;
use schemars::schema_for;
use serde_json::{Value, json};

#[test]
fn tax_period_schema_retains_its_format() -> Result<(), serde_json::Error> {
    let schema = serde_json::to_value(schema_for!(TaxPeriod))?;
    assert_eq!(schema["type"], "string");
    assert_eq!(schema["pattern"], r"^\d{4}/\d{2}$");
    Ok(())
}

#[test]
fn currency_amount_schema_requires_both_foreign_amount_fields() -> Result<(), serde_json::Error> {
    let schema = serde_json::to_value(schema_for!(CurrencyAmount))?;
    let alternatives = &schema["anyOf"];
    assert_eq!(alternatives.as_array().map(Vec::len), Some(2));
    let object = &alternatives[1];
    assert_eq!(object["type"], "object");
    assert_eq!(object["required"], json!(["amount", "currency"]));
    assert_eq!(object["properties"]["amount"], alternatives[0]);
    assert_ne!(object["properties"]["amount"], Value::Null);
    assert_eq!(object["properties"]["currency"]["type"], "string");
    Ok(())
}
