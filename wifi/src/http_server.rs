//! This submodule defines the functionality of the HTTP server. 

use esp_idf_svc::{
    http::{server, Method},
    http::server::{
        EspHttpServer,
        Request,
        EspHttpConnection,
    }
};
use esp_idf_svc::hal::io::EspIOError;
use esp_idf_svc::sys::EspError;
use std::fmt;
use std::fs;
use std::error::Error;

// const STACK_SIZE: usize = 4096;

/// Represents an OPEN instance of the http server. When dropped, the server closes.
pub struct PbHttpServer<'a> {
    serv: EspHttpServer<'a>,
}

/// Represents an error that the http server can throw.
#[derive(Debug)]
pub enum PbHttpServerError {
    BoardIO(EspIOError),
    FsIO(std::io::Error),
    Esp(EspError),
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

impl fmt::Display for PbHttpServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PbHttpServerError::BoardIO(e) => write!(f, "board IO error: {}", e),
            PbHttpServerError::FsIO(e) => write!(f, "LittleFS IO error: {}", e),
            PbHttpServerError::Esp(e) => write!(f, "Esp error: {}", e),
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
        serv.fn_handler("/browse", Method::Get, PbHttpServer::download_get_handler)?;

        Ok( PbHttpServer { serv })
    }

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
}