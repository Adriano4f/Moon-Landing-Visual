use super::*;
use crate::timeline::{self, Options};
use lunar_core::ephemeris::{Ephemeris, topocentric};

impl Panorama
{
	pub(super) fn options(&self) -> Options
	{
		self.timeline.lock().options
	}

	pub(super) fn refresh_summary(&mut self)
	{
		let options = self.options();
		let key = (options, self.earth_margin_deg);
		if self.summary_key == Some(key)
		{
			return;
		}
		if let Some(summary) = self
			.data
			.as_ref()
			.and_then(|d| timeline::summary(d, options, self.earth_margin_deg))
		{
			self.summary = summary;
			self.summary_key = Some(key);
		}
	}

	pub(super) fn path_range(&self) -> Range<usize>
	{
		let Some(data) = &self.data
		else
		{
			return 0..0;
		};
		let period = self.options().period;
		let paths = window(&data.sun.t, self.index, self.hours);
		paths
			.start
			.max(data.sun.t.partition_point(|&t| t < period.start))
			..paths
				.end
				.min(data.sun.t.partition_point(|&t| t <= period.end))
	}

	pub(super) fn sky_status(&self) -> String
	{
		if self
			.data
			.as_ref()
			.is_some_and(|d| self.options().period.covered(&d.sun.t))
		{
			return "Updating period statistics...".into();
		}
		self.sky_error
			.clone()
			.or_else(|| {
				self.timeline
					.lock()
					.sky
					.as_ref()
					.and_then(|r| r.as_ref().err().cloned())
			})
			.map_or_else(
				|| "Loading additional sky data for this period...".into(),
				|e| format!("Sky data unavailable: {e}"),
			)
	}

	fn append_sky(&mut self, ephemeris: &Ephemeris, end: i64) -> Result<(), String>
	{
		if ephemeris.step_s == 0
		{
			return Err("Invalid sky cadence.".into());
		}
		let Some(data) = &mut self.data
		else
		{
			return Ok(());
		};
		let altitude = data
			.terrain
			.as_ref()
			.map(|a| a.surface_height_m + a.observer_height_m)
			.filter(|h| h.is_finite())
			.ok_or("Missing observer altitude for additional sky data.")?;
		let last = *data.sun.t.last().unwrap();
		if last < ephemeris.start_utc
			|| (last - ephemeris.start_utc) % i64::from(ephemeris.step_s) != 0
		{
			return Err("Site and additional sky times do not align.".into());
		}
		let start = ((last - ephemeris.start_utc) / i64::from(ephemeris.step_s) + 1) as usize;
		let limit = ((end - ephemeris.start_utc) / i64::from(ephemeris.step_s) + 1) as usize;
		for index in start..(start + 512).min(limit).min(ephemeris.samples.len())
		{
			let epoch = ephemeris.epoch(index).ok_or("Missing sky epoch")?;
			let pair = ephemeris.samples[index]
				.map(|b| topocentric(b, data.site.lat_deg, data.site.lon_deg, altitude));
			let (a, e) = pair[0].as_ref().map_err(Clone::clone)?;
			data.sun.t.push(epoch);
			data.sun.az_deg.push(*a as f32);
			data.sun.el_deg.push(*e as f32);
			let (a, e) = pair[1].as_ref().map_err(Clone::clone)?;
			let (a, e) = (*a as f32, *e as f32);
			data.earth.t.push(epoch);
			data.earth.az_deg.push(a);
			data.earth.el_deg.push(e);
			data.earth_above
				.push(e > data.horizon_at(a) + self.earth_margin_deg);
		}
		Ok(())
	}

	pub(super) fn extend_sky(&mut self, ctx: &egui::Context)
	{
		let end = self.options().period.end;
		if self.sky_error.is_some()
			|| self
				.data
				.as_ref()
				.is_none_or(|d| d.sun.t.last().is_some_and(|&t| t >= end))
		{
			return;
		}
		timeline::request(&self.timeline, ctx);
		let ephemeris = self
			.timeline
			.lock()
			.sky
			.as_ref()
			.and_then(|r| r.as_ref().ok())
			.cloned();
		if let Some(ephemeris) = ephemeris
		{
			if let Err(error) = self.append_sky(&ephemeris, end)
			{
				self.sky_error = Some(error);
			}
			ctx.request_repaint();
		}
	}
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn extension_is_chunked_preserves_original_tracks_and_uses_observer_altitude()
	{
		let data =
			SiteData::parse(include_bytes!("../../assets/data/connecting-ridge-c1.json")).unwrap();
		let original = data.sun.t.len();
		let last = [data.sun.az_deg[original - 1], data.sun.el_deg[original - 1]];
		let timeline = std::sync::Arc::new(egui::mutex::Mutex::new(timeline::Timeline::default()));
		let mut p = Panorama {
			pending: Pending::default(),
			data: Some(data),
			error: None,
			index: 0,
			hours: 720,
			earth_margin_deg: 0.5,
			technical: false,
			summary: Default::default(),
			timeline,
			cursor: timeline::START,
			summary_key: None,
			sky_error: None,
		};
		p.refresh_summary();
		assert!(p.summary_key.is_some());
		p.timeline.lock().options.period.end = timeline::STOP;
		p.refresh_summary();
		assert_ne!(p.summary_key, Some((p.options(), p.earth_margin_deg)));
		let sky =
			Ephemeris::decode(include_bytes!("../../assets/data/moon-ephemeris.bin")).unwrap();
		p.append_sky(&sky, timeline::STOP).unwrap();
		assert_eq!(p.data.as_ref().unwrap().sun.t.len(), original + 512);
		while *p.data.as_ref().unwrap().sun.t.last().unwrap() < timeline::STOP
		{
			p.append_sky(&sky, timeline::STOP).unwrap();
		}
		let data = p.data.as_ref().unwrap();
		assert_eq!(data.sun.t.len(), 70129);
		assert_eq!(data.earth_above.len(), data.sun.t.len());
		assert_eq!(
			[data.sun.az_deg[original - 1], data.sun.el_deg[original - 1]],
			last
		);
		let (a, e) = topocentric(
			sky.samples.last().unwrap()[1],
			-89.4398,
			222.83,
			1960.5030063044644,
		)
		.unwrap();
		let actual = timeline::sample(&data.earth, timeline::STOP).unwrap();
		assert!((f64::from(actual[0]) - a).abs() < 1e-5 && (f64::from(actual[1]) - e).abs() < 1e-6);
		p.refresh_summary();
		assert_eq!(p.summary_key, Some((p.options(), 0.5)));
		p.data.as_mut().unwrap().terrain = None;
		assert!(p.append_sky(&sky, timeline::STOP).is_err());
		p.timeline.lock().sky = Some(Err("injected unavailable sky".into()));
		p.timeline.lock().options.period.start = timeline::START - 1;
		assert!(p.sky_status().contains("injected unavailable sky"));
	}
}
