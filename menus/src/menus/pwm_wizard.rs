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
    primitives::{Rectangle, PrimitiveStyleBuilder, PrimitiveStyle, Triangle},
};
use alloc::format;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Browse,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MenuElementType {
    Channel,
    Voltage,
    Min,
    Max,
    Solenoid,
}

#[derive(Debug, Clone, Copy)]
struct MenuElement {
    elem_type: MenuElementType,
    undraw: Rectangle,
    cursor_undraw_left: Rectangle,
    cursor_undraw_right: Rectangle,
    aux_element_undraw: Option<Rectangle>,
}

impl MenuElement {
    const CHANNEL_POS: Point = Point::new(64, 30);
    const VOLTAGE_POS: Point = Point::new(64, 60);
    const SOLENOID_TEXT_POS: Point = Point::new(64, 110);
    const MAX_TEXT_POS: Point = Point::new(90, 80);
    const VOLT_LABEL_POS_MAX: Point = Point::new(90, 90);
    const MIN_TEXT_POS: Point = Point::new(30, 80);
    const VOLT_LABEL_POS_MIN: Point = Point::new(30, 90);
    const CURSOR_LEFT: Triangle = Triangle::new(Point::new(0, 0), Point::new(5, 5), Point::new(5, -5));
    const CURSOR_RIGHT: Triangle = Triangle::new(Point::new(0, 0), Point::new(-5, 5), Point::new(-5, -5));

    const fn new(elem_type: MenuElementType) -> Self {
        MenuElement {
            elem_type,
            undraw: Rectangle::zero(),
            cursor_undraw_left: Rectangle::zero(),
            cursor_undraw_right: Rectangle::zero(),
            aux_element_undraw: None,
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

    const fn cursor_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.highlight().as_rgb565())
            .build()
    }

    async fn draw_cursors(&mut self, theme: &Theme, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let left = Self::CURSOR_LEFT.translate(self.undraw.top_left + Point::new(-10, self.undraw.size.height as i32 / 2)).into_styled(Self::cursor_style(theme));
        let right = Self::CURSOR_RIGHT.translate(self.undraw.top_left + Point::new(10 + self.undraw.size.width as i32, self.undraw.size.height as i32 / 2)).into_styled(Self::cursor_style(theme));
        self.cursor_undraw_left = left.bounding_box();
        self.cursor_undraw_right = right.bounding_box();
        left.draw(&mut h.screen)?;
        right.draw(&mut h.screen)?;
        Ok(())
    }

    async fn draw_voltage_element(&mut self, voltage: u8, theme: &Theme, channel_state: ChannelState, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        h.font.fgcol = theme.fg();
        h.font.font.borrow_mut().set_size(FontSize::Sz7)?;
        if let Some(k) = self.aux_element_undraw {
            k.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
            self.aux_element_undraw = None;
        }
        match self.elem_type {
            MenuElementType::Channel => {},
            MenuElementType::Voltage => {},
            MenuElementType::Min => {
                let voltage = (channel_state.min as f32 / h.pwm_output.max_duty as f32) * (voltage as f32) / 2.0;
                let volts = format!("({:.2}V)", voltage);
                let t = Text::with_alignment(&volts, Self::VOLT_LABEL_POS_MIN, &h.font, Alignment::Center);
                self.aux_element_undraw = Some(t.bounding_box());
                t.draw(&mut h.screen)?;
            },
            MenuElementType::Max => {
                let voltage = (channel_state.max as f32 / h.pwm_output.max_duty as f32) * (voltage as f32) / 2.0;
                let volts = format!("({:.2}V)", voltage);
                let t = Text::with_alignment(&volts, Self::VOLT_LABEL_POS_MAX, &h.font, Alignment::Center);
                self.aux_element_undraw = Some(t.bounding_box());
                t.draw(&mut h.screen)?;
            },
            MenuElementType::Solenoid => {},
        }
        h.font.font.borrow_mut().set_size(FontSize::Sz12)?;
        Ok(())
    }

