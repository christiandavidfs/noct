//! Noct core: sistema, temas, atajos y diagnóstico.

pub mod doctor;
pub mod env;
pub mod keys;
pub mod theme;

pub use doctor::{Check, Report, Severity, Summary};
pub use theme::{ApplyOptions, Backup, PaletteSet, Step, Theme, Wallpapers};
