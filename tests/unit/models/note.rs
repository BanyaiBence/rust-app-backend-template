use backend::{
    error::AppError,
    models::note::{CreateNote, UpdateNote},
};

fn create(title: &str, body: &str) -> CreateNote {
    CreateNote {
        title: title.to_string(),
        body: body.to_string(),
    }
}

#[test]
fn valid_note_passes() {
    assert!(create("Groceries", "milk, eggs").validate().is_ok());
}

#[test]
fn blank_title_is_rejected() {
    assert!(matches!(
        create("   ", "").validate(),
        Err(AppError::Validation(_))
    ));
}

#[test]
fn overlong_title_and_body_are_rejected() {
    assert!(create(&"a".repeat(201), "").validate().is_err());
    assert!(create("ok", &"a".repeat(10_001)).validate().is_err());
}

#[test]
fn empty_update_is_valid() {
    let changes = UpdateNote {
        title: None,
        body: None,
    };
    assert!(changes.validate().is_ok());
}

#[test]
fn update_validates_present_fields() {
    let changes = UpdateNote {
        title: Some(String::new()),
        body: None,
    };
    assert!(changes.validate().is_err());
}
