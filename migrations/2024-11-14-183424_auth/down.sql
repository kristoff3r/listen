-- Down Migration for users table
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_handle;

DROP TABLE IF EXISTS users CASCADE;

-- Down Migration for oidc_mapping table
DROP INDEX IF EXISTS idx_oidc_mapping;
DROP INDEX IF EXISTS idx_oidc_mapping_user;

DROP TABLE IF EXISTS oidc_mapping CASCADE;

-- Down Migration for user_sessions table
DROP INDEX IF EXISTS idx_user_sessions_user_id;

DROP TABLE IF EXISTS user_sessions CASCADE;
