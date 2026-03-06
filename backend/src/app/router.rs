use axum::{
    http::{header::{AUTHORIZATION, CONTENT_TYPE}, Method},
    routing::{get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    app::state::AppState,
    features::{auth, events, matches, system},
};

pub fn build_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    Router::new()
        .route("/health", get(system::health))
        .route("/api/hello", get(system::hello))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/refresh", post(auth::refresh))
        .route("/api/auth/logout", post(auth::logout))
        .route(
            "/api/events",
            get(events::list_events).post(events::create_event),
        )
        .route(
            "/api/events/:event_id",
            get(events::get_event)
                .put(events::update_event)
                .delete(events::delete_event),
        )
        .route(
            "/api/events/:event_id/matches",
            post(events::create_event_match),
        )
        .route(
            "/api/events/:event_id/players",
            post(events::add_event_player),
        )
        .route(
            "/api/events/:event_id/players/:player_id",
            put(events::update_event_player).delete(events::delete_event_player),
        )
        .route(
            "/api/events/:event_id/teams",
            post(events::create_event_team),
        )
        .route(
            "/api/events/:event_id/teams/:team_id",
            put(events::update_event_team).delete(events::delete_event_team),
        )
        .route(
            "/api/events/:event_id/team-members",
            post(events::assign_event_player_team),
        )
        .route(
            "/api/events/:event_id/matches/:match_id/matchup",
            post(events::set_matchup),
        )
        .route("/api/matches", get(matches::list_matches))
        .route(
            "/api/matches/:match_id",
            get(matches::get_match).delete(matches::delete_match),
        )
        .with_state(state)
        .layer(cors)
}
