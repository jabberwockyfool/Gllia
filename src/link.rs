use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

const SCHEME: &str = "gllia";
const PORT: u16 = 47_917;

pub fn register() -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(format!(r"Software\Classes\{SCHEME}"))?;
    key.set_value("", &"URL:Gllia")?;
    key.set_value("URL Protocol", &"")?;
    let (icon, _) = key.create_subkey("DefaultIcon")?;
    icon.set_value("", &format!("\"{}\",0", exe.display()))?;
    let (command, _) = key.create_subkey(r"shell\open\command")?;
    command.set_value("", &format!("\"{}\" \"%1\"", exe.display()))?;
    Ok(())
}

pub fn parse(arg: &str) -> Option<String> {
    let slug = arg.strip_prefix(&format!("{SCHEME}://installer/"))?.trim_end_matches('/');
    let valid = !slug.is_empty() && slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
    valid.then(|| slug.to_owned())
}

pub fn send_to_running(slug: &str) -> bool {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, PORT));
    match TcpStream::connect_timeout(&address, Duration::from_millis(300)) {
        Ok(mut stream) => writeln!(stream, "{slug}").is_ok(),
        Err(_) => false,
    }
}

pub fn listen(ctx: &egui::Context) -> Option<Receiver<String>> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, PORT)).ok()?;
    let (sender, receiver) = mpsc::channel();
    let ctx = ctx.clone();

    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut line = String::new();
            if BufReader::new(stream).read_line(&mut line).is_ok() {
                if let Some(slug) = parse(&format!("{SCHEME}://installer/{}", line.trim())) {
                    let _ = sender.send(slug);
                    ctx.request_repaint();
                }
            }
        }
    });

    Some(receiver)
}
