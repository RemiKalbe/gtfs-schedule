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
