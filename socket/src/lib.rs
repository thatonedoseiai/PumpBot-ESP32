#![no_std]
#![no_main]

extern crate alloc;

use embassy_net::{
    tcp::{TcpSocket, ConnectError},
    IpEndpoint,
    IpAddress,
    Ipv4Address,
    Stack
};
use embassy_executor::Spawner;
use embassy_sync::{
    channel::{Channel, Sender, Receiver},
    blocking_mutex::raw::CriticalSectionRawMutex,
};
use core::fmt;
use core::write;
use core::convert::From;

#[derive(Clone, Copy)]
pub enum ServerCommand {
    Connect(IpAddress, u16),
    Disconnect,
}

#[derive(Debug, Clone, Copy)]
pub enum ServerResponse {
    BlockHid(u16),          // centiseconds
    BlockHidRelative(i16),
    ToggleChannel(PwmNumber),
    ChannelOff(PwmNumber),
    ChannelOn(PwmNumber),
    SetPwmValue(u16, PwmNumber),   // pwm value, channel num
    SetPwmValueRelative(i16, PwmNumber),
    GetState(PwmNumber),
    None
}

pub struct ServerConnection {
    server_ip: IpAddress,
    server_port: u16,
}

#[derive(Debug)]
pub enum ServerError {
    ConnectError(ConnectError)
}
impl core::error::Error for ServerError { }
impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConnectError(c) => write!(f, "SERVER ERROR: ConnectError {}", c)
        }
    }
}

impl From<ConnectError> for ServerError {
    fn from(val: ConnectError) -> ServerError {
        Self::ConnectError(val)
    }
}

static COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, ServerCommand, 8> = Channel::new();
static RESPONSE_CHANNEL: Channel<CriticalSectionRawMutex, Result<ServerResponse, ServerError>, 8> = Channel::new();

impl ServerConnection {
    const RX_BUF_LEN: usize = 256;
    const TX_BUF_LEN: usize = 256;

    pub fn new(spawner: Spawner, netstack: Stack<'static>) -> Self {
        spawner.spawn(socket_runner_task(COMMAND_CHANNEL.receiver(), RESPONSE_CHANNEL.sender(), netstack).unwrap());

        ServerConnection {
            server_ip: IpAddress::Ipv4(Ipv4Address::new(0,0,0,0)),
            server_port: 0,
        }
    }

    pub async fn connect(&mut self) -> Result<ServerResponse, ServerError> {
        COMMAND_CHANNEL.send(ServerCommand::Connect(self.server_ip, self.server_port)).await;
        RESPONSE_CHANNEL.receive().await
    }

    pub async fn disconnect(&mut self) {
        COMMAND_CHANNEL.send(ServerCommand::Disconnect).await;
    }
}

#[embassy_executor::task]
async fn socket_runner_task(commands: Receiver<'static, CriticalSectionRawMutex, ServerCommand, 8>, responses: Sender<'static, CriticalSectionRawMutex, Result<ServerResponse, ServerError>, 8>, netstack: Stack<'static>) {
    let mut rx_buffer = [0u8; ServerConnection::RX_BUF_LEN];
    let mut tx_buffer = [0u8; ServerConnection::TX_BUF_LEN];
    let mut conn = TcpSocket::new(
        netstack, &mut rx_buffer, &mut tx_buffer,
    );
    loop {
        match commands.receive().await {
            ServerCommand::Connect(addr, port) => {
                let res = conn.connect(IpEndpoint::new(addr, port)).await;
                let response = match res {
                    Ok(_) => Ok(ServerResponse::None),
                    Err(e) => Err(e.into()),
                };
                responses.send(response).await;
            },
            ServerCommand::Disconnect => {
                conn.close();
            }
        }
    }
}