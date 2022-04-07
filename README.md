# Usual
## The nORM for everyday use

Usual is a simple wrapper for querying, and deserializing SQL queries in a (relatively) type safe way. Queries are still written in plain SQL, so you, the developer get all of the power (and responsibility) of not having a DSL.

### Models

Usual revolves around models. Models let you query the data you want, and only the data you want. Simply add a `derive` for `UsualModel`.

```rs
use usual::{base::Model, base::TryGetRow, query, UsualModel};

derive(UsualModel)
struct Post {
    id: i64,
    title: String,
    content: String,
}
```

This gives you access to deserialize like a boss (this example uses [tokio-postgres](https://github.com/sfackler/rust-postgres)):

```rs
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
```

The only special, usual-specific, language here is `{Post}`. This means "all of the fields in the `Post` model."

### Partials

Often, you don't want to query every field on a table, we have that too with the `partial` macro.

```rs
let partial_rows = client
    .query(
        query!("SELECT {Post::title,created_at} FROM posts").as_str(),
        &[],
    )
    .await?
    .iter()
    .map(partial!(Post, title as String, created_at as Time))
    .collect::<Vec<_>>();

let post = partial_rows.get(0).unwrap();

// This is fine, because we grabbed the title row
println!("title: {}", post.title);

// This is a compile-time error, because we haven't fetched that row from the table.
println!("content: {}", post.content);
```

The syntax is simple, it's just `ModelName::field,field,field`.

### Aliasing

Aliasing is supported via a query of the form:

```rs
query!("SELECT {TestModel::some_string as t} FROM test_model as t")
```

This query selects a field from the `test_model` table, which we've aliased as `t` in the query.

### Multiple tables

Fetching from multiple tables is also possible, simply add more `{}`:

```rs
query!("SELECT {TestModel as t}, {TestModel2 as t2} FROM test_model as t JOIN test_model as t2 on t.id = t2.id")
```

This will let you do a single query and hydrate multiple types of objects from the resulting rows.

### Including non-sql values

Including values not stored in SQL can be achieved by using the `#[unusual]` attribute. In order to be unusual, a field must implement `Default`, as when the struct is created this is what will be called for that field.

```rs
use usual::{base::Model, base::TryGetRow, query, UsualModel};

struct SomethingElse {}

derive(UsualModel)
struct Post {
    id: i64,
    title: String,
    content: String,
    #[unusual]
    non_sql: Option<SomethingElse>
}
```
