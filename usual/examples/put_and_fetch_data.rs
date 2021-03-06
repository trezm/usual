use anyhow::Error as ErrorStruct;
use chrono::offset::Utc;
use chrono::DateTime;
use dotenv::dotenv;
use std::any::{Any, TypeId};
use std::env;
use std::error::Error;
use tokio_postgres::NoTls;
use tokio_postgres::Row;
use usual::partial;

use usual::{base::Model, base::TryGetRow, impl_model, query, UsualModel};

// Note that Default is required for unusual fields
#[derive(Clone, Debug, Default)]
struct NonUsualField;

#[derive(Debug, UsualModel)]
struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[unusual]
    pub do_not_include: NonUsualField,
}

struct RowWrapper(Row);

impl TryGetRow for RowWrapper {
    fn try_get<T: 'static>(&self, index: &str) -> Result<T, ErrorStruct> {
        let t = TypeId::of::<T>();

        if t == TypeId::of::<i32>() {
            self.0
                .try_get::<_, i32>(index)
                .map(|v| *(Box::new(v) as Box<dyn Any>).downcast().unwrap())
                .map_err(|e| anyhow::Error::from(e))
        } else if t == TypeId::of::<i64>() {
            self.0
                .try_get::<_, i64>(index)
                .map(|v| *(Box::new(v) as Box<dyn Any>).downcast().unwrap())
                .map_err(|e| anyhow::Error::from(e))
        } else if t == TypeId::of::<String>() {
            self.0
                .try_get::<_, String>(index)
                .map(|v| *(Box::new(v) as Box<dyn Any>).downcast().unwrap())
                .map_err(|e| anyhow::Error::from(e))
        } else if t == TypeId::of::<DateTime<Utc>>() {
            self.0
                .try_get::<_, DateTime<Utc>>(index)
                .map(|v| *(Box::new(v) as Box<dyn Any>).downcast().unwrap())
                .map_err(|e| anyhow::Error::from(e))
        } else {
            Err(anyhow::anyhow!(
                "The type passed in for index {} is unhandled at this time.",
                index
            ))
        }
    }
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
        .into_iter()
        .map(|r| Post::from_row(&RowWrapper(r)))
        .collect::<Vec<_>>();

    println!("rows: {:#?}", rows);

    type Time = DateTime<Utc>;
    let partial_rows = client
        .query(
            query!("SELECT {Post::title,created_at} FROM posts").as_str(),
            &[],
        )
        .await?
        .into_iter()
        .map(|r| (partial!(Post, title as String, created_at as Time))(&RowWrapper(r)))
        .collect::<Vec<_>>();

    println!("partial_rows: {:#?}", partial_rows);

    Ok(())
}
