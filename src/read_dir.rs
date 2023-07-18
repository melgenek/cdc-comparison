use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub struct MultiFileRead {
    files: Vec<PathBuf>,
    current_file: Option<File>,
}

impl MultiFileRead {
    pub fn new(mut files: Vec<PathBuf>) -> std::io::Result<MultiFileRead> {
        files.reverse();
        let mut instance = MultiFileRead { files, current_file: None };
        instance.update_current_file()?;
        Ok(instance)
    }

    fn update_current_file(&mut self) -> std::io::Result<()> {
        self.current_file.take();
        let next_file = self.files.pop();
        match next_file {
            None => {}
            Some(path) => {
                self.current_file = Some(File::open(path)?);
            }
        }
        Ok(())
    }
}

impl Read for MultiFileRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        loop {
            match &mut self.current_file {
                None => break Ok(0),
                Some(file) => {
                    let bytes_read = file.read(buf)?;
                    if bytes_read > 0 {
                        break Ok(bytes_read);
                    } else {
                        self.update_current_file()?;
                    }
                }
            }
        }
    }
}
