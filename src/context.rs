#![allow(dead_code)]
#![allow(unreachable_code)]

//!
//! CONTEXT
//! -------
//!
//! So far, you have seen context-free web applications in Rust. These web applications
//! do not share context with a higher level or between themselves.
//!
//! While appropriate for very simple applications, most real world applications will need
//! some form of context. For example, a web application might need to access a database,
//! and it would be inefficient to open a new connection to the database for every request.
//! So most handlers will end up drawing from a database connection pool.
//!
//! Axum has been designed to facilitate sharing context, both between handlers, and
//! between handlers and higher levels of the application.
//!
//! In this section, you will explore these mechanisms.
//!

use std::{collections::HashMap, sync::Arc};

#[allow(unused_imports)]
use axum::extract::{State, Path};
use axum::{body::Body, response::{IntoResponse, Response}, Json};
#[allow(unused_imports)]
use axum::{http::Method, routing::*};
#[allow(unused_imports)]
use hyper::Request;
use hyper::StatusCode;
use tokio::sync::Mutex;

///
/// EXERCISE 1
///
/// While not a highly maintainable solution, it is possible to create contextual
/// web applications by using closures to capture context.
///
/// In this exercise, share the same `usd_to_gbp` rate between the two routes
/// by using closures.
///
#[tokio::test]
async fn closure_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let gbp_to_usd_rate = 1.3;

    let app = Router::new()
        .route("/usd_to_gbp", get(move |usd: String| async move { convert_usd_to_gbp(usd, gbp_to_usd_rate) }))
        .route("/gbp_to_usd", get(move |usd: String| async move { convert_gbp_to_usd(usd, gbp_to_usd_rate) }));

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}
fn convert_usd_to_gbp(usd: String, gbp_to_usd_rate: f64) -> String {
    (usd.parse::<f64>().unwrap() * gbp_to_usd_rate).to_string()
}
fn convert_gbp_to_usd(gbp: String, gbp_to_usd_rate: f64) -> String {
    (gbp.parse::<f64>().unwrap() / gbp_to_usd_rate).to_string()
}

///
/// EXERCISE 2
///
/// The previous exercise was almost too easy, because the context was of type
/// `f64`, which is `Copy`. This means that the context was copied into both
/// closures, rather than truly shared between them.
///
/// Of course, for any data type that you do not wish to mutate, you can always
/// implement `Clone`, and then manually clone the context into each closure.
///
/// But what if you want to share a mutable context between handlers?
///
/// In this exercise, you will share a mutable context between handlers.
/// Specifically, you will share a mutably editable exchange rate between
/// GBP and USD currencies. Consider using the `Arc` type, which you will
/// have to use atop Tokio's Mutex in order to support mutation.
///
/// When you are done, try to generalize what you have learned about sharing
/// context between handlers. What would you use if the context were
/// immutable? What would you use if the context were mutable?
///
#[tokio::test]
async fn shared_mutable_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let arc = Arc::new(Mutex::new(1.3));
    let arc_clone = arc.clone();

    let _app = Router::new()
        .route(
            "/usd_to_gbp",
            get(move |usd: String| async move {
                let rate = *arc.lock().await;
                convert_usd_to_gbp(usd, rate)
            }),
        )
        .route(
            "/gbp_to_usd",
            get(move |gbp: String| async move {
                let rate = *arc_clone.lock().await;
                convert_gbp_to_usd(gbp, rate)
            }),
        );

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}

// #[derive(Copy, Clone, Debug, PartialEq)]
// struct ConversionRate(f64);

///
/// EXERCISE 3
///
/// Having to write all your handlers as closures is not very ergonomic, and could
/// lead to either boilerplate or gigantic functions that define all handlers.
///
/// Instead, Axum provides direct support for sharing context. This shared context
/// can be specified in your Router, and it can be passed into your handlers as
/// a State parameter.
///
/// In this exercise, share the same `usd_to_gbp` rate between the two routes
/// by using the `State` extractor, defined in `axum::extract`. Note that you
/// will have to supply the state by using the `.with_state` method on your
/// Router. An example (using () as the state type) has been provided below.
///
#[tokio::test]
async fn state_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let _gbp_to_usd_rate = 1.3;

    let _app = Router::<f64>::new()
        .route("/usd_to_gbp", get(usd_to_gbp_handler))
        .route("/gbp_to_usd", get(gbp_to_usd_handler))
        .with_state(_gbp_to_usd_rate);

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}
async fn usd_to_gbp_handler(State(rate): State<f64>, amount: String) -> String {
    convert_usd_to_gbp(amount, rate)
}
async fn gbp_to_usd_handler(State(rate): State<f64>, amount: String) -> String {
    convert_gbp_to_usd(amount, rate)
}

