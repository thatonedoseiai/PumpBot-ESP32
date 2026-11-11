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
    signal::Signal,
    priority_channel::{PriorityChannel, Max, Receiver as PReceiver, Sender as PSender},
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::Mutex,
};
use embassy_futures::select::{
    Either, select
};
use core::fmt;
use core::write;
use core::convert::From;
use pwm::PwmNumber;
use alloc::string::{ToString, String};
use alloc::vec::Vec;
use alloc::vec;
use core::task::Poll;
use core::future::{poll_fn, pending};
use core::cmp::Ordering;
use core::sync::atomic::{AtomicBool, Ordering as AOrdering};
use alloc::sync::Arc;

const READ_BUF_SIZE: usize = 16;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ServerCommand {
    Connect(IpAddress, u16),
    Send(Vec<u8>),
    Disconnect,
    Listen,
}

impl ServerCommand {
    const fn priority(&self) -> u8 {
        match self {
            ServerCommand::Connect(_, _) => 3,
            ServerCommand::Disconnect => 2,
            ServerCommand::Send(_) => 1,
            ServerCommand::Listen => 0,
        }
    }
}

impl PartialOrd for ServerCommand {
    fn partial_cmp(&self, other: &ServerCommand) -> Option<Ordering> {
        self.priority().partial_cmp(&other.priority())
    }
}

