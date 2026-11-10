use std::{
    fmt::Debug,
    fs,
    io::{self, Write},
    path::Path,
    time::SystemTime,
};

use rook_platform::{
    Platform,
    clipboard::Clipboard,
    credentials::Credentials,
    files::{Entry, Files},
    network::{Network, Request, Response},
    parallel::Serial,
    threads::Threads,
};

#[derive(Debug)]
struct AndroidFiles;

impl Files for AndroidFiles {
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        fs::read(path)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::write(path, bytes)
    }

    fn create(&self, path: &Path) -> io::Result<Box<dyn Write + Send>> {
        Ok(Box::new(fs::File::create(path)?))
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<Entry>> {
        Ok(fs::read_dir(dir)?
            .flatten()
            .filter_map(|entry| {
                let metadata = entry.metadata().ok()?;
                Some(Entry {
                    path: entry.path(),
                    is_file: metadata.is_file(),
                    modified: metadata.modified()
                        .unwrap_or(SystemTime::UNIX_EPOCH),
                    len: metadata.len(),
                })
            })
            .collect())
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

#[derive(Debug)]
struct AndroidClipboard;

impl Clipboard for AndroidClipboard {
    fn text(&self) -> Option<String> {
        None
    }

    fn set_text(&self, _text: &str) {}
}

#[derive(Debug)]
struct AndroidCredentials;

impl Credentials for AndroidCredentials {
    fn get(&self, _name: &str) -> io::Result<Option<String>> {
        Ok(None)
    }

    fn set(&self, _name: &str, _secret: &str) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Android credential storage is not implemented",
        ))
    }

    fn clear(&self, _name: &str) -> io::Result<bool> {
        Ok(false)
    }
}

#[derive(Debug)]
struct AndroidNetwork;

impl Network for AndroidNetwork {
    fn send(&self, _request: &Request<'_>) -> Result<Response, String> {
        Err("Android network backend is not implemented yet".into())
    }
}

#[derive(Debug)]
struct AndroidThreads;

impl Threads for AndroidThreads {
    fn spawn(
        &self,
        name: &str,
        body: Box<dyn FnOnce() + Send>,
    ) -> io::Result<()> {
        std::thread::Builder::new()
            .name(name.into())
            .spawn(body)
            .map(drop)
    }
}

pub fn install() -> Result<(), Platform> {
    rook_platform::install(Platform {
        files: Box::new(AndroidFiles),
        workers: Box::new(Serial),
        threads: Box::new(AndroidThreads),
        network: Box::new(AndroidNetwork),
        clipboard: Box::new(AndroidClipboard),
        credentials: Box::new(AndroidCredentials),
    })
}