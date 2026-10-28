use backend::{
    error::AppError,
    models::note::{CreateNote, UpdateNote},
    repo::{note_repo, user_repo},
};
use sqlx::PgPool;
use uuid::Uuid;

async fn user(pool: &PgPool, username: &str) -> Uuid {
    user_repo::create(pool, username, "hash").await.unwrap().id
}

fn new_note(title: &str) -> CreateNote {
    CreateNote {
        title: title.to_string(),
        body: "body".to_string(),
    }
}

#[sqlx::test]
async fn create_then_find_returns_the_note(pool: PgPool) {
    let alice = user(&pool, "alice").await;

    let created = note_repo::create(&pool, alice, &new_note("first"))
        .await
        .unwrap();
    let found = note_repo::find(&pool, alice, created.id).await.unwrap();

    assert_eq!(found.title, "first");
    assert_eq!(found.user_id, alice);
}

#[sqlx::test]
async fn list_is_newest_first_and_paged(pool: PgPool) {
    let alice = user(&pool, "alice").await;
    for title in ["one", "two", "three"] {
        note_repo::create(&pool, alice, &new_note(title))
            .await
            .unwrap();
    }

    let first_page = note_repo::list(&pool, alice, 2, 0).await.unwrap();
    let second_page = note_repo::list(&pool, alice, 2, 2).await.unwrap();

    let titles: Vec<_> = first_page.iter().map(|n| n.title.as_str()).collect();
    assert_eq!(titles, ["three", "two"]);
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].title, "one");
}

#[sqlx::test]
async fn update_changes_only_given_fields(pool: PgPool) {
    let alice = user(&pool, "alice").await;
    let note = note_repo::create(&pool, alice, &new_note("old"))
        .await
        .unwrap();

    let changes = UpdateNote {
        title: Some("new".to_string()),
        body: None,
    };
    let updated = note_repo::update(&pool, alice, note.id, &changes)
        .await
        .unwrap();

    assert_eq!(updated.title, "new");
    assert_eq!(updated.body, "body");
    assert!(updated.updated_at >= note.updated_at);
}

#[sqlx::test]
async fn other_users_notes_are_not_found(pool: PgPool) {
    let alice = user(&pool, "alice").await;
    let bob = user(&pool, "bob").await;
    let note = note_repo::create(&pool, alice, &new_note("private"))
        .await
        .unwrap();
    let changes = UpdateNote {
        title: Some("hijacked".to_string()),
        body: None,
    };

    assert!(matches!(
        note_repo::find(&pool, bob, note.id).await,
        Err(AppError::NotFound)
    ));
    assert!(matches!(
        note_repo::update(&pool, bob, note.id, &changes).await,
        Err(AppError::NotFound)
    ));
    assert!(matches!(
        note_repo::delete(&pool, bob, note.id).await,
        Err(AppError::NotFound)
    ));
    assert!(note_repo::list(&pool, bob, 50, 0).await.unwrap().is_empty());
}

#[sqlx::test]
async fn delete_removes_the_note(pool: PgPool) {
    let alice = user(&pool, "alice").await;
    let note = note_repo::create(&pool, alice, &new_note("gone"))
        .await
        .unwrap();

    note_repo::delete(&pool, alice, note.id).await.unwrap();

    assert!(matches!(
        note_repo::find(&pool, alice, note.id).await,
        Err(AppError::NotFound)
    ));
}
