use button_idf::ButtonEvent;
use rotenc::EncoderEvent;

#[derive(Clone, Copy)]
pub enum Event {
    Button(ButtonEvent),
    Rotenc(EncoderEvent)
}

impl From<ButtonEvent> for Event {
    fn from(val: ButtonEvent) -> Self {
        Event::Button(val)
    }
}

impl From<EncoderEvent> for Event {
    fn from(val: EncoderEvent) -> Self {
        Event::Rotenc(val)
    }
}