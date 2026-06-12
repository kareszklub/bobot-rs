use embassy_time::{Duration, Ticker, Timer};

use crate::{
    hardware::Hardware,
    utils::color::{HSV, RGB},
};

pub async fn rainbow_led<'a>(t: u64, hw: &mut Hardware) {
    loop {
        for h in 0..360 {
            let rgb = RGB::from_hsv(&HSV {
                h: h as f32,
                s: 1.0,
                v: 1.0,
            });
            hw.rgb_led.set_color(rgb.r, rgb.g, rgb.b);
            Timer::after_millis(t / 360).await;
        }
    }
}

// pub async fn follow_line<'a>(speed_rpm: f32) {
//     #[derive(Debug, PartialEq, Clone)]
//     enum TrackState {
//         OnLine,
//         HalfLeft,
//         HalfRight,
//         Left,
//         Right,
//         Unknown,
//     }
//
//     let mut last_state = TrackState::Unknown;
//
//     let mut ticker = Ticker::every(Duration::from_hz(500));
//     loop {
//         let state = match (hw.track_left.is_high(), hw.track_right.is_high()) {
//             (true, true) => TrackState::OnLine,
//             (true, false) => TrackState::HalfRight,
//             (false, true) => TrackState::HalfLeft,
//             (false, false) => match last_state {
//                 TrackState::OnLine => TrackState::Unknown,
//                 TrackState::HalfLeft => TrackState::Left,
//                 TrackState::HalfRight => TrackState::Right,
//                 TrackState::Left => TrackState::Left,
//                 TrackState::Right => TrackState::Right,
//                 TrackState::Unknown => TrackState::Unknown,
//             },
//         };
//
//         if last_state != state {
//             let (left, right) = match state {
//                 TrackState::OnLine => (1.0, 1.0),
//                 TrackState::HalfLeft => (1.0, 0.9),
//                 TrackState::HalfRight => (0.9, 1.0),
//                 TrackState::Left => (1.0, 0.0),
//                 TrackState::Right => (0.0, 1.0),
//                 TrackState::Unknown => (0.5, -0.5),
//             };
//
//             let (r, g, b) = match state {
//                 TrackState::OnLine => (0, 255, 0),
//                 TrackState::HalfLeft => (128, 128, 0),
//                 TrackState::HalfRight => (0, 128, 128),
//                 TrackState::Left => (255, 0, 0),
//                 TrackState::Right => (0, 0, 255),
//                 TrackState::Unknown => (255, 255, 255),
//             };
//
//             hw.rgb_led.set_color(r, g, b);
//
//             log::info!("{:?}", state);
//
//             hw.sc.drive(left * speed_rpm, right * speed_rpm);
//
//             last_state = state;
//         }
//
//         ticker.next().await;
//     }
// }
