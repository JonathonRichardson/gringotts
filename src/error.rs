pub struct CorruptDataError {
    message: String
}

impl CorruptDataError {
    pub fn new(message: &str) -> CorruptDataError {
        return CorruptDataError {
            message: String::from(message)
        };
    }

    pub fn get_message(&self) -> String {
        return self.message.clone();
    }
}
