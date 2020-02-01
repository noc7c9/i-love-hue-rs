mod color_conversions;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    RGB { r: u8, g: u8, b: u8 },
    HSL { h: f64, s: f64, l: f64 },
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::RGB { r, g, b }
    }

    pub fn hsl(h: f64, s: f64, l: f64) -> Self {
        assert!(
            h >= 0.0 && h <= 360.0,
            "h must be within the range 0.0 to 360.0"
        );
        assert!(
            s >= 0.0 && s <= 1.0,
            "s must be within the range 0.0 to 1.0"
        );
        assert!(
            l >= 0.0 && l <= 1.0,
            "l must be within the range 0.0 to 1.0"
        );
        Color::HSL { h, s, l }
    }

    fn lerp(a: Self, b: Self, ratio: f64) -> Self {
        // Helper that just lerps between two f64 values
        fn lerp(start: f64, end: f64, ratio: f64) -> f64 {
            start * (1.0 - ratio) + end * ratio
        }

        let ratio = ratio.min(1.0).max(0.0); // clamp

        match (a, b) {
            (Self::RGB { .. }, Self::RGB { .. }) => {
                let a = a.unwrap_rgb();
                let b = b.unwrap_rgb();
                let lerp_u8 = move |a, b| lerp(f64::from(a), f64::from(b), ratio) as u8;

                Self::RGB {
                    r: lerp_u8(a.0, b.0),
                    g: lerp_u8(a.1, b.1),
                    b: lerp_u8(a.2, b.2),
                }
            }
            (Self::HSL { .. }, Self::HSL { .. }) => {
                let a = a.unwrap_hsl();
                let b = b.unwrap_hsl();

                fn lerp_hue(start: f64, end: f64, ratio: f64) -> f64 {
                    // This function linearly interpolates (lerps) between the two hue values
                    // (start & end) either positively or negatively (wrapping around) in
                    // whichever direction would be shortest.
                    //
                    // Note: this assumes start <= end;
                    //
                    // The hue values are from 0.0 to 360.0 inclusive
                    //
                    // The ratio is how much of start vs end to have in the calculated value
                    //  - a ratio of 0.0 means 100% start + 0% end
                    //  - a ratio of 1.0 means 0% start + 100% end
                    // The ratio goes from 0.0 to 1.0 inclusive
                    //
                    //
                    // There are two possibilities for which direction to lerp:
                    //  - Case A: The positive direction is shorter
                    // 0.0 --------------------------------------------------------------- 360.0
                    //                       ^ <==== shorter ====> ^
                    //                     Start                  End
                    //
                    //  - Case A: The negative direction is shorter
                    // 0.0 --------------------------------------------------------------- 360.0
                    //     == short- ==> ^                                  ^ <=== -er ===
                    //                 Start                               End

                    use std::cmp::Ordering;
                    if let Some(Ordering::Less) | Some(Ordering::Equal) = start.partial_cmp(&end) {
                    } else {
                        panic!("lerp_hue only works if start <= end");
                    }

                    // First figure out which direction is shorter
                    let positive_distance = end - start;
                    let negative_distance = 360.0 - positive_distance;
                    if positive_distance <= negative_distance {
                        // Case A: lerp positively from start to end
                        lerp(start, end, ratio)
                    } else {
                        // Case B: lerp negatively
                        //         from start to 0 (left side) and then
                        //         from 360 to end (right side)
                        //
                        // Both sides need to be lerped independantly to implement the wrapping
                        // behaviour

                        // For the given ratio, we need to figure out
                        // which side to lerp and
                        // how much much each side takes up
                        //
                        //             These are all the possible values of ratio
                        // 0.0 ------------------------------------------------------------- 1.0
                        //                 And it is split in two at point X
                        //     <==================> X <====================================>
                        //           left side                     right side

                        let length_left = start;
                        let length_right = 360.0 - end;
                        let length_total = length_left + length_right;

                        let x = length_left / length_total;

                        // We first need to figure out which side the ratio is on
                        let is_on_left = ratio < x;

                        // In order to lerp each side, we also need to map each side to the
                        // range 0.0 to 1.0
                        //
                        // ie. map
                        //   -  left: [0.0, X] => [0.0, 1.0]
                        //   - right: [X, 1.0] => [0.0, 1.0]
                        //
                        // 0.0 <==================> X <====================================> 1.0
                        // 0.0      left side      1.0
                        //                         0.0              right side               1.0
                        //
                        // This ends up being
                        //  left: ratio / x
                        // right: (ratio - x) / (1 - x)

                        // Now we can lerp each side
                        if is_on_left {
                            lerp(start, 0.0, ratio / x)
                        } else {
                            lerp(360.0, end, (ratio - x) / (1.0 - x))
                        }
                    }
                };

                let h = if a.0 < b.0 {
                    lerp_hue(a.0, b.0, ratio)
                } else {
                    let mut h = -lerp_hue(-a.0, -b.0, ratio);
                    while h < 0.0 {
                        h += 360.0
                    }
                    while h > 360.0 {
                        h -= 360.0
                    }
                    h
                };

                Self::HSL {
                    h,
                    // h: lerp(a.0, b.0, ratio),
                    s: lerp(a.1, b.1, ratio),
                    l: lerp(a.2, b.2, ratio),
                }
            }
            _ => panic!("Lerping between colors of two types is not supported"),
        }
    }

    fn unwrap_rgb(self) -> (u8, u8, u8) {
        match self {
            Self::RGB { r, g, b } => (r, g, b),
            _ => panic!("Attempted to unwrap non-rgb color as rgb"),
        }
    }

    fn unwrap_hsl(self) -> (f64, f64, f64) {
        match self {
            Self::HSL { h, s, l } => (h, s, l),
            _ => panic!("Attempted to unwrap non-hsl color as hsl"),
        }
    }

    pub fn to_css(self) -> String {
        match self {
            Self::RGB { r, g, b } => format!("rgb({}, {}, {})", r, g, b),
            Self::HSL { h, s, l } => format!("hsl({}, {}%, {}%)", h, s * 100.0, l * 100.0),
        }
    }

    // source: http://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl/
    pub fn to_rgb(self) -> Self {
        match self {
            Self::RGB { .. } => self,
            Self::HSL { h, s, l } => {
                let (r, g, b) = color_conversions::hsl_to_rgb((h, s, l));
                Self::rgb(r, g, b)
            }
        }
    }

    // source: http://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl/
    pub fn to_hsl(self) -> Self {
        match self {
            Self::RGB { r, g, b } => {
                let (h, s, l) = color_conversions::rgb_to_hsl((r, g, b));
                Self::hsl(h, s, l)
            }
            Self::HSL { .. } => self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    x: f64,
    y: f64,
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        assert!(
            x >= 0.0 && x <= 1.0,
            "x must be within the range 0.0 to 1.0"
        );
        assert!(
            y >= 0.0 && y <= 1.0,
            "y must be within the range 0.0 to 1.0"
        );
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Stop {
    color: Color,
    position: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    top_left: Stop,
    top_right: Stop,
    bottom_left: Stop,
    bottom_right: Stop,
}

impl Gradient {
    pub fn builder() -> GradientBuilder {
        GradientBuilder::new()
    }

    pub fn color_at(&self, at: Position) -> Color {
        let top_color = Color::lerp(self.top_left.color, self.top_right.color, at.x);
        let bottom_color = Color::lerp(self.bottom_left.color, self.bottom_right.color, at.x);

        Color::lerp(top_color, bottom_color, at.y)
    }
}

#[derive(Default)]
pub struct GradientBuilder {
    top_left: Option<Stop>,
    top_right: Option<Stop>,
    bottom_left: Option<Stop>,
    bottom_right: Option<Stop>,
}

impl GradientBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Gradient {
        assert!(
            self.top_left.is_some(),
            "Gradient requires top-left color stop"
        );
        assert!(
            self.top_right.is_some(),
            "Gradient requires top-right color stop"
        );
        assert!(
            self.bottom_left.is_some(),
            "Gradient requires bottom-left color sbottom"
        );
        assert!(
            self.bottom_right.is_some(),
            "Gradient requires bottom-right color sbottom"
        );

        Gradient {
            top_left: self.top_left.unwrap(),
            top_right: self.top_right.unwrap(),
            bottom_left: self.bottom_left.unwrap(),
            bottom_right: self.bottom_right.unwrap(),
        }
    }

    pub fn top_left(mut self, color: Color) -> Self {
        let position = Position::new(0.0, 0.0);
        self.top_left = Some(Stop { color, position });
        self
    }

    pub fn top_right(mut self, color: Color) -> Self {
        let position = Position::new(1.0, 0.0);
        self.top_right = Some(Stop { color, position });
        self
    }

    pub fn bottom_left(mut self, color: Color) -> Self {
        let position = Position::new(0.0, 1.0);
        self.bottom_left = Some(Stop { color, position });
        self
    }

    pub fn bottom_right(mut self, color: Color) -> Self {
        let position = Position::new(1.0, 1.0);
        self.bottom_right = Some(Stop { color, position });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rgb_near_eq(a: Color, b: Color) {
        fn u8_diff(a: u8, b: u8) -> u8 {
            use std::convert::TryInto;
            (a as i16 - b as i16).abs().try_into().unwrap()
        }

        let a = a.unwrap_rgb();
        let b = b.unwrap_rgb();

        // Accept if any of the channels are off by one
        assert!(u8_diff(a.0, b.0) <= 1);
        assert!(u8_diff(a.1, b.1) <= 1);
        assert!(u8_diff(a.2, b.2) <= 1);
    }

    #[test]
    fn rgb_to_hsl() {
        for r in 0..255 {
            for g in 0..255 {
                for b in 0..255 {
                    let orig = Color::rgb(r, g, b);
                    let conv = orig.to_hsl().to_rgb();
                    assert_rgb_near_eq(orig, conv);
                }
            }
        }
    }
}
