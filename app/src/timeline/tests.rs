use super::*;

fn epoch(s: &str) -> i64
{
	NaiveDate::parse_from_str(s, "%Y-%m-%d")
		.unwrap()
		.and_hms_opt(0, 0, 0)
		.unwrap()
		.and_utc()
		.timestamp()
}

#[test]
fn default_week_and_presets_use_the_cursor_and_clip_to_coverage()
{
	let state=Timeline::default();
	assert_eq!(state.options.period,Preset::Week.period(START));
	assert_eq!(state.options.period.label(),"This week: 2027-01-01 → 2027-01-08");
	let cursor=epoch("2028-01-31")+43200;
	assert_eq!(Preset::Week.period(cursor).end,cursor+7*86400);
	assert_eq!(Preset::Month.period(cursor).end,epoch("2028-02-29")+43200);
	assert_eq!(Preset::Year.period(epoch("2028-02-29")).end,epoch("2029-02-28"));
	for p in [Preset::Week,Preset::Month,Preset::Year,Preset::All] {
		for cursor in [i64::MIN,START,STOP-86400,STOP,i64::MAX] {
			let period=p.period(cursor);
			assert!(period.start>=START && period.end<=STOP && period.start<period.end);
		}
	}
	assert_eq!(Preset::All.period(cursor),Period {start:START,end:STOP});
}

#[test]
fn dates_validate_and_calendar_steps_clamp_month_ends()
{
	assert_eq!(epoch("2027-01-01"), START);
	assert_eq!(epoch("2035-01-01"), STOP);
	assert!(Period::parse("2027-01-01", "2035-01-01").is_ok());
	for (a, b) in [
		("", "2030-01-01"),
		("2027-02-30", "2030-01-01"),
		("2026-12-31", "2030-01-01"),
		("2027-01-01", "2035-01-02"),
		("2030-01-01", "2030-01-01"),
		("2030-01-02", "2030-01-01"),
	]
	{
		assert!(Period::parse(a, b).is_err());
	}
	assert_eq!(
		Step::Month.shift(epoch("2028-01-31"), 1),
		Some(epoch("2028-02-29"))
	);
	assert_eq!(
		Step::Month.shift(epoch("2027-03-31"), -1),
		Some(epoch("2027-02-28"))
	);
	assert_eq!(
		Step::Month.shift(epoch("2027-12-31"), 1),
		Some(epoch("2028-01-31"))
	);
	assert_eq!(
		Step::Month.shift(epoch("2027-02-01") + 43200, 1),
		Some(epoch("2027-03-01") + 43200)
	);
	assert_eq!(Step::Day.shift(START, -1), Some(START - 86400));
	assert_eq!(Step::Hour.shift(START, 1), Some(START + 3600));
	assert_eq!(Step::Hour.shift(i64::MAX, 1), None);
	assert_eq!(Step::Month.shift(i64::MAX, 1), None);
	assert_eq!(Step::Day.shift(START, 0), None);
}

#[test]
fn repeat_delays_then_runs_at_ten_hz_without_an_extra_release_step()
{
	let mut repeat = Repeat::default();
	assert!(!repeat.tick(true, false, 0.0));
	assert!(!repeat.tick(true, false, 0.399));
	let count = (400..1400)
		.map(|ms| {
			let down = repeat.held(ms < 800, true);
			repeat.tick(down, false, f64::from(ms) / 1000.0)
		})
		.filter(|&v| v)
		.count();
	assert_eq!(count, 10);
	assert!(!repeat.tick(false, true, 1.4));
	assert!(!repeat.tick(true, false, 2.0));
	assert!(repeat.tick(false, true, 2.1));
	assert!(!repeat.tick(true, false, 3.0));
	assert!(!repeat.tick(false, false, 3.1));
	assert!(!repeat.tick(false, false, 4.0));
}

#[test]
fn interpolation_takes_short_path_across_north_and_never_extrapolates()
{
	let track = Track {
		t: vec![0, 3600],
		az_deg: vec![359.0, 1.0],
		el_deg: vec![0.0, 0.0],
	};
	let [a, e] = sample(&track, 1800).unwrap();
	assert!(a.min(360.0 - a).abs() < 1e-5);
	assert_eq!(e, 0.0);
	assert_eq!(sample(&track, 0), Some([359.0, 0.0]));
	assert_eq!(sample(&track, 3600), Some([1.0, 0.0]));
	assert_eq!(sample(&track, -1), None);
	assert_eq!(sample(&track, 3601), None);
	assert_eq!(sample(&Track::default(), 0), None);
	let malformed = Track {
		t: vec![0, 3600],
		az_deg: vec![],
		el_deg: vec![0.0],
	};
	assert_eq!(sample(&malformed, 1800), None);
}

#[test]
fn samples_include_exact_end_and_month_lengths()
{
	let p = Period::parse("2027-01-01", "2027-03-15").unwrap();
	assert_eq!(
		p.samples(Step::Month),
		[
			epoch("2027-01-01"),
			epoch("2027-02-01"),
			epoch("2027-03-01"),
			epoch("2027-03-15")
		]
	);
	assert!(!p.covered(&[]));
	assert!(!p.covered(&[p.start]));
	assert!(p.covered(&[p.start, p.end]));
	let p = Period {
		start: 0,
		end: 90000,
	};
	assert_eq!(p.samples(Step::Day), [0, 86400, 90000]);
}

#[test]
fn statistics_use_selected_period_step_and_actual_elapsed_durations()
{
	let data=SiteData::parse(&serde_json::to_vec(&serde_json::json!({
		"site":{"name":"test","lat_deg":0.0,"lon_deg":0.0},
		"sun":{"t":[0,3600,86400,90000],"az_deg":[0.0,0.0,0.0,0.0],"el_deg":[-1.0,1.0,-1.0,-1.0]},
		"earth":{"t":[0,3600,86400,90000],"az_deg":[0.0,0.0,0.0,0.0],"el_deg":[0.75,0.75,0.75,0.75]},"horizon":null
	})).unwrap()).unwrap();
	let options = Options {
		period: Period {
			start: 0,
			end: 86400,
		},
		step: Step::Day,
	};
	let daily = summary(&data, options, 0.5).unwrap();
	assert_eq!(daily.pct_sun, 0.0);
	assert_eq!(daily.longest_night_h, 24.0);
	assert_eq!(daily.pct_earth, 100.0);
	let selected = Options {
		period: Period {
			start: 3600,
			end: 86400,
		},
		step: Step::Day,
	};
	assert_eq!(summary(&data, selected, 0.5).unwrap().pct_sun, 100.0);
	assert_eq!(summary(&data, selected, 1.0).unwrap().pct_both, 0.0);
	let partial = Options {
		period: Period {
			start: 0,
			end: 90000,
		},
		step: Step::Day,
	};
	assert_eq!(summary(&data, partial, 0.5).unwrap().longest_night_h, 25.0);
	let outside = Options {
		period: Period {
			start: 0,
			end: 90001,
		},
		step: Step::Day,
	};
	assert!(summary(&data, outside, 0.5).is_none());
	assert!(
		summary(
			&data,
			Options {
				period: Period { start: 0, end: 0 },
				step: Step::Hour
			},
			0.5
		)
		.is_none()
	);
}
