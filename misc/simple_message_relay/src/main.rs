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
async fn pop_user_message(
    path: web::Path<String>,
    data: web::Data<UserMessages>,
) -> impl Responder {
    let user_id = path.into_inner();
    let mut messages_by_user_id = data.messages_by_user_id.lock().unwrap();
    let messages_for_user = messages_by_user_id.get_mut(&user_id);

    let message_body = messages_for_user.and_then(|msgs| msgs.pop_front());

    if let Some(body) = message_body {
        HttpResponse::Ok().body(body)
    } else {
        HttpResponse::NoContent().into()
    }
}

#[post("/send_user_message/{user_id}")]
async fn send_user_message(
    path: web::Path<String>,
    body: Bytes,
    data: web::Data<UserMessages>,
) -> impl Responder {
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
            .service(send_user_message)
            .service(status)
    })
    .bind(("0.0.0.0", 8420))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{
        body,
        http::StatusCode,
        test::{self, TestRequest},
        web, App,
    };

    use crate::{pop_user_message, send_user_message, UserMessages};

    fn pop_message_request(user_id: &str) -> TestRequest {
        test::TestRequest::get().uri(&format!("/pop_user_message/{user_id}"))
    }

    fn send_message_request(user_id: &str, msg: &'static str) -> TestRequest {
        test::TestRequest::post()
            .uri(&format!("/send_user_message/{user_id}"))
            .set_payload(msg)
    }

    #[actix_web::test]
    async fn test_standard_user_flow() {
        // assemble service
        let user_messages = web::Data::new(UserMessages::default());

        let app = test::init_service(
            App::new()
                .app_data(user_messages)
                .service(send_user_message)
                .service(pop_user_message),
        )
        .await;

        let user_id = "user1";

        // pop for unknown user == NO CONTENT
        let pop_response =
            test::call_service(&app, pop_message_request(user_id).to_request()).await;
        assert_eq!(pop_response.status(), StatusCode::NO_CONTENT);
        let body = pop_response.into_body();
        assert!(body::to_bytes(body).await.unwrap().is_empty());

        // post a message in
        let message = "hello world";
        let req = send_message_request(user_id, message).to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // pop for known user == OK and body
        let pop_response =
            test::call_service(&app, pop_message_request(user_id).to_request()).await;
        assert_eq!(pop_response.status(), StatusCode::OK);
        let body = pop_response.into_body();
        assert_eq!(body::to_bytes(body).await.unwrap(), message.as_bytes());

        // pop for no messages == NO CONTENT
        let pop_response =
            test::call_service(&app, pop_message_request(user_id).to_request()).await;
        assert_eq!(pop_response.status(), StatusCode::NO_CONTENT);
        let body = pop_response.into_body();
        assert!(body::to_bytes(body).await.unwrap().is_empty());
    }

    #[actix_web::test]
    async fn test_multi_message_multi_user_flow() {
        // assemble service
        let user_messages = web::Data::new(UserMessages::default());

        let app = test::init_service(
            App::new()
                .app_data(user_messages)
                .service(send_user_message)
                .service(pop_user_message),
        )
        .await;

        let user_id1 = "user1";
        let user_id2 = "user2";

        let message1 = "message1";
        let message2 = "message2";
        let message3 = "message3";
        let message4 = "message4";

        // populate
        test::call_service(&app, send_message_request(user_id1, message1).to_request()).await;
        test::call_service(&app, send_message_request(user_id1, message2).to_request()).await;
        test::call_service(&app, send_message_request(user_id2, message3).to_request()).await;
        test::call_service(&app, send_message_request(user_id2, message4).to_request()).await;

        // pop and check
        let res = test::call_service(&app, pop_message_request(user_id1).to_request()).await;
        assert_eq!(
            body::to_bytes(res.into_body()).await.unwrap(),
            message1.as_bytes()
        );

        let res = test::call_service(&app, pop_message_request(user_id2).to_request()).await;
        assert_eq!(
            body::to_bytes(res.into_body()).await.unwrap(),
            message3.as_bytes()
        );

        let res = test::call_service(&app, pop_message_request(user_id1).to_request()).await;
        assert_eq!(
            body::to_bytes(res.into_body()).await.unwrap(),
            message2.as_bytes()
        );

        let res = test::call_service(&app, pop_message_request(user_id2).to_request()).await;
        assert_eq!(
            body::to_bytes(res.into_body()).await.unwrap(),
            message4.as_bytes()
        );
    }
}
