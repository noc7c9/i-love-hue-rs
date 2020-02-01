#[allow(clippy::many_single_char_names)]
pub fn hsl_to_rgb((h, s, l): (f64, f64, f64)) -> (u8, u8, u8) {
    fn f64_to_u8(v: f64) -> u8 {
        (v * 255.0) as u8
    }

    if s < std::f64::EPSILON {
        let l = f64_to_u8(l);
        return (l, l, l);
    }

    let temp1 = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let temp2 = 2.0 * l - temp1;

    let h = h / 360.0;

    let calc_channel = |mut v| {
        // wrap
        if v < 0.0 {
            v += 1.0
        } else if v > 1.0 {
            v -= 1.0
        }

        // calculate
        v = if 6.0 * v < 1.0 {
            temp2 + (temp1 - temp2) * 6.0 * v
        } else if 2.0 * v < 1.0 {
            temp1
        } else if 3.0 * v < 2.0 {
            temp2 + (temp1 - temp2) * 6.0 * (0.666_666_666 - v)
        } else {
            temp2
        };

        // convert to u8
        f64_to_u8(v)
    };

    let r = calc_channel(h + 0.333_333_333);
    let g = calc_channel(h);
    let b = calc_channel(h - 0.333_333_333);

    (r, g, b)
}

#[allow(clippy::many_single_char_names)]
pub fn rgb_to_hsl((r, g, b): (u8, u8, u8)) -> (f64, f64, f64) {
    let min = r.min(g).min(b);
    let max = r.max(g).max(b);

    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;

    let minf = rf.min(gf).min(bf);
    let maxf = rf.max(gf).max(bf);

    let l = (minf + maxf) / 2.0;

    let s = {
        if min == max {
            0.0
        } else if l < 0.5 {
            (maxf - minf) / (maxf + minf)
        } else {
            (maxf - minf) / (2.0 - maxf - minf)
        }
    };

    let mut h = {
        if min == max {
            0.0
        } else if max == r {
            (gf - bf) / (maxf - minf)
        } else if max == g {
            2.0 + (bf - rf) / (maxf - minf)
        } else {
            4.0 + (rf - gf) / (maxf - minf)
        }
    };
    h *= 60.0;
    if h < 0.0 {
        h += 360.0;
    }

    (h, s, l)
}
