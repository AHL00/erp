use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::Acquire;
use sqlx_core::transaction;

use crate::db::DB;

// TODO: Write description for the module
fn default_settings() -> Vec<Setting> {
    vec![
        Setting {
            key: "business_name".to_string(),
            long_name: "Business Name".to_string(),
            description: None,
            value: SettingValue::Text("Business Name".to_string()),
        },
        Setting {
            key: "business_address".to_string(),
            long_name: "Business Address".to_string(),
            description: None,
            value: SettingValue::Text("123 Business Avenue".to_string()),
        },
        Setting {
            key: "business_email".to_string(),
            long_name: "Business Email".to_string(),
            description: None,
            value: SettingValue::Text("business@gmail.com".to_string()),
        },
        Setting {
            key: "business_website".to_string(),
            long_name: "Business Website".to_string(),
            description: None,
            value: SettingValue::Text("https://business.com".to_string()),
        },
        Setting {
            key: "business_bank_accounts".to_string(),
            long_name: "Business Bank Accounts".to_string(),
            description: None,
            value: SettingValue::TextVec(vec![]),
        },
        Setting {
            key: "business_phone_numbers".to_string(),
            long_name: "Business Phone Numbers".to_string(),
            description: None,
            value: SettingValue::TextVec(vec![]),
        },
        Setting {
            key: "currency_prefix".to_string(),
            long_name: "Currency Prefix".to_string(),
            description: None,
            value: SettingValue::Text("".to_string()),
        },
        Setting {
            key: "currency_suffix".to_string(),
            long_name: "Currency Suffix".to_string(),
            description: None,
            value: SettingValue::Text("".to_string()),
        },
        Setting {
            key: "currency_decimal_places".to_string(),
            long_name: "Currency Decimal Places".to_string(),
            description: None,
            value: SettingValue::UnsignedInt(2),
        },
        Setting {
            key: "currency_decimal_separator".to_string(),
            long_name: "Currency Decimal Separator".to_string(),
            description: None,
            value: SettingValue::Text(".".to_string()),
        },
        Setting {
            key: "currency_thousand_separator".to_string(),
            long_name: "Currency Thousand Separator".to_string(),
            description: None,
            value: SettingValue::Text(",".to_string()),
        },
        Setting {
            key: "logo_high_resolution".to_string(),
            long_name: "Logo High Resolution".to_string(),
            description: Some(
                "High resolution logo used in places like invoices. 512x512 recommended."
                    .to_string(),
            ),
            value: SettingValue::ImageBase64URI(
                include_str!("misc/default_logo_high_res.txt").to_string(),
            ),
        },
        Setting {
            key: "logo_low_resolution".to_string(),
            long_name: "Logo Low Resolution".to_string(),
            description: Some(
                "Low resolution logo used in places like browser icons. 128x128 recommended."
                    .to_string(),
            ),
            value: SettingValue::ImageBase64URI(
                include_str!("misc/default_logo_low_res.txt").to_string(),
            ),
        },
        Setting {
            key: "theme_color".to_string(),
            long_name: "Theme Color".to_string(),
            description: Some("Primary theme color in hex".to_string()),
            value: SettingValue::Text("#d3d3d3".to_string()),
        },
        Setting {
            key: "invoice_signature_fields".to_string(),
            long_name: "Invoice Signature Fields".to_string(),
            description: Some(
                "Whether signature fields should be included in invoices".to_string(),
            ),
            value: SettingValue::Boolean(true),
        },
        Setting {
            key: "date_time_format".to_string(),
            long_name: "Date Time Format".to_string(),
            description: Some("Date and time format using this notation: https://www.npmjs.com/package/dateformat".to_string()),
            value: SettingValue::Text("dd/mm/yy hh:MM tt".to_string()),
        }
    ]
}

#[derive(Serialize, Deserialize, Debug, ts_rs::TS)]
#[ts(export)]
pub enum SettingValue {
    Boolean(bool),
    Text(String),
    Int(i32),
    Float(f32),
    UnsignedInt(u32),
    Decimal(BigDecimal),
    ImageBase64URI(String),
    TextVec(Vec<String>),
}

