use crate::{devices::{DisplayCommands, KeyboardCommands}, timer::Timed};


pub fn run<D, K, T>(display : D, keyboard: K) 
where
    D : DisplayCommands,
    K: KeyboardCommands,
    T: Timed
{
}