// Noisy pedantic/nursery style lints: allowing these crate-wide rather than
// doing large, behavior-risky refactors. Simple mechanical lints are fixed for real.
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::future_not_send,
    clippy::significant_drop_tightening,
    clippy::module_name_repetitions
)]

pub mod app;
pub mod controllers;
pub mod mailers;
pub mod models;
pub mod tasks;
pub mod views;
pub mod workers;
pub mod extractors;