#[derive(Serialize, Deserialize, Debug, ts_rs::TS)]
#[ts(export)]
pub enum SettingValueType {
    Boolean,
    Text,
    Int,
    Float,
    UnsignedInt,
    Decimal,
    ImageBase64URI,
    TextVec,
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct SettingRow {
    pub key: String,
    pub long_name: String,
    pub description: Option<String>,
    pub value: sqlx::types::Json<SettingValue>,
}

impl From<Setting> for SettingRow {
    fn from(setting: Setting) -> Self {
        Self {
            key: setting.key,
            long_name: setting.long_name,
            description: setting.description,
            value: sqlx::types::Json(setting.value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ts_rs::TS)]
#[ts(export)]
pub struct Setting {
    pub key: String,
    pub long_name: String,
    pub description: Option<String>,
    pub value: SettingValue,
}

impl From<SettingRow> for Setting {
    fn from(row: SettingRow) -> Self {
        Self {
            key: row.key,
            long_name: row.long_name,
            description: row.description,
            value: row.value.0,
        }
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

    for setting in default_settings {
        let setting_row = SettingRow::from(setting);
        let key = setting_row.key.clone();

        let res = sqlx::query(
            r#"
            INSERT INTO settings (key, long_name, description, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (key) DO NOTHING
            SET value = $4
            "#,
        )
        .bind(setting_row.key)
        .bind(setting_row.long_name)
        .bind(setting_row.description)
        .bind(setting_row.value)
        .execute(&mut ***db)
        .await?;

        if res.rows_affected() == 0 {
            log::warn!("Setting {} already exists", key);
        } else {
            log::warn!("Setting {} created", key);
        }
    }

    Ok(())
}

pub async fn get_setting(db: &mut DB, key: &str) -> Result<Option<Setting>, sqlx::Error> {
    ensure_settings_exist(db).await?;

    let setting = sqlx::query_as::<_, SettingRow>(
        r#"
        SELECT key, long_name, description, value
        FROM settings
        WHERE key = $1
        "#,
    )
    .bind(&key)
    .fetch_optional(&mut ***db)
    .await?;

    Ok(setting.map(Setting::from))
}

pub async fn reset_settings(db: &mut DB) -> Result<(), sqlx::Error> {
    let mut transaction = db.begin().await?;

    sqlx::query(
        r#"
        DELETE FROM settings
        "#,
    )
    .execute(&mut *transaction)
    .await?;

    log::warn!("Creating default settings");

    let default_settings = default_settings();

    for setting in default_settings {
        let setting_row = SettingRow::from(setting);

        sqlx::query(
            r#"
            INSERT INTO settings (key, long_name, description, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (key) DO UPDATE
            SET value = $4
            "#,
        )
        .bind(setting_row.key)
        .bind(setting_row.long_name)
        .bind(setting_row.description)
        .bind(setting_row.value)
        .execute(&mut *transaction)
        .await?;
    }

    let settings_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM settings
        "#,
    )
    .fetch_one(&mut *transaction)
    .await?;

    if settings_count == 0 {
        transaction.rollback().await?;
        return Err(sqlx::Error::RowNotFound);
    }

    transaction.commit().await?;

    Ok(())
}

pub async fn get_settings(db: &mut DB, keys: Vec<String>) -> Result<Vec<Setting>, sqlx::Error> {
    ensure_settings_exist(db).await?;

    let query_str = &format!(
        r#"
    SELECT key, long_name, description, value
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
        SELECT key, long_name, description, value
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
        INSERT INTO settings (key, long_name, description, value)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (key) DO UPDATE
        SET value = $4
        "#,
    )
    .bind(setting.key)
    .bind(setting.long_name)
    .bind(setting.description)
    .bind(setting.value)
    .execute(&mut ***db)
    .await?;

    Ok(())
}

pub async fn delete_setting(db: &mut DB, key: &str) -> Result<(), sqlx::Error> {
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
