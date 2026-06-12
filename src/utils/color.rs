// no_std partial port of https://docs.rs/color_space/0.5.4/color_space/

pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct HSV {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

#[allow(dead_code)]
impl HSV {
    pub fn from_rgb(rgb: &RGB) -> Self {
        let r = rgb.r as f32 / 255.0;
        let g = rgb.g as f32 / 255.0;
        let b = rgb.b as f32 / 255.0;

        let min = r.min(g.min(b));
        let max = r.max(g.max(b));
        let delta = max - min;

        let v = max;
        let s = match max > 1e-3 {
            true => delta / max,
            false => 0.0,
        };
        let h = match delta == 0.0 {
            true => 0.0,
            false => {
                if r == max {
                    (g - b) / delta
                } else if g == max {
                    2.0 + (b - r) / delta
                } else {
                    4.0 + (r - g) / delta
                }
            }
        };
        let h2 = ((h * 60.0) + 360.0) % 360.0;

        Self { h: h2, s, v }
    }
}

#[allow(dead_code)]
impl RGB {
    pub fn from_hsv(hsv: &HSV) -> Self {
        let range = (hsv.h / 60.0) as u8;
        let c = hsv.v * hsv.s;
        let x = c * (1.0 - (((hsv.h / 60.0) % 2.0) - 1.0).abs());
        let m = hsv.v - c;

        match range {
            0 => Self {
                r: ((c + m) * 255.0) as u8,
                g: ((x + m) * 255.0) as u8,
                b: (m * 255.0) as u8,
            },
            1 => Self {
                r: ((x + m) * 255.0) as u8,
                g: ((c + m) * 255.0) as u8,
                b: (m * 255.0) as u8,
            },
            2 => Self {
                r: (m * 255.0) as u8,
                g: ((c + m) * 255.0) as u8,
                b: ((x + m) * 255.0) as u8,
            },
            3 => Self {
                r: (m * 255.0) as u8,
                g: ((x + m) * 255.0) as u8,
                b: ((c + m) * 255.0) as u8,
            },
            4 => Self {
                r: ((x + m) * 255.0) as u8,
                g: (m * 255.0) as u8,
                b: ((c + m) * 255.0) as u8,
            },
            _ => Self {
                r: ((c + m) * 255.0) as u8,
                g: (m * 255.0) as u8,
                b: ((x + m) * 255.0) as u8,
            },
        }
    }
}
