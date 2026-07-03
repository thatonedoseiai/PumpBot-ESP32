//! This module contains an enum that represents all the different kinds of events we could receive
//! from things across the board. Currently it only contains events from the Button and Encoder,
//! but we may also use it for messages received from the remote server (if connected)

#[cfg(not(feature = "sim"))]
pub mod event {
    use button_idf::ButtonEvent;
    use rotenc::EncoderEvent;

    /// A wrapper around multiple kinds of events that we can receive from different IOs. Currently
    /// working only with the Button and Rotenc.
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
}

#[cfg(feature = "sim")]
pub mod event {
    #[derive(Clone, Copy)]
    pub enum Event {
        Button(BData),
        Rotenc(RData)
    }

    #[derive(Clone, Copy)]
    pub struct BData {
        pub pin: u8,
    }

    #[derive(Clone, Copy)]
    pub struct RData {
    }
}