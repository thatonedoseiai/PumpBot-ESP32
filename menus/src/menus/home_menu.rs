use crate::{MenuSignal, IOHandles, ComponentMenu, Menu};
use fontfile::FontSize;
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Circle, Arc},
    pixelcolor::Rgb565,
};
use embassy_futures::select::{Either4, select4};
use global_settings::{PB_GLOBAL_SETTINGS, Theme, rgb, rgb::RGB};
use pwm::{PwmNumber, Command, PwmAction, PwmPinState};
use button_idf::{ButtonEventKind, ButtonType};
use rotenc::Direction;
use alloc::string::ToString;

#[derive(Debug, Copy, Clone)]
enum HomeMenuMode {
    Browse,
    Edit
}

pub struct HomeMenu {
    mode: HomeMenuMode,
    selected_channel: PwmNumber,
    main_dial_undraw: Rectangle,
    pwm0_undraw: Rectangle,
    pwm1_undraw: Rectangle,
    pwm2_undraw: Rectangle,
    pwm3_undraw: Rectangle,
}

impl HomeMenu {
    const CHANNEL_BOX: Rectangle = Rectangle::new(Point::new(0, 0), Size::new(40, 30));

    pub const fn new() -> Self {
        HomeMenu {
            mode: HomeMenuMode::Browse,
            selected_channel: PwmNumber::Pwm0,
            main_dial_undraw: Rectangle::zero(),
            pwm0_undraw: Rectangle::zero(),
            pwm1_undraw: Rectangle::zero(),
            pwm2_undraw: Rectangle::zero(),
            pwm3_undraw: Rectangle::zero(),
        }
    }

