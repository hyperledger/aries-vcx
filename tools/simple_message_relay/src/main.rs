use env_logger;
use simple_message_relay::build_msg_relay;

#[macro_use]
extern crate log;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let binding_address = "0.0.0.0";
    let port = 8420;
    let (server, mut msg_receiver) = build_msg_relay(binding_address, port)?;

    tokio::task::spawn(async move {
        info!("Message receiver is active");
        while let Some(message) = msg_receiver.recv().await {
            println!("Received a message: {:?}", message);
        }
    });

    info!("Message relay listening on {binding_address}:{port}");
    server.await
}

#[cfg(test)]
mod tests {
    use actix_web::{
        body,
        http::StatusCode,
        test::{self, TestRequest},
        web, App,
    };
    use simple_message_relay::{pop_user_message, send_user_message, AppState, UserMessage};
    use tokio::sync::mpsc;

    fn pop_message_request(user_id: &str) -> TestRequest {
        test::TestRequest::get().uri(&format!("/pop_user_message/{user_id}"))
    }

    fn send_message_request(user_id: &str, msg: &'static str) -> TestRequest {
        test::TestRequest::post()
            .uri(&format!("/send_user_message/{user_id}"))
            .set_payload(msg)
    }

    // #[actix_web::test]
    async fn test_standard_user_flow() {
        // assemble service
        let (tx, mut rx) = mpsc::channel::<UserMessage>(100);
        let app_state = web::Data::new(AppState::new(Default::default(), tx));

        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .service(send_user_message)
                .service(pop_user_message),
        )
        .await;

        let user_id = "user1";

        // pop for unknown user == NO CONTENT
        let pop_response = test::call_service(&app, pop_message_request(user_id).to_request()).await;
        assert_eq!(pop_response.status(), StatusCode::NO_CONTENT);
        let body = pop_response.into_body();
        assert!(body::to_bytes(body).await.unwrap().is_empty());

        // post a message in
        let message = "hello world";
        let req = send_message_request(user_id, message).to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // pop for known user == OK and body
        let pop_response = test::call_service(&app, pop_message_request(user_id).to_request()).await;
        assert_eq!(pop_response.status(), StatusCode::OK);
        let body = pop_response.into_body();
        assert_eq!(body::to_bytes(body).await.unwrap(), message.as_bytes());

        // pop for no messages == NO CONTENT
        let pop_response = test::call_service(&app, pop_message_request(user_id).to_request()).await;
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
        assert_eq!(body::to_bytes(res.into_body()).await.unwrap(), message1.as_bytes());

        let res = test::call_service(&app, pop_message_request(user_id2).to_request()).await;
        assert_eq!(body::to_bytes(res.into_body()).await.unwrap(), message3.as_bytes());

        let res = test::call_service(&app, pop_message_request(user_id1).to_request()).await;
        assert_eq!(body::to_bytes(res.into_body()).await.unwrap(), message2.as_bytes());

        let res = test::call_service(&app, pop_message_request(user_id2).to_request()).await;
        assert_eq!(body::to_bytes(res.into_body()).await.unwrap(), message4.as_bytes());
    }
}
