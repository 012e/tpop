use std::{io, process::Command};

pub fn new_session_detached(name: &str, create_new: bool) -> Result<(), io::Error> {
    let mut cmd = Command::new("tmux");
    cmd.arg("new-session").arg("-d").arg("-s").arg(name);
    if !create_new {
        cmd.arg("-A");
    }
    cmd.spawn()?;
    Ok(())
}

pub fn new_session(name: &str, create_new: bool) -> Result<(), io::Error> {
    let mut cmd = Command::new("tmux");
    cmd.arg("new-session").arg("-s").arg(name);
    if !create_new {
        cmd.arg("-A");
    }
    cmd.spawn()?;
    Ok(())
}

pub fn run_command(command: &str, target: &str) -> Result<(), io::Error> {
    Command::new("tmux")
        .arg("send-keys")
        .arg("-t")
        .arg(target)
        .arg(command)
        .arg("Enter")
        .spawn()?;
    Ok(())
}

pub fn get_current_session_property(property: &str) -> Result<String, io::Error> {
    let output = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg(property)
        .output()?;
    Ok(String::from_utf8(output.stdout).unwrap().trim().into())
}

pub fn get_session_property(property: &str, target: &str) -> Result<String, io::Error> {
    let output = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg(property)
        .arg("-t")
        .arg(target)
        .output()?;
    Ok(String::from_utf8(output.stdout).unwrap())
}

pub fn get_session_name() -> Result<String, io::Error> {
    get_current_session_property("#S")
}
