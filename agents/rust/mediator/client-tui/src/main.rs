/// Aries Agent TUI
mod tui;
#[tokio::main]
async fn main() {
    use mediator::{
        aries_agent::AgentBuilder,
        utils::binary_utils::{load_dot_env, setup_logging},
    };

    load_dot_env();
    setup_logging();
    log::info!("TUI initializing!");
    let agent = AgentBuilder::new_demo_agent().await.unwrap();
    tui::init_tui(agent).await;
}

// fn main() {
//     print!(
//         "This is a placeholder binary. Please enable \"client_tui\" feature to to build the \
//          functional client_tui binary."
//     )
// }
