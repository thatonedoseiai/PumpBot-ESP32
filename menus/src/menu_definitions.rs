use crate::{ComponentMenuDefinition, ComponentDefinition, GenericHandler, Menu, ComponentMenu, MenuHandler, CustomMenu};
use crate::components::{ButtonDefinition, OptionSwitchDefinition, OptionScrollerDefinition, TextBoxDefinition, ValueSelectorDefinition, ColorSelectorDefinition};
use crate::handlers::{HandlerResult, OptionSwitchHandler, ButtonHandler, OptionScrollerHandler, OptionsGenerator, MenuInternalStateAction, TextGetterSetter, TextSubmitHandler, InitialValueGenerator, ValueSelectorHandler, ColorGetter, ColorSubmitHandler};
use embedded_graphics::{
    prelude::*,
    text::Alignment,
    primitives::rectangle::Rectangle,
};
use fontfile::FontSize;
use global_settings::{lang::*, ThemedColor};
use crate::static_element::StaticElement;
use alloc::borrow::Cow;

pub const COMPONENT_TESTING: ComponentMenuDefinition = ComponentMenuDefinition {
    components: &[
        ComponentDefinition::Button(
            &ButtonDefinition {
                pos: Point::new(10, 20),
                click: ButtonHandler::Generic(GenericHandler::Print("[first button clicked]")),
                left: ButtonHandler::Generic(GenericHandler::Print("[first button left]")),
                right: ButtonHandler::Generic(GenericHandler::Print("[first button right]")),
                font_size: FontSize::Sz7,
                text: &LanguageString::const_string("hello"),
            }),
        ComponentDefinition::Button(
            &ButtonDefinition {
                pos: Point::new(10, 50),
                click: ButtonHandler::Generic(GenericHandler::Print("[second button clicked]")),
                left: ButtonHandler::Generic(GenericHandler::Print("[second button left]")),
                right: ButtonHandler::Generic(GenericHandler::Print("[second button right]")),
                font_size: FontSize::Sz7,
                text: &LanguageString::const_string("blue!"),
            }),
        ComponentDefinition::Button(
            &ButtonDefinition {
                pos: Point::new(10, 80),
                click: ButtonHandler::Generic(GenericHandler::Print("[third button clicked]")),
                left: ButtonHandler::Generic(GenericHandler::Print("[third button left]")),
                right: ButtonHandler::Generic(GenericHandler::Print("[third button right]")),
                font_size: FontSize::Sz7,
                text: &LanguageString::const_string("click me!"),
            }),
        ComponentDefinition::OptionSwitch(
            &OptionSwitchDefinition {
                pos: Point::new(20, 110),
                click: OptionSwitchHandler::ToggleFocus,
                left: OptionSwitchHandler::PrevElement,
                right: OptionSwitchHandler::NextElement,
                font_size: FontSize::Sz12,
                options: &[
                    LanguageString::const_string("first"),
                    LanguageString::const_string("second"),
                    LanguageString::const_string("third")
                ],
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
                pos: Point::new(26, 50),
                click: OptionSwitchHandler::Generic(GenericHandler::SetLanguageAndTransition(Menu::CustomMenu(CustomMenu::SetupMethod))),
                left: OptionSwitchHandler::PrevElementUpdateLang,
                right: OptionSwitchHandler::NextElementUpdateLang,
                font_size: FontSize::Sz12,
                options: &[
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::En)),
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::Jp)),
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::Fr)),
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::Es)),
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::Pt)),
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::Zh)),
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::Cn)),
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::Ru)),
                    LanguageString::const_string(TEXT_LANGUAGE_NAME.const_index(Lang::De)),
                ],
            }),
    ],
    static_elements: &[
        StaticElement::Text(
            Point::new(64, 20),
            FontSize::Sz7,
            &TEXT_CHOOSE_LANG,
            ThemedColor::Fg,
            Alignment::Center,
        ),
        StaticElement::Text(
            Point::new(10, 150),
            FontSize::Sz7,
            &TEXT_BACK,
            ThemedColor::Fg,
            Alignment::Left,
        ),
        StaticElement::Text(
            Point::new(118, 150),
            FontSize::Sz7,
            &TEXT_OK,
            ThemedColor::Fg,
            Alignment::Right,
        ),
    ],
    left_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Back)),
    right_btn: MenuHandler::Generic(GenericHandler::SetLanguageAndTransition(Menu::CustomMenu(CustomMenu::SetupMethod))),
};

