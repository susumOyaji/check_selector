use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::Files;
use check_selector::{get_default_quotes, get_quote_by_code};

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("サーバーを http://127.0.0.1:8080 で起動しています");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .service(get_quote_by_code)
            .service(get_default_quotes)
            .service(Files::new("/", ".").index_file("index.html")) // 静的ファイルを提供
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
