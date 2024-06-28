use crate::error::Error;
use crate::node_type::Offset;
use crate::page::Page;
use crate::page_layout::PAGE_SIZE;
use std::io::Write;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use wasi::filesystem::{self, types::{Descriptor, DescriptorFlags, OpenFlags, PathFlags}};


pub struct Pager {
    file: Descriptor,
    curser: usize,
}

impl Pager {
    pub fn new(path: &Descriptor) -> Result<Pager, Error> {
        let file = path
            .open_at(
                PathFlags::empty(),
                "pager",
                OpenFlags::CREATE,
                DescriptorFlags::READ | DescriptorFlags::WRITE,
            )
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?;

        Ok(Pager {
            file,
            curser: 0,
        })
    }

    pub fn get_page(&mut self, offset: &Offset) -> Result<Page, Error> {
        let mut page: [u8; PAGE_SIZE] = [0x00; PAGE_SIZE];
        let stream =  self.file.read_via_stream(offset.0 as u64)
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?;
        let read_page_bytes = stream.blocking_read(PAGE_SIZE as u64)
            .map_err(|err| Error::FilesystemError(err.to_string()))?;
        drop(stream);
        page.clone_from_slice(&read_page_bytes);
        Ok(Page::new(page))
    }

    pub fn write_page(&mut self, page: Page) -> Result<Offset, Error> {
        let stream = self.file.write_via_stream(self.curser as u64)
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?;
        stream.blocking_write_and_flush(&page.get_data())
            .map_err(|err| Error::FilesystemError(err.to_string()))?;
        drop(stream);
        let res = Offset(self.curser);
        self.curser += PAGE_SIZE;
        Ok(res)
    }

    pub fn write_page_at_offset(&mut self, page: Page, offset: &Offset) -> Result<(), Error> {
        let stream = self.file.write_via_stream(offset.0 as u64)
            .map_err(|err| Error::FilesystemError(err.name().to_string()))?;
        stream.blocking_write_and_flush(&page.get_data())
            .map_err(|err| Error::FilesystemError(err.to_string()))?;
        drop(stream);
        Ok(())
    }
}
