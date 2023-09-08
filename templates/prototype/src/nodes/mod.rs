pub mod character;
pub mod enemy;
pub mod gui;
pub mod thunder;

use crate::nodes::{
    character::CharacterNode, enemy::EnemyNode, gui::GuiNode, thunder::ThunderNode,
};
use oxygengine::prelude::intuicio::prelude::*;

pub fn install(registry: &mut Registry) {
    GuiNode::install(registry);
    CharacterNode::install(registry);
    ThunderNode::install(registry);
    EnemyNode::install(registry);
}
