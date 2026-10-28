use backend::models::page::PageParams;

fn params(limit: Option<i64>, offset: Option<i64>) -> PageParams {
    PageParams { limit, offset }
}

#[test]
fn defaults_apply_when_absent() {
    assert_eq!(PageParams::default().resolve().unwrap(), (50, 0));
}

#[test]
fn explicit_values_are_kept() {
    assert_eq!(params(Some(10), Some(20)).resolve().unwrap(), (10, 20));
}

#[test]
fn out_of_range_values_are_rejected() {
    assert!(params(Some(0), None).resolve().is_err());
    assert!(params(Some(101), None).resolve().is_err());
    assert!(params(None, Some(-1)).resolve().is_err());
}
