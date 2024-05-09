#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_imports)]

//!
//! PERSISTENCE
//! -----------
//!
//! Every web application needs to store data. There are, of course, many Rust
//! crates for interacting with NoSQL databases and AWS services like DynamoDB.
//! There are even some ORM-like solutions for Rust that aim to emulate the
//! ORM solutions from the Java world. However, most web applications will rely
//! on relational databases for persistence because of their ubiquity,
//! flexibility, performance, and ACID guarantees.
//!
//! Rust has many solutions for interacting with relational databases. One of
//! the most common that does not try to hide SQL from the user, and which is
//! fully compatible with Tokio, is the `sqlx` crate.
//!
//! In this section, you will learn the basics of using the `sqlx` crate to
//! interact with a PostgreSQL database.
//!
//! To get started:
//!
//! 1. Run `cargo install sqlx-cli` to install the SQLx CLI.
//!
//! 2. Set the environment variable
//! `DATABASE_URL=postgres://<user>:<password>@<address>:<port>/<database>`.
//! For example, `DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres`.
//!
//! 3. Run `sqlx database create` to create the database.
//!
//! 4. Run `sqlx migrate run` to run the migrations in the `migrations` folder.
//!

use axum::{async_trait, body::Body, extract::{Path, State}, response::{IntoResponse, Response}, routing::{delete, get, post, put}, Json, Router};
use hyper::StatusCode;
use sqlx::{postgres::PgPoolOptions, types::time::PrimitiveDateTime, Pool, Postgres};

///
/// EXERCISE 1
///
/// Experiment with the `sqlx::query!` macro. If you have configured your
/// DATABASE_URL correctly (with a running Postgres), then you should be able
/// to get live feedback from the macro.
///
/// At the same time, try the `sqlx::query::<Postgres>` function, which is NOT a macro.
/// What can you say about the difference between the two?
///
/// Note that calling either `query` does not actually execute the query. For that, you
/// need to supply a database pool, which you can do so with the `fetch` family of
/// methods.
///
#[tokio::test]
async fn query_playground() {
    let _ = sqlx::query!("SELECT 1 + 1 AS sum");

    let _ = sqlx::query::<Postgres>("SELECT 1 + 1 AS sum");
}

///
/// EXERCISE 2
///
/// Use the `sqlx::query!` macro to select the result of `1 + 1` from the database,
/// being sure to name the column `sum` using SQL's `AS` keyword.
///
/// Then modify the test to reference a row, which you can obtain by using the
/// `fetch_one` method on the query result, and awaiting and unwrapping it.
///
#[tokio::test]
async fn select_one_plus_one() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let _sum: i32 = sqlx::query!("SELECT 1 + 1 AS sum")
        .fetch_one(&pool).await.unwrap().sum.unwrap();

    assert_eq!(_sum, 2);
}

///
/// EXERCISE 3
///
/// In this example, we are going to show the strength of sqlx by
/// doing a select star query.
///
/// Use the `sqlx::query!` macro to select all columns from the `todos` table.
/// Use a `fetch_all`, and iterate over them, printing out each row.
///
/// What do you notice about the type of the row?
///
#[tokio::test]
async fn select_star() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let todos = sqlx::query!("SELECT * FROM todos")
        .fetch_all(&pool).await.unwrap();

    for todo in todos {
        println!("{:?}", todo)
    }

    assert!(true);
}

///
/// EXERCISE 4
///
/// The `query!` macro supports parameterized queries, which you can create using the
/// placeholder syntax '$1', '$2', etc. You then supply these parameters after the
/// main query.
///
/// Use the `query!` macro to insert a row into the `todo` table, keeping
/// in mind every todo has a title, description, and a boolean indicating
/// whether it is done.
///
/// Using the `RETURNING` keyword, return the id of the inserted row,
/// and assert it is greater than zero.
///
#[tokio::test]
async fn insert_todo() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let title = "Learn SQLx";
    let description = "I should really learn SQLx for my Axum web app";
    let done = false;

    let id = sqlx::query!(
        "INSERT INTO todos (title, description, done) VALUES ($1, $2, $3) RETURNING id",
        title,
        description,
        done,
    ).fetch_one(&pool).await.unwrap().id;

    assert!(id > 0);
}

///
/// EXERCISE 5
///
/// Use the `query!` macro to update a row in the `todo` table.
///
/// You may want to use `execute` to execute the query, rather than one
/// of the fetch methods.
///
#[tokio::test]
async fn test_update_todo() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let id = 1;
    let done = true;

    sqlx::query!(
        "UPDATE todos SET done = $2 WHERE id = $1",
        id,
        done,
    ).execute(&pool).await.unwrap();

    assert!(true);
}

///
/// EXERCISE 6
///
/// Use the `query!` macro to delete a row in the `todo` table.
///
/// You may want to use `execute` to execute the query, rather than one
/// of the fetch methods.
///
#[tokio::test]
async fn test_delete_todo() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let id = 1;

    sqlx::query!("DELETE FROM todos WHERE id = $1", id).execute(&pool).await.unwrap();

    assert!(true);
}

