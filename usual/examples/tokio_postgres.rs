use chrono::offset::Utc;
use chrono::DateTime;
use dotenv::dotenv;
use std::env;
use std::error::Error;
use tokio_postgres::NoTls;

use usual::{base::Model, base::TryGetRow, partial, query, UsualModel};

#[derive(Debug, UsualModel)]
struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/test".to_string());

    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let _ = client
        .execute(
            query!("INSERT INTO posts (title, content) VALUES ($1, $2)").as_str(),
            &[
                &format!("title {}", Utc::now().timestamp_millis()),
                &"this is some content",
            ],
        )
        .await?;

    let rows = client
        .query(query!("SELECT {Post} FROM posts").as_str(), &[])
        .await?
        .iter()
        .map(Post::from_row)
        .collect::<Vec<_>>();

    println!("rows: {:#?}", rows);

    type Time = DateTime<Utc>;
    let partial_rows = client
        .query(
            query!("SELECT {Post::title,created_at} FROM posts").as_str(),
            &[],
        )
        .await?
        .iter()
        .map(partial!(Post, title as String, created_at as Time))
        .collect::<Vec<_>>();

    println!("partial_rows: {:#?}", partial_rows);

    Ok(())
}
