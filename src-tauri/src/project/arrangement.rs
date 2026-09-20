pub struct Arrangement {
    title: String,
}

impl Arrangement {
    pub fn default() -> Self {
        Self {
            title: String::from("New Arrangement"),
        }
    }

    pub fn new(title: String) -> Self {
        Self { title }
    }
}