    const fn undraw_style(theme: &Theme) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .fill_color(theme.bg().as_rgb565())
            .build()
    }

    const fn next_selection(&mut self) -> &mut Self {
        self.selected_channel = match self.selected_channel {
            PwmNumber::Pwm0 => PwmNumber::Pwm1,
            PwmNumber::Pwm1 => PwmNumber::Pwm0,
            PwmNumber::Pwm2 => PwmNumber::Pwm0,
            PwmNumber::Pwm3 => PwmNumber::Pwm0,
        };
        self
    }

    const fn channel_position(c: PwmNumber) -> Point {
        match c {
            PwmNumber::Pwm0 => Point::new(24, 130),
            PwmNumber::Pwm1 => Point::new(64, 130),
            PwmNumber::Pwm2 => Point::new(24, 100),
            PwmNumber::Pwm3 => Point::new(64, 100),
        }
    }

    async fn draw_main_dial(&mut self, theme: &Theme, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        h.font.font.borrow_mut().set_size(FontSize::Sz14)?;
        let current_state = self.selected_channel.get_state().await;
        let percentage = current_state.get_duty_pct();
        let percentage_string = percentage.to_string() + "%";
        self.main_dial_undraw.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
        let dial_end = (percentage as f32 * 2.7).deg();
        let dial = Arc::with_center(Point::new(64, 70), 80, 135.0.deg(), dial_end)
                    .into_styled(PrimitiveStyle::with_stroke(theme.fg().as_rgb565(), 5));
        let dial_text = Text::with_alignment(&percentage_string, Point::new(64, 78), &h.font, Alignment::Center);
        self.main_dial_undraw = dial.bounding_box(); // dial_text.bounding_box();
        dial.draw(&mut h.screen)?;
        dial_text.draw(&mut h.screen)?;
        let dial_colour = RGB::lerp(&rgb![255, 0, 0], &rgb![0, 255, 0], (percentage as f32) / 100.0).as_rgb565();
        Arc::with_center(Point::new(64, 70), 80, 135.0.deg(), dial_end)
                    .into_styled(PrimitiveStyle::with_stroke(dial_colour, 3))
                    .draw(&mut h.screen)?;
        Ok(())
    }

    const fn channel_style(theme: &Theme, selected: bool) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_color(if selected { theme.highlight() } else { theme.fg() }.as_rgb565())
            .stroke_width(2)
            .build()
    }

    const fn power_indicator_style(theme: &Theme, state: PwmPinState) -> PrimitiveStyle<Rgb565> {
        PrimitiveStyleBuilder::new()
            .stroke_color(theme.fg().as_rgb565())
            .stroke_width(1)
            .fill_color(match state {
                PwmPinState::On => theme.highlight(),
                PwmPinState::Off => theme.bg(),
            }.as_rgb565())
            .build()
    }

    async fn draw_channel(&mut self, channel: PwmNumber, theme: &Theme, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        h.font.font.borrow_mut().set_size(FontSize::Sz7)?;
        let current_state = channel.get_state().await;
        let pos = Self::channel_position(channel);
        let surrounding_rect = Self::CHANNEL_BOX.translate(pos).into_styled(Self::channel_style(theme, self.selected_channel == channel)).draw(&mut h.screen)?;

        let percentage = current_state.get_duty_pct();
        let percentage_string = percentage.to_string() + "%";
        match channel {
            PwmNumber::Pwm0 => { self.pwm0_undraw }
            PwmNumber::Pwm1 => { self.pwm1_undraw }
            PwmNumber::Pwm2 => { self.pwm2_undraw }
            PwmNumber::Pwm3 => { self.pwm3_undraw }
        }.into_styled(Self::undraw_style(theme)).draw(&mut h.screen)?;
        let channel_text = Text::with_alignment(&percentage_string, pos + Point::new(20, 10), &h.font, Alignment::Center);
        match channel {
            PwmNumber::Pwm0 => { self.pwm0_undraw = channel_text.bounding_box(); }
            PwmNumber::Pwm1 => { self.pwm1_undraw = channel_text.bounding_box(); }
            PwmNumber::Pwm2 => { self.pwm2_undraw = channel_text.bounding_box(); }
            PwmNumber::Pwm3 => { self.pwm3_undraw = channel_text.bounding_box(); }
        };
        channel_text.draw(&mut h.screen)?;

        Circle::new(pos + Point::new(15, 15), 10).into_styled(Self::power_indicator_style(theme, current_state.state)).draw(&mut h.screen)?;
        Ok(())
    }

    pub(crate) async fn run(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let theme = PB_GLOBAL_SETTINGS.read().await.theme;
        h.screen.clear(theme.bg().as_rgb565())?;
        h.font.fgcol = theme.fg();
        h.font.bgcol = theme.bg();
        self.draw_main_dial(&theme, h).await?;
        self.draw_channel(PwmNumber::Pwm1, &theme, h).await?;
        self.draw_channel(PwmNumber::Pwm0, &theme, h).await?;

        // let ip = h.wifi.wget("http://ifconfig.me").await.unwrap();
        // log::error!("my ip! {}", ip);

        loop {
            let result = select4(
                h.rotenc.receive(),
                h.button.receive(),
                h.pwm_output.wait_result(),
                h.server.receive(),
            ).await;
            match (result, self.mode) {
                (Either4::First(r), HomeMenuMode::Browse) => {
                    self.next_selection();
                    self.draw_main_dial(&theme, h).await?;
                    self.draw_channel(PwmNumber::Pwm1, &theme, h).await?;
                    self.draw_channel(PwmNumber::Pwm0, &theme, h).await?;
                },
                (Either4::Second(b), _) => {
                    match (&b.button_type, &b.event) {
                        (ButtonType::Right, ButtonEventKind::Down) => {
                            // transition to settings menu
                            // todo!("transition to settings menu")
                            return Ok(MenuSignal::Transition(Menu::ComponentMenu(ComponentMenu::Settings)));
                        },
                        (ButtonType::Left, ButtonEventKind::Down) => {
                            h.pwm_output.send_await(Command::new(
                                PwmAction::Toggle(self.selected_channel), 
                                0 
                            )).await;
                            let state = self.selected_channel.get_state().await;
                            self.draw_channel(self.selected_channel, &theme, h).await?;
                        },
                        (ButtonType::Rotenc, ButtonEventKind::Down) => {
                            self.mode = match self.mode {
                                HomeMenuMode::Edit => HomeMenuMode::Browse,
                                HomeMenuMode::Browse => HomeMenuMode::Edit,
                            };
                        },
                        _ => ()
                    }
                },
                (Either4::First(r), HomeMenuMode::Edit) => {
                    // control PWM values
                    let state = self.selected_channel.get_state().await;
                    if r > 0 {
                        // let new_duty = if state.duty >= (99 * 163) {
                        //     16383
                        // } else {
                        //     state.duty.saturating_add(163)
                        // };
                        h.pwm_output.send_and_forget(Command::new(
                            // PwmAction::SetDuty(self.selected_channel, new_duty), 
                            PwmAction::IncDutyPct(self.selected_channel, r.min(255) as u8),
                            0 
                        )).await;
                    } else {
                        // let new_duty = if state.duty == 16383 {
                        //     99 * 163
                        // } else {
                        //     state.duty.saturating_sub(163)
                        // };
                        h.pwm_output.send_and_forget(Command::new(
                            // PwmAction::SetDuty(self.selected_channel, new_duty), 
                            PwmAction::DecDutyPct(self.selected_channel, (-r).min(255) as u8),
                            0 
                        )).await;
                    }
                },
                (Either4::Third(_), _) => {
                    self.draw_main_dial(&theme, h).await?;
                    self.draw_channel(PwmNumber::Pwm1, &theme, h).await?;
                    self.draw_channel(PwmNumber::Pwm0, &theme, h).await?;
                }
                (Either4::Fourth(_), _) => {
                    todo!("handle a network command!")
                }
            }
            // let text = Text::with_alignment("menu", Point::new(10, 20), &h.font, Alignment::Left);
            // text.draw(&mut h.screen)?;
        }
    }
}
