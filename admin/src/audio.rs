#[cfg(target_arch = "wasm32")]
use web_sys::{AudioContext, OscillatorType};

pub struct SoundEffects;

impl SoundEffects {
    /// Play a pleasant chime tone when ore is deposited / points awarded
    pub fn play_deposit_chime() {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(ctx) = AudioContext::new() {
                if let Ok(osc) = ctx.create_oscillator() {
                    if let Ok(gain) = ctx.create_gain() {
                        let _ = osc.connect_with_audio_node(&gain);
                        let _ = gain.connect_with_audio_node(&ctx.destination());

                        osc.set_type(OscillatorType::Sine);
                        osc.frequency().set_value(587.33); // D5

                        let now = ctx.current_time();
                        let _ = gain.gain().set_value_at_time(0.15, now);
                        let _ = gain
                            .gain()
                            .exponential_ramp_to_value_at_time(0.001, now + 0.4);

                        let _ = osc.start();
                        let _ = osc.stop_with_when(now + 0.4);
                    }
                }
            }
        }
    }

    /// Play a gentle low alarm tone when a ship is stranded / in distress
    pub fn play_distress_alarm() {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(ctx) = AudioContext::new() {
                if let Ok(osc) = ctx.create_oscillator() {
                    if let Ok(gain) = ctx.create_gain() {
                        let _ = osc.connect_with_audio_node(&gain);
                        let _ = gain.connect_with_audio_node(&ctx.destination());

                        osc.set_type(OscillatorType::Sawtooth);
                        osc.frequency().set_value(220.0); // A3

                        let now = ctx.current_time();
                        let _ = gain.gain().set_value_at_time(0.12, now);
                        let _ = gain
                            .gain()
                            .exponential_ramp_to_value_at_time(0.001, now + 0.6);

                        let _ = osc.start();
                        let _ = osc.stop_with_when(now + 0.6);
                    }
                }
            }
        }
    }
}
