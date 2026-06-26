use chrono::{NaiveDateTime, TimeZone, Utc};
use serde::Serializer;

pub fn serialize_naive_as_utc<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let utc_date = Utc.from_utc_datetime(date);
    serializer.serialize_str(&utc_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestWrapper {
        #[serde(serialize_with = "serialize_naive_as_utc")]
        pub timestamp: NaiveDateTime,
    }

    #[test]
    fn test_serialize_naive_as_utc_standard() {
        let naive_dt = NaiveDate::from_ymd_opt(2026, 6, 26)
            .unwrap()
            .and_hms_opt(10, 45, 30)
            .unwrap();

        let wrapper = TestWrapper {
            timestamp: naive_dt,
        };

        let json_result = serde_json::to_string(&wrapper);

        assert!(json_result.is_ok());
        assert_eq!(
            json_result.unwrap(),
            r#"{"timestamp":"2026-06-26T10:45:30Z"}"#
        );
    }

    #[test]
    fn test_serialize_naive_as_utc_midnight() {
        let naive_dt = NaiveDate::from_ymd_opt(2026, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let wrapper = TestWrapper {
            timestamp: naive_dt,
        };

        let json_result = serde_json::to_string(&wrapper);

        assert!(json_result.is_ok());
        assert_eq!(
            json_result.unwrap(),
            r#"{"timestamp":"2026-01-01T00:00:00Z"}"#
        );
    }
}