///
/// EXERCISE 4
///
/// Now that you have seen Axum's first-class support for context sharing, it's
/// time to leverage your knowledge of Rust to enable sharing mutable context
/// between handlers, building upon what you have done in previous exercises.
///
/// Modify this exercise to share a mutable exchange rate between GBP and USD.
///
#[tokio::test]
async fn mutable_state_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let _gbp_to_usd_rate = 1.3;

    let app = Router::<Arc<Mutex<f64>>>::new()
        .route("/usd_to_gbp", get(mutable_usd_to_gbp_handler))
        .route("/gbp_to_usd", get(mutable_gbp_to_usd_handler))
        .route("/set_exchange_rate", put(set_exchange_rate_handler))
        .with_state(Arc::new(Mutex::new(_gbp_to_usd_rate)));

    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri("/set_exchange_rate")
                .body(Body::from("1.7"))
                .unwrap(),
        )
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "170");
}
async fn mutable_usd_to_gbp_handler(State(rate_arc): State<Arc<Mutex<f64>>>, amount: String) -> String {
    convert_usd_to_gbp(amount, *rate_arc.lock().await)
}
async fn mutable_gbp_to_usd_handler(State(rate_arc): State<Arc<Mutex<f64>>>, amount: String) -> String {
    convert_gbp_to_usd(amount, *rate_arc.lock().await)
}
async fn set_exchange_rate_handler(State(rate_arc): State<Arc<Mutex<f64>>>, new_amount: String) -> () {
    *(rate_arc.lock().await) = new_amount.parse::<f64>().unwrap();
}

