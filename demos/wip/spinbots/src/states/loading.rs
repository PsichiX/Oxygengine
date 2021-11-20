use crate::states::battle::BattleState;
use oxygengine::prelude::*;

const PUBLIC_PARTS_URL: &str =
    "https://raw.githubusercontent.com/PsichiX/Oxygengine/master/demos/spinbots/parts-registry/";

pub enum LoadingState {
    None,
    AssetPack(AssetPackPreloader),
    PublicParts(Option<AssetsPreloader>),
    PrivateParts(Option<AssetsPreloader>),
}

impl Default for LoadingState {
    fn default() -> Self {
        Self::None
    }
}

impl State for LoadingState {
    fn on_process(&mut self, universe: &mut Universe) -> StateChange {
        let mut assets = universe.expect_resource_mut::<AssetsDatabase>();
        match self {
            Self::None => {
                *self = Self::AssetPack(
                    AssetPackPreloader::new("assets.pack", &mut assets, vec!["set://assets.txt"])
                        .expect("could not create asset pack preloader"),
                );
            }
            Self::AssetPack(preloader) => {
                if preloader.process(&mut assets).unwrap() {
                    let params = universe.expect_resource::<AppParams>();
                    if params.has("nopublic") {
                        *self = Self::PublicParts(None);
                    } else {
                        *self = Self::PublicParts(Some(AssetsPreloader::new(
                            WebFetchEngine::new(PUBLIC_PARTS_URL),
                            vec!["parts://parts.yaml"],
                        )));
                    }
                }
            }
            Self::PublicParts(preloader) => {
                let passing = preloader
                    .as_mut()
                    .map(|preloader| preloader.process(&mut assets).unwrap())
                    .unwrap_or(true);
                if passing {
                    let params = universe.expect_resource::<AppParams>();
                    if let Some(port) = params.get("parts") {
                        *self = Self::PrivateParts(Some(AssetsPreloader::new(
                            WebFetchEngine::new(&format!("http://localhost:{}", port)),
                            vec!["parts://parts.yaml"],
                        )));
                    } else {
                        *self = Self::PrivateParts(None);
                    }
                }
            }
            Self::PrivateParts(preloader) => {
                let passing = preloader
                    .as_mut()
                    .map(|preloader| preloader.process(&mut assets).unwrap())
                    .unwrap_or(true);
                if passing {
                    return StateChange::Swap(Box::new(BattleState::<2>::default()));
                }
            }
        }
        StateChange::None
    }
}
