//! Core data and models for the search system.

/// Models for the engine.
pub mod engine;

/// Model for sanitizing URL's.
pub mod sanitized_url;

/// Models for searching.
pub mod search;

/// Model for building User Agents.
pub mod user_agent;

/// Model for refining the score of each search result.
pub mod score_refiner;