///
/// EXERCISE 5
///
/// The type `S` flows through a lot of the types in Axum (Router, MethodRouter,
/// Handler, etc.). If you examine closely the signatures for methods that combine
/// routers, you will see that their state types have to be exactly the same.
///
/// What happens if your handlers, from different parts of your application,
/// require totally different state?
///
/// One possible solution to this problem is to make your handlers polymorphic
/// in the type of state they handle, and to use traits that expose "accessors"
/// for the specific state type they require.
///
/// In this exercise, you will use this technique to complete the following
/// exercise.
///
/// Assume that some handlers require state type `GBPtoUSD`, and that other
/// handlers require state type `EURtoUSD`. Further, assume you have a
/// composite state type, `AllExchangeRates`, that contains both `GBPtoUSD`
/// and `EURtoUSD`.
///
/// Invent traits that can describe what each type of handler requires from
/// the "global state", and then make the handlers polymorphic in the state
/// type, requiring only an implementation of the appropriate trait.
///
/// You might have to supply some type hints to the compiler in order to
/// construct the routes with your polymorphic handlers.
///
/// This technique is very powerful, and it can allow state to vary across
/// a modular web application, where different types of endpoints have
/// different requirements for context.
///
#[tokio::test]
async fn generic_state_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let _app = Router::new()
        .route("/usd_to_gbp", get(generic_usd_to_gbp_handler))
        .route("/gbp_to_usd", get(generic_gbp_to_usd_handler))
        .route("/eur_to_usd", get(generic_eur_to_usd_handler))
        .route("/usd_to_eur", get(generic_usd_to_eur_handler))
        .with_state(AllExchangeRates {
            gbp_to_usd: GBPtoUSD(1.3),
            eur_to_usd: EURtoUSD(1.2),
        });

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}
async fn generic_usd_to_gbp_handler(_price: String) -> String {
    todo!("Use State to access the exchange rate")
}
async fn generic_gbp_to_usd_handler(_price: String) -> String {
    todo!("Use State to access the exchange rate")
}
async fn generic_eur_to_usd_handler(_price: String) -> String {
    todo!("Use State to access the exchange rate")
}
async fn generic_usd_to_eur_handler(_price: String) -> String {
    todo!("Use State to access the exchange rate")
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct AllExchangeRates {
    gbp_to_usd: GBPtoUSD,
    eur_to_usd: EURtoUSD,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct GBPtoUSD(f64);
#[derive(Clone, Copy, Debug, PartialEq)]
struct EURtoUSD(f64);

///
/// EXERCISE 6
///
/// Although it is possible to share virtually any kind of context using State,
/// with the appropriate type classes and polymorphic handlers allowing state
/// to vary across a web application, some would prefer to reduce the amount of
/// ceremony required to share varying context, and are willing to accept a
/// tradeoff in terms of static type safety.
///
/// For this audience, Axum has a solution called Extensions. Extensions can be
/// used to share context between middleware and handlers, or just to share
/// context either between handlers, between middleware, or between either
/// handlers or middleware and higher levels of the application.
///
/// In order to use extensions, your handler may require a parameter of type
/// `axum::extract::Extension<T>` where `T` is the type of the context you
/// wish to share. Then you must install a layer in your router, which holds
/// the context, and you can do that with the `Extension(...)` constructor.
///
/// In this exercise, you will implement the same exchange-rate-sharing
/// application, but this time using an extension to share state.
///
/// Experiment with what happens when you forget to install the extension.
/// Under what circumstances would you prefer extensions to state for
/// sharing context? Under what circumstances would you prefer the reverse?
///
#[tokio::test]
async fn extension_shared_context() {
    // for Body::collect
    use http_body_util::BodyExt;
    /// for ServiceExt::oneshot
    use tower::util::ServiceExt;

    let _gbp_to_usd_rate = 1.3;

    let _app = Router::new()
        .route("/usd_to_gbp", get(extension_usd_to_gbp_handler))
        .route("/gbp_to_usd", get(extension_gbp_to_usd_handler));

    let response = _app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/usd_to_gbp")
                .body(Body::from("100"))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();

    let _body_as_string = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(_body_as_string, "130");
}
async fn extension_usd_to_gbp_handler() -> String {
    todo!("Use Extensions to access the exchange rate")
}
async fn extension_gbp_to_usd_handler() -> String {
    todo!("Use Extensions to access the exchange rate")
}

///
/// GRADUATION PROJECT
///
/// Provide a complete implementation of the following API, which uses shared mutable
/// state across all the handlers to provide a fake implementation of the full CRUD
/// API.
///
/// GET /users
/// GET /users/:id
/// POST /users
/// PUT /users/:id
/// DELETE /users/:id
///
/// Place it into a web server and test to ensure it meets your requirements.
///
async fn run_users_server() {
    let app = Router::<Arc<Mutex<UsersState>>>::new()
        .route("/users", get(get_users))
        .route("/users/:id", get(get_user))
        .route("/users", post(create_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .with_state(Arc::new(Mutex::new(UsersState::new())));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn get_users(state: State<Arc<Mutex<UsersState>>>) -> Json<Vec<User>> {
    Json(state.lock().await.get_users())
}

async fn get_user(Path(id): Path<u64>, state: State<Arc<Mutex<UsersState>>>) -> Result<Json<User>, MissingUserError> {
    state.lock().await.get_user(id).map(Json).ok_or(MissingUserError("".to_string()))
}

async fn create_user(state: State<Arc<Mutex<UsersState>>>, Json(proto_user): Json<ProtoUser>) -> Json<User> {
    Json(state.lock().await.create_user(proto_user))
}

async fn update_user(Path(id): Path<u64>, state: State<Arc<Mutex<UsersState>>>, Json(updates): Json<UserUpdate>) -> Result<Json<User>, MissingUserError> {
    state.lock().await.update_user(id, updates).map(Json).ok_or(MissingUserError("".to_string()))
}

async fn delete_user(Path(id): Path<u64>, state: State<Arc<Mutex<UsersState>>>) -> Result<Json<User>, MissingUserError> {
    state.lock().await.delete_user(id).map(Json).ok_or(MissingUserError("".to_string()))
}

struct UsersState {
    users: HashMap<u64, User>,
    next_id: u64,
}

impl UsersState {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            next_id: 0,
        }
    }

    fn get_user(&self, id: u64) -> Option<User> {
        self.users.get(&id).map(|u| u.clone())
    }

    fn get_users(&self) -> Vec<User> {
        self.users.values().map(|u| u.clone()).collect()
    }

    fn create_user(&mut self, proto_user: ProtoUser) -> User {
        let new_user = User { id: self.next_id, name: proto_user.name, email: proto_user.email };
        self.users.insert(self.next_id, new_user.clone());
        self.next_id += 1;
        new_user
    }

    fn update_user(&mut self, id: u64, update: UserUpdate) -> Option<User> {
        let current_user = self.users.get(&id);
        if current_user.is_none() {
            return Option::None
        }
        let current_user = current_user.unwrap();
        let new_user = User {
            id: current_user.id,
            name: update.name.unwrap_or_else(|| current_user.name.clone()),
            email: update.email.unwrap_or_else(|| current_user.email.clone()),
        };
        self.users.insert(id, new_user.clone());
        Option::Some(new_user)
    }

    fn delete_user(&mut self, id: u64) -> Option<User> {
        self.users.remove(&id)
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct ProtoUser {
    name: String,
    email: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct UserUpdate {
    name: Option<String>,
    email: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq, Eq)]
struct MissingUserError(String);

impl IntoResponse for MissingUserError {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("Content-Type", "application/json")
            .body(Body::from(format!("{{message:{}}}", serde_json::json!(&self.0))))
            .unwrap()
    }
}