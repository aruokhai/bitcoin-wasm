use crate::error::Error;
use crate::node_type::Offset;
use crate::page_layout::PTR_SIZE;
use core::slice;
use std::convert::TryFrom;
use wasi::filesystem::{self, types::{Descriptor, DescriptorFlags, OpenFlags, PathFlags}};
use std::io::{Read, Seek, SeekFrom, Write};


pub struct Wal {
    file: Descriptor,
}

impl Wal {
    pub fn new() -> Result<Self, Error> {
        let preopens = filesystem::preopens::get_directories();
        let (dir, _) = &preopens[0];
        let file = dir
            .open_at(
                PathFlags::empty(),
                "wal",
                OpenFlags::CREATE,
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            )
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?;

        Ok(Self { file })
    }

    pub fn get_root(&mut self) -> Result<Offset, Error> {
        let mut buff: [u8; PTR_SIZE] = [0x00; PTR_SIZE];
        let file_len = self.file.stat()
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?.size;
        let mut root_offset: usize = 0;
        if file_len > 0 {
            root_offset = (file_len as usize / PTR_SIZE - 1) * PTR_SIZE;
        }
        let stream =  self.file.read_via_stream(root_offset as u64)
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?;
        let read_page_bytes = stream.blocking_read(PTR_SIZE as u64)
            .map_err(|err| Error::FilesystemError(err.to_string()))?;
        drop(stream);
        buff.clone_from_slice(read_page_bytes.as_slice());
        Offset::try_from(buff)        
    }

    pub fn set_root(&mut self, offset: Offset) -> Result<(), Error> {
        let file_len = self.file.stat()
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?.size;
        let stream = self.file.write_via_stream(file_len as u64)
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?;
        stream.blocking_write_and_flush(&offset.0.to_be_bytes())
            .map_err(|err| Error::FilesystemError(err.to_string()))?;
        drop(stream);
        Ok(())
    }
}
