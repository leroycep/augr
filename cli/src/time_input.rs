use chrono::{Date, DateTime, Datelike, Duration, Local, NaiveTime, TimeZone};
use std::ffi::{OsStr, OsString};

pub trait Context {
    type TZ: TimeZone;
    fn tz(&self) -> &Self::TZ;
    fn now(&self) -> &DateTime<Self::TZ>;
}

macro_rules! attempt {
    ($code:expr) => {
        match $code {
            Ok(s) => return Ok(s),
            Err(e) => e,
        }
    };
}

pub fn parse_default_local(text: &OsStr) -> Result<DateTime<Local>, OsString> {
    let text = text
        .to_str()
        .ok_or_else(|| OsString::from("OsStr was not a valid rust string"))?;
    struct LocalContext(DateTime<Local>);
    impl Context for LocalContext {
        type TZ = Local;
        fn tz(&self) -> &Self::TZ {
            &Local
        }
        fn now(&self) -> &DateTime<Self::TZ> {
            &self.0
        }
    }

    let c = LocalContext(Local::now());
    parse(&c, text).map_err(|_| OsString::from("No valid date, time, or duration was found"))
}

pub fn parse<C: Context>(c: &C, text: &str) -> Result<DateTime<C::TZ>, ()> {
    attempt!(parse_datetime(c.tz(), text));
    if let Ok(date) = parse_date(c, text) {
        return Ok(date.and_hms(0, 0, 0));
    }
    if let Ok(time) = parse_time(c, text) {
        if time <= c.now().time() {
            return Ok(c.now().date().and_time(time).unwrap());
        } else {
            let yesterday = c.now().date() - Duration::days(1);
            return Ok(yesterday.and_time(time).unwrap());
        }
    }
    if let Ok(Ok(duration)) = ::parse_duration::parse(text).map(Duration::from_std) {
        return Ok(c.now().clone() - duration);
    }
    Err(())
}

fn parse_datetime<T: TimeZone>(tz: &T, text: &str) -> Result<DateTime<T>, ()> {
    if let Ok(datetime) = DateTime::parse_from_rfc3339(text) {
        return Ok(datetime.with_timezone(tz));
    }
    attempt!(tz.datetime_from_str(text, "%Y-%m-%dT%H:%M:%S"));
    Err(())
}

fn parse_date<C: Context>(c: &C, text: &str) -> Result<Date<C::TZ>, ()> {
    if let Ok(parsed) = format_parse(fmts::FULL_DATE, text) {
        return Ok(c.tz().ymd(
            parsed.year.unwrap(),
            parsed.month.unwrap(),
            parsed.day.unwrap(),
        ));
    }
    if let Ok(parsed) = format_parse(fmts::PARTIAL_DATE, text) {
        return Ok(c.tz().ymd(
            c.now().with_timezone(c.tz()).year(),
            parsed.month.unwrap(),
            parsed.day.unwrap(),
        ));
    }
    Err(())
}

fn parse_time<C: Context>(_c: &C, text: &str) -> Result<NaiveTime, ()> {
    if let Ok(mut parsed) = format_parse(fmts::HOUR_AND_MINUTE, text) {
        let _ = parsed.set_second(0);
        return parsed.to_naive_time().map_err(|_| ());
    }
    Err(())
}

fn format_parse(fmt: &[chrono::format::Item], text: &str) -> Result<chrono::format::Parsed, ()> {
    use chrono::format;
    let fmt_iter = fmt.iter().cloned();
    let mut parsed = format::Parsed::new();
    match format::parse(&mut parsed, text, fmt_iter) {
        Ok(()) => Ok(parsed),
        Err(_) => Err(()),
    }
}

mod fmts {
    use chrono::format::{Item, Numeric::*, Pad};

    pub const FULL_DATE: &[Item] = &[
        Item::Numeric(Year, Pad::None),
        Item::Literal("-"),
        Item::Numeric(Month, Pad::None),
        Item::Literal("-"),
        Item::Numeric(Day, Pad::None),
    ];

    pub const PARTIAL_DATE: &[Item] = &[
        Item::Numeric(Month, Pad::None),
        Item::Literal("-"),
        Item::Numeric(Day, Pad::None),
    ];

    pub const HOUR_AND_MINUTE: &[Item] = &[
        Item::Numeric(Hour, Pad::None),
        Item::Literal(":"),
        Item::Numeric(Minute, Pad::None),
    ];

}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Utc;

    struct DummyContext(DateTime<Utc>);
    impl Context for DummyContext {
        type TZ = Utc;
        fn tz(&self) -> &Self::TZ {
            &Utc
        }
        fn now(&self) -> &DateTime<Self::TZ> {
            &self.0
        }
    }
    impl DummyContext {
        fn new() -> Self {
            DummyContext(Utc.ymd(2019, 07, 16).and_hms(19, 25, 0))
        }
    }

    #[test]
    fn full_datetime() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(19, 25, 0)),
            parse(&DummyContext::new(), "2019-07-16T14:25:00-05:00")
        );
    }

    #[test]
    fn datetime_no_timezone() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(19, 25, 0)),
            parse(&DummyContext::new(), "2019-07-16T19:25:00")
        );
    }

    #[test]
    fn just_the_date() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(0, 0, 0)),
            parse(&DummyContext::new(), "2019-07-16")
        );
    }

    #[test]
    fn just_the_month_and_day_no_padding() {
        assert_eq!(
            Ok(Utc.ymd(2019, 7, 6).and_hms(0, 0, 0)),
            parse(&DummyContext::new(), "7-6")
        );
    }

    #[test]
    fn just_the_month_and_day() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(0, 0, 0)),
            parse(&DummyContext::new(), "07-16")
        );
    }

    #[test]
    fn just_hour_and_minute() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(19, 25, 0)),
            parse(&DummyContext::new(), "19:25")
        );
    }

    #[test]
    fn time_from_yesterday() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 15).and_hms(20, 00, 0)),
            parse(&DummyContext::new(), "20:00")
        );
    }

    #[test]
    fn duration_20minutes() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(19, 05, 0)),
            parse(&DummyContext::new(), "20min")
        );
    }

    #[test]
    fn duration_1hour_12minutes() {
        assert_eq!(
            Ok(Utc.ymd(2019, 07, 16).and_hms(18, 13, 0)),
            parse(&DummyContext::new(), "1hr12min")
        );
    }
}
