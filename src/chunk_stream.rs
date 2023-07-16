use std::fmt;
use std::io::Read;

pub trait SplitPointFinder {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize;
}

///
/// The error type returned from the `StreamCDC` iterator.
///
#[derive(Debug)]
pub enum Error {
    /// End of source data reached.
    Empty,
    /// An I/O error occurred.
    IoError(std::io::Error),
    /// Something unexpected happened.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chunker error: {self:?}")
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IoError(error)
    }
}

impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        match error {
            Error::IoError(ioerr) => ioerr,
            Error::Empty => Self::from(std::io::ErrorKind::UnexpectedEof),
            Error::Other(str) => Self::new(std::io::ErrorKind::Other, str),
        }
    }
}

///
/// Represents a chunk returned from the StreamCDC iterator.
///
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ChunkData {
    /// Starting byte position within the source.
    pub offset: usize,
    /// Length of the chunk in bytes.
    pub length: usize,
    /// Source bytes contained in this chunk.
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug)]
pub struct ChunkSizes {
    min_size: usize,
    avg_size: usize,
    max_size: usize,
}

impl ChunkSizes {
    pub fn new(min_size: usize, avg_size: usize, max_size: usize) -> Self {
        assert!(min_size <= avg_size);
        assert!(avg_size <= max_size);
        Self { min_size, avg_size, max_size }
    }

    pub fn min_size(&self) -> usize { self.min_size }
    pub fn avg_size(&self) -> usize { self.avg_size }
    pub fn max_size(&self) -> usize { self.max_size }
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
    split_point_finder: &'a Box<dyn SplitPointFinder>,
    chunk_sizes: ChunkSizes,
}

impl<'a, R: Read> ChunkStream<'a, R> {
    pub fn new(
        source: R,
        split_point_finder: &'a Box<dyn SplitPointFinder>,
        chunk_sizes: ChunkSizes,
    ) -> Self {
        Self {
            buffer: vec![0_u8; chunk_sizes.max_size() as usize],
            length: 0,
            source,
            eof: false,
            processed: 0,
            chunk_sizes,
            split_point_finder,
        }
    }

    /// Fill the buffer with data from the source
    fn fill_buffer(&mut self) -> Result<(), Error> {
        while !self.eof && self.length < self.chunk_sizes.max_size() as usize {
            let bytes_read = self.source.read(&mut self.buffer[self.length..])?;
            if bytes_read == 0 {
                self.eof = true;
            } else {
                self.length += bytes_read;
            }
        }
        Ok(())
    }

    /// Drains a specified number of bytes from the buffer, then resizes the
    /// buffer back to `max_size` size in preparation for further reads.
    fn drain_bytes(&mut self, count: usize) -> Result<Vec<u8>, Error> {
        if count > self.length {
            Err(Error::Other(format!(
                "drain_bytes() called with count larger than length: {} > {}",
                count, self.length
            )))
        } else {
            let mut data: Vec<u8> = vec![0; count];
            data.copy_from_slice(&self.buffer[..count]);

            self.buffer.copy_within(count.., 0);
            self.length -= count;
            Ok(data)
        }
    }

    /// Find the next chunk in the source. If the end of the source has been
    /// reached, returns `Error::Empty` as the error.
    fn read_chunk(&mut self) -> Result<ChunkData, Error> {
        self.fill_buffer()?;
        if self.length == 0 {
            Err(Error::Empty)
        } else {
            let buf = &self.buffer[..self.length];
            let index = if buf.len() <= self.chunk_sizes.min_size() as usize {
                buf.len()
            } else {
                self.split_point_finder.find_split_point(
                    buf,
                    &self.chunk_sizes,
                )
            };
            if index == 0 {
                Err(Error::Empty)
            } else {
                let offset = self.processed;
                self.processed += index;
                let data = self.drain_bytes(index)?;
                Ok(ChunkData {
                    offset,
                    length: index,
                    data,
                })
            }
        }
    }
}

impl<'a, R: Read> Iterator for ChunkStream<'a, R> {
    type Item = Result<ChunkData, Error>;

    fn next(&mut self) -> Option<Result<ChunkData, Error>> {
        let slice = self.read_chunk();
        if let Err(Error::Empty) = slice {
            None
        } else {
            Some(slice)
        }
    }
}
