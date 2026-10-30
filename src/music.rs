use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, OutputStream, Sink};

/// Fichiers cherchés dans `assets/`, dans cet ordre.
const TRACKS: [&str; 2] = ["musique.ogg", "musique.mp3"];

/// Musique de fond jouée en boucle. Si aucun fichier n'est trouvé ou qu'il
/// n'y a pas de sortie audio, le launcher tourne simplement sans musique.
pub struct Music {
    // Doit rester en vie tant que la musique joue, sinon le son se coupe.
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
}

impl Music {
    pub fn start(volume: f32, muted: bool) -> Self {
        let Some((stream, sink)) = open() else {
            return Self { _stream: None, sink: None };
        };

        let music = Self { _stream: Some(stream), sink: Some(sink) };
        music.set_volume(volume, muted);
        music
    }

    pub fn is_available(&self) -> bool {
        self.sink.is_some()
    }

    pub fn set_volume(&self, volume: f32, muted: bool) {
        if let Some(sink) = &self.sink {
            sink.set_volume(if muted { 0.0 } else { volume });
        }
    }
}

fn open() -> Option<(OutputStream, Sink)> {
    let path = TRACKS.iter().find_map(|name| crate::assets::find(name))?;
    let source = Decoder::new_looped(BufReader::new(File::open(path).ok()?)).ok()?;

    let (stream, handle) = OutputStream::try_default().ok()?;
    let sink = Sink::try_new(&handle).ok()?;
    sink.append(source);

    Some((stream, sink))
}
