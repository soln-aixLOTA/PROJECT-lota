<<<<<<< HEAD
pub mod jwt;
pub mod middleware;

pub use jwt::AuthUser;
=======
mod jwt;
mod middleware;

pub use jwt::AuthUser;
pub use middleware::{require_auth, require_roles};
>>>>>>> 921251a (fetch)
