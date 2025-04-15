use {
  super::scheduler::TaskScheduler,
  crate::{
    BotData,
    BotError
  },
  poise::serenity_prelude::async_trait,
  std::{
    sync::{
      Arc,
      LazyLock,
      atomic::{
        AtomicU32,
        Ordering
      }
    },
    time::{
      SystemTime,
      UNIX_EPOCH
    }
  }
};

static CURRENT_EMBED_COLOR: LazyLock<AtomicU32> = LazyLock::new(|| AtomicU32::new(DEFAULT));

const DEFAULT: u32 = if cfg!(feature = "production") { 0x0052CF } else { 0x559999 };

struct Date {
  day:   u32,
  month: u32
}

struct Theme {
  name:  &'static str,
  start: Date,
  end:   Date,
  color: u32
}

static SEASONAL_THEMES: &[Theme] = &[
  Theme {
    name:  "Breast Cancer Awareness",
    start: Date { day: 1, month: 10 },
    end:   Date { day: 31, month: 10 },
    color: 0xFF69B4
  },
  Theme {
    name:  "Remembrance Day",
    // It's always 11 in AU but 8-12 in UK for some reason
    start: Date { day: 8, month: 11 },
    end:   Date { day: 12, month: 11 },
    color: 0xE35335
  },
  Theme {
    name:  "Christmas",
    start: Date { day: 1, month: 12 },
    end:   Date { day: 31, month: 12 },
    color: 0xFFFFFF
  }
];

fn is_leap_year(year: u32) -> bool { (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) }

fn get_current_date() -> Date {
  let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Incorrect system time");
  let seconds = now.as_secs();

  let days_since_epoch = seconds / 86400;
  let mut year = 1970;
  let mut remaining_days = days_since_epoch;

  loop {
    let days_in_year = if is_leap_year(year) { 366 } else { 365 };
    if remaining_days >= days_in_year as u64 {
      remaining_days -= days_in_year as u64;
      year += 1;
    } else {
      break;
    }
  }

  let days_in_month = if is_leap_year(year) {
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
  } else {
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
  };

  let mut month = 1;
  let mut day = 0;

  for (i, &days) in days_in_month.iter().enumerate() {
    if remaining_days < days as u64 {
      month = i + 1;
      day = remaining_days as u32 + 1;
      break;
    }
    remaining_days -= days as u64;
  }

  println!("SeasonalTheme[Debug] Current date: {day}/{month}");

  Date {
    day,
    month: month.try_into().unwrap()
  }
}

fn is_date_in_range(
  current: &Date,
  start: &Date,
  end: &Date
) -> bool {
  if start.month == end.month {
    current.month == start.month && current.day >= start.day && current.day <= end.day
  } else {
    (current.month > start.month || (current.month == start.month && current.day >= start.day))
      && (current.month < end.month || (current.month == end.month && current.day <= end.day))
  }
}

fn calculate_embed_color() -> u32 {
  let current_date = get_current_date();

  for theme in SEASONAL_THEMES {
    if is_date_in_range(&current_date, &theme.start, &theme.end) {
      println!("SeasonalTheme[Info] Matching theme '{}' active", theme.name);
      return theme.color;
    }
  }

  DEFAULT
}

pub fn get_embed_color() -> u32 { CURRENT_EMBED_COLOR.load(Ordering::Relaxed) }

fn update_embed_color() {
  let new_color = calculate_embed_color();
  let current = CURRENT_EMBED_COLOR.load(Ordering::Relaxed);

  if new_color != current {
    CURRENT_EMBED_COLOR.store(new_color, Ordering::Relaxed);
    println!("SeasonalTheme[Info] Updated embed color to use {new_color:06X}")
  }
}

pub struct SeasonalTheme;

#[async_trait]
impl TaskScheduler for SeasonalTheme {
  fn name(&self) -> &'static str { "Seasonal Theme" }

  fn interval_secs(&self) -> u64 { 3600 }

  async fn main_loop(
    &self,
    _: Arc<BotData>
  ) -> Result<(), BotError> {
    update_embed_color();
    Ok(())
  }
}
