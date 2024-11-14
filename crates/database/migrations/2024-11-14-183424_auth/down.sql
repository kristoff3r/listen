-- Down Migration for users table
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_users_handle;
DROP INDEX IF EXISTS idx_users_oidc_issuer_url;

DROP TABLE IF EXISTS users CASCADE;

-- Down Migration for user_sessions table
DROP INDEX IF EXISTS idx_user_sessions_user_id;

DROP TABLE IF EXISTS user_sessions CASCADE;
