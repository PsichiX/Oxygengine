use crate::resources::board::*;
use oxygengine_core::{
    prefab::{Prefab, PrefabComponent},
    Ignite, Scalar,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Ignite, Debug, Clone, Serialize, Deserialize)]
pub enum BoardAvatarAction {
    Move {
        duration: Scalar,
        x: isize,
        y: isize,
    },
    MoveStep {
        duration: Scalar,
        direction: BoardDirection,
    },
    Teleport {
        duration: Scalar,
        location: Location,
    },
}

impl BoardAvatarAction {
    pub fn duration(&self) -> Scalar {
        match self {
            Self::Move { duration, .. } => *duration,
            Self::MoveStep { duration, .. } => *duration,
            Self::Teleport { duration, .. } => *duration,
        }
    }

    pub fn progress(&self, time: Scalar) -> Scalar {
        let duration = self.duration();
        if duration > 0.0 {
            (time / duration).max(0.0).min(1.0)
        } else {
            1.0
        }
    }
}

#[derive(Ignite, Debug, Default, Clone, Serialize, Deserialize)]
pub struct BoardAvatar {
    pub(crate) location: Location,
    #[serde(default)]
    pub(crate) direction: Option<BoardDirection>,
    #[serde(default)]
    pub(crate) actions_queue: VecDeque<BoardAvatarAction>,
    /// (action, time, completed)?
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) active_action: Option<(BoardAvatarAction, Scalar, bool)>,
    #[serde(skip)]
    #[ignite(ignore)]
    pub(crate) token: Option<BoardToken>,
}

impl BoardAvatar {
    pub fn has_token(&self) -> bool {
        self.token.is_some()
    }

    pub fn token(&self) -> Option<BoardToken> {
        self.token
    }

    pub fn location(&self) -> Location {
        self.location
    }

    pub fn direction(&self) -> Option<BoardDirection> {
        self.direction
    }

    pub fn actions_queue(&self) -> impl Iterator<Item = BoardAvatarAction> + '_ {
        self.actions_queue.iter().cloned()
    }

    /// (action, time, completed)?
    pub fn active_action(&self) -> Option<(&BoardAvatarAction, Scalar, bool)> {
        self.active_action
            .as_ref()
            .map(|(action, time, completed)| (action, *time, *completed))
    }

    pub fn in_progress(&self) -> bool {
        self.active_action.is_some()
    }

    pub fn clear_actions_queue(&mut self) {
        self.actions_queue.clear();
    }

    pub fn perform_single_action(&mut self, action: BoardAvatarAction) {
        self.clear_actions_queue();
        self.enqueue_action(action);
    }

    pub fn enqueue_action(&mut self, action: BoardAvatarAction) {
        self.actions_queue.push_back(action);
    }
}

impl Prefab for BoardAvatar {}

impl PrefabComponent for BoardAvatar {}
