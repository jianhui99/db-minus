use crate::error::AppError;
use keyring::Entry;

const SERVICE: &str = "com.dbminus.app";

fn entry(conn_id: &str) -> Result<Entry, AppError> {
    Entry::new(SERVICE, conn_id).map_err(|e| AppError::Keychain(e.to_string()))
}

pub fn set_password(conn_id: &str, password: &str) -> Result<(), AppError> {
    entry(conn_id)?
        .set_password(password)
        .map_err(|e| AppError::Keychain(e.to_string()))
}

pub fn get_password(conn_id: &str) -> Result<Option<String>, AppError> {
    match entry(conn_id)?.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keychain(e.to_string())),
    }
}

pub fn delete_password(conn_id: &str) -> Result<(), AppError> {
    match entry(conn_id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Keychain(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "touches real macOS Keychain, run manually: cargo test -- --ignored secret"]
    fn roundtrip() {
        let id = "db-minus-test-entry";
        set_password(id, "s3cret").unwrap();
        assert_eq!(get_password(id).unwrap().as_deref(), Some("s3cret"));
        delete_password(id).unwrap();
        assert_eq!(get_password(id).unwrap(), None);
        delete_password(id).unwrap();
    }
}
