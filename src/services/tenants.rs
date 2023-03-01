use axum::http::StatusCode;
use sea_orm::{query::Statement, ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr};

#[derive(Debug)]
pub struct TenantResult {
    pub detail: String,
    pub status: StatusCode,
}

// TODO: Make this more robust.
// We should delete the created schema if any of the subsequent steps fail.
pub async fn signup(db: &DatabaseConnection, tenant_name: &String) -> TenantResult {
    let create_schema_stmt = Statement::from_string(
        DatabaseBackend::Postgres,
        format!("CREATE SCHEMA {tenant_name}"),
    );
    match db.execute(create_schema_stmt).await {
        Ok(_) => (),
        Err(err) => return map_db_err_to_http_status(&err),
    };

    let copy_tbls_stmt = Statement::from_string(DatabaseBackend::Postgres, format!("
    DO $$
    DECLARE
        tbl_name TEXT;
        seq_name TEXT;  -- Define a new loop variable for sequences
    BEGIN
        FOR tbl_name IN
            SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'
        LOOP
            EXECUTE 'CREATE TABLE {tenant_name}.' || quote_ident(tbl_name) || ' (LIKE public.' || quote_ident(tbl_name) || ' INCLUDING ALL)';
        END LOOP;

        FOR seq_name IN  -- Use the new variable in the loop for sequences
            SELECT sequence_name FROM information_schema.sequences WHERE sequence_schema = 'public'
        LOOP
            EXECUTE 'CREATE SEQUENCE {tenant_name}.' || quote_ident(seq_name) || '';
        END LOOP;
    END $$;"
    ));
    match db.execute(copy_tbls_stmt).await {
        Ok(_) => TenantResult {
            detail: format!("Created schema: {tenant_name}"),
            status: StatusCode::CREATED,
        },
        Err(err) => return map_db_err_to_http_status(&err),
    }
}

fn map_db_err_to_http_status(err: &DbErr) -> TenantResult {
    let status = match err {
        DbErr::ConnectionAcquire => StatusCode::SERVICE_UNAVAILABLE,
        DbErr::TryIntoErr { .. } => StatusCode::BAD_REQUEST,
        DbErr::Conn(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::Exec(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::Query(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::ConvertFromU64(_) => StatusCode::BAD_REQUEST,
        DbErr::UnpackInsertId => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::UpdateGetPrimaryKey => StatusCode::BAD_REQUEST,
        DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
        DbErr::AttrNotSet(_) => StatusCode::BAD_REQUEST,
        DbErr::Custom(_) => StatusCode::INTERNAL_SERVER_ERROR,
        DbErr::Type(_) => StatusCode::BAD_REQUEST,
        DbErr::Json(_) => StatusCode::BAD_REQUEST,
        DbErr::Migration(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let message = err.to_string();
    if message.contains("already exists") {
        return TenantResult {
            detail: "Tenant already exists".to_string(),
            status: StatusCode::CONFLICT,
        };
    }

    TenantResult {
        detail: message,
        status,
    }
}
