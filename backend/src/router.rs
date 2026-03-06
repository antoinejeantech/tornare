use axum::{
    http::{header::{AUTHORIZATION, CONTENT_TYPE}, Method},
    routing::{get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{handlers, state::AppState};

pub fn build_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/hello", get(handlers::hello))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/auth/refresh", post(handlers::refresh))
        .route("/api/auth/logout", post(handlers::logout))
        .route(
            "/api/events",
            get(handlers::list_events).post(handlers::create_event),
        )
        .route(
            "/api/events/:event_id",
            get(handlers::get_event)
                .put(handlers::update_event)
                .delete(handlers::delete_event),
        )
        .route(
            "/api/events/:event_id/matches",
            post(handlers::create_event_match),
        )
        .route(
            "/api/events/:event_id/players",
            post(handlers::add_event_player),
        )
        .route(
            "/api/events/:event_id/players/:player_id",
            put(handlers::update_event_player).delete(handlers::delete_event_player),
        )
        .route(
            "/api/events/:event_id/teams",
            post(handlers::create_event_team),
        )
        .route(
            "/api/events/:event_id/teams/:team_id",
            put(handlers::update_event_team).delete(handlers::delete_event_team),
        )
        .route(
            "/api/events/:event_id/team-members",
            post(handlers::assign_event_player_team),
        )
        .route(
            "/api/events/:event_id/matches/:match_id/matchup",
            post(handlers::set_matchup),
        )
        .route("/api/matches", get(handlers::list_matches))
        .route(
            "/api/matches/:match_id",
            get(handlers::get_match).delete(handlers::delete_match),
        )
        .with_state(state)
        .layer(cors)
}
