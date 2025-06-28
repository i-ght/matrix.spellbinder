#[derive(Debug, Copy, Clone)]
pub enum MayaEpoch {
    BC3114,
    CE2012
}

#[derive(Debug, Clone)]
pub struct LongDate {
    pub digits: Vec<i64>,
    pub days_since_epoch: i64,
    pub epoch: MayaEpoch
}

pub type TzolkinNumber = i64;

#[derive(Debug, PartialEq)]
pub enum TzolkinDayName {
    Imix = 1,
    Ik,
    Akbal,
    Kan,
    Chikchan,
    Kimi,
    Manik,
    Lamat,
    Muluk,
    Ok,
    Chuwen,
    Eb,
    Ben,
    Ix,
    Men,
    Kib,
    Kaban,
    Etznab,
    Kawak,
    Ajaw
}

#[derive(Debug, PartialEq)]
pub struct TzolkinDate(pub TzolkinNumber, pub TzolkinDayName);

pub type HaabDay = i64;

#[derive(Debug, PartialEq)]
pub enum HaabMonth {
    Pop = 1,
    Wo,
    Sip,
    Sotz,
    Sek,
    Xul,
    Yaxkin,
    Mol,
    Chen,
    Yax,
    Sak,
    Keh,
    Mak,
    Kankin,
    Muwan,
    Pax,
    Kayab,
    Kumku,
    Wayeb
}

#[derive(Debug, PartialEq)]
pub struct HaabDate(pub HaabDay, pub HaabMonth);

#[derive(Debug, PartialEq)]
pub struct RoundDate(pub TzolkinDate, pub HaabDate);

pub mod long;
pub mod tzolkin;
pub mod haab;
pub mod round; 

#[cfg(test)]
mod tests {

    use crate::maya::{HaabDate, HaabMonth, LongDate, MayaEpoch, RoundDate, TzolkinDate, TzolkinDayName};

    #[test]
    fn zero() {
        let long = LongDate::new(
            MayaEpoch::BC3114,
            2012,
            12,
            21
        );

        assert_eq!(
            long.digits,
            vec![13, 0, 0, 0, 0]
        );

        let round = RoundDate::from(&long);

        assert_eq!(
            round,
            RoundDate(
                TzolkinDate(4, TzolkinDayName::Ajaw),
                HaabDate(3, HaabMonth::Kankin)
            )
        );
    }
}