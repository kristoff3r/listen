use leptos::*;
use leptos_router::*;

use crate::{
    downloads::DownloadsPage,
    errors::{AppError, ErrorTemplate},
    hooks::auth::AuthRequired,
    nav::Nav,
    settings::SettingsPage,
    videos::VideosPage,
};

#[component]
pub fn ListenRoutes() -> impl IntoView {
    view! {
        <div id="root" class="grid grid-cols-main grid-rows-1">
            <Router fallback=|| {
                let mut outside_errors = Errors::default();
                outside_errors.insert_with_default_key(AppError::NotFound);
                let errors = create_rw_signal(outside_errors);
                view! { <ErrorTemplate errors/> }
            }>
                <Nav/>
                <main class="flex flex-1 my-0 w-full h-screen text-center justif">
                    <ErrorBoundary fallback=|errors| {
                        view! { <ErrorTemplate errors/> }
                    }>
                        <Routes>
                            <Route path="/" view=VideosPage/>
                            <Route path="/videos" view=VideosPage/>
                            <Route path="/downloads" view=DownloadsPage/>
                            <Route path="/settings" view=SettingsPage/>
                            <Route
                                path="/authed"
                                view=|| {
                                    view! {
                                        <AuthRequired>
                                            <Outlet/>
                                        </AuthRequired>
                                    }
                                }
                            />

                        </Routes>
                    </ErrorBoundary>
                </main>
            </Router>
        </div>
    }
}
