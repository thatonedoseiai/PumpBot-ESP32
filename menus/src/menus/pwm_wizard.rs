use crate::{MenuSignal, IOHandles, Menu, ComponentMenu};
use pwm::{PwmNumber, PwmMode};
use embassy_futures::select::{select, Either};
use fontfile::FontSize;
use button_idf::{ButtonType, ButtonEventKind};
use global_settings::{ PB_GLOBAL_SETTINGS, Theme };

use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    pixelcolor::Rgb565,
    primitives::{Rectangle, PrimitiveStyleBuilder, PrimitiveStyle},
};
use alloc::format;

#[derive(Debug, Clone, Copy)]
enum Mode {
    Browse,
    Edit,
}

#[derive(Debug, Clone, Copy)]
enum MenuElementType {
    Channel,
    Min,
    Max,
    Solenoid,
}

#[derive(Debug, Clone, Copy)]
struct MenuElement {
    elem_type: MenuElementType,
    undraw: Rectangle,
}

impl MenuElement {
    const SOLENOID_TEXT_POS: Point = Point::new(64, 80);
    const MAX_TEXT_POS: Point = Point::new(90, 60);
    const MIN_TEXT_POS: Point = Point::new(20, 60);

    const fn new(elem_type: MenuElementType) -> Self {
        MenuElement {
            elem_type,
            undraw: Rectangle::zero(),
        }
    }

