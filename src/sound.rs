use rodio::*;

pub struct SoundGenerator {
    sink: Sink,
}

impl SoundGenerator {
    pub fn new() -> SoundGenerator {
        let frequency = source::SineWave::new(440.0);
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(1.);
        sink.append(frequency);
        sink.play();

        SoundGenerator { sink }
    }

    pub fn play(self: &Self) {
        self.sink.play();
    }

    pub fn pause(self: &Self) {
        self.sink.play();
    }
}
