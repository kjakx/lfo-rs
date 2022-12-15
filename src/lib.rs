use std::f64::consts::TAU;
use dasp_signal::{Signal};

fn phase(freq: f64, time: f64, theta: f64) -> f64 {
    (freq * time + theta).fract()
}

fn sine(phase: f64) -> f64 {
    (TAU * phase).sin()
}

fn triangle(phase: f64) -> f64 {
    if phase < 0.5 {
        4.0 * phase - 1.0
    } else {
        3.0 - 4.0 * phase
    }
}

fn saw(phase: f64) -> f64 {
    2.0 * phase - 1.0
}

fn square(phase: f64) -> f64 {
    if phase < 0.5 {
        1.0
    } else {
        -1.0
    }
}

pub enum LFOWaveform {
    Sine,
    Triangle,
    Saw,
    Square,
}

struct LFO {
    waveform: LFOWaveform,
    freq: f64,
    theta: f64,
    gain: f64, // -1.0 <= g <= 1.0
    time_step: f64,
    sample_rate: f64,
}

impl LFO {
    pub fn new(waveform: LFOWaveform, freq: f64, sample_rate: f64) -> Self {
        LFO {
            waveform: waveform,
            freq: freq,
            theta: 0.0,
            gain: 1.0,
            time_step: 0.0,
            sample_rate: sample_rate,
        }
    }

    pub fn set_waveform(&mut self, waveform: LFOWaveform) {
        self.waveform = waveform;
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
    }

    pub fn set_theta(&mut self, theta: f64) {
        self.theta = theta;
    }

    pub fn set_gain(&mut self, gain: f64) {
        self.gain = gain;
    }

    fn increment_time_step(&mut self) {
        self.time_step = ((self.time_step + 1.0) as usize % self.sample_rate as usize) as f64;
    }

    fn generate(&mut self) -> f64 {
        use LFOWaveform::*;
        let phase = phase(self.freq, self.time_step as f64 / self.sample_rate, self.theta);
        self.increment_time_step();
        match self.waveform {
            Sine => {
                sine(phase)
            },
            Triangle => {
                triangle(phase)
            },
            Saw => {
                saw(phase)
            },
            Square => {
                square(phase)
            },
        }
    }
}

impl Signal for LFO {
    type Frame = f64;

    fn next(&mut self) -> Self::Frame {
        self.gain * self.generate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotters::prelude::*;

    fn create_chart(lfo: &mut LFO, t_sec: f64, filename: &str, cap: &str) {
        let data_len: usize = (lfo.sample_rate * t_sec) as usize;
        let lfo_vec: Vec<f64> = (0..=data_len).map(|_i| {
            lfo.next()
        }).collect();

        let root = BitMapBackend::new(filename, (1024, 768)).into_drawing_area();

        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .set_label_area_size(LabelAreaPosition::Left, 60)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .caption(cap, ("sans-serif", 40))
            .build_cartesian_2d(-0.1_f64..t_sec+0.1, -1.1_f64..1.1_f64)
            .unwrap();

        chart
            .configure_mesh()
            //.disable_x_mesh()
            //.disable_y_mesh()
            .draw()
            .unwrap();

        chart.draw_series(
            AreaSeries::new(
                (0..=data_len).zip(lfo_vec.iter()).map(|(x, y)| ((x as f64 / lfo.sample_rate, *y))),
                0.0,
                &RED.mix(0.2),
            )
            .border_style(&RED),
        ).unwrap();

        // To avoid the IO failure being ignored silently, we manually call the present function
        root.present().unwrap();
    }

    #[test]
    fn sine_10hz() {
        let mut lfo = LFO::new(LFOWaveform::Sine, 10.0, 1000.0);
        create_chart(&mut lfo, 1.0, "chart/sine_10hz.png", "sine_10hz");
    }

    #[test]
    fn triangle_3hz() {
        let mut lfo = LFO::new(LFOWaveform::Triangle, 3.0, 1000.0);
        lfo.set_theta(0.25);
        create_chart(&mut lfo, 1.0, "chart/triangle_3hz.png", "triangle_3hz");
    }

    #[test]
    fn saw_5hz() {
        let mut lfo = LFO::new(LFOWaveform::Saw, 5.0, 1000.0);
        lfo.set_theta(0.5);
        create_chart(&mut lfo, 1.0, "chart/saw_5hz.png", "saw_5hz");
    }

    #[test]
    fn square_2hz() {
        let mut lfo = LFO::new(LFOWaveform::Square, 2.0, 1000.0);
        lfo.set_gain(0.5);
        create_chart(&mut lfo, 1.0, "chart/square_2hz.png", "square_2hz");
    }
}
