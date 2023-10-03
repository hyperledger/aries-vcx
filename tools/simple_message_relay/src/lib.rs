use actix_web::dev::Server;
use actix_web::{
    get, post,
    web::{self, Bytes},
    App, HttpResponse, HttpServer, Responder,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::Mutex,
};

pub type UserMessage = Vec<u8>;
pub type UserMessages = VecDeque<UserMessage>;

#[macro_use]
extern crate log;

#[derive(Default)]
pub struct UserMessagesById {
    messages_by_user_id: Mutex<HashMap<String, UserMessages>>,
}

pub struct AppState {
    user_messages: UserMessagesById
}

impl AppState {
    pub fn new(user_messages: UserMessagesById) -> Self {
        AppState {
            user_messages
        }
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/pop_user_message/{user_id}")]
async fn pop_user_message(path: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    let user_id = path.into_inner();
    info!("Popping message, user_id: {user_id}");
    let mut messages_by_user_id = data.user_messages.messages_by_user_id.lock().unwrap();
    let messages_for_user = messages_by_user_id.get_mut(&user_id);

    let message_body = messages_for_user.and_then(|msgs| msgs.pop_front());

    if let Some(body) = message_body {
        return HttpResponse::Ok().body(body);
    } else {
        return HttpResponse::NoContent().into();
    }
}

#[post("/send_user_message/{user_id}")]
async fn send_user_message(path: web::Path<String>, body: Bytes, state: web::Data<AppState>) -> impl Responder {
    let user_id = path.into_inner();
    info!("Received message, user_id: {user_id}");

    let msg: UserMessage = body.to_vec();

    let mut messages_by_user_id = state.user_messages.messages_by_user_id.lock().unwrap();

    let messages_for_user = messages_by_user_id.get_mut(&user_id);

    if let Some(messages) = messages_for_user {
        messages.push_back(msg.clone());
    } else {
        messages_by_user_id.insert(user_id.clone(), vec![msg.clone()].into());
    }

    HttpResponse::Ok()
}

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok()
}

pub fn build_msg_relay(bind_address: &str, port: u16) -> std::io::Result<Server> {
    let app_state = web::Data::new(AppState::new(Default::default()));

    let server_future = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(pop_user_message)
            .service(send_user_message)
            .service(status)
    })
    .bind((bind_address, port))?
    .run();
    Ok(server_future)
}
