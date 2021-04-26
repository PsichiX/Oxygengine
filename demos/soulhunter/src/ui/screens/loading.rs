use oxygengine::user_interface::raui::core::prelude::*;

const TEXTS: [&str; 4] = ["Loading", "Loading.", "Loading..", "Loading..."];

fn make_animation() -> Animation {
    Animation::Looped(Box::new(Animation::Value(AnimatedValue {
        name: "phase".to_owned(),
        duration: 2.0,
    })))
}

fn use_loading(context: &mut WidgetContext) {
    context.life_cycle.mount(|context| {
        drop(context.animator.change("", Some(make_animation())));
    });
}

#[pre_hooks(use_loading)]
pub fn loading(mut context: WidgetContext) -> WidgetNode {
    let WidgetContext {
        key,
        animator,
        ..
    } = context;

    let value = animator.value_progress_or_zero("", "phase");
    let phase = (value * std::f32::consts::PI * 2.0).sin();
    let alpha = 1.0 + 0.2 * phase;
    let index = ((value * 4.0) as usize + 3) % 4;

    let text_props = Props::new(ContentBoxItemLayout {
        anchors: Rect {
            left: 0.0,
            right: 1.0,
            top: 0.5,
            bottom: 0.5,
        },
        margin: Rect {
            left: 0.0,
            right: 0.0,
            top: -30.0,
            bottom: -30.0,
        },
        ..Default::default()
    })
    .with(TextBoxProps {
        text: TEXTS[index].to_owned(),
        alignment: TextBoxAlignment::Center,
        font: TextBoxFont {
            name: "fonts/aquatico.json".to_owned(),
            size: 60.0,
        },
        color: Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: alpha,
        },
        ..Default::default()
    });

    widget! {
        (#{key} content_box [
            (#{"text"} text_box: {text_props})
        ])
    }
}
