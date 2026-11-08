//! visos Input library (InputManager.cpp, InputDevice.cpp) on top of SDL3: physical device state for the binding
//! layer (`velocity_game::controller::user_input`) and every other reader (frontend, debug menu).
pub mod input_device;
pub mod input_manager;
pub mod raw_input;
