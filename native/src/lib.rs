use after_effects as ae;
mod engine;
#[cfg(test)]
#[path = "../benches/reference_v21.rs"]
mod reference_v21;
use engine::{Channel, Settings, Source};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
enum Params {
    Amount,
    Direction,
    Softness,
    Mix,
    Repeat,
    Red,
    RedDistance,
    RedX,
    RedY,
    RedColour,
    RedSoftness,
    RedEnd,
    Green,
    GreenDistance,
    GreenX,
    GreenY,
    GreenColour,
    GreenSoftness,
    GreenEnd,
    Blue,
    BlueDistance,
    BlueX,
    BlueY,
    BlueColour,
    BlueSoftness,
    BlueEnd,
    // Append enum variants to preserve the persisted IDs of every 2.0 control.
    BlurDimensions,
    RedDimensions,
    GreenDimensions,
    BlueDimensions,
    ExpandBounds,
    CentreRestore,
}
#[derive(Default)]
struct Plugin;
ae::define_effect!(Plugin, (), Params);

fn slider(
    params: &mut ae::Parameters<Params>,
    id: Params,
    name: &str,
    default: f64,
    min: f32,
    max: f32,
    percent: bool,
) -> Result<(), Error> {
    params.add(
        id,
        name,
        ae::FloatSliderDef::setup(|f| {
            f.set_valid_min(min);
            f.set_valid_max(max);
            f.set_slider_min(min.max(-100.0));
            f.set_slider_max(max.min(100.0));
            f.set_default(default);
            f.set_value(default);
            f.set_precision(1);
            if percent {
                f.set_display_flags(ae::ValueDisplayFlag::PERCENT);
            }
        }),
    )
}
fn scalar(params: &ae::Parameters<Params>, id: Params) -> Result<f32, Error> {
    Ok(params.get(id)?.as_float_slider()?.value() as f32)
}
fn settings(params: &ae::Parameters<Params>, data: InData) -> Result<Settings, Error> {
    let amount = scalar(params, Params::Amount)? / 100.0;
    let angle = params.get(Params::Direction)?.as_angle()?.float_value()? as f32
        * std::f32::consts::PI
        / 180.0;
    let softness = scalar(params, Params::Softness)?;
    let sx = f32::from(data.downsample_x());
    let sy = f32::from(data.downsample_y());
    let dimensions = params.get(Params::BlurDimensions)?.as_popup()?.value();
    let mut channels = [Channel {
        offset: [0.0; 2],
        colour: [0.0; 3],
        sigma: [0.0; 2],
    }; 3];
    let keys = [
        [
            Params::RedDistance,
            Params::RedX,
            Params::RedY,
            Params::RedColour,
            Params::RedSoftness,
        ],
        [
            Params::GreenDistance,
            Params::GreenX,
            Params::GreenY,
            Params::GreenColour,
            Params::GreenSoftness,
        ],
        [
            Params::BlueDistance,
            Params::BlueX,
            Params::BlueY,
            Params::BlueColour,
            Params::BlueSoftness,
        ],
    ];
    for (i, k) in keys.iter().enumerate() {
        let distance = scalar(params, k[0])?;
        let col = params.get(k[3])?.as_color()?.float_value()?;
        let blur = (softness + scalar(params, k[4])?).max(0.0);
        let dimension_key = [
            Params::RedDimensions,
            Params::GreenDimensions,
            Params::BlueDimensions,
        ][i];
        let channel_dimensions = params.get(dimension_key)?.as_popup()?.value();
        let mode = if channel_dimensions == 1 {
            dimensions
        } else {
            channel_dimensions - 1
        };
        // 1 = both, 2 = horizontal, 3 = vertical.
        channels[i] = Channel {
            offset: [
                (distance * angle.cos() + scalar(params, k[1])?) * amount * sx,
                (distance * angle.sin() + scalar(params, k[2])?) * amount * sy,
            ],
            colour: [col.red, col.green, col.blue],
            sigma: [
                if mode == 3 { 0.0 } else { blur * sx },
                if mode == 2 { 0.0 } else { blur * sy },
            ],
        };
    }
    Ok(Settings {
        channels,
        mix: (scalar(params, Params::Mix)? / 100.0).clamp(0.0, 1.0),
        repeat: params.get(Params::Repeat)?.as_checkbox()?.value(),
        centre_restore: (scalar(params, Params::CentreRestore)? / 100.0).clamp(0.0, 1.0),
        reference_bounds: None,
    })
}
struct World<'a>(&'a ae::Layer, i16);
impl Source for World<'_> {
    fn size(&self) -> (usize, usize) {
        (self.0.width(), self.0.height())
    }
    fn pixel(&self, x: usize, y: usize) -> engine::Pixel {
        let mut pixel = match self.1 {
            8 => {
                let p = self.0.as_pixel8(x, y);
                [
                    p.red as f32 / 255.0,
                    p.green as f32 / 255.0,
                    p.blue as f32 / 255.0,
                    p.alpha as f32 / 255.0,
                ]
            }
            16 => {
                let p = self.0.as_pixel16(x, y);
                [
                    p.red as f32 / 32768.0,
                    p.green as f32 / 32768.0,
                    p.blue as f32 / 32768.0,
                    p.alpha as f32 / 32768.0,
                ]
            }
            _ => {
                let p = self.0.as_pixel32(x, y);
                [p.red, p.green, p.blue, p.alpha]
            }
        };
        // AE effect worlds expose straight RGB. Filter in premultiplied space
        // so transparent pixels cannot darken coloured edges.
        pixel[3] = pixel[3].clamp(0.0, 1.0);
        for c in 0..3 {
            pixel[c] *= pixel[3];
        }
        pixel
    }
}
fn render(
    input: &ae::Layer,
    output: &mut ae::Layer,
    data: InData,
    params: &ae::Parameters<Params>,
    legacy: bool,
) -> Result<(), Error> {
    let mut s = settings(params, data)?;
    let depth = input.bit_depth();
    if ![8, 16, 32].contains(&depth) || output.bit_depth() != depth {
        return Err(Error::BadCallbackParameter);
    }
    let source = World(input, depth);
    let a = input.origin();
    let b = output.origin();
    // Keep the restoration centred on the original layer, not on a cropped ROI
    // or a buffer expanded by an earlier effect.
    let source_origin = if legacy {
        data.pre_effect_source_origin()
    } else {
        ae::Point { h: -a.h, v: -a.v }
    };
    s.reference_bounds = Some([
        source_origin.h as f32,
        source_origin.v as f32,
        source_origin.h as f32 + data.width() as f32 * f32::from(data.downsample_x()),
        source_origin.v as f32 + data.height() as f32 * f32::from(data.downsample_y()),
    ]);
    let pixels = engine::render(
        &source,
        (output.width(), output.height()),
        if legacy {
            let o = data.output_origin();
            [-(o.h as f32), -(o.v as f32)]
        } else {
            [(b.h - a.h) as f32, (b.v - a.v) as f32]
        },
        &s,
        || data.interact().abort().is_err(),
    )
    .map_err(|e| match e {
        engine::RenderError::Memory => Error::OutOfMemory,
        engine::RenderError::Cancelled => Error::InterruptCancel,
    })?;
    #[cfg(feature = "trace")]
    if let Ok(path) = std::env::var("CHROMATIC_AE_TRACE") {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let (w, h) = source.size();
            if w > 0 && h > 0 {
                let _ = writeln!(
                    f,
                    "depth={depth} size={w}x{h} in-origin={a:?} out-origin={b:?} centre={:?} settings={s:?} out29={:?}",
                    source.pixel(w / 2, h / 2),
                    pixels.get(output.width() * 64 + 29)
                );
            }
        }
    }
    for y in 0..output.height() {
        if y % 32 == 0 {
            data.interact().abort()?;
        }
        for x in 0..output.width() {
            let mut p = pixels[y * output.width() + x];
            // Return straight RGB to the host after coverage-aware filtering.
            if p[3] > 0.0 {
                for c in 0..3 {
                    p[c] /= p[3];
                }
            } else {
                p[0] = 0.0;
                p[1] = 0.0;
                p[2] = 0.0;
            }
            match depth {
                8 => {
                    let q = output.as_pixel8_mut(x, y);
                    q.red = (p[0] * 255.0).round().clamp(0.0, 255.0) as u8;
                    q.green = (p[1] * 255.0).round().clamp(0.0, 255.0) as u8;
                    q.blue = (p[2] * 255.0).round().clamp(0.0, 255.0) as u8;
                    q.alpha = (p[3] * 255.0).round().clamp(0.0, 255.0) as u8;
                }
                16 => {
                    let q = output.as_pixel16_mut(x, y);
                    q.red = (p[0] * 32768.0).round().clamp(0.0, 32768.0) as u16;
                    q.green = (p[1] * 32768.0).round().clamp(0.0, 32768.0) as u16;
                    q.blue = (p[2] * 32768.0).round().clamp(0.0, 32768.0) as u16;
                    q.alpha = (p[3] * 32768.0).round().clamp(0.0, 32768.0) as u16;
                }
                _ => {
                    let q = output.as_pixel32_mut(x, y);
                    q.red = p[0];
                    q.green = p[1];
                    q.blue = p[2];
                    q.alpha = p[3].clamp(0.0, 1.0);
                }
            }
        }
    }
    Ok(())
}
impl AdobePluginGlobal for Plugin {
    fn params_setup(
        &self,
        params: &mut ae::Parameters<Params>,
        _in_data: InData,
        _: OutData,
    ) -> Result<(), Error> {
        slider(
            params,
            Params::Amount,
            "Split Amount",
            100.0,
            -1000.0,
            1000.0,
            true,
        )?;
        params.add(
            Params::Direction,
            "Direction",
            ae::AngleDef::setup(|f| {
                f.set_default(0.0);
            }),
        )?;
        slider(
            params,
            Params::Softness,
            "Edge Softness",
            0.0,
            0.0,
            200.0,
            false,
        )?;
        params.add(
            Params::BlurDimensions,
            "Blur Dimensions",
            ae::PopupDef::setup(|f| {
                f.set_options(&["Horizontal and Vertical", "Horizontal", "Vertical"]);
                f.set_default(1);
                f.set_value(1);
            }),
        )?;
        slider(
            params,
            Params::CentreRestore,
            "Centre Colour Restore",
            0.0,
            0.0,
            100.0,
            true,
        )?;
        slider(params, Params::Mix, "Mix", 100.0, 0.0, 100.0, true)?;
        params.add(
            Params::Repeat,
            "Repeat Edge Pixels",
            ae::CheckBoxDef::setup(|f| {
                f.set_default(true);
                f.set_value(true);
            }),
        )?;
        params.add(
            Params::ExpandBounds,
            "Expand Layer Bounds",
            ae::CheckBoxDef::setup(|f| {
                f.set_default(true);
                f.set_value(true);
            }),
        )?;
        let groups = [
            (
                "Red",
                Params::Red,
                Params::RedEnd,
                [
                    Params::RedDistance,
                    Params::RedX,
                    Params::RedY,
                    Params::RedColour,
                    Params::RedSoftness,
                ],
                -8.0,
                ae::Pixel8 {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
            ),
            (
                "Green",
                Params::Green,
                Params::GreenEnd,
                [
                    Params::GreenDistance,
                    Params::GreenX,
                    Params::GreenY,
                    Params::GreenColour,
                    Params::GreenSoftness,
                ],
                0.0,
                ae::Pixel8 {
                    red: 0,
                    green: 255,
                    blue: 0,
                    alpha: 255,
                },
            ),
            (
                "Blue",
                Params::Blue,
                Params::BlueEnd,
                [
                    Params::BlueDistance,
                    Params::BlueX,
                    Params::BlueY,
                    Params::BlueColour,
                    Params::BlueSoftness,
                ],
                8.0,
                ae::Pixel8 {
                    red: 0,
                    green: 0,
                    blue: 255,
                    alpha: 255,
                },
            ),
        ];
        for (name, start, end, k, distance, colour) in groups {
            params.add_group(start, end, name, false, |params| {
                slider(
                    params,
                    k[0],
                    &format!("{name} Distance"),
                    distance,
                    -30000.0,
                    30000.0,
                    false,
                )?;
                slider(
                    params,
                    k[1],
                    &format!("{name} X Offset"),
                    0.0,
                    -30000.0,
                    30000.0,
                    false,
                )?;
                slider(
                    params,
                    k[2],
                    &format!("{name} Y Offset"),
                    0.0,
                    -30000.0,
                    30000.0,
                    false,
                )?;
                params.add(
                    k[3],
                    &format!("{name} Colour"),
                    ae::ColorDef::setup(|f| {
                        f.set_default(colour);
                        f.set_value(colour);
                    }),
                )?;
                slider(
                    params,
                    k[4],
                    &format!("{name} Softness"),
                    0.0,
                    0.0,
                    200.0,
                    false,
                )?;
                let dimension_key = match start {
                    Params::Red => Params::RedDimensions,
                    Params::Green => Params::GreenDimensions,
                    _ => Params::BlueDimensions,
                };
                params.add(
                    dimension_key,
                    &format!("{name} Blur Dimensions"),
                    ae::PopupDef::setup(|f| {
                        f.set_options(&[
                            "Use Master",
                            "Horizontal and Vertical",
                            "Horizontal",
                            "Vertical",
                        ]);
                        f.set_default(1);
                        f.set_value(1);
                    }),
                )
            })?;
        }
        Ok(())
    }
    fn handle_command(
        &self,
        cmd: ae::Command,
        in_data: InData,
        mut out_data: OutData,
        params: &mut ae::Parameters<Params>,
    ) -> Result<(), Error> {
        match cmd {
            ae::Command::About => out_data.set_return_msg("Chromatic AE 2.2\rRGB split, recolouring and directional edge softness.\rExpanded output for footage and adjustment layers."),
            ae::Command::FrameSetup {in_layer,..} => {
                let pad=if params.get(Params::ExpandBounds)?.as_checkbox()?.value() {
                    engine::expansion(&settings(params,in_data)?)
                } else {[0;4]};
                out_data.set_width((in_layer.width() as i32+pad[0]+pad[2]) as u32);
                out_data.set_height((in_layer.height() as i32+pad[1]+pad[3]) as u32);
                out_data.set_origin(ae::Point{h:pad[0],v:pad[1]});
            },
            ae::Command::Render {in_layer,mut out_layer} => render(&in_layer,&mut out_layer,in_data,params,true)?,
            ae::Command::SmartPreRender {mut extra} => {
                let original=extra.output_request();
                let mut request=original;
                request.rect.left=-30000;request.rect.top=-30000;
                request.rect.right=30000;request.rect.bottom=30000;
                let result=extra.callbacks().checkout_layer(0,0,&request,in_data.current_time(),in_data.time_step(),in_data.time_scale())?;
                let pad=if params.get(Params::ExpandBounds)?.as_checkbox()?.value() {
                    engine::expansion(&settings(params,in_data)?)
                } else {[0;4]};
                let b=result.max_result_rect;
                let bounds=if b.right<=b.left || b.bottom<=b.top {
                    ae::Rect{left:0,top:0,right:0,bottom:0}
                } else {
                    ae::Rect{left:b.left-pad[0],top:b.top-pad[1],right:b.right+pad[2],bottom:b.bottom+pad[3]}
                };
                extra.set_max_result_rect(bounds);
                let r=ae::Rect{left:original.rect.left.max(bounds.left),top:original.rect.top.max(bounds.top),right:original.rect.right.min(bounds.right),bottom:original.rect.bottom.min(bounds.bottom)};
                // Empty requests are valid bounds-only queries. Never return inverted rectangles.
                extra.set_result_rect(if r.right>r.left && r.bottom>r.top {r} else {ae::Rect{left:0,top:0,right:0,bottom:0}});
            },
            ae::Command::SmartRender {extra} => {
                let cb=extra.callbacks();
                let input=cb.checkout_layer_pixels(0)?;
                let outcome:Result<(),Error>=(|| {
                    if let Some(mut output)=cb.checkout_output()? {
                        if let Some(input)=input.as_ref() {render(input,&mut output,in_data,params,false)?;}
                        else {output.buffer_mut().fill(0);}
                    }
                    Ok(())
                })();
                let checkin=cb.checkin_layer_pixels(0);
                outcome?;checkin?;
            },
            _=>{}
        }
        Ok(())
    }
}
