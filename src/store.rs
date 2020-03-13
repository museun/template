use std::io::Result;
use std::path::PathBuf;
use std::time::SystemTime;

/// A backing store for a set of templates
pub trait TemplateStore {
    /// Data of the templates
    fn data(&mut self) -> Result<String>;
    /// Returns whether the template changed
    fn changed(&mut self) -> bool;
}

/// A file-based backing for templates
#[derive(Debug)]
pub struct FileStore {
    file: PathBuf,
    last: SystemTime,
}

impl FileStore {
    /// Create a store from this `PathBuf`
    pub fn new(file: PathBuf) -> Result<Self> {
        std::fs::metadata(&file)?
            .modified()
            .map(|last| Self { file, last })
    }
}

impl TemplateStore for FileStore {
    fn data(&mut self) -> Result<String> {
        std::fs::read_to_string(&self.file)
    }

    fn changed(&mut self) -> bool {
        match std::fs::metadata(&self.file)
            .and_then(|md| md.modified())
            .ok()
            .filter(|&last| last > self.last)
        {
            Some(time) => {
                self.last = time;
                true
            }
            None => false,
        }
    }
}

/// A memory-backed store for a template
#[derive(Debug)]
pub struct MemoryStore {
    data: String,
    changed: bool,
}

impl MemoryStore {
    /// Create a new store for the templates in `data`
    pub fn new(data: impl ToString) -> Self {
        Self {
            data: data.to_string(),
            changed: true,
        }
    }

    /// Update the templates with `data` (replaces it)
    pub fn update(&mut self, data: impl ToString) {
        self.changed = true;
        self.data = data.to_string()
    }
}

impl TemplateStore for MemoryStore {
    fn data(&mut self) -> Result<String> {
        self.changed = false;
        Ok(self.data.clone())
    }

    fn changed(&mut self) -> bool {
        self.changed
    }
}
