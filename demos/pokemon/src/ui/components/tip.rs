use oxygengine::user_interface::raui::{core::prelude::*, material::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(PropsData, Default, Debug, Clone, Serialize, Deserialize)]
pub struct TipProps {
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub label: String,
}

pub fn tip(context: WidgetContext) -> WidgetNode {
    let WidgetContext { key, props, .. } = context;

    let tip_props = props.read_cloned_or_default::<TipProps>();

    let action_props = TextPaperProps {
        text: tip_props.action,
        variant: "roboto5".to_owned(),
        use_main_color: true,
        horizontal_align_override: Some(TextBoxHorizontalAlign::Right),
        ..Default::default()
    };

    let label_props = TextPaperProps {
        text: tip_props.label,
        variant: "5".to_owned(),
        use_main_color: true,
        horizontal_align_override: Some(TextBoxHorizontalAlign::Left),
        ..Default::default()
    };

    widget! {
        (#{key} horizontal_box: {props.clone()} [
            (#{"action"} text_paper: {action_props})
            (#{"label"} text_paper: {label_props})
        ])
    }
}

pub fn confirm_tip(context: WidgetContext) -> WidgetNode {
    widget! {
        (#{context.key} tip: {TipProps { action: "ENTER: ".to_owned(), label: "Confirm".to_owned() }})
    }
}

pub fn save_tip(context: WidgetContext) -> WidgetNode {
    widget! {
        (#{context.key} tip: {TipProps { action: "F5: ".to_owned(), label: "Save".to_owned() }})
    }
}

pub fn quit_tip(context: WidgetContext) -> WidgetNode {
    widget! {
        (#{context.key} tip: {TipProps { action: "ESC: ".to_owned(), label: "Quit".to_owned() }})
    }
}
