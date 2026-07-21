use core::fmt;
use crate::menus::component_combined_menu::MenuComponent;
use crate::components::button_component::{ButtonComponent, ButtonComponentError};
use fontfile::pb_font_renderer::PbFontRenderer;

#[macro_export]
macro_rules! components {
    [ $(<$($gens_enum:tt),*>;)? $($component:ident $(<$($gens:tt),*>)? -> $componenterror:ident),*$(,)? ] => {
        pub enum Components$(<$($gens_enum),*>)? {
            $($component($component $(<$($gens),*>)?),)*
        }

        #[derive(Debug)]
        pub enum ComponentsError {
            $($component($componenterror),)*
        }

        impl fmt::Display for ComponentsError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(ComponentsError::$component(t) => write!(f, "COMPONENT ERROR: [{}]", t),)*
                }
            }
        }

        impl core::error::Error for ComponentsError { }

        $(
        impl$(<$($gens:tt),*>)? From<$component$(<$($gens:tt),*>)?> for Components { // $(<$($gens_enum),*>)? {
            fn from(val: $component$(<$($gens:tt),*>)?) -> Self {
                Components::$component(val)
            }
        }
        )*

        impl MenuComponent for Components {
            type Error = ComponentsError;

            async fn draw<D: DrawTarget<Color = Rgb565>>(&self, f: PbFontRenderer, d: &mut D) -> Result<(), <D as DrawTarget>::Error> {
                match self {
                    $(Components::$component(t) => t.draw(f, d).await,)*
                }
            }

            async fn position(&self) -> (u8, u8) {
                match self {
                    $(Components::$component(t) => t.position().await,)*
                }
            }

            async fn highlight(&self) -> &Self {
                match self {
                    $(Components::$component(t) => { t.highlight().await; &self },)*
                }
            }

            async fn unhighlight(&self) -> &Self {
                match self {
                    $(Components::$component(t) => { t.unhighlight().await; &self },)*
                }
            }

            async fn click(&self, h: &mut IOHandles<'_>) -> Result<ComponentSignal, Self::Error> {
                match self {
                    $(Components::$component(t) => t.click(h).await.map_err(|e| ComponentsError::$component(e)),)*
                }
            }

            async fn right(&self, h: &mut IOHandles<'_>) -> &Self {
                match self {
                    $(Components::$component(t) => { t.right(h).await; &self },)*
                }
            }

            async fn left(&self, h: &mut IOHandles<'_>) -> &Self {
                match self {
                    $(Components::$component(t) => { t.left(h).await; &self },)*
                }
            }

            async fn receive_signal(&self, c: ComponentSignal) -> () {
                match self {
                    $(Components::$component(t) => t.receive_signal(c).await,)*
                }
            }
        }
    }
}