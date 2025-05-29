use rusqlite::{Connection, Result};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

use crate::AppResult;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DayNote {
    pub day: String, // YYYY-MM-DD
    pub content: String,
    pub updated_at: String,
}

pub fn init_database(app_handle: AppHandle) -> AppResult<std::path::PathBuf> {
    let app_data_dir: PathBuf = match app_handle.path().app_data_dir() {
        Ok(dir) => dir,
        Err(_) => {
            let error_msg = "Failed to get app data directory";
            eprintln!("{}", error_msg);
            return Err(Box::new(Error::new(ErrorKind::NotFound, error_msg)));
        }
    };

    let db_path: PathBuf = app_data_dir.join("wt-database.sqlite");

    let conn: Connection = match Connection::open(&db_path) {
        Ok(conn) => conn,
        Err(e) => {
            let error_msg = format!("Failed to open database connection: {}", e);
            eprintln!("{}", error_msg);
            return Err(Box::new(Error::new(ErrorKind::Other, error_msg)));
        }
    };

    let notes_table = r#"
    CREATE TABLE IF NOT EXISTS notes (
        id TEXT PRIMARY KEY,                    
        body_markdown TEXT NOT NULL DEFAULT '',
        body_json TEXT NOT NULL DEFAULT '',    
        folder_id TEXT,                        
        note_type TEXT NOT NULL DEFAULT 'day', -- 'day', 'regular', etc.
        is_archived INTEGER NOT NULL DEFAULT 0,
        created_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
        updated_at DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    );"#;

    let notes_indexes = vec![
        "CREATE INDEX IF NOT EXISTS idx_notes_folder ON notes(folder_id);",
        "CREATE INDEX IF NOT EXISTS idx_notes_type ON notes(note_type);",
        "CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at);",
        "CREATE INDEX IF NOT EXISTS idx_notes_day_type ON notes(id, note_type);",
    ];

    let mut statements = vec![notes_table];
    statements.extend(notes_indexes);

    for (i, stmt) in statements.iter().enumerate() {
        if let Err(e) = conn.execute(stmt, []) {
            let error_msg = format!("Error executing statement #{}: {}", i + 1, e);
            eprintln!("{}", error_msg);
            return Err(Box::new(Error::new(ErrorKind::Other, error_msg)));
        }
    }

    println!("Database initialized");
    Ok(db_path)
}

fn get_db_connection(app_handle: &AppHandle) -> Result<Connection> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    let db_path = app_data_dir.join("wt-database.sqlite");
    Connection::open(db_path)
}

#[tauri::command]
pub async fn save_note(app_handle: AppHandle, id: String, content: String) -> Result<(), String> {
    let conn = get_db_connection(&app_handle).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT OR REPLACE INTO notes (id, body_markdown, note_type, folder_id, updated_at) 
         VALUES (?1, ?2, 'day', 'daily', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        [&id, &content],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn load_note(app_handle: AppHandle, id: String) -> Result<Option<DayNote>, String> {
    let conn = get_db_connection(&app_handle).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, body_markdown, updated_at FROM notes WHERE id = ?1 AND note_type = 'day'",
        )
        .map_err(|e| e.to_string())?;

    let note = stmt
        .query_row([&id], |row| {
            Ok(DayNote {
                day: row.get(0)?,
                content: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(Some(note))
}

#[tauri::command]
pub async fn load_notes_for_days(
    app_handle: AppHandle,
    days: Vec<String>,
) -> Result<Vec<DayNote>, String> {
    let conn = get_db_connection(&app_handle).map_err(|e| e.to_string())?;

    if days.is_empty() {
        return Ok(vec![]);
    }

    let placeholders: Vec<&str> = days.iter().map(|_| "?").collect();
    let query = format!(
        "SELECT id, body_markdown, updated_at FROM notes 
         WHERE id IN ({}) AND note_type = 'day' 
         ORDER BY id DESC",
        placeholders.join(",")
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let day_refs: Vec<&str> = days.iter().map(|s| s.as_str()).collect();
    let notes = stmt
        .query_map(rusqlite::params_from_iter(day_refs), |row| {
            Ok(DayNote {
                day: row.get(0)?,
                content: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
pub async fn list_notes(app_handle: AppHandle) -> Result<Vec<DayNote>, String> {
    let conn = get_db_connection(&app_handle).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, body_markdown, updated_at FROM notes WHERE note_type = 'day' ORDER BY id DESC")
        .map_err(|e| e.to_string())?;

    let notes = stmt
        .query_map([], |row| {
            Ok(DayNote {
                day: row.get(0)?,
                content: row.get(1)?,
                updated_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
pub async fn delete_note(app_handle: AppHandle, id: String) -> Result<(), String> {
    let conn = get_db_connection(&app_handle).map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM notes WHERE id = ?1 AND note_type = 'day'",
        [&id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
