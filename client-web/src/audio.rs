#[cfg(target_arch = "wasm32")]
use web_sys::{AudioContext, OscillatorType};

pub struct SoundEffects;

impl SoundEffects {
    /// Kurzer, angenehmer Synth-Ping beim erfolgreichen Stufenaufstieg (C5 -> G5)
    pub fn play_success_ping() {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(ctx) = AudioContext::new() {
                if let Ok(osc) = ctx.create_oscillator() {
                    if let Ok(gain) = ctx.create_gain() {
                        let _ = osc.connect_with_audio_node(&gain);
                        let _ = gain.connect_with_audio_node(&ctx.destination());

                        osc.set_type(OscillatorType::Sine);
                        let now = ctx.current_time();
                        osc.frequency().set_value(523.25); // C5
                        let _ = osc.frequency().set_value_at_time(783.99, now + 0.1); // G5

                        let _ = gain.gain().set_value_at_time(0.15, now);
                        let _ = gain
                            .gain()
                            .exponential_ramp_to_value_at_time(0.001, now + 0.45);

                        let _ = osc.start();
                        let _ = osc.stop_with_when(now + 0.45);
                    }
                }
            }
        }
    }

    /// Warn-Ton bei kritischen Events (z. B. Signalverlust oder Drohne gestrandet)
    pub fn play_warning() {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(ctx) = AudioContext::new() {
                if let Ok(osc) = ctx.create_oscillator() {
                    if let Ok(gain) = ctx.create_gain() {
                        let _ = osc.connect_with_audio_node(&gain);
                        let _ = gain.connect_with_audio_node(&ctx.destination());

                        osc.set_type(OscillatorType::Triangle);
                        let now = ctx.current_time();
                        osc.frequency().set_value(330.0);

                        let _ = gain.gain().set_value_at_time(0.12, now);
                        let _ = gain
                            .gain()
                            .exponential_ramp_to_value_at_time(0.001, now + 0.5);

                        let _ = osc.start();
                        let _ = osc.stop_with_when(now + 0.5);
                    }
                }
            }
        }
    }
}
