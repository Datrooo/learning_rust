use actix_web::{delete, get, post, put, web, HttpResponse};
use deadpool_postgres::Pool;
use validator::Validate;

use crate::error::AppError;
use crate::models::{CreateTodo, Todo, UpdateTodo};

async fn get_client(pool: &Pool) -> Result<deadpool_postgres::Client, AppError> {
    pool.get().await.map_err(|e| {
        log::debug!("unable to get postgres client: {:?}", e);
        AppError::from(e)
    })
}

#[get("/todos")]
pub async fn list_todos(pool: web::Data<Pool>) -> Result<HttpResponse, AppError> {
    let client = get_client(&pool).await?;
    let todos = Todo::all(& **client).await?;
    Ok(HttpResponse::Ok().json(todos))
}

#[get("/todos/{id}")]
pub async fn get_todo(pool: web::Data<Pool>, id: web::Path<i32>) -> Result<HttpResponse, AppError> {
    let client = get_client(&pool).await?;
    match Todo::get_by_id(&**client, *id).await? {
        Some(todo) => Ok(HttpResponse::Ok().json(todo)),
        None => Err(AppError::NotFound("Todo not found".to_string())),
    }
}

#[post("/todos")]
pub async fn create_todo(
    pool: web::Data<Pool>,
    data: web::Json<CreateTodo>,
) -> Result<HttpResponse, AppError> {
    data.validate().map_err(|e| {
        log::debug!("validation failed: {:?}", e);
        AppError::ValidationError(e.to_string())
    })?;

    let client = get_client(&pool).await?;
    let todo = Todo::create(&**client, data.into_inner()).await?;
    Ok(HttpResponse::Created().json(todo))
}

#[put("/todos/{id}")]
pub async fn update_todo(
    pool: web::Data<Pool>,
    id: web::Path<i32>,
    data: web::Json<UpdateTodo>,
) -> Result<HttpResponse, AppError> {
    data.validate().map_err(|e| {
        log::debug!("validation failed: {:?}", e);
        AppError::ValidationError(e.to_string())
    })?;

    let client = get_client(&pool).await?;
    match Todo::update(&**client, *id, data.into_inner()).await? {
        Some(todo) => Ok(HttpResponse::Ok().json(todo)),
        None => Err(AppError::NotFound("Todo not found".to_string())),
    }
}

#[delete("/todos/{id}")]
pub async fn delete_todo(pool: web::Data<Pool>, id: web::Path<i32>) -> Result<HttpResponse, AppError> {
    let client = get_client(&pool).await?;
    match Todo::delete(&**client, *id).await? {
        true => Ok(HttpResponse::NoContent().finish()),
        false => Err(AppError::NotFound("Todo not found".to_string())),
    }
}
