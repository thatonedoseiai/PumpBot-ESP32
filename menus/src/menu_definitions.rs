use crate::{ComponentMenuDefinition, ComponentDefinition, GenericHandler, Menu, ComponentMenu, MenuHandler};
use crate::components::{ButtonDefinition, OptionSwitchDefinition};
use crate::handlers::{HandlerResult, OptionSwitchHandler, ButtonHandler};
use embedded_graphics::prelude::*;
use fontfile::FontSize;

pub const TITLE: ComponentMenuDefinition = ComponentMenuDefinition {
    components: &[
        ComponentDefinition::Button(
            &ButtonDefinition {
                pos: Point::new(10, 20),
                click: ButtonHandler::Generic(GenericHandler::Print("[first button clicked]")),
                left: ButtonHandler::Generic(GenericHandler::Print("[first button left]")),
                right: ButtonHandler::Generic(GenericHandler::Print("[first button right]")),
                font_size: FontSize::Sz7,
                text: "hello",
            }),
        ComponentDefinition::Button(
            &ButtonDefinition {
                pos: Point::new(10, 50),
                click: ButtonHandler::Generic(GenericHandler::Print("[second button clicked]")),
                left: ButtonHandler::Generic(GenericHandler::Print("[second button left]")),
                right: ButtonHandler::Generic(GenericHandler::Print("[second button right]")),
                font_size: FontSize::Sz7,
                text: "blue!",
            }),
        ComponentDefinition::Button(
            &ButtonDefinition {
                pos: Point::new(10, 80),
                click: ButtonHandler::Generic(GenericHandler::Print("[third button clicked]")),
                left: ButtonHandler::Generic(GenericHandler::Print("[third button left]")),
                right: ButtonHandler::Generic(GenericHandler::Print("[third button right]")),
                font_size: FontSize::Sz7,
                text: "click me!",
            }),
        ComponentDefinition::OptionSwitch(
            &OptionSwitchDefinition {
                pos: Point::new(20, 110),
                click: OptionSwitchHandler::ToggleFocus,
                left: OptionSwitchHandler::PrevElement,
                right: OptionSwitchHandler::NextElement,
                font_size: FontSize::Sz12,
                options: &["first", "second", "third"],
            }),
    ],
    left_btn: MenuHandler::Generic(GenericHandler::Print("title left btn")),
    right_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::Lang(5))))),
};

pub const LANG: ComponentMenuDefinition = ComponentMenuDefinition {
    components: &[
        ComponentDefinition::OptionSwitch(
            &OptionSwitchDefinition {
                pos: Point::new(20, 110),
                click: OptionSwitchHandler::PrintSelection,
                left: OptionSwitchHandler::PrevElement,
                right: OptionSwitchHandler::NextElement,
                font_size: FontSize::Sz12,
                options: &["hey", "you", "guuuys"],
            }),
    ],
    left_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Back)),
    right_btn: MenuHandler::Generic(GenericHandler::Print("lang right btn")),
};

