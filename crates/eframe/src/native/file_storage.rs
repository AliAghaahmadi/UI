use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

/// Determines the directory where `eframe` will store its state.
///
/// The `app_id` argument is used to generate the storage path based on the app's identifier.
/// This path is used to store application-specific data such as settings or state.
///
/// On different operating systems, the path is determined as follows:
/// * Linux:   `/home/UserName/.local/share/APP_ID`
/// * macOS:   `/Users/UserName/Library/Application Support/APP_ID`
/// * Windows: `C:\Users\UserName\AppData\Roaming\APP_ID`
pub fn storage_dir(app_id: &str) -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", app_id)
        .map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
}

// ----------------------------------------------------------------------------

/// A key-value store that persists data to a RON (Rusty Object Notation) file on disk.
/// This is used to restore various application states such as egui settings and window dimensions.
pub struct FileStorage {
    ron_filepath: PathBuf,
    kv: HashMap<String, String>,
    dirty: bool,
    last_save_join_handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for FileStorage {
    /// Ensures that any pending save operation completes when `FileStorage` is dropped.
    ///
    /// This avoids potential data loss by waiting for the save thread to finish before the `FileStorage`
    /// instance is destroyed.
    fn drop(&mut self) {
        if let Some(join_handle) = self.last_save_join_handle.take() {
            crate::profile_scope!("wait_for_save");
            join_handle.join().ok();
        }
    }
}

impl FileStorage {
    /// Creates a `FileStorage` instance from the specified RON file path.
    ///
    /// This initializes the storage by loading existing data from the RON file, if available.
    ///
    /// # Arguments
    /// * `ron_filepath` - The path to the RON file used for storing application state.
    ///
    /// # Returns
    /// A new `FileStorage` instance with the loaded state from the specified RON file.
    pub(crate) fn from_ron_filepath(ron_filepath: impl Into<PathBuf>) -> Self {
        crate::profile_function!();
        let ron_filepath: PathBuf = ron_filepath.into();
        log::debug!("Loading app state from {:?}…", ron_filepath);
        Self {
            kv: read_ron(&ron_filepath).unwrap_or_default(),
            ron_filepath,
            dirty: false,
            last_save_join_handle: None,
        }
    }

    /// Creates a `FileStorage` instance by determining a suitable directory for storing files
    /// based on the application ID.
    ///
    /// # Arguments
    /// * `app_id` - An identifier for the application used to determine the storage directory.
    ///
    /// # Returns
    /// An `Option<FileStorage>` that contains the storage instance if the directory creation succeeds,
    /// or `None` if it fails.
    pub fn from_app_id(app_id: &str) -> Option<Self> {
        crate::profile_function!(app_id);
        if let Some(data_dir) = storage_dir(app_id) {
            if let Err(err) = std::fs::create_dir_all(&data_dir) {
                log::warn!(
                    "Saving disabled: Failed to create app path at {:?}: {}",
                    data_dir,
                    err
                );
                None
            } else {
                Some(Self::from_ron_filepath(data_dir.join("app.ron")))
            }
        } else {
            log::warn!("Saving disabled: Failed to find path to data_dir.");
            None
        }
    }
}

impl crate::Storage for FileStorage {
    /// Retrieves a string value associated with the given key from the storage.
    ///
    /// # Arguments
    /// * `key` - The key whose associated value is to be retrieved.
    ///
    /// # Returns
    /// An `Option<String>` containing the value if it exists, or `None` if the key is not found.
    fn get_string(&self, key: &str) -> Option<String> {
        self.kv.get(key).cloned()
    }

    /// Sets a string value for the specified key in the storage.
    ///
    /// If the value is different from the current value associated with the key, it updates the storage
    /// and marks the storage as dirty.
    ///
    /// # Arguments
    /// * `key` - The key for which the value is to be set.
    /// * `value` - The new value to be associated with the key.
    fn set_string(&mut self, key: &str, value: String) {
        if self.kv.get(key) != Some(&value) {
            self.kv.insert(key.to_owned(), value);
            self.dirty = true;
        }
    }

    /// Persists the current state to disk if there are any changes.
    ///
    /// This function ensures that changes are written to the RON file in a separate thread to avoid blocking
    /// the main thread. It waits for any ongoing save operations to complete before starting a new one.
    fn flush(&mut self) {
        if self.dirty {
            crate::profile_function!();
            self.dirty = false;

            let file_path = self.ron_filepath.clone();
            let kv = self.kv.clone();

            if let Some(join_handle) = self.last_save_join_handle.take() {
                // Wait for the previous save operation to complete.
                join_handle.join().ok();
            }

            let result = std::thread::Builder::new()
                .name("eframe_persist".to_owned())
                .spawn(move || {
                    save_to_disk(&file_path, &kv);
                });
            match result {
                Ok(join_handle) => {
                    self.last_save_join_handle = Some(join_handle);
                }
                Err(err) => {
                    log::warn!("Failed to spawn thread to save app state: {}", err);
                }
            }
        }
    }
}

/// Saves the key-value pairs to a RON file on disk.
///
/// # Arguments
/// * `file_path` - The path to the RON file where the state should be saved.
/// * `kv` - The key-value pairs to be written to the file.
fn save_to_disk(file_path: &PathBuf, kv: &HashMap<String, String>) {
    crate::profile_function!();

    if let Some(parent_dir) = file_path.parent() {
        if !parent_dir.exists() {
            if let Err(err) = std::fs::create_dir_all(parent_dir) {
                log::warn!("Failed to create directory {:?}: {}", parent_dir, err);
            }
        }
    }

    match std::fs::File::create(file_path) {
        Ok(file) => {
            let mut writer = std::io::BufWriter::new(file);
            let config = Default::default();

            crate::profile_scope!("ron::serialize");
            if let Err(err) = ron::ser::to_writer_pretty(&mut writer, &kv, config)
                .and_then(|_| writer.flush().map_err(|err| err.into()))
            {
                log::warn!("Failed to serialize app state: {}", err);
            } else {
                log::trace!("Persisted to {:?}", file_path);
            }
        }
        Err(err) => {
            log::warn!("Failed to create file {:?}: {}", file_path, err);
        }
    }
}

// ----------------------------------------------------------------------------

/// Reads and deserializes data from a RON file.
///
/// # Arguments
/// * `ron_path` - The path to the RON file from which data is to be read.
///
/// # Returns
/// An `Option<T>` containing the deserialized value if successful, or `None` if reading or deserialization fails.
///
/// # Generic Parameters
/// * `T` - The type of the value to be deserialized. It must implement `serde::de::DeserializeOwned`.
fn read_ron<T>(ron_path: impl AsRef<Path>) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    crate::profile_function!();
    match std::fs::File::open(ron_path) {
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            match ron::de::from_reader(reader) {
                Ok(value) => Some(value),
                Err(err) => {
                    log::warn!("Failed to parse RON: {}", err);
                    None
                }
            }
        }
        Err(_err) => {
            // File might not exist, which is acceptable.
            None
        }
    }
}