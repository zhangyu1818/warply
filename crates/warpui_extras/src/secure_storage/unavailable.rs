use super::Error;

pub struct SecureStorage;

impl super::SecureStorage for SecureStorage {
    fn write_value(&self, _key: &str, _value: &str) -> Result<(), Error> {
        Ok(())
    }

    fn read_value(&self, _key: &str) -> Result<String, Error> {
        Err(Error::NotFound)
    }

    fn remove_value(&self, _key: &str) -> Result<(), Error> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "unavailable_tests.rs"]
mod tests;
