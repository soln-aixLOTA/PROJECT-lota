        if let Some(token) = parts.get(AUTHORIZATION).and_then(|value| value.to_str().ok()) {
            if token.starts_with("Bearer ") {
                let jwt_token = token.trim_start_matches("Bearer ");
                match validate_token(jwt_token) {
                    Ok(claims) => {
                        let mut extensions = req.extensions_mut();
                        extensions.insert(claims.clone());

                        // ... rest of the code ...
                    }
                    Err(_) => {
                        // ...
                    }
                }
            }
        } 