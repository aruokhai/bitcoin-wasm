use std::{any::Any, collections::BTreeMap, fs::{File, OpenOptions}};

use bdk::{bitcoin::Script, database::BatchOperations};
use wasi::filesystem::{self, types::{Descriptor, DescriptorFlags, OpenFlags, PathFlags}};


pub struct Store {
    magic_len: usize,
    db_file: Descriptor,
    map: BTreeMap<Vec<u8>, Box<dyn Any + Send + Sync>>,
    deleted_keys: Vec<Vec<u8>>,
}


pub enum StoreError {
    FilesystemError(String),
    InvalidMagicBytes,
    
}

impl Store {

    fn create(magic: &[u8], file_path: &str) -> Result<Self, StoreError> {
        let preopens = filesystem::preopens::get_directories();
        let (dir, _) = &preopens[0];

        let file = dir
            .open_at(
                PathFlags::empty(),
                file_path,
                OpenFlags::CREATE,
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            )
            .map_err(|err| StoreError::FilesystemError(err.name().to_string()))?;
        let stream = file.write_via_stream(0)
            .map_err(|err| StoreError::FilesystemError(err.name().to_string()))?;
        stream.blocking_write_and_flush(magic)
            .map_err(|err| StoreError::FilesystemError(err.name().to_string()))?;
        drop(stream);
        Ok(Store{ 
            magic_len:  magic.len(),
            db_file: file,
            map: BTreeMap::new(),
            deleted_keys: BTreeMap::new()
        })
        
    }

    pub fn open(magic: &[u8], file_path: &str) -> Result<Self, StoreError> {
        let preopens = filesystem::preopens::get_directories();
        let (dir, _) = &preopens[0];
        let file = dir
            .open_at(
                PathFlags::empty(),
                file_path,
                OpenFlags::empty(),
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            )
        .map_err(|err| StoreError::FilesystemError(err.name().to_string()))?;
        let stream =  file.read_via_stream(0)
            .map_err(|err| StoreError::FilesystemError(err.name().to_string()))?;
        let read_magic_bytes = stream.blocking_read(magic.len())
            .map_err(|err| StoreError::FilesystemError(err.name().to_string()))?;
        let mut f = OpenOptions::new().read(true).write(true).open(file_path)?;
        if read_magic_bytes != magic.to_vec() {
            return Err(StoreError::InvalidMagicBytes);
        }

        Ok(Self {
            magic_len: magic.len(),
            db_file: file,
            map: BTreeMap::new(),
            deleted_keys: BTreeMap::new()
        })
    }

    
    pub fn open_or_create_new(magic: &[u8], file_path: &str) -> Result<Self, StoreError> {
        let preopens = filesystem::preopens::get_directories();
        let (dir, _) = &preopens[0];
        if dir.metadata_hash_at(PathFlags::empty(), file_path).is_err() {
            Self::create_new(magic, file_path)
        }
         else {
            Self::open(magic, file_path)
        }
    }

}


