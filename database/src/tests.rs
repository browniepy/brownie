use sqlx::PgPool;

#[sqlx::test]
async fn create_member(pool: PgPool) {
    let res = sqlx::query!("INSERT INTO member (id) VALUES (12) RETURNING id;")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(res.id, 12)
}

#[sqlx::test]
async fn set_referee(pool: PgPool) {
    sqlx::query!("INSERT INTO member (id) VALUES (2)")
        .execute(&pool)
        .await
        .unwrap();

    let res = sqlx::query!("UPDATE member SET roles = array_append(roles, 'Referee');")
        .execute(&pool)
        .await;

    assert!(res.is_ok())
}

#[sqlx::test]
async fn member_as_referee(pool: PgPool) {
    sqlx::query!("INSERT INTO member (id) VALUES (1)")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query!("UPDATE member SET roles = array_append(roles, 'Member') WHERE id = 1;")
        .execute(&pool)
        .await
        .unwrap();

    let res = sqlx::query!("SELECT assign_referee_range(1)")
        .execute(&pool)
        .await;

    assert!(res.is_err())
}
