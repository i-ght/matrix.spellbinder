use crate::maya::{LongDate, MayaEpoch};

fn jd(y: i32, m: i32, d: i32) -> f64 {
    let (y, m, d) = (f64::from(y), f64::from(m), f64::from(d));
    let y = if y < 0.0 { y + 1.0 } else { y };
    let (y, m, d) = (y, m, d);
    let (jy, jm) = if m > 2.0 {
        (y, m + 1.0)
    } else {
        (y - 1.0, m + 13.0)
    };
    let jd0 = (365.25 * jy + 30.6001 * jm + d + 1720995.0).floor();
    let jd1 = if d + 31.0 * (m + 12.0 * y) >= 15.0 + 31.0 * (10.0 + 12.0 * 1582.0) {
        let ja = (0.01 * jy).floor();
        jd0 + 2.0 - ja + (0.25 * ja).floor()
    } else {
        jd0
    };
    jd1 - 0.5
}

fn epoch_jd(epoch: &MayaEpoch) -> f64 {
    match epoch {
        MayaEpoch::BC3114 => 584282.5,
        MayaEpoch::CE2012 => jd(2012, 12, 21)
    }
}

fn days_diff(epoch: &MayaEpoch, y: i32, m: i32, d: i32) -> i64 {
    return (jd(y, m, d) - epoch_jd(epoch)) as i64;
}

fn maya_digis(days: i64, place_index: usize) -> Vec<i64> {
    let mut acc = Vec::with_capacity(5);
    let mut days_remaining = days;

    for current_index in (0..place_index).rev() {
        let f_index = current_index as f64;
        let unit_of_days = if current_index >= 2 {
            18.0 * f64::powf(20.0, f_index - 1.0)
        } else {
            f64::powf(20.0, f_index)
        } as i64;
        let value = days_remaining / unit_of_days;
        acc.push(value);
        days_remaining %= unit_of_days;
    }

    acc
}


impl LongDate {
    pub fn new(epoch: MayaEpoch, y: i32, m: i32, d: i32) -> LongDate {
        let days = days_diff(&epoch, y, m, d);
        let places_needed = 5;
        let digis = maya_digis(days, places_needed);
        LongDate {
            digits: digis,
            days_since_epoch: days,
            epoch
        }
    }
}