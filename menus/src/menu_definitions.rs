use crate::{ComponentMenuDefinition, ComponentDefinition, GenericHandler, Menu, ComponentMenu, MenuHandler};
use crate::components::{ButtonDefinition, OptionSwitchDefinition, OptionScrollerDefinition};
use crate::handlers::{HandlerResult, OptionSwitchHandler, ButtonHandler, OptionScrollerHandler, OptionsGenerator};
use embedded_graphics::prelude::*;
use fontfile::FontSize;
use global_settings::lang::{TEXT_LANGUAGE_NAME, Lang};

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
        // ComponentDefinition::OptionSwitch(
        //     &OptionSwitchDefinition {
        //         pos: Point::new(20, 110),
        //         click: OptionSwitchHandler::PrintSelection,
        //         left: OptionSwitchHandler::PrevElement,
        //         right: OptionSwitchHandler::NextElement,
        //         font_size: FontSize::Sz12,
        //         options: &["hey", "you", "guuuys"],
        //     }),
        ComponentDefinition::OptionScroller(
            &OptionScrollerDefinition{
                pos: Point::new(20, 20),
                click: OptionScrollerHandler::PrintSelection,
                left: OptionScrollerHandler::PrevOption,
                right: OptionScrollerHandler::NextOption,
                num_visible_elements: 5,
                width: 100,
                font_size: FontSize::Sz12,
                // options: OptionsGenerator::Const(&["first", "second", "third", "fourth", "secret fifth" , "last", "stupid", "you", "belly", "pick me!", "gwargh"]),
                options: OptionsGenerator::WifiGenerator,
            }),
    ],
    static_elements: &[],
    left_btn: MenuHandler::Generic(GenericHandler::Print("title left btn")),
    right_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::Lang(0))))),
};

pub const LANG: ComponentMenuDefinition = ComponentMenuDefinition {
    components: &[
        ComponentDefinition::OptionSwitch(
            &OptionSwitchDefinition {
                pos: Point::new(20, 110),
                click: OptionSwitchHandler::ToggleFocus,
                left: OptionSwitchHandler::PrevElement,
                right: OptionSwitchHandler::NextElement,
                font_size: FontSize::Sz12,
                options: &[
                    TEXT_LANGUAGE_NAME.const_index(Lang::En),
                    TEXT_LANGUAGE_NAME.const_index(Lang::Jp),
                    TEXT_LANGUAGE_NAME.const_index(Lang::Fr),
                    TEXT_LANGUAGE_NAME.const_index(Lang::Es),
                    TEXT_LANGUAGE_NAME.const_index(Lang::Pt),
                    TEXT_LANGUAGE_NAME.const_index(Lang::Zh),
                    TEXT_LANGUAGE_NAME.const_index(Lang::Cn),
                    TEXT_LANGUAGE_NAME.const_index(Lang::Ru),
                    TEXT_LANGUAGE_NAME.const_index(Lang::De),
                ],
            }),
    ],
    static_elements: &[],
    left_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Back)),
    right_btn: MenuHandler::Generic(GenericHandler::Print("lang right btn")),
};

