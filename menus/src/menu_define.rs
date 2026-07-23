#[macro_export]
macro_rules! menus_define {
    [
        $($selection_name:ident, 
          $state_enum_name:ident($state_name:ident)
        );*
    ] => {
        /// The list of all currently implemented menus.
        #[derive(PartialEq, Debug)]
        pub enum Menu {
            $($selection_name,)*
            Unimplemented
        }

        /// A wrapper around the current states of every menu.
        enum MenuState {
            $($state_enum_name { 
                layout: &'static Layout, 
                state: $state_name 
            },)*
        }

        impl MenuState {
            fn layout(&self) -> &Layout {
                $(MenuState::$state_enum_name { layout, .. } => layout,)*
            }
        }

        impl fmt::Display for MenuState {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        MenuStates::$state_enum_name(_) => write!(f, stringify!($state_name)),
                    )*
                }
            }
        }

        impl From<&'static Layout> for MenuState {
            fn from(val: &'static Layout) -> MenuState {
                match val.defined_menu {
                    $(Menu::$selection_name => MenuState::$state_enum_name {
                        layout: val, state: $state_type::new()
                    },)*
                    MenuSelection::Unimplemented => unreachable!(),
                }
            }
        }

        impl MenuBehaviour for MenuStates {
            // Args = MenuSelection;
            async fn init(&mut self, spawner: Spawner, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
                match self {
                    $(
                    MenuStates::$state_enum_name(t) => t.init(spawner, io_handles).await,
                    )*
                }
            }

            async fn update(&mut self, io_handles: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
                match self {
                    $(
                    MenuStates::$state_enum_name(t) => t.update(io_handles).await,
                    )*
                }
            }
        }
    }
}
