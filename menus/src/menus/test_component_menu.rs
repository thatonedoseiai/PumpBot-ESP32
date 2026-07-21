use crate::menus::component_combined_menu::{ComponentCombinedMenuButtonBehaviours, ComponentCombinedMenu, ComponentSignal, ComponentMenuError, Components};
use crate::components::button_component::{ButtonComponent};
use embedded_graphics::prelude::*;
use log::info;
use alloc::boxed::Box;
use alloc::vec;
use crate::{MenuSignal, MenuBehaviour, IOHandles};
use embassy_executor::Spawner;

struct TestComponentMenuButtonBehaviours;
impl ComponentCombinedMenuButtonBehaviours for TestComponentMenuButtonBehaviours {
    async fn left_behaviour(_: &mut IOHandles<'_>) -> Result<ComponentSignal, ComponentMenuError> {
        Ok(ComponentSignal::None)
    }

    async fn right_behaviour(_: &mut IOHandles<'_>) -> Result<ComponentSignal, ComponentMenuError> {
        Ok(ComponentSignal::None)
    }
}

pub struct TestComponentMenu(ComponentCombinedMenu<TestComponentMenuButtonBehaviours>);


impl TestComponentMenu {
    pub fn new() -> Self {
        let comps = vec![
            ButtonComponent::new(Point::new(10, 10), Size::new(50, 10), "first button", |h| Box::pin(async { 
                info!("[EVENT] first button clicked!");
                Ok(ComponentSignal::None)
            })).into(),
            ButtonComponent::new(Point::new(30, 30), Size::new(50, 10), "second button", |h| Box::pin(async { 
                info!("[EVENT] second button clicked!");
                Ok(ComponentSignal::None)
            })).into(),
        ];
        TestComponentMenu (ComponentCombinedMenu::new(
            comps,
        ))
    }
}

impl MenuBehaviour for TestComponentMenu {
    async fn init(&mut self, spawner: Spawner, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        self.0.init(spawner, io_handles).await
    }

    async fn update(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        self.0.update(io_handles).await
    }
}
