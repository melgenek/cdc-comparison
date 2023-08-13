use std::io::Read;

use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::chunkers::chunker::Chunker;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Chunk {
    /// Starting byte position within the source.
    pub offset: usize,
    /// Length of the chunk in bytes.
    pub length: usize,
    /// Source bytes contained in this chunk.
    pub data: Vec<u8>,
}

pub struct ChunkStream<'a, R: Read> {
    /// Buffer of data from source for finding cut points.
    buffer: Vec<u8>,
    /// Number of relevant bytes in the `buffer`.
    length: usize,
    /// Source from which data is read into `buffer`.
    source: R,
    /// Number of bytes read from the source so far.
    processed: usize,
    /// True when the source produces no more data.
    eof: bool,
    chunker: &'a Box<dyn Chunker>,
    chunk_sizes: ChunkSizes,
}

impl<'a, R: Read> ChunkStream<'a, R> {
    pub fn new(source: R, chunker: &'a Box<dyn Chunker>, chunk_sizes: ChunkSizes) -> Self {
        Self {
            buffer: vec![0_u8; chunk_sizes.max_size()],
            length: 0,
            source,
            eof: false,
            processed: 0,
            chunk_sizes,
            chunker,
        }
    }

    /// Fill the buffer with data from the source
    fn fill_buffer(&mut self) -> std::io::Result<()> {
        while !self.eof && self.length < self.chunk_sizes.max_size() {
            let bytes_read = self.source.read(&mut self.buffer[self.length..])?;
            if bytes_read == 0 {
                self.eof = true;
            } else {
                self.length += bytes_read;
            }
        }
        Ok(())
    }

    fn cut_chunk(&mut self, chunk_length: usize) -> Vec<u8> {
        let mut data: Vec<u8> = vec![0; chunk_length];
        data.copy_from_slice(&self.buffer[..chunk_length]);

        self.buffer.copy_within(chunk_length.., 0);
        self.length -= chunk_length;
        data
    }
}

impl<'a, R: Read> Iterator for ChunkStream<'a, R> {
    type Item = std::io::Result<Chunk>;

    fn next(&mut self) -> Option<std::io::Result<Chunk>> {
        match self.fill_buffer() {
            Err(err) => Some(Err(err)),
            Ok(_) => {
                let chunk_length = if self.length <= self.chunk_sizes.min_size() {
                    self.length
                } else {
                    self.chunker.find_split_point(&self.buffer[..self.length], &self.chunk_sizes)
                };
                if chunk_length == 0 {
                    None
                } else if chunk_length > self.length {
                    panic!(
                        "The chunk size is bigger than the buffer: {} > {}. The splitter implementation is wrong.",
                        chunk_length, self.length
                    );
                } else {
                    let offset = self.processed;
                    self.processed += chunk_length;
                    let data = self.cut_chunk(chunk_length);
                    Some(Ok(Chunk { offset, length: chunk_length, data }))
                }
            }
        }
    }
}
