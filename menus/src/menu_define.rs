#[macro_export]
macro_rules! menus_define {
    [
        $($selection_name:ident, 
          $state_enum_name:ident($state_name:ident)
        );*
    ] => {
        /// The list of all currently implemented menus.
        #[derive(PartialEq, Debug)]
        pub enum MenuSelection {
            $($selection_name,)*
            Unimplemented
        }

        /// A wrapper around the current states of every menu.
        enum MenuStates {
            $($state_enum_name($state_name),)*
        }

        impl fmt::Display for MenuStates {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(
                        MenuStates::$state_enum_name(_) => write!(f, stringify!($state_name)),
                    )*
                }
            }
        }

        impl From<MenuSelection> for MenuStates {
            fn from(val: MenuSelection) -> MenuStates {
                match val {
                    $(MenuSelection::$selection_name => MenuStates::$state_enum_name($state_name::new()),)*
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
