use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

use actix_web::{
    get, post,
    web::{self, Bytes},
    App, HttpResponse, HttpServer, Responder,
};

#[derive(Default)]
struct UserMessages {
    messages_by_user_id: Mutex<HashMap<String, VecDeque<Vec<u8>>>>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/pop_user_message/{user_id}")]
async fn pop_user_message(path: web::Path<String>, data: web::Data<UserMessages>) -> impl Responder {
    let user_id = path.into_inner();
    let mut messages_by_user_id = data.messages_by_user_id.lock().unwrap();
    let messages_for_user = messages_by_user_id.get_mut(&user_id);

    let message_body = messages_for_user.and_then(|msgs| msgs.pop_front());

    if let Some(body) = message_body {
        return HttpResponse::Ok().body(body);
    } else {
        return HttpResponse::NoContent().into();
    }
}

#[post("/receive_user_message/{user_id}")]
async fn receive_user_message(path: web::Path<String>, body: Bytes, data: web::Data<UserMessages>) -> impl Responder {
    let user_id = path.into_inner();

    let body = body.to_vec();

    let mut messages_by_user_id = data.messages_by_user_id.lock().unwrap();

    let messages_for_user = messages_by_user_id.get_mut(&user_id);

    if let Some(messages) = messages_for_user {
        messages.push_back(body);
    } else {
        messages_by_user_id.insert(user_id, vec![body].into());
    }

    HttpResponse::Ok()
}

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let user_messages = web::Data::new(UserMessages::default());

    HttpServer::new(move || {
        App::new()
            .app_data(user_messages.clone())
            .service(pop_user_message)
            .service(receive_user_message)
            .service(status)
    })
    .bind(("127.0.0.1", 8420))?
    .run()
    .await
}