pub const WIFI: ComponentMenuDefinition = ComponentMenuDefinition {
    components: &[
        ComponentDefinition::OptionScroller(
            &OptionScrollerDefinition {
                pos: Point::new(0, 20),
                click: OptionScrollerHandler::SetMenuState(MenuInternalStateAction::SetWifi),
                left: OptionScrollerHandler::PrevOption,
                right: OptionScrollerHandler::NextOption,
                num_visible_elements: 5,
                width: 123,
                font_size: FontSize::Sz12,
                options: OptionsGenerator::WifiGenerator,
            }),
    ],
    static_elements: &[
        StaticElement::Text(
            Point::new(64, 15),
            FontSize::Sz12,
            &TEXT_WIFI_SETTINGS,
            ThemedColor::Fg,
            Alignment::Center,
        ),
    ],
    left_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Back)),
    right_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::ServerDetails)))),
};

pub const WIFI_DETAILS: ComponentMenuDefinition = ComponentMenuDefinition {
    components: &[
        ComponentDefinition::TextBox(
            &TextBoxDefinition {
                pos: Point::new(10, 50),
                max_length: 30,
                width: 120,
                preview_chars: 10,
                font_size: FontSize::Sz12,
                empty_text: "ssid...",
                initial_text: TextGetterSetter::WifiMenuSSIDName,
                on_submit: TextSubmitHandler::SetWifiSSID,
            }),
        ComponentDefinition::TextBox(
            &TextBoxDefinition {
                pos: Point::new(10, 90),
                max_length: 64,
                width: 120,
                preview_chars: 10,
                font_size: FontSize::Sz12,
                empty_text: "password...",
                initial_text: TextGetterSetter::WifiMenuPassword,
                on_submit: TextSubmitHandler::SetWifiPassword,
            }),
        ComponentDefinition::Button(
            &ButtonDefinition {
                pos: Point::new(32, 140),
                click: ButtonHandler::Generic(GenericHandler::ConnectWifi),
                left: ButtonHandler::Generic(GenericHandler::Signal(HandlerResult::None)),
                right: ButtonHandler::Generic(GenericHandler::Signal(HandlerResult::None)),
                font_size: FontSize::Sz12,
                text: &TEXT_CONNECT,
            }),
    ],
    static_elements: &[
        StaticElement::Text(
            Point::new(64, 20),
            FontSize::Sz12,
            &TEXT_SETTINGS_NETWORK,
            ThemedColor::Fg,
            Alignment::Center,
        ),
        StaticElement::Text(
            Point::new(64, 30),
            FontSize::Sz7,
            &TEXT_NETWORK_NAME,
            ThemedColor::Fg,
            Alignment::Center,
        ),
        StaticElement::Text(
            Point::new(64, 70),
            FontSize::Sz7,
            &TEXT_PASSWORD,
            ThemedColor::Fg,
            Alignment::Center,
        ),
    ],
    left_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Back)),
    right_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::ServerDetails)))),
};

