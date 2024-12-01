use anyhow::{Context, Result};

macro_rules! app_env_vars {
    (
        $(
            $name:ident: $var:expr
        ),* $(,)?
    ) => {
        pub struct AppEnvVars {
            $(
                pub $name: String,
            )*
        }
        impl AppEnvVars {
            pub fn load_from_env() -> Result<Self> {
                Ok(Self {
                    $(
                        $name: std::env::var($var).context(format!("Missing env variable {}", $var))?,
                    )*
                })
            }
        }
    };
}

app_env_vars!(
    database_url: "DATABASE_URL",
    jwt_secret: "JWT_SECRET",
    google_oidc_client_id: "GOOGLE_OIDC_CLIENT_ID",
    google_oidc_client_secret: "GOOGLE_OIDC_CLIENT_SECRET",
);
