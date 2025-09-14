#![no_std]

mod board;

/// Currently only one supported board
pub use board::nucleo_f411re as bsp;

pub mod drivers;