///
/// EXERCISE 7
///
/// You do not have to rely on SQLx generating anonymous structs for you.
/// With the `sqlx::query_as!` macro, you can specify the type of the row
/// yourself.
///
/// In this exercise, introduce a struct called `Todo` that models the `todos`
/// table, and use the `sqlx::query_as!` macro to select all columns from the
/// `todos` table.
///
#[tokio::test]
async fn select_star_as() {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    sqlx::query_as!(TodoRecord, "SELECT * FROM todos")
        .fetch_all(&pool).await.unwrap();

    assert!(true);
}
struct TodoRecord {
    id: i64,
    title: String,
    description: String,
    done: bool,
    created_at: PrimitiveDateTime,
}

#[async_trait]
trait TodoRepo: Send + Sync {
    async fn get_all(&self) -> Vec<Todo>;
    async fn create(&self, title: String, description: String) -> Todo;
    async fn get(&self, id: i64) -> Option<Todo>;
    async fn update(&self, id: i64, title: Option<String>, description: Option<String>, done: Option<bool>) -> Option<Todo>;
    async fn delete(&self, id: i64) -> Option<Todo>;
}

#[derive(Debug, Clone)]
struct TodoRepoPostgres {
    pool: Pool<Postgres>,
}

impl TodoRepoPostgres {
    async fn new() -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(16)
            .connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        Self { pool }
    }
}

#[async_trait]
impl TodoRepo for TodoRepoPostgres {
    async fn get_all(&self) -> Vec<Todo> {
        sqlx::query_as!(TodoRecord, "SELECT * FROM todos")
            .fetch_all(&self.pool).await.unwrap()
            .into_iter()
            .map(|r| Todo::from_record(r))
            .collect()

    }

    async fn create(&self, title: String, description: String) -> Todo {
        Todo::from_record(
            sqlx::query_as!(
                TodoRecord,
                "INSERT INTO todos (title, description, done) VALUES ($1, $2, $3) RETURNING *",
                title,
                description,
                false,
            )
                .fetch_one(&self.pool).await.unwrap()
        )
    }

    async fn get(&self, id: i64) -> Option<Todo> {
        sqlx::query_as!(TodoRecord, "SELECT * FROM todos WHERE id = $1", &id)
            .fetch_optional(&self.pool).await.unwrap()
            .map(|r| Todo::from_record(r))
    }

    async fn update(&self, id: i64, title: Option<String>, description: Option<String>, done: Option<bool>) -> Option<Todo> {
        sqlx::query_as!(
            TodoRecord,
            "UPDATE todos SET title = COALESCE($1, title), description = COALESCE($2, description), done = COALESCE($3, done) WHERE id = $4 RETURNING *",
            title,
            description,
            done,
            id,
        )
            .fetch_optional(&self.pool).await.unwrap()
            .map(|r| Todo::from_record(r))
    }

    async fn delete(&self, id: i64) -> Option<Todo> {
        sqlx::query_as!(
            TodoRecord,
            "DELETE FROM todos WHERE id = $1 RETURNING *",
            id,
        )
            .fetch_optional(&self.pool).await.unwrap()
            .map(|r| Todo::from_record(r))
    }
}

///
/// GRADUATION PROJECT
///
/// In this project, you will build a simple CRUD API for a todo list,
/// which uses sqlx for persistence.
///
pub async fn run_todo_app() {
    let app = Router::<TodoRepoPostgres>::new()
        .route("/todos", get(get_todos::<TodoRepoPostgres>))
        .route("/todos/:id", get(get_todo::<TodoRepoPostgres>))
        .route("/todos", post(create_todo::<TodoRepoPostgres>))
        .route("/todos/:id", put(update_todo::<TodoRepoPostgres>))
        .route("/todos/:id", delete(delete_todo::<TodoRepoPostgres>))
        .with_state(TodoRepoPostgres::new().await);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn get_todos<R: TodoRepo>(state: State<R>) -> Json<Vec<Todo>> {
    Json((*state).get_all().await)
}

async fn get_todo<R: TodoRepo>(Path(id): Path<i64>, state: State<R>) -> Result<Json<Todo>, MissingTodoError> {
    (*state).get(id).await.map(Json).ok_or_else(|| MissingTodoError("".to_string()))
}

async fn create_todo<R: TodoRepo>(state: State<R>, Json(spec): Json<CreateTodo>) -> Json<Todo> {
    Json((*state).create(spec.title, spec.description).await)
}

async fn update_todo<R: TodoRepo>(Path(id): Path<i64>, state: State<R>, Json(update): Json<UpdateTodo>) -> Result<Json<Todo>, MissingTodoError> {
    (*state).update(id, update.title, update.description, update.done).await
        .map(Json).ok_or_else(|| MissingTodoError("".to_string()))
}

async fn delete_todo<R: TodoRepo>(Path(id): Path<i64>, state: State<R>) -> Result<Json<Todo>, MissingTodoError> {
    (*state).delete(id).await.map(Json).ok_or_else(|| MissingTodoError("".to_string()))
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct Todo {
    id: i64,
    title: String,
    description: String,
    done: bool,
}

impl Todo {
    fn from_record(record: TodoRecord) -> Self {
        Todo {
            id: record.id,
            title: record.title,
            description: record.description,
            done: record.done,
        }
    }
}

#[derive(serde::Deserialize)]
struct CreateTodo {
    title: String,
    description: String,
}

#[derive(serde::Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    description: Option<String>,
    done: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct MissingTodoError(String);

impl IntoResponse for MissingTodoError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(Body::from(format!("{{message:{}}}", serde_json::json!(&self.0))))
            .unwrap()
    }
}