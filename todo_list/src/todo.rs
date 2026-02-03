use tokio_postgres::{Error, GenericClient, Row};
use validator::Validate;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Todo{
    pub id: i32,
    pub task: String,
    pub is_finished: bool,
}

impl From<Row> for Todo {
    fn from(value: Row) -> Self {
        Todo {
            id: value.get(0),
            task: value.get(1),
            is_finished: value.get(2),
        }
    }
}

#[derive(Debug, serde::Deserialize, Validate)]
pub struct CreateTodo {
    #[validate(length(min = 1, max = 500, message = "Task must be between 1 and 500 characters"))]
    pub task: String,
    pub is_finished: Option<bool>,
}

#[derive(Debug, serde::Deserialize, Validate)]
pub struct UpdateTodo {
    #[validate(length(min = 1, max = 500, message = "Task must be between 1 and 500 characters"))]
    pub task: Option<String>,
    pub is_finished: Option<bool>,
}

impl Todo {
    pub async fn all<C: GenericClient>(client: &C) -> Result<Vec<Todo>, Error> {
        let rows = client
            .query("SELECT id, task, is_finished FROM todo", &[])
            .await?;
        Ok(rows.into_iter().map(Todo::from).collect())
    }

    pub async fn get_by_id<C: GenericClient>(client: &C, id: i32) -> Result<Option<Todo>, Error> {
        let row = client
            .query_opt("SELECT id, task, is_finished FROM todo WHERE id = $1", &[&id])
            .await?;
        Ok(row.map(Todo::from))
    }

    pub async fn create<C: GenericClient>(client: &C, data: CreateTodo) -> Result<Todo, Error> {
        let is_finished = data.is_finished.unwrap_or(false);
        let row = client
            .query_one(
                "INSERT INTO todo (task, is_finished) VALUES ($1, $2) RETURNING id, task, is_finished",
                &[&data.task, &is_finished],
            )
            .await?;
        Ok(Todo::from(row))
    }

    pub async fn update<C: GenericClient>(client: &C, id: i32, data: UpdateTodo) -> Result<Option<Todo>, Error> {
        // Сначала получаем текущую задачу
        let current = match Self::get_by_id(client, id).await? {
            Some(todo) => todo,
            None => return Ok(None),
        };

        let task = data.task.unwrap_or(current.task);
        let is_finished = data.is_finished.unwrap_or(current.is_finished);

        let row = client
            .query_one(
                "UPDATE todo SET task = $1, is_finished = $2 WHERE id = $3 RETURNING id, task, is_finished",
                &[&task, &is_finished, &id],
            )
            .await?;
        Ok(Some(Todo::from(row)))
    }

    pub async fn delete<C: GenericClient>(client: &C, id: i32) -> Result<bool, Error> {
        let rows_affected = client
            .execute("DELETE FROM todo WHERE id = $1", &[&id])
            .await?;
        Ok(rows_affected > 0)
    }
}