    async fn draw(&mut self, voltage: u8, selected_channel: PwmNumber, highlighted: bool, selected: bool, theme: &Theme, channel_state: ChannelState, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        h.font.fgcol = if highlighted {
            theme.highlight()
        } else {
            theme.fg()
        };
        if !self.cursor_undraw_left.is_zero_sized() {
            self.cursor_undraw_left.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
            self.cursor_undraw_right.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
            self.cursor_undraw_left = Rectangle::zero();
            self.cursor_undraw_right = Rectangle::zero();
        }
        self.undraw.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
        match self.elem_type {
            MenuElementType::Channel => {
                let t = Text::with_alignment(Self::channel_label(selected_channel), Self::CHANNEL_POS, &h.font, Alignment::Center);
                self.undraw = t.bounding_box();
                t.draw(&mut h.screen)?;
                if selected && highlighted {
                    self.draw_cursors(theme, h).await?;
                }
            },
            MenuElementType::Voltage => {
                let v = format!("{}.{}V", voltage / 2, 5 * (voltage & 1));
                let t = Text::with_alignment(&v, Self::VOLTAGE_POS, &h.font, Alignment::Center);
                self.undraw = t.bounding_box();
                t.draw(&mut h.screen)?;
                if selected && highlighted {
                    self.draw_cursors(theme, h).await?;
                }
            },
            MenuElementType::Min => {
                // let (m, _) = h.pwm_output.get_channel_limits(selected_channel).await;
                let min = format!("{}", channel_state.min);
                let t = Text::with_alignment(&min, Self::MIN_TEXT_POS, &h.font, Alignment::Center);
                self.undraw = t.bounding_box();
                t.draw(&mut h.screen)?;
                self.draw_voltage_element(voltage, theme, channel_state, h).await?;
                if selected && highlighted {
                    self.draw_cursors(theme, h).await?;
                }
            },
            MenuElementType::Max => {
                // let (_, m) = h.pwm_output.get_channel_limits(selected_channel).await;
                let max = format!("{}", channel_state.max);
                let t = Text::with_alignment(&max, Self::MAX_TEXT_POS, &h.font, Alignment::Center);
                self.undraw = t.bounding_box();
                t.draw(&mut h.screen)?;
                self.draw_voltage_element(voltage, theme, channel_state, h).await?;
                if selected && highlighted {
                    self.draw_cursors(theme, h).await?;
                }
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

    pub async fn set(&self, chan: PwmNumber, h: &mut IOHandles<'_>) {
        h.pwm_output.set_channel_min_duty(chan, self.min).await;
        h.pwm_output.set_channel_max_duty(chan, self.max).await;
        h.pwm_output.set_channel_operating_mode(chan, self.solenoid).await;
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
    elements: [MenuElement; Self::NUM_ELEMENTS as usize],
    voltage: u8,
}

impl PwmWizardState {
    const NUM_ELEMENTS: i16 = 5;
    const MIN_VOLTAGE: u8 = 8;  // 4 in increments of 0.5
    const MAX_VOLTAGE: u8 = 48; // 24 in increments of 0.5
    const DEFAULT_VOLTAGE: u8 = 24; // 12 in increments of 0.5

    pub const fn new() -> Self {
        PwmWizardState {
            selected_channel: PwmNumber::Pwm0,
            mode: Mode::Browse,
            selected_element: 0,
            channel_state: ChannelState::zero(),
            elements: [
                MenuElement::new(MenuElementType::Channel),
                MenuElement::new(MenuElementType::Voltage),
                MenuElement::new(MenuElementType::Min),
                MenuElement::new(MenuElementType::Max),
                MenuElement::new(MenuElementType::Solenoid),
            ],
            voltage: Self::DEFAULT_VOLTAGE,
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
        self.elements[index].draw(self.voltage, self.selected_channel, index == self.selected_element, self.mode == Mode::Edit, theme, self.channel_state, h).await?;
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
                            if self.elements[self.selected_element].elem_type != MenuElementType::Solenoid {
                                self.mode = match self.mode {
                                    Mode::Browse => Mode::Edit,
                                    Mode::Edit => Mode::Browse,
                                };
                            } else {
                                self.channel_state.solenoid = match self.channel_state.solenoid {
                                    PwmMode::Pwm => PwmMode::Solenoid,
                                    PwmMode::Solenoid => PwmMode::Pwm,
                                };
                                h.pwm_output.set_channel_operating_mode(self.selected_channel, self.channel_state.solenoid).await;
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
                            // self.channel_state.set(self.selected_channel, h).await;
                            self.next_channel();
                            self.channel_state = ChannelState::get(self.selected_channel, h).await;
                            for i in 0..self.elements.len() {
                                self.draw_element(i, &theme, h).await?
                            }
                        },
                        MenuElementType::Voltage =>{
                            self.voltage = self.voltage.saturating_add_signed(r.clamp(i8::MIN.into(), i8::MAX.into()) as i8).clamp(Self::MIN_VOLTAGE, Self::MAX_VOLTAGE);
                            self.elements[2].draw_voltage_element(self.voltage, &theme, self.channel_state, h).await?;
                            self.elements[3].draw_voltage_element(self.voltage, &theme, self.channel_state, h).await?;
                        },
                        MenuElementType::Min => {
                            self.channel_state.min = self.channel_state.min.saturating_add_signed(r).min(self.channel_state.max);
                            h.pwm_output.set_channel_min_duty(self.selected_channel, self.channel_state.min).await;
                        },
                        MenuElementType::Max => {
                            self.channel_state.max = self.channel_state.max.saturating_add_signed(r).max(self.channel_state.min).min(h.pwm_output.max_duty);
                            h.pwm_output.set_channel_max_duty(self.selected_channel, self.channel_state.max).await;
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
