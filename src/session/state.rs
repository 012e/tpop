use std::io;

#[derive(PartialEq, Debug)]
pub enum State {
	Popup,
	Normal,
}

pub fn session_is_popup() -> Result<bool, io::Error> {
	let session_name = crate::tmux::commands::get_session_name()?;
	Ok(session_name.starts_with("popup"))
}

pub fn get_state() -> Result<State, io::Error> {
	if session_is_popup()? {
		Ok(State::Popup)
	} else {
		Ok(State::Normal)
	}
}
