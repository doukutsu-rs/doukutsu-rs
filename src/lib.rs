//#![cfg_attr(target_os = "horizon", feature(restricted_std))]

#[macro_use]
extern crate log;
extern crate strum;
#[macro_use]
extern crate strum_macros;

pub mod common;
mod components;
mod data;
#[cfg(feature = "discord-rpc")]
pub mod discord;
#[cfg(feature = "editor")]
mod editor;
mod engine_constants;
mod entity;
mod framework;
pub mod game;
mod graphics;
mod i18n;
mod input;
mod live_debugger;
mod macros;
mod menu;
mod mod_list;
mod mod_requirements;
mod scene;
mod sound;
mod util;
