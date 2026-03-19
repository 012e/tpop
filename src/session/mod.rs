pub mod state;
use core::fmt;
use std::{
	hint::assert_unchecked,
	io::{stdin, Read, Write},
	process::Command,
};

use color_eyre::eyre::Result;

use crate::tmux::{
	self,
	commands::{get_session_name, kill_current_window, kill_session_by_session_name},
	popup::{self, open_popup, PopupConfig},
};

pub struct NewCommandPaneConfig {
	pub command: String,
	pub path: String,
	pub silent: bool,
	pub name: Option<String>,
}

// TODO: actually implement errors
#[derive(thiserror::Error, Debug)]
pub enum PopupEror {
	#[error("Popup already exists")]
	AlreadyExists,
	#[error("Popup does not exist")]
	DoesNotExist,
	#[error("Currently in normal mode, can't hide anything")]
	CantHideNormalMode,
	#[error("Currently in popup mode, can't show anything")]
	CantShowPopup,
}

pub struct Session {
	pub state: state::State,
	pub name: String,
	pub current_path: String,
	pub current_window_id: String,
}

const POPUP_PREFIX: &str = "popup-";

fn extract_session_state(session_name: &String) -> state::State {
	if session_name.starts_with(POPUP_PREFIX) {
		state::State::Popup
	} else {
		state::State::Normal
	}
}
fn format_session_name(session_name: &str, current_window_id: &str) -> String {
	format!("{}[{}][{}]", POPUP_PREFIX, current_window_id, session_name)
}

impl Session {
	pub fn current() -> Result<Self> {
		let session_name = get_session_name()?;

		let state = extract_session_state(&session_name);
		let name = session_name;
		let current_path = tmux::commands::get_current_session_property("#{pane_current_path}")?;
		let current_window_id = tmux::commands::get_current_session_property("#{window_id}")?;
		Ok(Self {
			state,
			name,
			current_path,
			current_window_id,
		})
	}

	fn get_popup_session_name(&self) -> String {
		if self.name.starts_with(POPUP_PREFIX) {
			self.name.clone()
		} else {
			format_session_name(&self.name, &self.current_window_id)
		}
	}

	fn get_parent_session_name(&self) -> String {
		if !self.name.starts_with(POPUP_PREFIX) {
			self.name.clone()
		} else {
			format!(
				"{}[{}][{}]",
				POPUP_PREFIX, self.current_window_id, self.name
			)
		}
	}

	fn kill_popups(&self) -> Result<()> {
		assert!(self.state == state::State::Normal);
		kill_session_by_session_name(self.get_popup_session_name())?;
		Ok(())
	}

	pub fn kill_current_window_and_popups(&self) -> Result<()> {
		match self.state {
			state::State::Normal => {
				self.kill_popups()?;
				kill_current_window()?;
			}
			state::State::Popup => {
				kill_current_window()?;
			}
		};
		Ok(())
	}

	pub fn toggle_popup(&self) -> Result<()> {
		match self.state {
			state::State::Normal => self.show_popup()?,
			state::State::Popup => self.hide_popup()?,
		}
		Ok(())
	}

	pub fn hide_popup(&self) -> Result<()> {
		if self.state == state::State::Popup {
			Command::new("tmux").arg("detach").status()?;
		}
		Ok(())
	}

	pub fn show_popup(&self) -> Result<()> {
		if let state::State::Normal = self.state {
			self.ensure_popup_session_exist()?;
			open_popup(PopupConfig {
				// Using the more robust 'new -A' approach from the incoming branch
				command: Some("tmux new -A -s ".to_owned() + &self.get_popup_session_name()),
				path: self.current_path.clone(),
				height: Some(80),
				width: Some(80),
			})?;
		}
		Ok(())
	}

	pub fn add_popup_pane(&self) -> Result<()> {
		Ok(())
	}

	pub fn spawn_command_window(
		&self,
		NewCommandPaneConfig {
			command,
			path,
			silent,
			name,
		}: NewCommandPaneConfig,
	) -> Result<()> {
		self.ensure_popup_session_exist()?;
		let mut cmd = Command::new("tmux");
		cmd.arg("new-window")
			.arg("-t")
			.arg(&self.get_popup_session_name())
			.arg("-n")
			.arg(name.unwrap_or("cmdRunner".to_owned()))
			.arg(command)
			.status()?;

		if !silent {
			Command::new("tmux")
				.arg("set-hook")
				.arg("-t")
				.arg(&self.get_popup_session_name())
				.arg("pane-exited")
				.arg("detach")
				.status()?;
			self.show_popup()?;
		}

		Ok(())
	}

	fn pause(&self) {
		let mut stdout = std::io::stdout();
		let _ = stdout.write(b"Press Enter to continue...");
		let _ = stdout.flush();
		let _ = stdin().read(&mut [0]);
	}

	/// Creates new tmux popup session without displaying it (detached).
	/// Popup sessions is defined as a session that starts with "popup" prefix.
	/// The path of the popup session is the same as the current session.
	pub fn ensure_popup_session_exist(&self) -> Result<()> {
		// TODO: check for duplication
		let popup_session_name = self.get_popup_session_name();
		if tmux::commands::has_session(popup_session_name.clone())? {
			return Ok(());
		}
		Command::new("tmux")
			.arg("new-session")
			.arg("-d")
			.arg("-s")
			.arg(popup_session_name)
			.arg("-c")
			.arg(self.current_path.clone())
			.status()?;
		Ok(())
	}

	pub fn convert_pane_to_popup(&self) -> Result<()> {
		if self.state == state::State::Popup {
			return Ok(());
		}

		let current_window_index = tmux::commands::get_current_session_property("#{window_index}")?;

		// Create new window to replace the old one.
		Command::new("tmux")
			.arg("new-window")
			.arg("-c")
			.arg(self.current_path.clone())
			.status()
			.expect("must be able to create new window");

		// Ensure that popup session must exists so that we can move current window there
		self.ensure_popup_session_exist()?;

		tmux::commands::move_window(
			self.name.clone(),
			current_window_index,
			self.get_popup_session_name(),
		)?;

		Ok(())
	}
}
