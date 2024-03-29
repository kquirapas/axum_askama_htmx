use anyhow::Context;
use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    name: &'a str,
    counter: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_askama=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // let html_string = index.render().unwrap();

    info!("initializing router...");
    // build our application with a single route
    let assets_path = std::env::current_dir().unwrap();
    let port = 3000;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    let api_router = Router::new().route("/hello", get(hello_from_the_server));

    let app = Router::new()
        .nest("/api", api_router)
        .route("/", get(index))
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );

    // run our app with hyper, listening globally on port 3000
    info!("router initialized, now listening on port {}", port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .await
        .context("Error while starting server")?;

    Ok(())
}

async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        name: "world",
        counter: 0,
    };
    HtmlTemplate(template)
}

async fn hello_from_the_server() -> &'static str {
    "Hello!"
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}
