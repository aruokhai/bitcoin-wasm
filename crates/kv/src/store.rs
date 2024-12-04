use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;

use wasi::filesystem;
use wasi::filesystem::types::{Descriptor, DescriptorFlags, OpenFlags, PathFlags};

use crate::errors::Error;
use crate::segment::{SEGMENT_FILE_PREFIX, SEGMENT_FILE_SUFFIX};

pub trait Store: Clone   {
    fn append(&mut self, bytes: &[u8]) -> Result<i64, Error>;
    fn read(&self, offset: i64, size: u32) -> Result<Vec<u8>, Error>;
    fn read_full(&self) -> Result<Vec<u8>, Error>;
    fn size_in_bytes(&self) -> i64;
    fn sync(&self);
    fn get_files(directory_path: &str )-> Result<Vec<String>, Error>;
    fn open(file_path: &str, directory_path:  &str) -> Result<Self, Error> where Self: Sized ;
    fn remove(&mut self);
}

#[derive(Clone)]
pub struct WasiStore {
    file_descriptor: Arc<Descriptor>,
    current_write_offset: i64,
    directory_path: String,
    file_name: String,
}




impl Store for WasiStore {


    fn open(file_path: &str, directory_path:  &str) -> Result<Self, Error> {
        let (directory_descriptor, _) = &filesystem::preopens::get_directories()[0];
        if let Err(err) =  directory_descriptor
            .create_directory_at(
                directory_path
            ) {
                if err != filesystem::types::ErrorCode::Exist {
                    panic!("{}",err.name().to_string());
                }
            }
            ;
        
        let opened_directory = directory_descriptor.open_at(PathFlags::empty(),
            directory_path,
            OpenFlags::DIRECTORY,
            DescriptorFlags::MUTATE_DIRECTORY)
            .map_err(|err| Error::OpenFileError)?;


        let file_descriptor = opened_directory
            .open_at(
                PathFlags::empty(),
                file_path,
                OpenFlags::CREATE,
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            ).map_err(|_| Error::OpenFileError)?;
           
        Ok(WasiStore {
            file_descriptor: Arc::new(file_descriptor),
            current_write_offset: 0,
            directory_path: directory_path.into(),
            file_name: file_path.into()
        })
    }



    fn append(&mut self, bytes: &[u8]) -> Result<i64, Error> {
        self.file_descriptor.write(bytes,self.current_write_offset as u64)
            .map_err(|_| Error::OpenFileError)?;
        let offset = self.current_write_offset;
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
    
    fn get_files(directory_path: &str) -> Result<Vec<String>, Error> {
        let mut store_files = Vec::new();
        let (directory_descriptor, _) = &filesystem::preopens::get_directories()[0];

        let opened_directory = directory_descriptor.open_at(PathFlags::empty(),
            directory_path,
            OpenFlags::DIRECTORY,
            DescriptorFlags::MUTATE_DIRECTORY)
            .map_err(|_| Error::OpenFileError)?;

        let files = opened_directory.read_directory().unwrap();

        while let Ok(entry_option) = files.read_directory_entry() {
            match entry_option {
                Some(entry) => {
                    store_files.push(entry.name);
                }
                None => {
                    break;
                }
            }
        }
        
        Ok(store_files)
    }

    fn remove(&mut self) {
        let (directory_descriptor, _) = &filesystem::preopens::get_directories()[0];
        
        let opened_directory = directory_descriptor.open_at(PathFlags::empty(),
            self.directory_path.as_str(),
            OpenFlags::DIRECTORY,
            DescriptorFlags::MUTATE_DIRECTORY)
            .map_err(|_| Error::OpenFileError);

        if let Ok(dir) = opened_directory {
            dir.unlink_file_at(&self.file_name);
        }
    }
    

}
