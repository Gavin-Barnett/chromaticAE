//! Premultiplied, HDR-preserving channel separation and continuous edge softness.
pub type Pixel = [f32; 4]; // RGB, alpha
pub trait Source {
    fn size(&self) -> (usize, usize);
    fn pixel(&self, x: usize, y: usize) -> Pixel;
}
#[derive(Clone, Copy, Debug)]
pub struct Channel {
    pub offset: [f32; 2],
    pub colour: [f32; 3],
    pub sigma: [f32; 2],
}
#[derive(Clone, Debug)]
pub struct Settings {
    pub channels: [Channel; 3],
    pub mix: f32,
    pub repeat: bool,
}
#[derive(Debug, PartialEq)]
pub enum RenderError {
    Memory,
    Cancelled,
}
fn zeros<T: Default + Clone>(len: usize) -> Result<Vec<T>, RenderError> {
    let mut v = Vec::new();
    v.try_reserve_exact(len).map_err(|_| RenderError::Memory)?;
    v.resize(len, T::default());
    Ok(v)
}
fn index(x: i64, n: usize, repeat: bool) -> Option<usize> {
    if n == 0 {
        None
    } else if repeat {
        Some(x.clamp(0, n as i64 - 1) as usize)
    } else if x >= 0 && x < n as i64 {
        Some(x as usize)
    } else {
        None
    }
}
fn sample<const N: usize>(x: f32, y: f32, get: impl Fn(i64, i64) -> [f32; N]) -> [f32; N] {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let tx = x - x.floor();
    let ty = y - y.floor();
    let p = [
        get(ix, iy),
        get(ix + 1, iy),
        get(ix, iy + 1),
        get(ix + 1, iy + 1),
    ];
    let weights = [
        (1.0 - tx) * (1.0 - ty),
        tx * (1.0 - ty),
        (1.0 - tx) * ty,
        tx * ty,
    ];
    let mut result = [0.0; N];
    for c in 0..N {
        for k in 0..4 {
            result[c] += p[k][c] * weights[k];
        }
    }
    result
}
fn source_pixel(src: &impl Source, x: i64, y: i64, repeat: bool) -> Pixel {
    let (w, h) = src.size();
    match (index(x, w, repeat), index(y, h, repeat)) {
        (Some(x), Some(y)) => src.pixel(x, y),
        _ => [0.0; 4],
    }
}
// Three fractional box passes approximate a Gaussian. Fractional endpoints make
// animated softness continuous, including the transition from zero blur.
fn box_pass(
    input: &[[f32; 2]],
    output: &mut [[f32; 2]],
    size: (usize, usize),
    sigma: f32,
    vertical: bool,
    repeat: bool,
    abort: &mut impl FnMut() -> bool,
) -> Result<(), RenderError> {
    if sigma <= 0.0 {
        output.copy_from_slice(input);
        return Ok(());
    }
    let radius = ((1.0 + 4.0 * sigma * sigma).sqrt() - 1.0) * 0.5;
    let r = radius.floor() as i64;
    let f = (radius - r as f32) as f64;
    let norm = 2.0 * r as f64 + 1.0 + 2.0 * f;
    let (w, h) = size;
    let (lines, len) = if vertical { (w, h) } else { (h, w) };
    for line in 0..lines {
        if line % 32 == 0 && abort() {
            return Err(RenderError::Cancelled);
        }
        let at = |pos: i64| -> [f32; 2] {
            match index(pos, len, repeat) {
                Some(i) => input[if vertical { i * w + line } else { line * w + i }],
                None => [0.0; 2],
            }
        };
        let mut sum = [0.0f64; 2];
        for k in -r..=r {
            let p = at(k);
            for c in 0..2 {
                sum[c] += p[c] as f64;
            }
        }
        for pos in 0..len {
            let p = pos as i64;
            let left = at(p - r - 1);
            let right = at(p + r + 1);
            let dst = if vertical {
                pos * w + line
            } else {
                line * w + pos
            };
            for c in 0..2 {
                output[dst][c] = ((sum[c] + f * (left[c] as f64 + right[c] as f64)) / norm) as f32;
            }
            let remove = at(p - r);
            let add = at(p + r + 1);
            for c in 0..2 {
                sum[c] += add[c] as f64 - remove[c] as f64;
            }
        }
    }
    Ok(())
}
pub fn render(
    src: &impl Source,
    output_size: (usize, usize),
    origin: [f32; 2],
    settings: &Settings,
    mut abort: impl FnMut() -> bool,
) -> Result<Vec<Pixel>, RenderError> {
    let (w, h) = src.size();
    let (ow, oh) = output_size;
    let n = ow.checked_mul(oh).ok_or(RenderError::Memory)?;
    let mut result: Vec<Pixel> = zeros(n)?;
    if w == 0 || h == 0 {
        return Ok(result);
    }
    if settings.mix > 0.0 {
        for (c, ch) in settings.channels.iter().enumerate() {
            let blurred = ch.sigma[0] > 0.0 || ch.sigma[1] > 0.0;
            let mut field: Vec<[f32; 2]> = Vec::new();
            if blurred {
                let count = w.checked_mul(h).ok_or(RenderError::Memory)?;
                field = zeros(count)?;
                let mut scratch = zeros(count)?;
                for y in 0..h {
                    if y % 32 == 0 && abort() {
                        return Err(RenderError::Cancelled);
                    }
                    for x in 0..w {
                        let p = src.pixel(x, y);
                        field[y * w + x] = [p[c], p[3]];
                    }
                }
                for _ in 0..3 {
                    box_pass(
                        &field,
                        &mut scratch,
                        (w, h),
                        ch.sigma[0],
                        false,
                        settings.repeat,
                        &mut abort,
                    )?;
                    box_pass(
                        &scratch,
                        &mut field,
                        (w, h),
                        ch.sigma[1],
                        true,
                        settings.repeat,
                        &mut abort,
                    )?;
                }
            }
            for y in 0..oh {
                if y % 32 == 0 && abort() {
                    return Err(RenderError::Cancelled);
                }
                for x in 0..ow {
                    let sx = x as f32 + origin[0] - ch.offset[0];
                    let sy = y as f32 + origin[1] - ch.offset[1];
                    let p = sample(sx, sy, |ix, iy| {
                        if blurred {
                            match (index(ix, w, settings.repeat), index(iy, h, settings.repeat)) {
                                (Some(ix), Some(iy)) => field[iy * w + ix],
                                _ => [0.0; 2],
                            }
                        } else {
                            let p = source_pixel(src, ix, iy, settings.repeat);
                            [p[c], p[3]]
                        }
                    });
                    let out = &mut result[y * ow + x];
                    for j in 0..3 {
                        out[j] += p[0] * ch.colour[j];
                    }
                    out[3] = out[3].max(p[1]);
                }
            }
        }
    }
    if settings.mix < 1.0 {
        for y in 0..oh {
            if y % 32 == 0 && abort() {
                return Err(RenderError::Cancelled);
            }
            for x in 0..ow {
                let base = sample(x as f32 + origin[0], y as f32 + origin[1], |ix, iy| {
                    source_pixel(src, ix, iy, false)
                });
                for c in 0..4 {
                    result[y * ow + x][c] =
                        result[y * ow + x][c] * settings.mix + base[c] * (1.0 - settings.mix);
                }
            }
        }
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Image(usize, usize, Vec<Pixel>);
    impl Source for Image {
        fn size(&self) -> (usize, usize) {
            (self.0, self.1)
        }
        fn pixel(&self, x: usize, y: usize) -> Pixel {
            self.2[y * self.0 + x]
        }
    }
    fn settings() -> Settings {
        Settings {
            channels: std::array::from_fn(|i| Channel {
                offset: [0.0; 2],
                colour: std::array::from_fn(|j| if i == j { 1.0 } else { 0.0 }),
                sigma: [0.0; 2],
            }),
            mix: 1.0,
            repeat: false,
        }
    }
    fn run(src: &Image, s: &Settings) -> Vec<Pixel> {
        render(src, src.size(), [0.0; 2], s, || false).unwrap()
    }
    fn near(a: f32, b: f32) {
        assert!((a - b).abs() < 0.00001, "{a} != {b}");
    }
    #[test]
    fn identity_retains_hdr_and_premultiplication() {
        let im = Image(2, 1, vec![[2.0, 0.1, -0.2, 1.0], [0.2, 0.1, 0.05, 0.25]]);
        assert_eq!(run(&im, &settings()), im.2);
    }
    #[test]
    fn signed_independent_offsets() {
        let mut p = vec![[0.0; 4]; 5];
        p[2] = [1.0; 4];
        let im = Image(5, 1, p);
        let mut s = settings();
        s.channels[0].offset[0] = -1.0;
        s.channels[2].offset[0] = 1.0;
        let p = run(&im, &s);
        assert_eq!(p[1], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(p[2], [0.0, 1.0, 0.0, 1.0]);
        assert_eq!(p[3], [0.0, 0.0, 1.0, 1.0]);
    }
    #[test]
    fn green_to_teal() {
        let im = Image(1, 1, vec![[0.0, 0.5, 0.0, 1.0]]);
        let mut s = settings();
        s.channels[1].colour = [0.0, 0.8, 0.7];
        let p = run(&im, &s)[0];
        near(p[1], 0.4);
        near(p[2], 0.35);
    }
    #[test]
    fn fractional_offset() {
        let im = Image(2, 1, vec![[1.0; 4], [0.0; 4]]);
        let mut s = settings();
        s.channels[0].offset[0] = 0.5;
        near(run(&im, &s)[1][0], 0.5);
    }
    #[test]
    fn blur_conserves_energy() {
        let mut pixels = vec![[0.0; 4]; 41];
        pixels[20] = [1.0; 4];
        let im = Image(41, 1, pixels);
        let mut s = settings();
        s.repeat = true;
        s.channels[0].sigma = [2.0, 0.0];
        let p = run(&im, &s);
        near(p.iter().map(|p| p[0]).sum(), 1.0);
        assert!(p[19][0] > 0.0);
        near(p[19][0], p[21][0]);
        assert_eq!(p[19][1], 0.0);
    }
    #[test]
    fn blur_preserves_constant() {
        let im = Image(9, 7, vec![[0.3, 0.2, 0.1, 0.5]; 63]);
        let mut s = settings();
        s.repeat = true;
        for ch in &mut s.channels {
            ch.sigma = [3.4, 1.1];
        }
        for p in run(&im, &s) {
            for (a, b) in p.into_iter().zip(im.2[0]) {
                near(a, b);
            }
        }
    }
    #[test]
    fn no_colour_bleed_from_transparent_black() {
        let im = Image(9, 1, vec![[0.0; 4]; 9]);
        let mut s = settings();
        s.channels[1].sigma = [2.0, 0.0];
        assert_eq!(run(&im, &s), im.2);
    }
    #[test]
    fn zero_mix_bypasses() {
        let im = Image(1, 1, vec![[0.2, 0.3, 0.4, 0.5]]);
        let mut s = settings();
        s.mix = 0.0;
        s.channels[0].offset = [100.0, 100.0];
        assert_eq!(run(&im, &s), im.2);
    }
    #[test]
    fn cropped_output_origin() {
        let im = Image(3, 1, vec![[1.0; 4], [0.5; 4], [0.2; 4]]);
        let p = render(&im, (1, 1), [1.0, 0.0], &settings(), || false).unwrap();
        assert_eq!(p[0], [0.5; 4]);
    }
    #[test]
    fn cancellation() {
        let im = Image(1, 1, vec![[1.0; 4]]);
        assert_eq!(
            render(&im, (1, 1), [0.0; 2], &settings(), || true).unwrap_err(),
            RenderError::Cancelled
        );
    }
    #[test]
    fn repeated_edge() {
        let im = Image(1, 1, vec![[1.0; 4]]);
        let mut s = settings();
        s.repeat = true;
        s.channels[0].offset = [10.0, 20.0];
        assert_eq!(run(&im, &s), im.2);
        s.repeat = false;
        assert_eq!(run(&im, &s)[0][0], 0.0);
    }
}
