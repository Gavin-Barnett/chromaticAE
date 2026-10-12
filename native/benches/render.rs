//! Standalone benchmark: old/new kernels, same input, alternating run order.
#[allow(dead_code)]
#[path = "../src/engine.rs"]
mod engine;
#[path = "reference_v21.rs"]
mod reference_v21;
use std::{hint::black_box, time::Instant};
struct Image {
    w: usize,
    h: usize,
    pixels: Vec<engine::Pixel>,
}
impl engine::Source for Image {
    fn size(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    fn pixel(&self, x: usize, y: usize) -> engine::Pixel {
        self.pixels[y * self.w + x]
    }
}
impl reference_v21::Source for Image {
    fn size(&self) -> (usize, usize) {
        (self.w, self.h)
    }
    fn pixel(&self, x: usize, y: usize) -> engine::Pixel {
        self.pixels[y * self.w + x]
    }
}
fn main() {
    let image = Image {
        w: 1920,
        h: 1080,
        pixels: (0..1920 * 1080)
            .map(|i| {
                let a = if i % 29 == 0 { 0.25 } else { 1.0 };
                [a * ((i % 251) as f32 / 250.0), a * 0.5, a * 0.2, a]
            })
            .collect(),
    };
    println!("1080p CPU kernel; one warm-up then median of five runs; alternating version order.");
    println!("Case | 2.1 ms | 2.2 ms | Speedup | 2.2 with restoration ms");
    for (name, sigma, fraction, roi) in [
        ("Integer split", [0.0, 0.0], 0.0, false),
        ("Fractional split", [0.0, 0.0], 0.37, false),
        ("Horizontal blur", [12.0, 0.0], 0.37, false),
        ("Both-axis blur", [12.0, 12.0], 0.37, false),
        ("Blur, 256x256 output region", [12.0, 12.0], 0.37, true),
    ] {
        let s = engine::Settings {
            channels: std::array::from_fn(|i| engine::Channel {
                offset: [(i as f32 - 1.0) * 8.0 + fraction, 0.0],
                colour: std::array::from_fn(|j| if i == j { 1.0 } else { 0.0 }),
                sigma,
            }),
            mix: 1.0,
            repeat: true,
            centre_restore: 0.0,
            reference_bounds: None,
        };
        let old = reference_v21::Settings {
            channels: std::array::from_fn(|i| reference_v21::Channel {
                offset: s.channels[i].offset,
                colour: s.channels[i].colour,
                sigma,
            }),
            mix: s.mix,
            repeat: s.repeat,
        };
        let size = if roi { (256, 256) } else { (1920, 1080) };
        let origin = if roi { [800.0, 400.0] } else { [0.0, 0.0] };
        let mut restored = s.clone();
        restored.centre_restore = 1.0;
        let run = |version| {
            let t = Instant::now();
            let pixels = if version == 0 {
                reference_v21::render(black_box(&image), size, origin, black_box(&old), || false)
                    .unwrap()
            } else {
                engine::render(
                    black_box(&image),
                    size,
                    origin,
                    black_box(if version == 2 { &restored } else { &s }),
                    || false,
                )
                .unwrap()
            };
            black_box(pixels);
            t.elapsed().as_secs_f64() * 1000.0
        };
        run(0);
        run(1);
        run(2);
        let mut times = [Vec::new(), Vec::new(), Vec::new()];
        for i in 0..5 {
            for j in 0..3 {
                let v = (i + j) % 3;
                times[v].push(run(v));
            }
        }
        for t in &mut times {
            t.sort_by(f64::total_cmp);
        }
        println!(
            "{name} | {:.2} | {:.2} | {:.2}x | {:.2}",
            times[0][2],
            times[1][2],
            times[0][2] / times[1][2],
            times[2][2]
        );
    }
}
