use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub size: usize,
}

impl Document {
    #[allow(dead_code)]
    pub fn new(name: String, content_type: String, size: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            content_type,
            size,
        }
    }
} 