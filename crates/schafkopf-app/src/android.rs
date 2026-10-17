//! Zugriff auf Android über JNI, ohne Java-Code im Projekt.
//!
//! `tao` legt beim Start die Activity im `ndk_context` ab. Das Modul wird auf allen Systemen
//! mitübersetzt, damit Tippfehler auch am Desktop auffallen, und läuft nur auf Android.

use jni::errors::Error as JniError;
use jni::objects::{JObject, JValue};
use jni::{JNIEnv, JavaVM};

/// Legt eine Datei im öffentlichen Ordner Downloads an (`MediaStore.Downloads`).
///
/// Das braucht ab Android 10 keine Berechtigung. Gibt es den Namen schon, hängt Android selbst
/// eine Nummer an. Fehler kommen als Text zurück, damit die App sie anzeigen kann.
pub fn save_to_downloads(name: &str, mime: &str, data: &[u8]) -> Result<(), String> {
    let ctx = ndk_context::android_context();
    // SAFETY: Zeiger stammen von `tao`, das sie beim Start der Activity gesetzt hat.
    let vm = unsafe { JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| format!("JavaVM: {e}"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("JNI: {e}"))?;
    // SAFETY: Die Activity lebt, solange die App läuft. `from_raw` löscht die Referenz nicht.
    let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

    let result = insert_download(&mut env, &activity, name, mime, data);
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
    result.map_err(|e| format!("Android-Fehler: {e}"))
}

fn insert_download(
    env: &mut JNIEnv,
    activity: &JObject,
    name: &str,
    mime: &str,
    data: &[u8],
) -> Result<(), JniError> {
    // ContentValues: Name, Typ und Zielordner der neuen Datei.
    let values = env.new_object("android/content/ContentValues", "()V", &[])?;
    for (key, value) in [
        ("_display_name", name),
        ("mime_type", mime),
        ("relative_path", "Download"),
    ] {
        let key = env.new_string(key)?;
        let value = env.new_string(value)?;
        env.call_method(
            &values,
            "put",
            "(Ljava/lang/String;Ljava/lang/String;)V",
            &[JValue::from(&key), JValue::from(&value)],
        )?;
    }

    let resolver = env
        .call_method(
            activity,
            "getContentResolver",
            "()Landroid/content/ContentResolver;",
            &[],
        )?
        .l()?;
    let collection = env
        .get_static_field(
            "android/provider/MediaStore$Downloads",
            "EXTERNAL_CONTENT_URI",
            "Landroid/net/Uri;",
        )?
        .l()?;
    let uri = env
        .call_method(
            &resolver,
            "insert",
            "(Landroid/net/Uri;Landroid/content/ContentValues;)Landroid/net/Uri;",
            &[JValue::from(&collection), JValue::from(&values)],
        )?
        .l()?;
    if uri.is_null() {
        return Err(JniError::NullPtr("Android hat die Datei nicht angelegt"));
    }
    let out = env
        .call_method(
            &resolver,
            "openOutputStream",
            "(Landroid/net/Uri;)Ljava/io/OutputStream;",
            &[JValue::from(&uri)],
        )?
        .l()?;
    if out.is_null() {
        return Err(JniError::NullPtr("Android hat die Datei nicht geöffnet"));
    }
    let bytes = env.byte_array_from_slice(data)?;
    env.call_method(&out, "write", "([B)V", &[JValue::from(&bytes)])?;
    env.call_method(&out, "flush", "()V", &[])?;
    env.call_method(&out, "close", "()V", &[])?;
    Ok(())
}
