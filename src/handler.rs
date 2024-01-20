
use uuid::Uuid;
use crate::model::Todo;

use crate::{
    model::{AppState, QueryOptions},
    response::{GenericResponse, SingleTodoResponse, TodoData, TodoListResponse},
};

use actix_web::{delete, get, post, web, HttpResponse, Responder};
use chrono::prelude::*;

#[get("/book")]
async fn book_handler() -> impl Responder {
    const MESSAGE: &str = "Build Simple CRUD API with Rust and Actix Web";

    let response_json = &GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };
    HttpResponse::Ok().json(response_json)
}

#[get("/todos")]
pub async fn todos_list_handler(
    opts: web::Query<QueryOptions>,
    data: web::Data<AppState>,
) -> impl Responder {
    let todos = data.todo_db.lock().unwrap();

    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;
    
    let todos: Vec<Todo> = todos.clone().into_iter().skip(offset).take(limit).collect();
    

    let json_response = TodoListResponse {
        status: "success".to_string(),
        results: todos.len(),
        todos,
    };
    HttpResponse::Ok().json(json_response)
}

#[post("/todos")]
async fn create_todo_handler(
    mut body: web::Json<Todo>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut vec = data.todo_db.lock().unwrap();

    let todo = vec.iter().find(|todo| todo.title == body.title);

    if todo.is_some(){
        let err_response = GenericResponse{
            status: "fail".to_string(),
            message: format!("Todo with title: '{}' allready exist",body.title),
        };
        return HttpResponse::Conflict().json(err_response);
    }

    let uuid_id = Uuid::new_v4();
    let datetime = Utc::now();

    body.id = Some(uuid_id.to_string());
    body.completed = Some(false);
    body.createdAt = Some(datetime);
    body.updatedAt = Some(datetime);

    let todo = body.clone();

    vec.push(body.to_owned());

    let json_response = SingleTodoResponse{
        status: "success".to_string(),
        data: TodoData { todo },
    };

    HttpResponse::Ok().json(json_response)
}

// get id
// edit

#[delete("/todos/{id}")]
async fn delete_todo_handler(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder{
    let mut vec = data.todo_db.lock().unwrap();

    let id = path.into_inner();
    let todo = vec.iter_mut().find(|todo| todo.id == Some(id.to_owned()));

    if todo.is_none(){
        let err_response = GenericResponse{
            status: "fail".to_string(),
            message: format!("Todo with ID: {} not found", id),
        };
        return HttpResponse::NotFound().json(err_response)
    }

    vec.retain(|todo| todo.id != Some(id.to_owned()));
    HttpResponse::NoContent().finish()
}

pub fn config(conf: &mut web::ServiceConfig){
    let scope = web::scope("/api")
        .service(book_handler)
        .service(todos_list_handler)
        .service(create_todo_handler)
        .service(delete_todo_handler);

    conf.service(scope);
}