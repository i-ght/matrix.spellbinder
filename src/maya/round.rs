use crate::maya::{HaabDate, LongDate, MayaEpoch, RoundDate, TzolkinDate};

impl From<&LongDate> for RoundDate {
    fn from(value: &LongDate) -> Self {
        RoundDate(
            TzolkinDate::new(value),
            HaabDate::new(value)
        )
    }
}

impl From<LongDate> for RoundDate {
    fn from(value: LongDate) -> Self {
        RoundDate::from(&value)
    }
}

impl RoundDate {
    pub fn from_date(epoch: MayaEpoch, y: i32, m: i32, d: i32) -> RoundDate {
        let long = LongDate::new(epoch, y, m, d);
        RoundDate::from(long)
    }
}