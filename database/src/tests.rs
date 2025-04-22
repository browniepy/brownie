use sqlx::PgPool;

#[sqlx::test]
async fn create_member(pool: PgPool) {
    let res = sqlx::query!("INSERT INTO member (id) VALUES (12) RETURNING id;")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(res.id, 12)
}
