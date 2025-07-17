use actix_web::{App, HttpServer};
use check_selector::get_quote;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(get_quote)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
