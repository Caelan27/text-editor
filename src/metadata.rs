use std::time::SystemTime;

pub struct FileMetadata {
    pub last_write_time: Option<SystemTime>,
    pub file_size: Option<usize>,
    pub file_path: String,
}

impl FileMetadata {
    pub fn new(file_path: String) -> Self {
        Self {
            last_write_time: None,
            file_size: None,
            file_path,
        }
    }

    pub fn update(&mut self, file_size: usize) {
        self.last_write_time = Some(SystemTime::now());
        self.file_size = Some(file_size);
    }
}
