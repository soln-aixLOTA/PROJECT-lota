use sqlx::PgPool;
use uuid::Uuid;

            "#,
            document.id,
            document.name,
            document.content_type,
            document.size,
            document.path,
            FROM documents
            WHERE id = $1
            "#,
            Uuid::parse_str(id)?
        )
        let result = sqlx::query!(
            r#"
            DELETE FROM documents
            WHERE id = $1
            "#,
            Uuid::parse_str(id)?
        )
        .await?;

        if result.rows_affected() == 0 {
            return Err(DocumentError::NotFound("Document not found".to_string()));
        }

        Ok(())
    }
}
