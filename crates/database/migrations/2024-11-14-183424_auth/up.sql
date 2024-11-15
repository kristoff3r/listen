CREATE TABLE users (
    user_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_activity TIMESTAMPTZ NOT NULL DEFAULT now(),
    email VARCHAR(255) NOT NULL,
    handle VARCHAR(255) NOT NULL,
    profile_picture_url TEXT NOT NULL,
    is_approved BOOLEAN NOT NULL DEFAULT false,
    is_admin BOOLEAN NOT NULL DEFAULT false,

    CONSTRAINT email_unique UNIQUE(email),
    CONSTRAINT handle_unique UNIQUE(handle)
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_handle ON users(handle);

CREATE TABLE oidc_mapping (
    oidc_mapping_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    oidc_issuer_url VARCHAR(255) NOT NULL,
    oidc_issuer_id VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL,

    CONSTRAINT oidc_unique UNIQUE(oidc_issuer_url, oidc_issuer_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE INDEX idx_oidc_mapping ON oidc_mapping(oidc_issuer_url, oidc_issuer_id);
CREATE INDEX idx_oidc_mapping_user ON oidc_mapping(user_id);

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
