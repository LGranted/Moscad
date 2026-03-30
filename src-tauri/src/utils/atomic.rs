use std::fs;
use std::path::Path;

/// Атомарная запись файла:
/// 1. Пишем в .tmp файл
/// 2. Переименовываем в основной (атомарная операция на уровне ОС)
/// Если процесс убьют в момент записи — основной файл остаётся нетронутым
pub fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    let tmp_path = path.with_extension("tmp");

    // Создаём директорию если не существует
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed_to_create_dir: {}", e))?;
    }

    // Пишем во временный файл
    fs::write(&tmp_path, content)
        .map_err(|e| format!("failed_to_write_tmp: {}", e))?;

    // Атомарное переименование
    fs::rename(&tmp_path, path)
        .map_err(|e| {
            // Если rename упал — чистим tmp
            let _ = fs::remove_file(&tmp_path);
            format!("failed_to_rename_atomic: {}", e)
        })?;

    Ok(())
}
