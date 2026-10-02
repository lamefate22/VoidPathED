pub trait SoundPlayer: Send + Sync {
    fn play_success(&self);
    fn play_advance(&self);
    fn play_error(&self);
}
