use gtfs_schedule::Dataset;
use std::path::Path;

fn test_dataset(dataset_name: &str, expected_result: Result<(), gtfs_schedule::error::Error>) {
    let path = Path::new("tests/_data")
        .join(dataset_name)
        .canonicalize()
        .unwrap();

    let csv_result = Dataset::from_csv(&path);

    match (&csv_result, &expected_result) {
        (Ok(_), Ok(_)) => assert!(true, "Dataset loaded successfully as expected"),
        (Err(e), Ok(_)) => panic!("Expected dataset to load, but got error: {:?}", e),
        (Ok(_), Err(_)) => panic!("Expected dataset to fail loading, but it passed"),
        (Err(actual), Err(expected)) => {
            assert_eq!(
                actual.to_string(),
                expected.to_string(),
                "Error message doesn't match. Expected: {:?}, Got: {:?}",
                expected,
                actual
            );
            return;
        }
    }

    let result = csv_result.unwrap().validate();

    match (result, expected_result) {
        (Ok(_), Ok(_)) => assert!(true, "Dataset validated successfully as expected"),
        (Err(e), Ok(_)) => panic!("Expected dataset to validate, but got error: {:?}", e),
        (Ok(_), Err(_)) => panic!("Expected dataset to fail validation, but it passed"),
        (Err(actual), Err(expected)) => {
            assert_eq!(
                actual.to_string(),
                expected.to_string(),
                "Error message doesn't match. Expected: {:?}, Got: {:?}",
                expected,
                actual
            );
        }
    }
}

//
// Good datasets
//

#[test]
fn test_good_feed_default() {
    test_dataset("good_feed", Ok(()));
}

#[test]
fn test_good_feed_au_sydney_entrances() {
    temp_env::with_var(
        "__TEST__IGNORE_MISSING_CALENDAR_DATES",
        Some("true"),
        || {
            test_dataset("au-sydney-entrances", Ok(()));
        },
    );
}

#[test]
fn test_good_flatten_feed() {
    test_dataset("flatten_feed", Ok(()));
}

#[test]
fn test_good_google_transit() {
    test_dataset("googletransit", Ok(()));
}

#[test]
fn test_good_utf8_bom() {
    test_dataset("utf8bom", Ok(()));
}

//
// Bad datasets
//

// Utils

/// Unsafe function to create a fake `csv::Error` for testing purposes.
///
/// ### Safety
/// This function uses `mem::transmute` to bypass the private constructor of `csv::Error`.
pub unsafe fn create_fake_csv_error(kind: csv::ErrorKind) -> csv::Error {
    std::mem::transmute(Box::new(kind))
}

/// Unsafe function to create a fake `DeserializeError` for testing.
///
/// ### Safety
/// This function uses `mem::transmute` to bypass private field restrictions.
pub unsafe fn create_fake_deserialize_error(
    field: Option<u64>,
    kind: csv::DeserializeErrorKind,
) -> csv::DeserializeError {
    std::mem::transmute((field, kind))
}

pub fn create_fake_csv_position(byte: u64, line: u64, record: u64) -> csv::Position {
    let mut position = csv::Position::new();
    position.set_byte(byte);
    position.set_line(line);
    position.set_record(record);

    let position = position;
    position
}

#[test]
fn test_bad_date_format() {
    let position = create_fake_csv_position(31, 1, 1);
    let deserialize_error = unsafe {
        create_fake_deserialize_error(
            None,
            csv::DeserializeErrorKind::Message(
                "Invalid date format: input contains invalid characters".to_string(),
            ),
        )
    };

    let error_kind = csv::ErrorKind::Deserialize {
        pos: Some(position),
        err: deserialize_error,
    };

    let fake_error = unsafe { create_fake_csv_error(error_kind) };

    test_dataset(
        "bad_date_format",
        Err(gtfs_schedule::error::Error::ParseError(
            gtfs_schedule::error::ParseError::from(gtfs_schedule::error::ParseErrorKind::Csv(
                fake_error,
            )),
        )),
    );
}
