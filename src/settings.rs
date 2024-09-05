use std::sync::LazyLock;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

use crate::db::DB;

fn default_settings() -> Vec<(&'static str, SettingType)> {
    vec![
        ("Business Name", SettingType::Text("___".to_string())),
        ("Business Address", SettingType::Text("___".to_string())),
        ("Business Phone Numbers", SettingType::TextVec(vec![])),
        ("Currency Prefix", SettingType::Text("".to_string())),
        ("Currency Suffix", SettingType::Text("".to_string())),
        ("Currency Decimal Places", SettingType::Int(2)),
        (
            "Currency Decimal Separator",
            SettingType::Text(".".to_string()),
        ),
        (
            "Currency Thousand Separator",
            SettingType::Text(",".to_string()),
        ),
    ]
}

#[derive(Serialize, Deserialize, Debug, ts_rs::TS)]
#[ts(export)]
pub enum SettingType {
    Boolean(bool),
    Text(String),
    Int(i32),
    Float(f32),
    UnsignedInt(u32),
    Decimal(BigDecimal),
    TextVec(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct SettingRow {
    key: String,
    value: sqlx::types::Json<SettingType>,
}

impl From<Setting> for SettingRow {
    fn from(setting: Setting) -> Self {
        Self {
            key: setting.key,
            value: sqlx::types::Json(setting.value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ts_rs::TS)]
#[ts(export)]
pub struct Setting {
    pub key: String,
    pub value: SettingType,
}

impl From<SettingRow> for Setting {
    fn from(row: SettingRow) -> Self {
        Self {
            key: row.key,
            value: row.value.0,
        }
    }
}

impl SettingRow {
    pub fn new(key: &str, value: SettingType) -> Self {
        Self {
            key: key.to_string(),
            value: sqlx::types::Json(value),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &SettingType {
        &self.value
    }

    pub fn value_json(&self) -> &sqlx::types::Json<SettingType> {
        &self.value
    }
}

pub async fn ensure_settings_exist(db: &mut DB) -> Result<(), sqlx::Error> {
    let settings_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM settings
        "#,
    )
    .fetch_one(&mut ***db)
    .await?;

    if settings_count == 0 {
        create_default_settings(db).await?;
    }

    Ok(())
}

async fn create_default_settings(db: &mut DB) -> Result<(), sqlx::Error> {
    log::warn!("Creating default settings");

    let default_settings = default_settings();

    for (key, value) in default_settings {
        let setting = SettingRow::new(key, value);
        sqlx::query(
            r#"
            INSERT INTO settings (key, value)
            VALUES ($1, $2)
            ON CONFLICT (key) DO UPDATE
            SET value = $2
            "#,
        )
        .bind(setting.key())
        .bind(setting.value_json())
        .execute(&mut ***db)
        .await?;
    }

    Ok(())
}

pub async fn get_setting(db: &mut DB, key: String) -> Result<Option<Setting>, sqlx::Error> {
    ensure_settings_exist(db).await?;

    let setting = sqlx::query_as::<_, SettingRow>(
        r#"
        SELECT key, value
        FROM settings
        WHERE key = $1
        "#,
    )
    .bind(&key)
    .fetch_optional(&mut ***db)
    .await?;

    Ok(setting.map(Setting::from))
}

pub async fn get_settings(db: &mut DB, keys: Vec<String>) -> Result<Vec<Setting>, sqlx::Error> {
    ensure_settings_exist(db).await?;

    let query_str = &format!(
        r#"
    SELECT key, value
    FROM settings
    WHERE key IN ({})
    ORDER BY key
    "#,
        keys.iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut query = sqlx::query_as::<_, SettingRow>(query_str);

    for key in keys {
        query = query.bind(key);
    }

    let settings = query.fetch_all(&mut ***db).await?;

    Ok(settings.into_iter().map(Setting::from).collect())
}

pub async fn get_all_settings(db: &mut DB) -> Result<Vec<Setting>, sqlx::Error> {
    ensure_settings_exist(db).await?;

    let settings = sqlx::query_as::<_, SettingRow>(
        r#"
        SELECT key, value
        FROM settings
        ORDER BY key
        "#,
    )
    .fetch_all(&mut ***db)
    .await?;

    Ok(settings.into_iter().map(Setting::from).collect())
}

pub async fn set_setting(db: &mut DB, setting: SettingRow) -> Result<(), sqlx::Error> {
    ensure_settings_exist(db).await?;

    sqlx::query(
        r#"
        INSERT INTO settings (key, value)
        VALUES ($1, $2)
        ON CONFLICT (key) DO UPDATE
        SET value = $2
        "#,
    )
    .bind(setting.key())
    .bind(setting.value_json())
    .execute(&mut ***db)
    .await?;

    Ok(())
}

pub async fn delete_setting(db: &mut DB, key: String) -> Result<(), sqlx::Error> {
    ensure_settings_exist(db).await?;

    sqlx::query(
        r#"
        DELETE FROM settings
        WHERE key = $1
        "#,
    )
    .bind(&key)
    .execute(&mut ***db)
    .await?;

    Ok(())
}
