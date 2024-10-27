use crate::error::Error;
use crate::node_type::Offset;
use crate::page_layout::PTR_SIZE;
use std::convert::TryFrom;
use wasi::filesystem::{types::{Descriptor, DescriptorFlags, OpenFlags, PathFlags}};


pub struct Wal {
    file: Descriptor,
}

impl Wal {
    pub fn new(path: &Descriptor) -> Result<Self, Error> {
        let file = path
            .open_at(
                PathFlags::empty(),
                "wal",
                OpenFlags::CREATE,
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            )
            .map_err(|err| Error::FilesystemError(err as u8))?;

        Ok(Self { file })
    }


    pub fn get_root(&mut self) -> Result<Offset, Error> {
        let mut buff: [u8; PTR_SIZE] = [0x00; PTR_SIZE];
        let file_len = self.file.stat()
            .map_err(|err| Error::FilesystemError(err as u8))?.size;
        let mut root_offset: usize = 0;
        if file_len > 0 {
            root_offset = (file_len as usize / PTR_SIZE - 1) * PTR_SIZE;
        }
        let stream =  self.file.read_via_stream(root_offset as u64)
            .map_err(|err| Error::FilesystemError(err as u8))?;
        let read_page_bytes = stream.blocking_read(PTR_SIZE as u64)
            .map_err(|_| Error::StreamError)?;
        drop(stream);
        buff.clone_from_slice(read_page_bytes.as_slice());
        Offset::try_from(buff)        
    }

    pub fn set_root(&mut self, offset: Offset) -> Result<(), Error> {
        let file_len = self.file.stat()
            .map_err(|err| Error::FilesystemError(err as u8))?.size;
        let stream = self.file.write_via_stream(file_len)
            .map_err(|err| Error::FilesystemError(err as u8))?;
        stream.blocking_write_and_flush(&offset.0.to_be_bytes())
            .map_err(|_| Error::StreamError)?;
        drop(stream);
        Ok(())
    }
}