    const fn channel_label(selected_channel: PwmNumber) -> &'static str {
        match selected_channel {
            PwmNumber::Pwm0 => "0",
            PwmNumber::Pwm1 => "1",
            PwmNumber::Pwm2 => "2",
            PwmNumber::Pwm3 => "3",
        }
    }

    const fn undraw_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg().as_rgb565())
            .build()
    }

    async fn draw(&mut self, selected_channel: PwmNumber, highlighted: bool, theme: &Theme, channel_state: ChannelState, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        h.font.fgcol = if highlighted {
            theme.highlight()
        } else {
            theme.fg()
        };
        self.undraw.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
        match self.elem_type {
            MenuElementType::Channel => {
                let t = Text::with_alignment(Self::channel_label(selected_channel), Point::new(64, 30), &h.font, Alignment::Center);
                self.undraw = t.bounding_box();
                t.draw(&mut h.screen)?;
            },
            MenuElementType::Min => {
                // let (m, _) = h.pwm_output.get_channel_limits(selected_channel).await;
                let min = format!("{}", channel_state.min);
                let t = Text::with_alignment(&min, Self::MIN_TEXT_POS, &h.font, Alignment::Center);
                self.undraw = t.bounding_box();
                t.draw(&mut h.screen)?;
            },
            MenuElementType::Max => {
                // let (_, m) = h.pwm_output.get_channel_limits(selected_channel).await;
                let max = format!("{}", channel_state.max);
                let t = Text::with_alignment(&max, Self::MAX_TEXT_POS, &h.font, Alignment::Center);
                self.undraw = t.bounding_box();
                t.draw(&mut h.screen)?;
            },
            MenuElementType::Solenoid => {
                // let operating_mode = h.pwm_output.get_operating_mode(selected_channel).await;
                let t = Text::with_alignment(match channel_state.solenoid {
                    PwmMode::Solenoid => "On",
                    PwmMode::Pwm => "Off",
                }, Self::SOLENOID_TEXT_POS, &h.font, Alignment::Center);
                self.undraw = t.bounding_box();
                t.draw(&mut h.screen)?;
            },
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct ChannelState {
    min: u16,
    max: u16,
    solenoid: PwmMode,
}

impl ChannelState {
    pub async fn get(chan: PwmNumber, h: &mut IOHandles<'_>) -> Self {
        let (min, max) = h.pwm_output.get_channel_limits(chan).await;
        let solenoid = h.pwm_output.get_operating_mode(chan).await;
        ChannelState { min, max, solenoid }
    }

    pub const fn zero() -> Self {
        ChannelState { min: 0, max: 0, solenoid: PwmMode::Pwm }
    }
}

pub struct PwmWizardState {
    selected_channel: PwmNumber,
    mode: Mode,
    selected_element: usize,
    channel_state: ChannelState,
    elements: [MenuElement; 4],
}

impl PwmWizardState {
    const NUM_ELEMENTS: i16 = 4;

    pub const fn new() -> Self {
        PwmWizardState {
            selected_channel: PwmNumber::Pwm0,
            mode: Mode::Browse,
            selected_element: 0,
            channel_state: ChannelState::zero(),
            elements: [
                MenuElement::new(MenuElementType::Channel),
                MenuElement::new(MenuElementType::Min),
                MenuElement::new(MenuElementType::Max),
                MenuElement::new(MenuElementType::Solenoid),
            ],
        }
    }

    const fn next_channel(&mut self) {
        self.selected_channel = match self.selected_channel {
            PwmNumber::Pwm0 => PwmNumber::Pwm1,
            PwmNumber::Pwm1 => PwmNumber::Pwm0,
            PwmNumber::Pwm2 => PwmNumber::Pwm0,
            PwmNumber::Pwm3 => PwmNumber::Pwm0,
        }
    }

    async fn draw_element(&mut self, index: usize, theme: &Theme, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        self.elements[index].draw(self.selected_channel, index == self.selected_element, theme, self.channel_state, h).await?;
        Ok(())
    }

    pub(crate) async fn run(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let theme = {
            let settings = &PB_GLOBAL_SETTINGS.read().await;
            settings.theme
        };
        h.font.font.borrow_mut().set_size(FontSize::Sz12)?;
        h.font.fgcol = theme.fg();
        h.font.bgcol = theme.bg();

        self.channel_state = ChannelState::get(self.selected_channel, h).await;
        // self.draw_dynamic_elements(&theme, h).await?;
        for i in 0..self.elements.len() {
            self.draw_element(i, &theme, h).await?
        }

        loop {
            let result = select(
                h.button.receive(),
                h.rotenc.receive(10),
            ).await;
            match (result, self.mode) {
                (Either::First(b), _) => {
                    match (b.button_type, b.event) {
                        (ButtonType::Rotenc, ButtonEventKind::Down) => {
                            if self.selected_element != 3 {
                                self.mode = match self.mode {
                                    Mode::Browse => Mode::Edit,
                                    Mode::Edit => Mode::Browse,
                                };
                            } else {
                                self.channel_state.solenoid = match self.channel_state.solenoid {
                                    PwmMode::Pwm => PwmMode::Solenoid,
                                    PwmMode::Solenoid => PwmMode::Pwm,
                                };
                                self.draw_element(self.selected_element, &theme, h).await?;
                            }
                            self.draw_element(self.selected_element, &theme, h).await?;
                        },
                        (ButtonType::Right, _) => return Ok(MenuSignal::Back),
                        (ButtonType::Left, ButtonEventKind::Down) => todo!("on/off button in drawing does what??"),
                        (_, _) => {}
                    };
                },
                (Either::Second(r), Mode::Browse) => {
                    let old_selection = self.selected_element;
                    self.selected_element = (self.selected_element + r.rem_euclid(Self::NUM_ELEMENTS) as usize).rem_euclid(Self::NUM_ELEMENTS as usize);
                    self.draw_element(old_selection, &theme, h).await?;
                    self.draw_element(self.selected_element, &theme, h).await?;
                },
                (Either::Second(r), Mode::Edit) => {
                    match self.elements[self.selected_element].elem_type {
                        MenuElementType::Channel => {
                            self.next_channel();
                            self.channel_state = ChannelState::get(self.selected_channel, h).await;
                            for i in 0..self.elements.len() {
                                self.draw_element(i, &theme, h).await?
                            }
                        },
                        MenuElementType::Min => {
                            self.channel_state.min = self.channel_state.min.saturating_add_signed(r).min(self.channel_state.max);
                        },
                        MenuElementType::Max => {
                            self.channel_state.max = self.channel_state.max.saturating_add_signed(r).max(self.channel_state.min).min(h.pwm_output.max_duty);
                        },
                        MenuElementType::Solenoid => {
                            unreachable!("selecting the solenoid button should not be possible");
                        },
                    };
                    self.draw_element(self.selected_element, &theme, h).await?;
                },
            }
        }
    }
}
