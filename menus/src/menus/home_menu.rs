use crate::{MenuSignal, IOHandles};
use embedded_graphics::{
    prelude::*,
    text::{Text, Alignment},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder},
    pixelcolor::Rgb565,
};
use embassy_futures::select::{Either, select};
use global_settings::{PB_GLOBAL_SETTINGS, Theme};
use pwm::{PwmNumber, Command, PwmAction};
use button_idf::{ButtonEventKind, ButtonType};

#[derive(Debug, Copy, Clone)]
enum HomeMenuMode {
    Browse,
    Edit
}

pub struct HomeMenu {
    mode: HomeMenuMode,
    selected_channel: PwmNumber,
}

impl HomeMenu {
    pub const fn new() -> Self {
        HomeMenu {
            mode: HomeMenuMode::Browse,
            selected_channel: PwmNumber::Pwm0,
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

    fn draw_main_dial(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<()> {
        let current_state = self.selected_channel.get_state();
        todo!()
    }

    pub(crate) async fn run(&mut self, h: &mut IOHandles<'_>) -> anyhow::Result<MenuSignal> {
        let theme = PB_GLOBAL_SETTINGS.read().await.theme;
        h.screen.clear(theme.bg().as_rgb565())?;
        h.font.fgcol = theme.fg();
        h.font.bgcol = theme.bg();
        loop {
            let result = select(
                h.rotenc.receive(),
                h.button.receive(),
            ).await;
            match (result, self.mode) {
                (Either::First(r), HomeMenuMode::Browse) => {
                    self.next_selection();
                },
                (Either::Second(b), _) => {
                    match (&b.button_type, &b.event) {
                        (ButtonType::Right, ButtonEventKind::Down) => {
                            // transition to settings menu
                            todo!("transition to settings menu")
                        },
                        (ButtonType::Left, ButtonEventKind::Down) => {
                            h.pwm_output.send(Command::new(
                                PwmAction::Toggle(self.selected_channel), 
                                0 
                            )).await;
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
                (Either::First(r), HomeMenuMode::Edit) => {
                    // control PWM values
                    todo!("control PWM values")
                },
            }
            let text = Text::with_alignment("menu", Point::new(10, 20), &h.font, Alignment::Left);
            text.draw(&mut h.screen)?;
        }
    }
}
