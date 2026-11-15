//! Shared UTC period, calendar stepping and sampled-period statistics.
mod controls;

use crate::data::{self, SiteData};
use chrono::{DateTime, Months, NaiveDate, Utc};
use eframe::egui::{self, mutex::Mutex};
use lunar_core::{Track, ephemeris::Ephemeris};
use std::sync::Arc;

pub(crate) const START: i64 = 1798761600;
pub(crate) const STOP: i64 = 2051222400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Step
{
	Hour,
	Day,
	Month,
}

impl Step
{
	pub fn label(self) -> &'static str
	{
		match self
		{
			Self::Hour => "1 hour",
			Self::Day => "1 day",
			Self::Month => "1 calendar month",
		}
	}

	pub fn shift(self, time: i64, direction: i32) -> Option<i64>
	{
		if ![-1, 1].contains(&direction)
		{
			return None;
		}
		match self
		{
			Self::Hour => time.checked_add(i64::from(direction) * 3600),
			Self::Day => time.checked_add(i64::from(direction) * 86400),
			Self::Month =>
			{
				let date = DateTime::<Utc>::from_timestamp(time, 0)?;
				if direction > 0
				{
					date.checked_add_months(Months::new(1))
				}
				else
				{
					date.checked_sub_months(Months::new(1))
				}
				.map(|d| d.timestamp())
			}
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Period
{
	pub start: i64,
	pub end: i64,
}

#[derive(Clone, Copy)]
enum Preset { Week, Month, Year, All }

impl Preset
{
	fn period(self, cursor:i64) -> Period
	{
		if matches!(self,Self::All) { return Period {start:START,end:STOP}; }
		let shift=|time:i64,direction:i32| match self {
			Self::Week=>time.checked_add(i64::from(direction)*7*86400),
			Self::Month=>Step::Month.shift(time,direction),
			Self::Year=> {
				let date=DateTime::<Utc>::from_timestamp(time,0)?;
				if direction>0 {date.checked_add_months(Months::new(12))} else {date.checked_sub_months(Months::new(12))}.map(|d|d.timestamp())
			},
			Self::All=>None,
		};
		let cursor=cursor.clamp(START,STOP);
		let start=if cursor==STOP {shift(STOP,-1).unwrap_or(START).max(START)} else {cursor};
		Period {start,end:shift(start,1).unwrap_or(STOP).min(STOP)}
	}
}

impl Period
{
	pub fn label(self) -> String
	{
		let date=|t|DateTime::<Utc>::from_timestamp(t,0).map(|d|d.format("%Y-%m-%d").to_string()).unwrap_or_default();
		format!("{}: {} → {}",if self.end-self.start==7*86400 {"This week"} else {"Period"},date(self.start),date(self.end))
	}
	pub fn parse(start: &str, end: &str) -> Result<Self, String>
	{
		let parse = |s: &str| {
			NaiveDate::parse_from_str(s, "%Y-%m-%d")
				.ok()
				.and_then(|d| d.and_hms_opt(0, 0, 0))
				.map(|d| d.and_utc().timestamp())
		};
		let (Some(start), Some(end)) = (parse(start), parse(end))
		else
		{
			return Err("Use valid dates: YYYY-MM-DD.".into());
		};
		if start < START || end > STOP || start >= end
		{
			return Err("Choose start before end, within 2027-01-01 to 2035-01-01.".into());
		}
		Ok(Self { start, end })
	}
	pub fn covered(self, times: &[i64]) -> bool
	{
		times
			.first()
			.zip(times.last())
			.is_some_and(|(&a, &b)| a <= self.start && b >= self.end)
	}
	pub fn samples(self, step: Step) -> Vec<i64>
	{
		let mut times = vec![self.start];
		while *times.last().unwrap() < self.end
		{
			let next = step
				.shift(*times.last().unwrap(), 1)
				.unwrap_or(self.end)
				.min(self.end);
			times.push(next);
		}
		times
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Options
{
	pub period: Period,
	pub step: Step,
}

#[derive(Default)]
struct Repeat
{
	since: Option<f64>,
	next: f64,
	repeated: bool,
}

impl Repeat
{
	fn held(&self, widget_down: bool, primary_down: bool) -> bool
	{
		primary_down && (widget_down || self.since.is_some())
	}

	fn tick(&mut self, down: bool, clicked: bool, now: f64) -> bool
	{
		if !down
		{
			let fire = clicked && !self.repeated;
			*self = Self::default();
			return fire;
		}
		if self.since.is_none()
		{
			self.since = Some(now);
			self.next = now + 0.4;
		}
		if now + 1e-9 >= self.next
		{
			self.next = now + 0.1;
			self.repeated = true;
			return true;
		}
		false
	}
}

pub(crate) struct Timeline
{
	pub options: Options,
	dates: [String; 2],
	validation: Option<String>,
	repeat: [Repeat; 2],
	pub sky: Option<Result<Arc<Ephemeris>, String>>,
	started: bool,
}

impl Default for Timeline
{
	fn default() -> Self
	{
		Self {
			options: Options {
				period: Period {
					start: START,
					end: START+7*86400,
				},
				step: Step::Hour,
			},
			dates: ["2027-01-01".into(), "2027-01-08".into()],
			validation: None,
			repeat: Default::default(),
			sky: None,
			started: false,
		}
	}
}

pub(crate) type Shared = Arc<Mutex<Timeline>>;

pub(crate) fn request(shared: &Shared, ctx: &egui::Context)
{
	{
		let mut state = shared.lock();
		if state.started
		{
			return;
		}
		state.started = true;
	}
	let shared = Arc::clone(shared);
	let ctx = ctx.clone();
	data::read_asset("moon-ephemeris.bin".into(), move |bytes| {
		let result = bytes.and_then(|b| Ephemeris::decode(&b)).and_then(|e| {
			if e.start_utc != START
				|| e.step_s != 3600
				|| e.epoch(e.samples.len() - 1) != Some(STOP)
			{
				return Err("Unexpected complete sky coverage.".into());
			}
			Ok(Arc::new(e))
		});
		shared.lock().sky = Some(result);
		ctx.request_repaint();
	});
}

pub(crate) fn sample(track: &Track, time: i64) -> Option<[f32; 2]>
{
	let (&start, &end) = track.t.first().zip(track.t.last())?;
	if time < start
		|| time > end
		|| track.az_deg.len() != track.t.len()
		|| track.el_deg.len() != track.t.len()
	{
		return None;
	}
	let i = track.t.partition_point(|&t| t <= time).saturating_sub(1);
	if track.t[i] == time
	{
		return Some([track.az_deg[i], track.el_deg[i]]);
	}
	let j = i + 1;
	let unit = |k: usize| {
		let (a, e) = (
			f64::from(track.az_deg[k]).to_radians(),
			f64::from(track.el_deg[k]).to_radians(),
		);
		[e.cos() * a.sin(), e.cos() * a.cos(), e.sin()]
	};
	let (a, b) = (unit(i), unit(j));
	let f = (time - track.t[i]) as f64 / (track.t[j] - track.t[i]) as f64;
	let v: [f64; 3] = std::array::from_fn(|k| a[k] * (1.0 - f) + b[k] * f);
	if v.iter().any(|x| !x.is_finite()) || v.iter().map(|x| x * x).sum::<f64>() < 1e-12
	{
		return None;
	}
	Some([
		v[0].atan2(v[1]).to_degrees().rem_euclid(360.0) as f32,
		v[2].atan2(v[0].hypot(v[1])).to_degrees() as f32,
	])
}

pub(crate) fn summary(data: &SiteData, options: Options, margin: f32) -> Option<lunar_core::Stats>
{
	if options.period.start >= options.period.end || !options.period.covered(&data.sun.t)
	{
		return None;
	}
	let times = options.period.samples(options.step);
	let mut sun = Vec::with_capacity(times.len());
	let mut earth = Vec::with_capacity(times.len());
	for &time in &times
	{
		let [a, e] = sample(&data.sun, time)?;
		sun.push(lunar_core::sun_fraction(e, data.horizon_at(a)) > 0.0);
		let [a, e] = sample(&data.earth, time)?;
		earth.push(e > data.horizon_at(a) + margin);
	}
	Some(lunar_core::stats(&times, &sun, &earth))
}

#[cfg(test)]
mod tests;
