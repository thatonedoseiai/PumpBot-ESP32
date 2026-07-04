//! This submodule defines the functionality of the HTTP server. 

// use esp_idf_svc::{
//     http::{server, Method},
//     http::server::{
//         EspHttpServer,
//         Request,
//         EspHttpConnection,
//     }
// };
// use esp_idf_svc::hal::io::EspIOError;
// use esp_idf_svc::sys::EspError;
use global_settings::PbGlobalSettings;

use std::fs;

use core::fmt;
use core::error::Error;
use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::sync::mpsc::{channel, Receiver, Sender, RecvError, SendError};
use crate::PbWifi;

// const STACK_SIZE: usize = 4096;

/// Represents an OPEN instance of the http server. When dropped, the server closes.
pub struct PbHttpServer<'a> {
    serv: EspHttpServer<'a>,
    // settings: Arc<Mutex<PbGlobalSettings>>,
    cmds_incoming: Receiver<PbHttpServerCommands>,
    response_outgoing: Sender<PbHttpServerResponse>,
}

/// All the possible commands the http server can send to the main thread
pub enum PbHttpServerCommands {
    GetLanguage,
    GetWifi,
    GetSettings,
    SetSettings(String),
}

/// All the possible responses the main thread could send back to the user through http
pub enum PbHttpServerResponse {
    Language(u8),
    WifiList(Vec<String>),
    Settings(String),
    SetSettingsSuccess,
}

/// Represents an error that the http server can throw.
#[derive(Debug)]
pub enum PbHttpServerError {
    BoardIO(EspIOError),
    FsIO(std::io::Error),
    Esp(EspError),
    ArcReclaim,
    MutexError(PoisonError<PbGlobalSettings>),
    UseAfterReclaim,
    LockError,
    RecvError(RecvError),
    BadResponse,
    ResponseError(SendError<PbHttpServerResponse>),
    CommandError(SendError<PbHttpServerCommands>),
}

impl From<EspIOError> for PbHttpServerError {
    fn from(val: EspIOError) -> Self {
        PbHttpServerError::BoardIO(val)
    }
}

impl From<std::io::Error> for PbHttpServerError {
    fn from(val: std::io::Error) -> Self {
        PbHttpServerError::FsIO(val)
    }
}

impl From<EspError> for PbHttpServerError {
    fn from(val: EspError) -> Self {
        PbHttpServerError::Esp(val)
    }
}

impl From<PoisonError<PbGlobalSettings>> for PbHttpServerError {
    fn from(val: PoisonError<PbGlobalSettings>) -> Self {
        PbHttpServerError::MutexError(val)
    }
}

impl From<RecvError> for PbHttpServerError {
    fn from(val: RecvError) -> Self {
        PbHttpServerError::RecvError(val)
    }
}

impl From<SendError<PbHttpServerResponse>> for PbHttpServerError {
    fn from(val: SendError<PbHttpServerResponse>) -> Self {
        PbHttpServerError::ResponseError(val)
    }
}

impl From<SendError<PbHttpServerCommands>> for PbHttpServerError {
    fn from(val: SendError<PbHttpServerCommands>) -> Self {
        PbHttpServerError::CommandError(val)
    }
}

impl fmt::Display for PbHttpServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PbHttpServerError::BoardIO(e) => write!(f, "HTTP: board IO error: {}", e),
            PbHttpServerError::FsIO(e) => write!(f, "HTTP: LittleFS IO error: {}", e),
            PbHttpServerError::Esp(e) => write!(f, "HTTP: Esp error: {}", e),
            PbHttpServerError::ArcReclaim => write!(f, "HTTP: Arc containing settings data could not be reclaimed succesfully!"),
            PbHttpServerError::MutexError(e) => write!(f, "HTTP: Mutex Error: {}", e),
            PbHttpServerError::UseAfterReclaim => write!(f, "HTTP: Attempted to access PB settings after settings struct has been reclaimed! Note: this should not be possible because callbacks shouldn't happen when HTTP server struct has been dropped."),
            PbHttpServerError::LockError => write!(f, "Could not lock the PB settings struct!"),
            PbHttpServerError::RecvError(e) => write!(f, "Receiver error: {}", e),
            PbHttpServerError::CommandError(e) => write!(f, "Command Sender error: {}", e),
            PbHttpServerError::ResponseError(e) => write!(f, "Response Sender error: {}", e),
            PbHttpServerError::BadResponse => write!(f, "Bad response received from command thread!"),
        }
    }
}