pub const SERVER_DETAILS: ComponentMenuDefinition = ComponentMenuDefinition {
    components: &[
        ComponentDefinition::TextBox(
            &TextBoxDefinition {
                pos: Point::new(10, 50),
                max_length: 16,
                width: 120,
                preview_chars: 10,
                font_size: FontSize::Sz12,
                empty_text: "ip address...",
                initial_text: TextGetterSetter::ServerIP,
                on_submit: TextSubmitHandler::SetServerIP,
            }),
        ComponentDefinition::TextBox(
            &TextBoxDefinition {
                pos: Point::new(10, 90),
                max_length: 5,
                width: 120,
                preview_chars: 10,
                font_size: FontSize::Sz12,
                empty_text: "port...",
                initial_text: TextGetterSetter::ServerPort,
                on_submit: TextSubmitHandler::SetServerPort,
            }),
        ComponentDefinition::Button(
            &ButtonDefinition {
                pos: Point::new(32, 140),
                click: ButtonHandler::Generic(GenericHandler::ConnectServer),
                left: ButtonHandler::Generic(GenericHandler::Signal(HandlerResult::None)),
                right: ButtonHandler::Generic(GenericHandler::Signal(HandlerResult::None)),
                font_size: FontSize::Sz12,
                text: &TEXT_CONNECT,
            }),
    ],
    static_elements: &[
        StaticElement::Text(
            Point::new(64, 20),
            FontSize::Sz12,
            &TEXT_SERVER_SETTINGS,
            ThemedColor::Fg,
            Alignment::Center,
        ),
        StaticElement::Text(
            Point::new(64, 30),
            FontSize::Sz7,
            &TEXT_SERVER_ADDR,
            ThemedColor::Fg,
            Alignment::Center,
        ),
        StaticElement::Text(
            Point::new(64, 70),
            FontSize::Sz7,
            &TEXT_PORT,
            ThemedColor::Fg,
            Alignment::Center,
        ),
    ],
    left_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Back)),
    right_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Transition(Menu::ComponentMenu(ComponentMenu::DisplaySettings)))),
};

pub const DISPLAY_SETTINGS: ComponentMenuDefinition = ComponentMenuDefinition {
    components: &[
        ComponentDefinition::OptionSwitch(
            &OptionSwitchDefinition {
                pos: Point::new(20, 60),
                click: OptionSwitchHandler::ToggleFocusAndSetTheme,
                left: OptionSwitchHandler::PrevElement,
                right: OptionSwitchHandler::NextElement,
                font_size: FontSize::Sz12,
                options: &[
                    TEXT_DARK_MODE,
                    TEXT_LIGHT_MODE,
                    TEXT_CUSTOM,
                ],
            }),
        ComponentDefinition::ColorSelector(
            &ColorSelectorDefinition {
                preview_rect: Rectangle::new(Point::new(64-15, 70), Size::new(30, 30)),
                initial_color: ColorGetter::ThemeMenuCustomColor,
                on_submit: ColorSubmitHandler::SetThemeMenuColor,
            }),
        ComponentDefinition::ValueSelector(
            &ValueSelectorDefinition {
                pos: Point::new(64, 140),
                click: ValueSelectorHandler::ToggleFocus,
                left: ValueSelectorHandler::DecrementBrightness,
                right: ValueSelectorHandler::IncrementBrightness,
                suffix: "",
                font_size: FontSize::Sz12,
                low_limit: 0,
                high_limit: 100,
                initial_value: InitialValueGenerator::BacklightBrightness,
            }),
    ],
    static_elements: &[
        StaticElement::Text(
            Point::new(64, 15),
            FontSize::Sz7,
            &TEXT_TOOLTIP_DISPLAY_SETTING,
            ThemedColor::Fg,
            Alignment::Center,
        ),
        StaticElement::Text(
            Point::new(64, 40),
            FontSize::Sz7,
            &TEXT_THEME,
            ThemedColor::Fg,
            Alignment::Center,
        ),
        StaticElement::Text(
            Point::new(64, 120),
            FontSize::Sz7,
            &TEXT_BRIGHTNESS,
            ThemedColor::Fg,
            Alignment::Center,
        ),
    ],
    left_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Back)),
    right_btn: MenuHandler::Generic(GenericHandler::Signal(HandlerResult::Transition(Menu::CustomMenu(CustomMenu::HomeMenu)))),
};
