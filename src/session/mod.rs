pub mod state;
use std::process::Command;

use color_eyre::eyre::Result;

use crate::tmux::{
	self,
	popup::{open_popup, PopupConfig},
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
}

impl Session {
	pub fn current() -> Result<Self> {
		let state = state::get_state()?;
		let name = tmux::commands::get_session_name()?;
		let current_path = tmux::commands::get_current_session_property("#{pane_current_path}")?;
		Ok(Self {
			state,
			name,
			current_path,
		})
	}

	fn get_popup_session_name(&self) -> String {
		if self.name.starts_with("popup") {
			self.name.clone()
		} else {
			format!("popup{}", self.name)
		}
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
				command: Some("tmux attach -t ".to_owned() + &self.get_popup_session_name()),
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

	/// Creates new tmux popup session without displaying it (detached).
	/// Popup sessions is defined as a session that starts with "popup" prefix.
	/// The path of the popup session is the same as the current session.
	pub fn ensure_popup_session_exist(&self) -> Result<()> {
		// TODO: check for duplication
		let popup_session_name = format!("popup{}", self.name);
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
		// Can only be ran after we have gotten the window index we wanted
		// or it will replace the index of the target window.
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
