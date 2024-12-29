use crate::repositories::user::UserRepository;

pub struct AppState {
    pub user_repo: UserRepository,
}

impl AppState {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }
}
