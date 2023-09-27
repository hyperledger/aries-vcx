use std::sync::Arc;

use aries_vcx_core::wallet::{base_wallet::BaseWallet, indy::IndySdkWallet};
use axum::{extract::State, Json};
use cursive::{
    direction::Orientation,
    event::Key,
    view::{Nameable, SizeConstraint},
    views::{
        Dialog, DummyView, LinearLayout, Panel, ResizedView, ScrollView, SelectView, TextArea,
        TextView,
    },
    Cursive, CursiveExt, View,
};
use futures::executor::block_on;
use log::info;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;

use crate::{agent::Agent, routes::client::handle_register};

pub async fn init_tui(agent: Agent<impl BaseWallet + 'static>) {
    let mut cursive = Cursive::new();
    cursive.add_global_callback(Key::Esc, |s| s.quit());
    cursive.set_user_data(Arc::new(agent));

    let mut main = LinearLayout::horizontal().with_name("main");
    let endpoint_selector = endpoints_ui();
    main.get_mut().add_child(endpoint_selector);
    cursive.add_layer(main);
    cursive.run()
}

pub fn endpoints_ui() -> Panel<LinearLayout> {
    let mut endpoint_selector = SelectView::new();
    // Set available endpoints
    endpoint_selector.add_item_str("/client/register");

    endpoint_selector.set_on_submit(|s, endpoint: &str| {
        // Match ui generators for available endpoints
        let view = match endpoint {
            "/client/register" => client_register_ui(),
            _ => dummy_ui(),
        };
        // Replace previously exposed ui
        s.find_name::<LinearLayout>("main").unwrap().remove_child(1);
        s.find_name::<LinearLayout>("main")
            .unwrap()
            .insert_child(1, view);
    });

    make_standard(endpoint_selector, Orientation::Vertical).title("Select endpoint")
}

pub fn client_register_ui() -> Panel<LinearLayout> {
    let input = TextArea::new().with_name("oob_text_area");
    let input = ResizedView::new(
        SizeConstraint::AtLeast(20),
        SizeConstraint::AtLeast(5),
        input,
    );
    let input = Dialog::around(input)
        .button("Clear", |s| {
            s.find_name::<TextArea>("oob_text_area")
                .unwrap()
                .set_content("");
        })
        .button("Connect", client_register_connect_cb)
        .title("OOB Invite");
    let input = Panel::new(input);

    let output = ScrollView::new(ResizedView::new(
        SizeConstraint::AtLeast(20),
        SizeConstraint::Free,
        TextView::new("").with_name("client_register_result"),
    ));
    let output = Panel::new(output).title("Result");

    let ui = LinearLayout::horizontal().child(input).child(output);

    make_standard(ui, Orientation::Horizontal).title("Register client using Out Of Band Invitation")
}

pub fn client_register_connect_cb(s: &mut Cursive) {
    let oob_text_area = s.find_name::<TextArea>("oob_text_area").unwrap();
    let mut output = s.find_name::<TextView>("client_register_result").unwrap();
    let oob_text = oob_text_area.get_content();
    info!("{:#?}", oob_text);

    let oob_invite = match serde_json::from_str::<OOBInvitation>(oob_text) {
        Ok(oob_invite) => oob_invite,
        Err(err) => {
            output.set_content(format!("{:?}", err));
            return;
        }
    };
    info!("{:#?}", oob_invite);
    s.with_user_data(|arc_agent: &mut Arc<Agent<IndySdkWallet>>| {
        output.set_content(format!("{:#?}", oob_invite));
        match block_on(handle_register(
            State(arc_agent.to_owned()),
            Json(oob_invite),
        )) {
            Ok(Json(res_json)) => {
                output.set_content(serde_json::to_string_pretty(&res_json).unwrap())
            }
            Err(err) => output.set_content(err),
        };
    });
}

fn dummy_ui() -> Panel<LinearLayout> {
    let ui = DummyView;
    make_standard(ui, Orientation::Horizontal)
}

fn make_standard(view: impl View, orientation: Orientation) -> Panel<LinearLayout> {
    Panel::new(LinearLayout::new(orientation).child(view))
}