impl Ord for ServerCommand {
    fn cmp(&self, other: &ServerCommand) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ServerResponseAction {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerResponseType {
    Receiving,
    Sending,
    Connecting,
}

#[derive(Debug, Clone, Copy)]
pub struct ServerResponse {
    pub action: ServerResponseAction,
    pub response_type: ServerResponseType,
    pub time: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum ServerResponseDecodeError {
    BadLength(usize),
    NotEnoughArguments(u8, usize, usize),   // cmd, expected, found
    IllegalCommand(u8),
    PwmOutOfBounds,
}

impl ServerResponse {
    const fn none(resp_type: ServerResponseType) -> Self {
        ServerResponse { action: ServerResponseAction::None, response_type: resp_type, time: 0 }
    }

    const fn new(action: ServerResponseAction, response_type: ServerResponseType, time: u16) -> Self {
        ServerResponse { action, response_type, time }
    }

    fn decode_from(v: &Vec<u8>) -> Result<Self, ServerResponseDecodeError> {
        log::info!("server response decoding: buffer = {:?}", v);
        if v.len() > 6 {
            return Err(ServerResponseDecodeError::BadLength(v.len()));
        }
        let flag_block_hid = (v[0] & 0x8) != 0;
        let flag_channel_state = (v[0] & 0x4) != 0;
        let flag_value_relative = (v[0] & 0x2) != 0;
        let flag_set_value = (v[0] & 0x1) != 0;
        match (flag_block_hid, 
                            flag_channel_state,
                            flag_value_relative,
                            flag_set_value) {
            (true, false, false, false) => {
                let timeout = u16::from_be_bytes(v[1..=2].try_into().map_err(|_| ServerResponseDecodeError::NotEnoughArguments(v[0], 5, v.len()))?);
                let time = u16::from_be_bytes(v[3..=4].try_into().map_err(|_| ServerResponseDecodeError::NotEnoughArguments(v[0], 5, v.len()))?);
                Ok(ServerResponse::new(ServerResponseAction::BlockHid(timeout), ServerResponseType::Receiving, time))
            },
            (true, false, true, false) => {
                let timeout = i16::from_be_bytes(v[1..=2].try_into().map_err(|_| ServerResponseDecodeError::NotEnoughArguments(v[0], 5, v.len()))?);
                let time = u16::from_be_bytes(v[3..=4].try_into().map_err(|_| ServerResponseDecodeError::NotEnoughArguments(v[0], 5, v.len()))?);
                Ok(ServerResponse::new(ServerResponseAction::BlockHidRelative(timeout), ServerResponseType::Receiving, time))
            },
            (false, false, false, true) => {
                let val = u16::from_be_bytes(v[1..=2].try_into().map_err(|_| ServerResponseDecodeError::NotEnoughArguments(v[0], 6, v.len()))?);
                let channel = v.get(3).ok_or(ServerResponseDecodeError::NotEnoughArguments(v[0], 6, v.len()))?;
                let time = u16::from_be_bytes(v[4..=5].try_into().map_err(|_| ServerResponseDecodeError::NotEnoughArguments(v[0], 6, v.len()))?);
                Ok(ServerResponse::new(
                    ServerResponseAction::SetPwmValue(val, PwmNumber::try_from(*channel).map_err(|_| ServerResponseDecodeError::PwmOutOfBounds)?),
                    ServerResponseType::Receiving,
                    time
                ))
            },
            (false, false, true, true) => {
                let val = i16::from_be_bytes(v[1..=2].try_into().map_err(|_| ServerResponseDecodeError::NotEnoughArguments(v[0], 6, v.len()))?);
                let channel = v.get(3).ok_or(ServerResponseDecodeError::NotEnoughArguments(v[0], 6, v.len()))?;
                let time = u16::from_be_bytes(v[4..=5].try_into().map_err(|_| ServerResponseDecodeError::NotEnoughArguments(v[0], 6, v.len()))?);
                Ok(ServerResponse::new(
                    ServerResponseAction::SetPwmValueRelative(val, PwmNumber::try_from(*channel).map_err(|_| ServerResponseDecodeError::PwmOutOfBounds)?),
                    ServerResponseType::Receiving,
                    time
                ))
            },
            (false, true, false, false) => {
                let channel = PwmNumber::try_from(*v.get(1).ok_or(ServerResponseDecodeError::NotEnoughArguments(v[0], 2, v.len()))?).map_err(|_| ServerResponseDecodeError::PwmOutOfBounds)?;
                match v[0] & 0xf0 {
                    0x80 => Ok(ServerResponse::new(ServerResponseAction::ToggleChannel(channel), ServerResponseType::Receiving, 0)),
                    0x40 => Ok(ServerResponse::new(ServerResponseAction::ChannelOff(channel), ServerResponseType::Receiving, 0)),
                    0x20 => Ok(ServerResponse::new(ServerResponseAction::ChannelOn(channel), ServerResponseType::Receiving, 0)),
                    0x10 => Ok(ServerResponse::new(ServerResponseAction::GetState(channel), ServerResponseType::Receiving, 0)),
                    _ => Err(ServerResponseDecodeError::IllegalCommand(v[0]))
                }
            },
            _ => Err(ServerResponseDecodeError::IllegalCommand(v[0]))
        }
    }
}

pub struct ServerConnection {
    pub server_ip: IpAddress,
    pub server_port: u16,
    thread_listening_enabled: Arc<AtomicBool>,
}

#[derive(Debug)]
pub enum ServerError {
    ConnectError(ConnectError),
    TcpError(tcp::Error),
    EOFWhileReading,
}
impl core::error::Error for ServerError { }
impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConnectError(c) => write!(f, "SERVER ERROR: ConnectError {}", match c {
                ConnectError::InvalidState => "Invalid State",
                ConnectError::ConnectionReset => "Connection Reset",
                ConnectError::TimedOut => "Timed Out",
                ConnectError::NoRoute => "No Route",
            }),
            Self::TcpError(e) => write!(f, "SERVER ERROR: Tcp Error during read: {}", e),
            Self::EOFWhileReading => write!(f, "SERVER ERROR: EOF while reading port! port closing..."),
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

static COMMAND_CHANNEL: PriorityChannel<CriticalSectionRawMutex, ServerCommand, Max, 8> = PriorityChannel::new();
static RESPONSE_CHANNEL: Channel<CriticalSectionRawMutex, Result<ServerResponse, ServerError>, 8> = Channel::new();
// static THREAD_STATE: Mutex<CriticalSectionRawMutex, SocketThreadState> = Mutex::new();

impl ServerConnection {
    const RX_BUF_LEN: usize = 256;
    const TX_BUF_LEN: usize = 256;

    pub fn new(spawner: Spawner, netstack: Stack<'static>) -> Self {
        let thread_listening_enabled: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        spawner.spawn(socket_runner_task(COMMAND_CHANNEL.receiver(), RESPONSE_CHANNEL.sender(), netstack, thread_listening_enabled.clone()).unwrap());

        ServerConnection {
            // TODO: remove this and turn it back to 0s everywhere
            server_ip: IpAddress::v4(192,168,0,138),
            server_port: 3939,
            thread_listening_enabled,
        }
    }

    pub async fn connect(&mut self) -> Result<ServerResponse, ServerError> {
        log::info!("connecting at {}, {}", self.server_ip, self.server_port);
        COMMAND_CHANNEL.send(ServerCommand::Connect(self.server_ip, self.server_port)).await;
        RESPONSE_CHANNEL.receive().await
    }

    pub async fn disconnect(&mut self) {
        COMMAND_CHANNEL.send(ServerCommand::Disconnect).await;
    }

    pub fn enable_receive_packets(&self) {
        self.thread_listening_enabled.store(true, AOrdering::Relaxed);
    }

    pub fn disable_receive_packets(&self) {
        self.thread_listening_enabled.store(false, AOrdering::Relaxed);
    }

    pub async fn receive(&self) -> Result<ServerResponse, ServerError> {
        // if !self.thread_listening_enabled.load(AOrdering::Relaxed) {
        //     COMMAND_CHANNEL.send(ServerCommand::Listen).await;
        // }
        log::error!("receiving from server. len: {}. first elem: {:?} awaiting response...", COMMAND_CHANNEL.len(), COMMAND_CHANNEL.try_peek());
        loop {
            let packet = RESPONSE_CHANNEL.receive().await;
            if let 
                Ok(ServerResponse { response_type: ServerResponseType::Receiving, .. }) |
                Err(ServerError::EOFWhileReading) |
                Err(ServerError::TcpError(_)) = packet {
                return packet;
            }
            log::error!("received unexpected other packet while listening!");
        }
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<ServerResponse, ServerError> {
        log::error!("placing command on buffer.");
        COMMAND_CHANNEL.send(ServerCommand::Send(data)).await;
        log::error!("placed command on buffer. len: {}. first elem: {:?} awaiting response...", COMMAND_CHANNEL.len(), COMMAND_CHANNEL.try_peek());
        RESPONSE_CHANNEL.receive().await
        // Ok(ServerResponse::none())
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

#[derive(Debug)]
pub enum SocketThreadState {
    Listening,
    Other,
}

#[embassy_executor::task]
async fn socket_runner_task(
    commands: PReceiver<'static, CriticalSectionRawMutex, ServerCommand, Max, 8>, 
    responses: Sender<'static, CriticalSectionRawMutex, Result<ServerResponse, ServerError>, 8>, 
    netstack: Stack<'static>,
    thread_listening_enabled: Arc<AtomicBool>) {
    let mut rx_buffer = [0u8; ServerConnection::RX_BUF_LEN];
    let mut tx_buffer = [0u8; ServerConnection::TX_BUF_LEN];
    let mut conn = TcpSocket::new(
        netstack, &mut rx_buffer, &mut tx_buffer,
    );
    loop {
        let read_next_command = async { 
            log::error!("awaiting commands..."); 
            let ret = commands.receive().await;
            log::error!("cmd received: {:?}", ret); 
            ret
        };
        let read_next_socket_message = async {
            if conn.local_endpoint().is_some() {
                let mut buf: Vec<u8> = vec![0;READ_BUF_SIZE];
                wait_until_truly_readable(&conn).await;
                conn.wait_read_ready().await;
                let res = conn.read(&mut buf).await;
                (res, buf)
            } else {
                pending().await
            }
        };
        match select(read_next_command, read_next_socket_message).await {
            Either::First(ServerCommand::Connect(addr, port)) => {
                let res = conn.connect(IpEndpoint::new(addr, port)).await;
                let response = match res {
                    Ok(_) => Ok(ServerResponse::none(ServerResponseType::Connecting)),
                    Err(e) => Err(e.into()),
                };
                responses.send(response).await;
            },
            Either::First(ServerCommand::Disconnect) => {
                conn.close();
            },
            Either::First(ServerCommand::Listen) => {
                log::info!("unused now!");
            },
            Either::First(ServerCommand::Send(data)) => {
                log::error!("thread entered sending");
                if conn.local_endpoint().is_some() {
                    log::error!("endpoint exists");
                    let res = async {
                        log::error!("writing into connection: {:?}", data);
                        let sz = conn.write(&data).await?;
                        log::error!("socket flushing: {:?}", data);
                        conn.flush().await?;
                        log::error!("socket flushed.");
                        Ok(sz)
                    }.await;
                    let response = match res {
                        Ok(_size) => Ok(ServerResponse::none(ServerResponseType::Sending)),
                        Err(e) => Err(ServerError::TcpError(e)),
                    };
                    responses.send(response).await;
                }
            },
            Either::Second((result, mut buf)) => {
                if thread_listening_enabled.load(AOrdering::Relaxed) {
                    let response = match result {
                        Ok(0) => {
                            log::error!("received no bytes: state {}, may_send {}", conn.state(), conn.may_send());
                            conn.close();
                            Err(ServerError::EOFWhileReading)
                        },
                        Ok(size) => {
                            buf.truncate(size);
                            // TODO: WILL INTRODUCE BUGS IF BUFFER IS TOO SMALL. LOOP READING
                            // FOR CORRECTNESS!
                            match ServerResponse::decode_from(&buf) {
                                Ok(resp) => Ok(resp),
                                Err(e) => {
                                    log::error!("received error: {:?}. Ignoring packet.", e);
                                    Ok(ServerResponse::none(ServerResponseType::Receiving))
                                }
                            }
                            // Ok(ServerResponse::decode_from(&buf))
                        },
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