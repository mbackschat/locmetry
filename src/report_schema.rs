//! The schema is generated from the same contract that reads and writes reports.
use serde_json::Value;

pub fn report_schema() -> Value {
    crate::schema::json::<crate::contract::MeasurementReport>()
}
