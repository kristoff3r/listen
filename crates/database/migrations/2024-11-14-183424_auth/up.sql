CREATE TABLE users (
    user_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    email VARCHAR(255) NOT NULL,
    handle VARCHAR(255) NOT NULL,
    oidc_issuer_url VARCHAR(255) NOT NULL,
    profile_picture_url TEXT NOT NULL,
    is_approved BOOLEAN NOT NULL,
    is_admin BOOLEAN NOT NULL,

    CONSTRAINT email_unique UNIQUE(email),
    CONSTRAINT handle_unique UNIQUE(handle),
    CONSTRAINT email_oidc_issuer_url_unique UNIQUE(email, oidc_issuer_url)
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_handle ON users(handle);
CREATE INDEX idx_users_oidc_issuer_url ON users(oidc_issuer_url);

CREATE TABLE user_sessions (
    user_session_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    oidc_issuer_url VARCHAR(255),
    csrf_token VARCHAR(255),
    nonce VARCHAR(255),
    pkce_code_verifier VARCHAR(255),

    user_id UUID,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
