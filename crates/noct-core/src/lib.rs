//! Noct core: sistema, temas, atajos y diagnóstico.

pub mod config;
pub mod doctor;
pub mod effects;
pub mod env;
pub mod kdl;
pub mod keys;
pub mod profile;
pub mod run;
pub mod sync;
pub mod theme;
pub mod wallpaper;

pub use doctor::{Check, Report, Severity, Summary};
pub use theme::{ApplyOptions, Backup, PaletteSet, Step, Theme, Wallpapers};
