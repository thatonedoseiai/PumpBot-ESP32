#![no_std]
#![no_main]
#![feature(iter_intersperse)]

extern crate alloc;

use embassy_net::{
    tcp::{TcpSocket, ConnectError},
    tcp,
    IpEndpoint,
    IpAddress,
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
use pwm::PwmNumber;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use core::task::Poll;
use core::future::poll_fn;

#[derive(Clone, Copy)]
pub enum ServerCommand {
    Connect(IpAddress, u16),
    Disconnect,
    Listen,
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

impl ServerResponse {
    const fn decode_from(v: Vec<u8>) -> Self {
        todo!()
    }
}

pub struct ServerConnection {
    pub server_ip: IpAddress,
    pub server_port: u16,
}

#[derive(Debug)]
pub enum ServerError {
    ConnectError(ConnectError),
    TcpError(tcp::Error),
}
impl core::error::Error for ServerError { }
impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConnectError(c) => write!(f, "SERVER ERROR: ConnectError {}", c),
            Self::TcpError(e) => write!(f, "SERVER ERROR: Tcp Error during read: {}", e),
        }
    }
}

impl From<ConnectError> for ServerError {
    fn from(val: ConnectError) -> ServerError {
        Self::ConnectError(val)
    }
}

impl From<tcp::Error> for ServerError {
    fn from(val: tcp::Error) -> ServerError {
        Self::TcpError(val)
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
            server_ip: IpAddress::v4(0,0,0,0),
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

    pub async fn receive(&self) -> Result<ServerResponse, ServerError> {
        COMMAND_CHANNEL.send(ServerCommand::Listen).await;
        RESPONSE_CHANNEL.receive().await
    }

    pub fn string_from_ip(ip: &IpAddress) -> String {
        match ip {
            IpAddress::Ipv4(i) => {
                i.octets().iter().map(|f| f.to_string()).intersperse(".".to_string()).collect()
            },
            IpAddress::Ipv6(k) => {
                let octets = k.octets();
                let mut s = String::new();
                const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
                
                let mut i = 0;
                while i < 8 {
                    if i > 0 {
                        // If push fails (which shouldn't happen for a valid IPv6 size), we ignore or handle it
                        let _ = s.push(':');
                    }
                    
                    let hextet = ((octets[i * 2] as u16) << 8) | (octets[i * 2 + 1] as u16);
                    
                    let mut shift = 12;
                    let mut leading_zero = true;
                    
                    while shift >= 0 {
                        let digit = ((hextet >> shift) & 0xf) as usize;
                        if digit != 0 || shift == 0 || !leading_zero {
                            leading_zero = false;
                            let _ = s.push(HEX_CHARS[digit] as char);
                        }
                        shift -= 4;
                    }
                    
                    i += 1;
                }
                
                s
            },
        }
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
            },
            ServerCommand::Listen => {
                if conn.local_endpoint().is_some() {
                    let mut buf: Vec<u8> = Vec::with_capacity(256);
                    wait_until_truly_readable(&conn).await;
                    let res = conn.read(&mut buf).await;
                    let response = match res {
                        Ok(_size) => Ok(ServerResponse::decode_from(buf)),
                        Err(e) => Err(ServerError::TcpError(e)),
                    };
                    responses.send(response).await;
                }
            },
        }
    }
}

async fn wait_until_truly_readable(socket: &TcpSocket<'_>) -> () {
    poll_fn(|cx| {
        if socket.may_recv() {
            return Poll::Ready(());
        } else {
            return Poll::Pending; 
        }
    }).await
}