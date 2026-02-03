use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer};
use actix_cors::Cors;
use actix_web_lab::middleware::from_fn;
use deadpool_postgres::Pool;
use validator::Validate;
mod todo;
mod postgres;

// Middleware для логирования запросов
async fn log_request(
    req: actix_web::dev::ServiceRequest,
    next: actix_web_lab::middleware::Next<impl actix_web::body::MessageBody>,
) -> Result<actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>, actix_web::Error> {
    log::info!("{} {}", req.method(), req.path());
    next.call(req).await
}

fn address() -> String {
    std::env::var("ADDRESS").unwrap_or_else(|_| "127.0.0.1:8000".into())
}

// GET /todos - получить все задачи
#[get("/todos")]
async fn list_todos(pool: web::Data<Pool>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match todo::Todo::all(&**client).await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => {
            log::debug!("unable to fetch todos: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to fetch todos");
        }
    }
}

// GET /todos/{id} - получить задачу по ID
#[get("/todos/{id}")]
async fn get_todo(pool: web::Data<Pool>, id: web::Path<i32>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match todo::Todo::get_by_id(&**client, *id).await {
        Ok(Some(todo)) => HttpResponse::Ok().json(todo),
        Ok(None) => HttpResponse::NotFound().json("todo not found"),
        Err(err) => {
            log::debug!("unable to fetch todo: {:?}", err);
            HttpResponse::InternalServerError().json("unable to fetch todo")
        }
    }
}

// POST /todos - создать новую задачу
#[post("/todos")]
async fn create_todo(
    pool: web::Data<Pool>,
    data: web::Json<todo::CreateTodo>,
) -> HttpResponse {
    // Валидация входных данных
    if let Err(errors) = data.validate() {
        log::debug!("validation failed: {:?}", errors);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match todo::Todo::create(&**client, data.into_inner()).await {
        Ok(todo) => HttpResponse::Created().json(todo),
        Err(err) => {
            log::debug!("unable to create todo: {:?}", err);
            HttpResponse::InternalServerError().json("unable to create todo")
        }
    }
}

// PUT /todos/{id} - обновить задачу
#[put("/todos/{id}")]
async fn update_todo(
    pool: web::Data<Pool>,
    id: web::Path<i32>,
    data: web::Json<todo::UpdateTodo>,
) -> HttpResponse {
    // Валидация входных данных
    if let Err(errors) = data.validate() {
        log::debug!("validation failed: {:?}", errors);
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "details": errors.to_string()
        }));
    }

    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match todo::Todo::update(&**client, *id, data.into_inner()).await {
        Ok(Some(todo)) => HttpResponse::Ok().json(todo),
        Ok(None) => HttpResponse::NotFound().json("todo not found"),
        Err(err) => {
            log::debug!("unable to update todo: {:?}", err);
            HttpResponse::InternalServerError().json("unable to update todo")
        }
    }
}

// DELETE /todos/{id} - удалить задачу
#[delete("/todos/{id}")]
async fn delete_todo(pool: web::Data<Pool>, id: web::Path<i32>) -> HttpResponse {
    let client = match pool.get().await {
        Ok(client) => client,
        Err(err) => {
            log::debug!("unable to get postgres client: {:?}", err);
            return HttpResponse::InternalServerError().json("unable to get postgres client");
        }
    };
    match todo::Todo::delete(&**client, *id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json("todo not found"),
        Err(err) => {
            log::debug!("unable to delete todo: {:?}", err);
            HttpResponse::InternalServerError().json("unable to delete todo")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let pg_pool = postgres::create_pool();
    postgres::migrate_up(&pg_pool).await;

    let address = address();
    HttpServer::new(move || {
        // Настройка CORS для фронтенда
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(from_fn(log_request))
            .app_data(web::Data::new(pg_pool.clone()))
            .service(list_todos)
            .service(get_todo)
            .service(create_todo)
            .service(update_todo)
            .service(delete_todo)
    })
    .bind(&address)?
    .run()
    .await
}
