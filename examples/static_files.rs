use poem::{route, service::Files, Server};

#[tokio::main]
async fn main() {
    let app = route().nest(
        "/",
        Files::new("./examples/static_files").show_files_listing(),
    );
    let server = Server::bind("127.0.0.1:3000").await.unwrap();
    server.run(app).await.unwrap();
}
