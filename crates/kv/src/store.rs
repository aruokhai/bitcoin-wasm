use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use wasi::filesystem;
use wasi::filesystem::types::{Descriptor, DescriptorFlags, OpenFlags, PathFlags};

use crate::errors::Error;

pub trait Store {
    fn append(&mut self, bytes: &[u8]) -> Result<i64, Error>;
    fn read(&self, offset: i64, size: u32) -> Result<Vec<u8>, Error>;
    fn read_full(&self) -> Result<Vec<u8>, Error>;
    fn size_in_bytes(&self) -> i64;
    fn sync(&self);

}

pub struct WasiStore {
    file_descriptor: Descriptor,
    current_write_offset: i64,
}

impl WasiStore {
    pub fn new(file_path: &str) -> Result<Self, Error> {
        let (director_descriptor, _) = &filesystem::preopens::get_directories()[0];
        let file_descriptor = director_descriptor
            .open_at(
                PathFlags::empty(),
                file_path,
                OpenFlags::CREATE,
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            ).map_err(|_| Error::OpenFileError)?;
           
        Ok(WasiStore {
            file_descriptor,
            current_write_offset: 0,
        })
    }

    pub fn reload(file_path: &str) -> Result<Self, Error> {
        let (director_descriptor, _) = &filesystem::preopens::get_directories()[0];
        let file_descriptor = director_descriptor
            .open_at(
                PathFlags::empty(),
                file_path,
                OpenFlags::CREATE,
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            ).map_err(|_| Error::OpenFileError)?;
           
        Ok(WasiStore {
            file_descriptor,
            current_write_offset: 0,
        })
    }
}

impl Store for WasiStore {
    fn append(&mut self, bytes: &[u8]) -> Result<i64, Error> {
        let offset = self.file_descriptor.write(bytes,self.current_write_offset as u64)
            .map_err(|_| Error::OpenFileError)?;
        self.current_write_offset += bytes.len() as i64;
        Ok(offset as i64)
    }

    fn read(&self, offset: i64, size: u32) -> Result<Vec<u8>, Error> {
        let (buffer,   _) =  self.file_descriptor.read(size as u64, offset as u64)
            .map_err(|err| Error::OpenFileError)?;
        Ok(buffer)
    }

    fn read_full(&self) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        let mut stream =  self.file_descriptor.read_via_stream(0 as u64)
            .map_err(|err| Error::OpenFileError)?;
        stream.read_to_end(&mut buffer)
            .map_err(|_| Error::StreamError)?;
        drop(stream);
        return Ok(buffer)
    }

    fn size_in_bytes(&self) -> i64 {
        self.current_write_offset
    }

    fn sync(&self) {
        self.file_descriptor.sync();
    }

}