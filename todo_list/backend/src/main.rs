use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use actix_web_lab::middleware::from_fn;

use todo_list::{create_pool, handlers, middleware, migrate_up, Config};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = Config::from_env();
    let pg_pool = create_pool();
    migrate_up(&pg_pool).await;

    log::info!("Starting server at {}", config.address);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(from_fn(middleware::log_request))
            .app_data(web::Data::new(pg_pool.clone()))
            .service(handlers::list_todos)
            .service(handlers::get_todo)
            .service(handlers::create_todo)
            .service(handlers::update_todo)
            .service(handlers::delete_todo)
    })
    .bind(&config.address)?
    .run()
    .await
}
