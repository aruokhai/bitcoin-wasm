// A buffer that can be:
// * rolled back and re-read from last commit
// * read ahead without moving read position
// * advance position
pub struct Buffer {
    // a deque of chunks
    chunks: VecDeque<Vec<u8>>,
    // pos.0 - current chunk
    // pos.1 - position to read next in the current chunk
    pos: (usize, usize),
    // a copy of pos at last checkpoint call
    checkpoint: (usize, usize)
}

impl Buffer {
    // create new buffer
    fn new () -> Buffer {
        Buffer{ chunks: VecDeque::new(), pos: (0, 0), checkpoint: (0, 0) }
    }

    /// not yet consumed length of the buffer
    pub fn len(&self) -> usize {
        self.chunks.iter().skip(self.pos.0).map(|c| c.len()).sum::<usize>() - self.pos.1
    }

    /// rollback to last commit
    pub fn rollback (&mut self) {
        self.pos = self.checkpoint;
    }

    /// checkpoint and drop already read content
    pub fn commit (&mut self) {
        // drop read chunks
        self.chunks.drain(0 .. self.pos.0);
        // current chunk is now the first
        self.pos.0 = 0;
        self.checkpoint = self.pos;
    }

    // read without advancing position
    // subsequent read would deliver the same data again
    fn read_ahead (&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let mut pos = (self.pos.0, self.pos.1);
        if self.chunks.len() == 0 {
            // no chunks -> no content
            Ok(0)
        }
        else {
            // number of bytes already collected for the read
            let mut have = 0;
            // until read enough
            while have < buf.len() {
                // current chunk
                let current = &self.chunks[pos.0];
                // number of bytes available to read from current chunk
                let available = min(buf.len() - have, current.len() - pos.1);
                // copy those
                buf[have..have+available].copy_from_slice(&current[pos.1..pos.1 + available]);
                // move pointer
                pos.1 += available;
                // have more
                have += available;
                // if current chunk was wholly read
                if pos.1 == current.len() {
                    // there are more chunks
                    if pos.0 < self.chunks.len() - 1 {
                        // move pointer to begin of next chunk
                        pos.0 += 1;
                        pos.1 = 0;
                    }
                    else {
                        // we can't have more now
                        break;
                    }
                }
            }
            // return the number of bytes that could be read
            Ok(have)
        }
    }

    // advance position by len
    fn advance (&mut self, len: usize) -> usize {
        let mut have = 0;
        // until read enough
        while have < len {
            // current chunk
            let current = &self.chunks[self.pos.0];
            // number of bytes available to read from current chunk
            let available = min(len - have, current.len() - self.pos.1);
            // move pointer
            self.pos.1 += available;
            // have more
            have += available;
            // if current chunk was wholly read
            if self.pos.1 == current.len() {
                // there are more chunks
                if self.pos.0 < self.chunks.len() - 1 {
                    // move pointer to begin of next chunk
                    self.pos.0 += 1;
                    self.pos.1 = 0;
                } else {
                    // we can't have more now
                    break;
                }
            }
        }
        // return the number of bytes that could be read
        have
    }

    // read and advance position in one step
    fn read_advance (&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        if self.chunks.len() == 0 {
            // no chunks -> no content
            Ok(0)
        }
        else {
            // number of bytes already collected for the read
            let mut have = 0;
            // until read enough
            while have < buf.len() {
                // current chunk
                let current = &self.chunks[self.pos.0];
                // number of bytes available to read from current chunk
                let available = min(buf.len() - have, current.len() - self.pos.1);
                // copy those
                buf[have..have+available].copy_from_slice(&current[self.pos.1..self.pos.1 + available]);
                // move pointer
                self.pos.1 += available;
                // have more
                have += available;
                // if current chunk was wholly read
                if self.pos.1 == current.len() {
                    // there are more chunks
                    if self.pos.0 < self.chunks.len() - 1 {
                        // move pointer to begin of next chunk
                        self.pos.0 += 1;
                        self.pos.1 = 0;
                    }
                    else {
                        // we can't have more now
                        break;
                    }
                }
            }
            // return the number of bytes that could be read
            Ok(have)
        }
    }
}

// write adapter for above buffer
impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        if buf.len() > 0 {
            // number of chunks in buffer
            let mut nc = self.chunks.len();
            // if no chunks or append to last chunk would create a too big chunk
            if nc == 0 || (buf.len() + self.chunks[nc - 1].len()) > IO_BUFFER_SIZE {
                // allocate and append a new chunk sufficient to hold buf but not smaller than IO_BUFFER_SIZE
                self.chunks.push_back(Vec::with_capacity(max(buf.len(), IO_BUFFER_SIZE)));
                nc += 1;
            }
            // append buf to current chunk
            self.chunks[nc - 1].extend_from_slice(buf);
        }
        Ok(buf.len())
    }
    // nop
    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

// read adapter for above buffer
impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.read_advance(buf)
    }
}