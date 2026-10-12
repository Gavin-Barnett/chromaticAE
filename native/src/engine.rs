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
    pub centre_restore: f32,
    /// Original layer bounds in input-buffer coordinates, independent of ROI/expansion.
    pub reference_bounds: Option<[f32; 4]>,
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
    // Exact integer and one-dimensional translations are common in AE. Avoid
    // loading neighbours whose bilinear weights are zero.
    if tx == 0.0 && ty == 0.0 {
        return get(ix, iy);
    }
    if ty == 0.0 {
        let a = get(ix, iy);
        let b = get(ix + 1, iy);
        return std::array::from_fn(|c| a[c] * (1.0 - tx) + b[c] * tx);
    }
    if tx == 0.0 {
        let a = get(ix, iy);
        let b = get(ix, iy + 1);
        return std::array::from_fn(|c| a[c] * (1.0 - ty) + b[c] * ty);
    }
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
    std::array::from_fn(|c| {
        p[0][c] * weights[0] + p[1][c] * weights[1] + p[2][c] * weights[2] + p[3][c] * weights[3]
    })
}
fn source_pixel(src: &impl Source, x: i64, y: i64, repeat: bool) -> Pixel {
    let (w, h) = src.size();
    match (index(x, w, repeat), index(y, h, repeat)) {
        (Some(x), Some(y)) => src.pixel(x, y),
        _ => [0.0; 4],
    }
}
/// Full finite support of the three fractional box passes (in render pixels).
pub fn blur_support(sigma: f32) -> usize {
    if sigma <= 0.0 {
        0
    } else {
        (3.0 * (((1.0 + 4.0 * sigma * sigma).sqrt() - 1.0) * 0.5).ceil()) as usize
    }
}
/// Additional output pixels, in left/top/right/bottom order. Retain the original
/// canvas as well, so Mix and edge repetition have a stable reference.
pub fn expansion(settings: &Settings) -> [i32; 4] {
    let mut pad = [0.0f32; 4];
    if settings.mix > 0.0 {
        for ch in &settings.channels {
            let rx = blur_support(ch.sigma[0]) as f32;
            let ry = blur_support(ch.sigma[1]) as f32;
            pad[0] = pad[0].max(rx - ch.offset[0]);
            pad[1] = pad[1].max(ry - ch.offset[1]);
            pad[2] = pad[2].max(rx + ch.offset[0]);
            pad[3] = pad[3].max(ry + ch.offset[1]);
        }
    }
    pad.map(|p| p.ceil() as i32)
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
    if vertical {
        // Accumulate complete rows. Scanning down each column caused a cache
        // miss per pixel on larger frames; adjacent columns share the row cache.
        let mut sums: Vec<[f64; 2]> = zeros(w)?;
        let zero: Vec<[f32; 2]> = zeros(w)?;
        let row = |y: i64| -> &[[f32; 2]] {
            match index(y, h, repeat) {
                Some(y) => &input[y * w..(y + 1) * w],
                None => &zero,
            }
        };
        for y in -r..=r {
            let p = row(y);
            for x in 0..w {
                for c in 0..2 {
                    sums[x][c] += p[x][c] as f64;
                }
            }
        }
        for y in 0..h {
            if y % 32 == 0 && abort() {
                return Err(RenderError::Cancelled);
            }
            let i = y as i64;
            let left = row(i - r - 1);
            let right = row(i + r + 1);
            let remove = row(i - r);
            let out = &mut output[y * w..(y + 1) * w];
            for x in 0..w {
                for c in 0..2 {
                    out[x][c] =
                        ((sums[x][c] + f * (left[x][c] as f64 + right[x][c] as f64)) / norm) as f32;
                    sums[x][c] += right[x][c] as f64 - remove[x][c] as f64;
                }
            }
        }
        return Ok(());
    }
    let (lines, len) = (h, w);
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
/// Region of the padded source required for this channel's requested output.
fn work_rect(
    size: (usize, usize),
    out: (usize, usize),
    origin: [f32; 2],
    ch: &Channel,
    repeat: bool,
) -> [i64; 4] {
    let axis = |n: usize, len: usize, o: f32, offset: f32, sigma: f32| -> (i64, i64) {
        let p = blur_support(sigma) as i64;
        let lo = -p;
        let hi = n as i64 + p;
        if len == 0 || hi <= lo {
            return (0, 0);
        }
        let a = (o - offset).floor() as i64;
        let b = (o + (len - 1) as f32 - offset).ceil() as i64;
        let (a, b) = if repeat {
            (a.clamp(lo, hi - 1), b.clamp(lo, hi - 1))
        } else {
            (a.max(lo), b.min(hi - 1))
        };
        if a > b {
            return (0, 0);
        }
        ((a - p).max(lo), (b + 1 + p).min(hi))
    };
    let (l, r) = axis(size.0, out.0, origin[0], ch.offset[0], ch.sigma[0]);
    let (t, b) = axis(size.1, out.1, origin[1], ch.offset[1], ch.sigma[1]);
    [l, t, r, b]
}
fn centre_weight(x: f32, y: f32, size: (usize, usize), settings: &Settings) -> f32 {
    if settings.centre_restore <= 0.0 {
        return 0.0;
    }
    let [l, t, r, b] =
        settings
            .reference_bounds
            .unwrap_or([0.0, 0.0, size.0 as f32, size.1 as f32]);
    let rx = (r - l) * 0.5;
    let ry = (b - t) * 0.5;
    if rx <= 0.0 || ry <= 0.0 {
        return 0.0;
    }
    let nx = (x + 0.5 - (l + rx)) / rx;
    let ny = (y + 0.5 - (t + ry)) / ry;
    let radius = (nx * nx + ny * ny).sqrt();
    // Keep an inner plateau completely original at 100%, then use a smooth
    // elliptical falloff. No original-image blend at the outer layer edges.
    let fade = ((radius - 0.35) / 0.55).clamp(0.0, 1.0);
    settings.centre_restore.clamp(0.0, 1.0) * (1.0 - fade * fade * (3.0 - 2.0 * fade))
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
            // Only blur pixels needed by this output region, plus complete
            // convolution support. The host's full input remains available for
            // stable repeat-edge semantics and effects earlier in the stack.
            let blur_rect = work_rect((w, h), output_size, origin, ch, settings.repeat);
            let [left, top, right, bottom] = blur_rect;
            let fw = (right - left).max(0) as usize;
            let fh = (bottom - top).max(0) as usize;
            if blurred {
                if fw == 0 || fh == 0 {
                    continue;
                }
                let count = fw.checked_mul(fh).ok_or(RenderError::Memory)?;
                field = zeros(count)?;
                let mut scratch = zeros(count)?;
                for y in 0..fh {
                    if y % 32 == 0 && abort() {
                        return Err(RenderError::Cancelled);
                    }
                    for x in 0..fw {
                        let p = source_pixel(src, x as i64 + left, y as i64 + top, settings.repeat);
                        field[y * fw + x] = [p[c], p[3]];
                    }
                }
                for _ in 0..3 {
                    if ch.sigma[0] > 0.0 {
                        box_pass(
                            &field,
                            &mut scratch,
                            (fw, fh),
                            ch.sigma[0],
                            false,
                            settings.repeat,
                            &mut abort,
                        )?;
                        std::mem::swap(&mut field, &mut scratch);
                    }
                    if ch.sigma[1] > 0.0 {
                        box_pass(
                            &field,
                            &mut scratch,
                            (fw, fh),
                            ch.sigma[1],
                            true,
                            settings.repeat,
                            &mut abort,
                        )?;
                        std::mem::swap(&mut field, &mut scratch);
                    }
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
                            match (
                                index(ix - left, fw, settings.repeat),
                                index(iy - top, fh, settings.repeat),
                            ) {
                                (Some(ix), Some(iy)) => field[iy * fw + ix],
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
    if settings.mix < 1.0 || settings.centre_restore > 0.0 {
        for y in 0..oh {
            if y % 32 == 0 && abort() {
                return Err(RenderError::Cancelled);
            }
            for x in 0..ow {
                let base = sample(x as f32 + origin[0], y as f32 + origin[1], |ix, iy| {
                    source_pixel(src, ix, iy, false)
                });
                let protect =
                    centre_weight(x as f32 + origin[0], y as f32 + origin[1], (w, h), settings);
                let mix = settings.mix * (1.0 - protect);
                for c in 0..4 {
                    result[y * ow + x][c] = result[y * ow + x][c] * mix + base[c] * (1.0 - mix);
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
            centre_restore: 0.0,
            reference_bounds: None,
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
    #[test]
    fn expanded_blur_preserves_energy_past_source_edges() {
        let im = Image(1, 1, vec![[1.0; 4]]);
        let mut s = settings();
        for ch in &mut s.channels {
            ch.sigma = [2.0, 2.0];
        }
        let p = render(&im, (21, 21), [-10.0, -10.0], &s, || false).unwrap();
        near(p.iter().map(|p| p[0]).sum(), 1.0);
        assert!(p[10 * 21 + 8][3] > 0.0);
        assert!(p[8 * 21 + 10][3] > 0.0);
        assert_eq!(p[0], [0.0; 4]);
    }
    #[test]
    fn horizontal_blur_has_no_vertical_spread() {
        let im = Image(1, 1, vec![[1.0; 4]]);
        let mut s = settings();
        for ch in &mut s.channels {
            ch.sigma = [2.0, 0.0];
        }
        let p = render(&im, (21, 5), [-10.0, -2.0], &s, || false).unwrap();
        assert!(p[2 * 21 + 8][3] > 0.0);
        assert_eq!(p[21 + 10], [0.0; 4]);
        near(p.iter().map(|p| p[0]).sum(), 1.0);
    }
    #[test]
    fn vertical_blur_has_no_horizontal_spread() {
        let im = Image(1, 1, vec![[1.0; 4]]);
        let mut s = settings();
        for ch in &mut s.channels {
            ch.sigma = [0.0, 2.0];
        }
        let p = render(&im, (5, 21), [-2.0, -10.0], &s, || false).unwrap();
        assert!(p[8 * 5 + 2][3] > 0.0);
        assert_eq!(p[10 * 5 + 1], [0.0; 4]);
        near(p.iter().map(|p| p[0]).sum(), 1.0);
    }
    #[test]
    fn expansion_covers_signed_offsets_and_blur_support() {
        let mut s = settings();
        s.channels[0].offset = [-7.25, 3.5];
        s.channels[0].sigma = [2.0, 0.0];
        s.channels[2].offset = [9.5, -12.0];
        assert_eq!(blur_support(2.0), 6);
        assert_eq!(expansion(&s), [14, 12, 10, 4]);
        s.mix = 0.0;
        assert_eq!(expansion(&s), [0; 4]);
    }
    #[test]
    fn expanded_region_matches_cropped_region() {
        let im = Image(2, 2, vec![[1.0; 4], [0.0; 4], [0.0; 4], [0.5; 4]]);
        let mut s = settings();
        s.channels[0].sigma = [2.3, 1.2];
        s.channels[0].offset = [-2.5, 0.4];
        let all = render(&im, (20, 20), [-10.0, -10.0], &s, || false).unwrap();
        let roi = render(&im, (2, 3), [-3.0, -2.0], &s, || false).unwrap();
        for y in 0..3 {
            for x in 0..2 {
                assert_eq!(roi[y * 2 + x], all[(y + 8) * 20 + x + 7]);
            }
        }
    }
    impl crate::reference_v21::Source for Image {
        fn size(&self) -> (usize, usize) {
            (self.0, self.1)
        }
        fn pixel(&self, x: usize, y: usize) -> Pixel {
            self.2[y * self.0 + x]
        }
    }
    #[test]
    fn optimized_renderer_matches_v21_across_regions() {
        use crate::reference_v21 as old;
        let im = Image(
            43,
            37,
            (0..43 * 37)
                .map(|i| {
                    let a = if i % 11 == 0 {
                        0.0
                    } else if i % 3 == 0 {
                        0.25
                    } else {
                        1.0
                    };
                    [
                        a * ((i * 71 % 113) as f32 / 40.0),
                        a * ((i * 43 % 79) as f32 / 55.0),
                        a * ((i * 17 % 97) as f32 / 80.0),
                        a,
                    ]
                })
                .collect(),
        );
        for repeat in [false, true] {
            for mix in [0.0, 0.35, 1.0] {
                for sigma in [[0.0, 0.0], [3.2, 0.0], [0.0, 2.7], [2.3, 0.6]] {
                    let mut s = settings();
                    s.repeat = repeat;
                    s.mix = mix;
                    for (i, c) in s.channels.iter_mut().enumerate() {
                        c.sigma = sigma;
                        c.offset = [(i as f32 - 1.0) * 3.7, 1.2 * i as f32];
                        c.colour = [0.2, 0.8, 1.5];
                    }
                    let o = old::Settings {
                        channels: s.channels.map(|c| old::Channel {
                            offset: c.offset,
                            colour: c.colour,
                            sigma: c.sigma,
                        }),
                        mix,
                        repeat,
                    };
                    for (size, origin) in [
                        ((55, 49), [-6.0, -5.0]),
                        ((9, 7), [4.0, 8.0]),
                        ((5, 5), [-70.0, 90.0]),
                        ((7, 9), [38.0, -8.0]),
                    ] {
                        let expected = old::render(&im, size, origin, &o, || false).unwrap();
                        let actual = render(&im, size, origin, &s, || false).unwrap();
                        for (a, b) in actual.iter().zip(expected) {
                            for c in 0..4 {
                                assert!(
                                    (a[c] - b[c]).abs() < 0.00003,
                                    "repeat={repeat} mix={mix} sigma={sigma:?} origin={origin:?}: {a:?} != {b:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn centre_restore_protects_middle_and_keeps_edges() {
        let im = Image(101, 101, vec![[0.2, 0.5, 0.1, 1.0]; 101 * 101]);
        let mut s = settings();
        s.channels[1].colour = [0.0, 0.8, 0.7];
        let aberrated = run(&im, &s);
        s.centre_restore = 1.0;
        let restored = run(&im, &s);
        assert_eq!(restored[50 * 101 + 50], im.2[0]);
        assert_eq!(restored[50 * 101], aberrated[50 * 101]);
        let intermediate = restored[50 * 101 + 80];
        assert!(intermediate[2] > im.2[0][2] && intermediate[2] < aberrated[0][2]);
        s.centre_restore = 0.5;
        let half = run(&im, &s);
        near(half[50 * 101 + 50][2], (im.2[0][2] + aberrated[0][2]) * 0.5);
    }
    #[test]
    fn centre_restore_is_stable_under_roi_and_upstream_expansion() {
        let im = Image(151, 151, vec![[0.2, 0.4, 0.1, 0.5]; 151 * 151]);
        let mut s = settings();
        s.centre_restore = 1.0;
        s.reference_bounds = Some([25.0, 25.0, 126.0, 126.0]);
        s.channels[1].colour = [0.0, 0.8, 0.7];
        let all = run(&im, &s);
        let crop = render(&im, (9, 7), [71.0, 72.0], &s, || false).unwrap();
        for y in 0..7 {
            for x in 0..9 {
                assert_eq!(crop[y * 9 + x], all[(y + 72) * 151 + x + 71]);
            }
        }
        assert_eq!(all[75 * 151 + 75], im.2[0]);
    }
    #[test]
    fn centre_restore_keeps_expanded_fringe_and_transparency() {
        let im = Image(21, 21, vec![[0.1, 0.3, 0.2, 0.5]; 441]);
        let mut s = settings();
        s.channels[0].offset = [-5.0, 0.0];
        s.channels[2].offset = [5.0, 0.0];
        let original = render(&im, (35, 21), [-7.0, 0.0], &s, || false).unwrap();
        s.centre_restore = 1.0;
        let restored = render(&im, (35, 21), [-7.0, 0.0], &s, || false).unwrap();
        assert_eq!(restored[10 * 35 + 3], original[10 * 35 + 3]);
        assert_eq!(restored[10 * 35 + 17], im.2[0]);
        assert_eq!(restored[0], [0.0; 4]);
    }
}
