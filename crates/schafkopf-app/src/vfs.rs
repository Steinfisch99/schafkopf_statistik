//! Dateizugriffe der App an einer Stelle.
//!
//! Auf Android und am Desktop reicht dieses Modul `std::fs` unverändert durch. Im Browser (WebAssembly) gibt es
//! kein Dateisystem: Dort liegen die Dateien in einer Tabelle im Speicher, die beim Start aus IndexedDB geladen
//! und bei jeder Änderung dorthin zurückgeschrieben wird. So bleibt der Rest der App (`storage.rs`) derselbe.

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::io::{self, Write};
    use std::path::Path;

    pub use std::fs::{create_dir_all, read, read_dir, read_to_string, remove_file, rename, write};
    // Nur die Tests räumen ihre Ordner mit auf.
    #[allow(unused_imports)]
    pub use std::fs::remove_dir_all;

    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Hängt eine Zeile an die Datei an und legt sie bei Bedarf an.
    pub fn append_line(path: &Path, line: &str) -> io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?;
        writeln!(file, "{line}")
    }

    /// Im Browser lädt dies die gespeicherten Dateien. Hier gibt es nichts zu tun.
    #[allow(dead_code)]
    pub async fn init() {}
}

#[cfg(target_arch = "wasm32")]
mod imp {
    use std::cell::RefCell;
    use std::collections::BTreeMap;
    use std::ffi::OsString;
    use std::io;
    use std::path::{Path, PathBuf};

    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(inline_js = r#"
let opened = null;
function db() {
    if (!opened) {
        opened = new Promise((resolve, reject) => {
            const request = indexedDB.open('schafkopf', 1);
            request.onupgradeneeded = () => request.result.createObjectStore('files');
            request.onsuccess = () => resolve(request.result);
            request.onerror = () => reject(request.error);
        });
    }
    return opened;
}
export async function load_all() {
    try {
        if (navigator.storage && navigator.storage.persist) { navigator.storage.persist(); }
    } catch (e) { /* nur eine Bitte an den Browser */ }
    const d = await db();
    return new Promise((resolve, reject) => {
        const tx = d.transaction('files');
        const store = tx.objectStore('files');
        const keys = store.getAllKeys();
        const values = store.getAll();
        tx.oncomplete = () => resolve(keys.result.map((k, i) => [k, values.result[i]]));
        tx.onerror = () => reject(tx.error);
    });
}
export function put(key, bytes) {
    const copy = bytes.slice();
    db().then((d) => {
        const tx = d.transaction('files', 'readwrite');
        tx.objectStore('files').put(copy, key);
        tx.onerror = () => console.error('Speichern fehlgeschlagen', tx.error);
    });
}
export function del(key) {
    db().then((d) => d.transaction('files', 'readwrite').objectStore('files').delete(key));
}
"#)]
    extern "C" {
        #[wasm_bindgen(catch)]
        async fn load_all() -> Result<JsValue, JsValue>;
        fn put(key: &str, bytes: &[u8]);
        fn del(key: &str);
    }

    thread_local! {
        static FILES: RefCell<BTreeMap<String, Vec<u8>>> = const { RefCell::new(BTreeMap::new()) };
    }

    fn key(path: &Path) -> String {
        path.to_string_lossy().trim_start_matches("./").to_owned()
    }

    fn not_found() -> io::Error {
        io::Error::new(io::ErrorKind::NotFound, "Datei nicht gefunden")
    }

    /// Lädt alle gespeicherten Dateien aus IndexedDB in den Speicher. Muss vor dem Start der App fertig sein.
    pub async fn init() {
        let Ok(all) = load_all().await else { return };
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            for entry in js_sys::Array::from(&all).iter() {
                let pair = js_sys::Array::from(&entry);
                if let Some(name) = pair.get(0).as_string() {
                    files.insert(name, js_sys::Uint8Array::new(&pair.get(1)).to_vec());
                }
            }
        });
    }

    pub fn exists(path: &Path) -> bool {
        FILES.with(|f| f.borrow().contains_key(&key(path)))
    }

    pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        FILES.with(|f| {
            f.borrow()
                .get(&key(path.as_ref()))
                .cloned()
                .ok_or_else(not_found)
        })
    }

    pub fn read_to_string(path: impl AsRef<Path>) -> io::Result<String> {
        String::from_utf8(read(path)?)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    pub fn write(path: impl AsRef<Path>, data: impl AsRef<[u8]>) -> io::Result<()> {
        let name = key(path.as_ref());
        put(&name, data.as_ref());
        FILES.with(|f| f.borrow_mut().insert(name, data.as_ref().to_vec()));
        Ok(())
    }

    pub fn append_line(path: &Path, line: &str) -> io::Result<()> {
        let mut data = read(path).unwrap_or_default();
        data.extend_from_slice(line.as_bytes());
        data.push(b'\n');
        write(path, data)
    }

    /// Ordner gibt es im Browser nicht, ein Dateiname enthält seinen Pfad.
    pub fn create_dir_all(_path: impl AsRef<Path>) -> io::Result<()> {
        Ok(())
    }

    pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
        let name = key(path.as_ref());
        match FILES.with(|f| f.borrow_mut().remove(&name)) {
            Some(_) => {
                del(&name);
                Ok(())
            }
            None => Err(not_found()),
        }
    }

    pub fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
        let data = read(from.as_ref())?;
        write(to, data)?;
        remove_file(from)
    }

    pub struct DirEntry(PathBuf);

    pub struct FileType;

    impl FileType {
        pub fn is_file(&self) -> bool {
            true
        }
    }

    impl DirEntry {
        pub fn path(&self) -> PathBuf {
            self.0.clone()
        }

        pub fn file_name(&self) -> OsString {
            self.0.file_name().unwrap_or_default().to_owned()
        }

        pub fn file_type(&self) -> io::Result<FileType> {
            Ok(FileType)
        }
    }

    /// Die Dateien direkt unter dem Ordner (Unterordner gibt es nicht).
    pub fn read_dir(
        path: impl AsRef<Path>,
    ) -> io::Result<std::vec::IntoIter<io::Result<DirEntry>>> {
        let prefix = format!("{}/", key(path.as_ref()));
        let entries: Vec<io::Result<DirEntry>> = FILES.with(|f| {
            f.borrow()
                .keys()
                .filter(|k| {
                    k.strip_prefix(&prefix)
                        .is_some_and(|rest| !rest.contains('/'))
                })
                .map(|k| Ok(DirEntry(PathBuf::from(k))))
                .collect()
        });
        Ok(entries.into_iter())
    }
}

pub use imp::*;
