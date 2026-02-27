use chrono::NaiveDate;
use std::cmp::Ordering;

/// Compare records by date, then ticker.
pub fn compare_date_ticker(
    left_date: NaiveDate,
    left_ticker: &str,
    right_date: NaiveDate,
    right_ticker: &str,
) -> Ordering {
    left_date
        .cmp(&right_date)
        .then_with(|| left_ticker.cmp(right_ticker))
}

/// Sort records by date, then ticker.
pub fn sort_by_date_ticker<T, F, G>(items: &mut [T], get_date: F, get_ticker: G)
where
    F: Fn(&T) -> NaiveDate,
    G: Fn(&T) -> &str,
{
    items.sort_by(|left, right| {
        compare_date_ticker(
            get_date(left),
            get_ticker(left),
            get_date(right),
            get_ticker(right),
        )
    });
}

#[cfg(test)]
mod tests {
    use super::sort_by_date_ticker;
    use chrono::NaiveDate;

    #[derive(Debug, PartialEq)]
    struct Item {
        date: NaiveDate,
        ticker: String,
    }

    #[test]
    fn sorts_by_date_then_ticker() {
        let mut items = vec![
            Item {
                date: NaiveDate::from_ymd_opt(2024, 5, 2).expect("valid date"),
                ticker: "MSFT".to_string(),
            },
            Item {
                date: NaiveDate::from_ymd_opt(2024, 5, 1).expect("valid date"),
                ticker: "TSLA".to_string(),
            },
            Item {
                date: NaiveDate::from_ymd_opt(2024, 5, 1).expect("valid date"),
                ticker: "AAPL".to_string(),
            },
        ];

        sort_by_date_ticker(&mut items, |item| item.date, |item| &item.ticker);

        assert_eq!(items[0].ticker, "AAPL");
        assert_eq!(items[1].ticker, "TSLA");
        assert_eq!(items[2].ticker, "MSFT");
    }
}
