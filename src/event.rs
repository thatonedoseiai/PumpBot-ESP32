use button_idf::ButtonEvent;
use rotenc::RotaryEncoderEvent;

#[derive(Clone, Copy)]
pub enum Event {
    Button(ButtonEvent),
    Rotenc(RotaryEncoderEvent)
}

impl From<ButtonEvent> for Event {
    fn from(val: ButtonEvent) -> Self {
        Event::Button(val)
    }
}

impl From<RotaryEncoderEvent> for Event {
    fn from(val: RotaryEncoderEvent) -> Self {
        Event::Rotenc(val)
    }
}