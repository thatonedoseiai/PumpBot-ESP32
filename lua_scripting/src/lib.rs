#![no_std]
#![no_main]
#![feature(iterator_try_collect)]
#![feature(allocator_api)]

extern crate alloc;

use flash_storage::PbFlashStorage;
use littlefs2::io;
use littlefs2::path;
use littlefs2::path::Path;
use littlefs2::fs::{Filesystem, DirEntry};
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use core::error::Error;
use core::fmt;
// use heapless::vec::Vec;
// use piccolo::{io::buffered_read, Closure, Executor, Lua};

#[derive(Debug)]
pub enum LuaScriptError {
    IOErr(io::Error),
    PathErr(path::Error),
}

impl Error for LuaScriptError { }
impl fmt::Display for LuaScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IOErr(e) => write!(f, "[LUA] IO error: {}", e.code()),
            Self::PathErr(e) => write!(f, "[LUA] Path error: {:?}", e),
        }
    }
}

impl From<io::Error> for LuaScriptError {
    fn from(val: io::Error) -> Self {
        Self::IOErr(val)
    }
}

impl From<path::Error> for LuaScriptError {
    fn from(val: path::Error) -> Self {
        Self::PathErr(val)
    }
}

// type File<'a, 'b, 'c> = littlefs2::fs::File<'a, 'b, PbFlashStorage<'c>>;
pub struct ScriptManager {
    fs: Arc<Filesystem<'static, PbFlashStorage<'static>>>,
}

const SCRIPTS_DIR: &Path = path!("/scripts/");

impl ScriptManager {
    pub fn get_scripts(&self) -> Result<Vec<DirEntry>, LuaScriptError> {
        Ok(self.fs.read_dir_and_then(SCRIPTS_DIR, |c| {
            c.try_collect()
        })?)
    }

    pub fn run_script(&self, name: &str) -> Result<(), LuaScriptError> {
        let full_name = SCRIPTS_DIR.join(Path::from_str_with_nul(&(String::from(name) + "\0"))?);
        let mut script_buf: heapless::vec::Vec<u8, 512> = heapless::vec::Vec::new();
        // let mut lua = Lua::full();
        Ok(self.fs.open_file_and_then(&full_name, |script| {
            script.read_to_end(&mut script_buf)?;
            Ok(())
        })?)
    }
}
