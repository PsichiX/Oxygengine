use crate::{
    components::{History, Typing},
    consts::{HOST_URL, MSG_TEXT, VERSION},
    resources::{history_res::HistoryRes, text_input_res::TextInputRes},
};
use bincode::{deserialize, serialize};
use oxygengine::prelude::*;

#[derive(Default)]
pub struct MainState {
    client: Option<ClientID>,
}

impl State for MainState {
    fn on_enter(&mut self, world: &mut World) {
        info!("* CLIENT CONNECTING");

        {
            let history = &mut world.write_resource::<HistoryRes>();
            history.lines_limit = 25;
        }

        world
            .create_entity()
            .with(CompositeCamera::with_scaling_target(
                CompositeScalingMode::CenterAspect,
                CompositeScalingTarget::Height,
            ))
            .with(CompositeTransform::scale(800.0.into()))
            .with(Name("main-camera".into()))
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(
                Text {
                    color: Color::yellow(),
                    font: "Verdana".into(),
                    align: TextAlign::Center,
                    text: ">".into(),
                    position: 0.0.into(),
                    size: 24.0,
                }
                .into(),
            ))
            .with(CompositeTransform::translation([0.0, -300.0].into()))
            .with(Typing)
            .build();

        world
            .create_entity()
            .with(CompositeRenderable(
                Text {
                    color: Color::white(),
                    font: "Verdana".into(),
                    align: TextAlign::Center,
                    text: "".into(),
                    position: 0.0.into(),
                    size: 20.0,
                }
                .into(),
            ))
            .with(CompositeTransform::translation([0.0, -270.0].into()))
            .with(History)
            .build();
    }

    fn on_exit(&mut self, world: &mut World) {
        info!("* CLIENT STOP");
        let network = &mut world.write_resource::<Network<WebClient>>();
        if let Some(client) = &self.client {
            network.close_client(*client);
        }
    }

    fn on_process(&mut self, world: &mut World) -> StateChange {
        let network = &mut world.write_resource::<Network<WebClient>>();
        if self.client.is_none() {
            self.client = network.open_client(HOST_URL);
            info!("* CLIENT START");
        }
        if let Some(id) = &self.client {
            if let Some(client) = network.client_mut(*id) {
                let history = &mut world.write_resource::<HistoryRes>();
                let text_message = MessageID::new(MSG_TEXT, VERSION);
                let messages = client.read_all();
                for (mid, data) in messages {
                    if mid == text_message {
                        if let Ok(msg) = deserialize::<String>(&data) {
                            history.write(&msg);
                            info!("* MESSAGE: {:?}", msg);
                        }
                    }
                }

                let input = &mut world.write_resource::<TextInputRes>();
                if let Some(values) = input.read_values() {
                    for value in values {
                        let data = serialize(&value).unwrap();
                        if client.send(text_message, &data).is_some() {
                            info!("* SENT: {:?}", value);
                        }
                    }
                }
            }
        }
        StateChange::None
    }
}
