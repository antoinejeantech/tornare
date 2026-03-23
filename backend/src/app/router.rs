use axum::{
    http::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        HeaderName, HeaderValue, Method, Request,
    },
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info_span, Level};

use crate::{
    app::state::AppState,
    features::{auth, events, matches, system, users},
};

pub fn build_app(state: AppState) -> Router {
    let request_id_header = HeaderName::from_static("x-request-id");

    let allow_any = state.config.cors_allowed_origins.iter().any(|origin| origin == "*");
    let parsed_allowed_origins: Vec<HeaderValue> = state
        .config
        .cors_allowed_origins
        .iter()
        .filter_map(|origin| origin.parse::<HeaderValue>().ok())
        .collect();

    let cors = if allow_any || parsed_allowed_origins.is_empty() {
        CorsLayer::new().allow_origin(Any)
    } else {
        CorsLayer::new().allow_origin(parsed_allowed_origins)
    }
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

    Router::new()
        .route("/health", get(system::health))
        .route("/api/hello", get(system::hello))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/me", get(auth::me))
        .route("/api/auth/refresh", post(auth::refresh))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/battlenet/authorize", get(auth::battlenet_authorize))
        .route("/api/auth/battlenet/callback", get(auth::battlenet_callback))
        .route("/api/auth/battlenet/connect-init", post(auth::battlenet_connect_init))
        .route("/api/auth/battlenet/disconnect", delete(auth::battlenet_disconnect))
        .route(
            "/api/users/{user_id}",
            get(users::get_user_profile).put(users::update_user_profile),
        )
        .route(
            "/api/events",
            get(events::list_events).post(events::create_event),
        )
        .route(
            "/api/events/kpi",
            get(events::get_events_kpis),
        )
        .route(
            "/api/events/featured",
            get(events::get_featured_event),
        )
        .route(
            "/api/events/{event_id}",
            get(events::get_event)
                .put(events::update_event)
                .delete(events::delete_event),
        )
        .route(
            "/api/events/{event_id}/matches",
            post(events::create_event_match),
        )
        .route(
            "/api/events/{event_id}/players",
            post(events::add_event_player),
        )
        .route(
            "/api/events/{event_id}/players/{player_id}",
            put(events::update_event_player).delete(events::delete_event_player),
        )
        .route(
            "/api/events/{event_id}/teams",
            post(events::create_event_team),
        )
        .route(
            "/api/events/{event_id}/teams/auto-solo",
            post(events::auto_create_solo_teams),
        )
        .route(
            "/api/events/{event_id}/teams/auto-balance",
            post(events::auto_balance_teams),
        )
        .route(
            "/api/events/{event_id}/teams/{team_id}",
            put(events::update_event_team).delete(events::delete_event_team),
        )
        .route(
            "/api/events/{event_id}/team-members",
            post(events::assign_event_player_team),
        )
        .route(
            "/api/events/{event_id}/matches/{match_id}/matchup",
            post(events::set_matchup),
        )
        .route(
            "/api/events/{event_id}/tourney/generate",
            post(events::generate_tourney_bracket),
        )
        .route(
            "/api/events/{event_id}/tourney/clear",
            post(events::clear_tourney_bracket),
        )
        .route(
            "/api/events/{event_id}/matches/{match_id}/winner",
            post(events::report_match_winner),
        )
        .route(
            "/api/events/{event_id}/matches/{match_id}/winner/cancel",
            post(events::cancel_match_winner),
        )
        .route(
            "/api/events/{event_id}/matches/{match_id}/start-date",
            post(events::update_match_start_date),
        )
        .route(
            "/api/events/{event_id}/signup-link",
            get(events::get_event_signup_link),
        )
        .route(
            "/api/events/{event_id}/signup-link/rotate",
            post(events::rotate_event_signup_link),
        )
        .route(
            "/api/events/{event_id}/signup-visibility",
            put(events::set_event_public_signup),
        )
        .route(
            "/api/events/{event_id}/featured",
            put(events::set_event_featured),
        )
        .route(
            "/api/events/{event_id}/ended",
            put(events::set_event_ended),
        )
        .route(
            "/api/events/{event_id}/signup-requests",
            get(events::list_event_signup_requests),
        )
        .route(
            "/api/events/{event_id}/signup-requests/{request_id}/accept",
            post(events::accept_event_signup_request),
        )
        .route(
            "/api/events/{event_id}/signup-requests/{request_id}/decline",
            post(events::decline_event_signup_request),
        )
        .route(
            "/api/public/event-signups/{signup_token}",
            get(events::get_public_signup_info),
        )
        .route(
            "/api/public/event-signups/{signup_token}/requests",
            post(events::create_public_signup_request),
        )
        .route("/api/matches", get(matches::list_matches))
        .route(
            "/api/matches/{match_id}",
            get(matches::get_match).delete(matches::delete_match),
        )
        .with_state(state)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = request
                    .headers()
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or("-");

                info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    request_id = %request_id,
                )
            })
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(PropagateRequestIdLayer::new(request_id_header.clone()))
        .layer(SetRequestIdLayer::new(request_id_header, MakeRequestUuid))
        .layer(cors)
}