impl Error for PbHttpServerError { }

impl PbHttpServer<'_> {
    /// Creates a new instance of the open server - in other words, opens the http server to
    /// receive new connections. If the network is not connected, this will fail.
    pub fn start() -> Result<Self, PbHttpServerError> {
        let server_configuration = server::Configuration {
            // stack_size: STACK_SIZE,
            ..Default::default()
        };
        let mut serv = EspHttpServer::new(&server_configuration)?;
        let (cmd_in, cmd_out) = channel();
        let (resp_in, resp_out) = channel();
        let result_listener = Arc::new(Mutex::new(resp_out));
        // let settings = Arc::new(Mutex::new(owned_settings));
        // let weak_settings = Arc::downgrade(&settings);
        serv.fn_handler("/browse", Method::Get, PbHttpServer::download_get_handler)?;
        let lang_cmd = cmd_in.clone();
        let lang_res = result_listener.clone();
        serv.fn_handler("/get_language", Method::Get, move |r| { PbHttpServer::get_language_handler(lang_cmd.clone(), lang_res.clone(), r) })?;
        let wifi_cmd = cmd_in.clone();
        let wifi_res = result_listener.clone();
        serv.fn_handler("/get_wifi", Method::Get, move |r| {PbHttpServer::get_wifi_handler(wifi_cmd.clone(), wifi_res.clone(), r)})?;
        let set_cmd = cmd_in.clone();
        let set_res = result_listener.clone();
        serv.fn_handler("/get_settings", Method::Get, move |r| { PbHttpServer::get_settings_handler(set_cmd.clone(), set_res.clone(), r) })?;
        serv.fn_handler("/set_settings", Method::Post, move |r| { PbHttpServer::set_settings_handler(cmd_in.clone(), result_listener.clone(), r) })?;

        Ok( PbHttpServer { serv, cmds_incoming: cmd_out, response_outgoing: resp_in })
    }

    /// Responds to requests using owned instances of the necessary peripherals. Must be called on
    /// the main thread because sending these peripherals across threads is nasty nasty business.
    pub fn update(&self, settings: &mut PbGlobalSettings, wifi_driver: &mut PbWifi) -> Result<(), PbHttpServerError> {
        if let Ok(cmd) = self.cmds_incoming.try_recv() {
            match cmd {
                PbHttpServerCommands::GetLanguage => {
                    self.response_outgoing.send(PbHttpServerResponse::Language(settings.lang.into()))?;
                }
                PbHttpServerCommands::GetWifi => {
                    let wifis = wifi_driver.get_wifis()?;
                    self.response_outgoing.send(PbHttpServerResponse::WifiList(wifis.into_iter().map(|w| w.ssid.to_string()).collect()))?;
                }
                PbHttpServerCommands::GetSettings => {
                    let (ssid, pass) = wifi_driver.currently_connected()?;
                    self.response_outgoing.send(PbHttpServerResponse::Settings(format!("{},{},{}", ssid, pass, settings.to_string())))?;
                }
                PbHttpServerCommands::SetSettings(instr) => { 
                    let mut split = instr.split('&');
                    if let (Some(new_ssid), Some(new_pass)) = (split.next(), split.next()) {
                        let (ssid, pass) = wifi_driver.currently_connected()?;
                        if new_ssid[3..] != ssid || new_pass[4..] != pass {
                            wifi_driver.connect(new_ssid.try_into().unwrap(), new_pass.try_into().unwrap())?;
                        }
                    }
                    if let Some(rest) = split.remainder() {
                        settings.set_from_string(rest.to_string());
                        self.response_outgoing.send(PbHttpServerResponse::SetSettingsSuccess)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// show either: the contents of a directory, or the contents of a file.
    fn download_get_handler(request: Request<&mut EspHttpConnection>) -> Result<(), PbHttpServerError> {
        let mut requested_path = "/fs/".to_string();
        requested_path.push_str(&request.uri()[8..]);

        if fs::metadata(&requested_path)?.is_dir() {
            let directory_contents = fs::read_dir(requested_path)?
                .map(|d| d.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()?;
            let files = if directory_contents.is_empty() {
                "No files".to_string()
            } else {
                format!("{:?}", directory_contents)
            };
            request.into_ok_response()?.write(files.as_bytes())?;
        } else {
            let file_contents = fs::read(requested_path)?;
            request.into_ok_response()?.write(&file_contents)?;
        }

        Ok(())
    }

    /// Send the language currently being used as a number.
    fn get_language_handler(cmd_out: Sender<PbHttpServerCommands>, resp_in: Arc<Mutex<Receiver<PbHttpServerResponse>>>, request: Request<&mut EspHttpConnection>) -> Result<(), PbHttpServerError> {
        cmd_out.send(PbHttpServerCommands::GetLanguage)?;
        let resp = resp_in.lock().map_err(|_| PbHttpServerError::LockError)?;
        let response: PbHttpServerResponse = resp.recv()?;
        if let PbHttpServerResponse::Language(linx) = response {
            request.into_ok_response()?.write(&[linx + ('0' as u8)])?;
            Ok(())
        } else {
            Err(PbHttpServerError::BadResponse)
        }
    }

    /// Get a comma-separated list of all wifis currently detected by PB.
    fn get_wifi_handler(cmd_out: Sender<PbHttpServerCommands>, resp_in: Arc<Mutex<Receiver<PbHttpServerResponse>>>, request: Request<&mut EspHttpConnection>) -> Result<(), PbHttpServerError> {
        cmd_out.send(PbHttpServerCommands::GetWifi)?;
        let resp = resp_in.lock().map_err(|_| PbHttpServerError::LockError)?;
        let response: PbHttpServerResponse = resp.recv()?;
        if let PbHttpServerResponse::WifiList(wifis) = response {
            request.into_ok_response()?.write(wifis.join(",").as_bytes())?;
            Ok(())
        } else {
            Err(PbHttpServerError::BadResponse)
        }
    }

    fn get_settings_handler(cmd_out: Sender<PbHttpServerCommands>, resp_in: Arc<Mutex<Receiver<PbHttpServerResponse>>>, request: Request<&mut EspHttpConnection>) -> Result<(), PbHttpServerError> {
        cmd_out.send(PbHttpServerCommands::GetSettings)?;
        let resp = resp_in.lock().map_err(|_| PbHttpServerError::LockError)?;
        let response: PbHttpServerResponse = resp.recv()?;
        if let PbHttpServerResponse::Settings(s) = response {
            request.into_ok_response()?.write(s.as_bytes())?;
            Ok(())
        } else {
            Err(PbHttpServerError::BadResponse)
        }
    }

    fn set_settings_handler(cmd_out: Sender<PbHttpServerCommands>, resp_in: Arc<Mutex<Receiver<PbHttpServerResponse>>>, mut request: Request<&mut EspHttpConnection>) -> Result<(), PbHttpServerError> {
        let mut post_contents = [0u8;128];
        let bytes_read = request.read(&mut post_contents)?;
        let value = String::from_utf8_lossy(&post_contents[..bytes_read]);
        cmd_out.send(PbHttpServerCommands::SetSettings(value.to_string()))?;
        let resp = resp_in.lock().map_err(|_| PbHttpServerError::LockError)?;
        let response: PbHttpServerResponse = resp.recv()?;
        if let PbHttpServerResponse::SetSettingsSuccess = response {
            request.into_ok_response()?.write(b"OK")?;
            Ok(())
        } else {
            request.into_status_response(521)?;
            Err(PbHttpServerError::BadResponse)
        }
    }
}