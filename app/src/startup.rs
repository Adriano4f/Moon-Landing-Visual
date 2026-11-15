use lunar_core::ephemeris::Ephemeris;

/// Decode one validated initial epoch without fetching the complete eight-year series.
pub(super) fn directions(bytes: &[u8]) -> Result<[[f32; 3]; 2], String>
{
	let initial = Ephemeris::decode(bytes)?;
	if initial.start_utc != 1798761600 || initial.step_s != 3600 || initial.samples.len() != 1
	{
		return Err("Initial sky must contain only 2027-01-01 00:00 UTC".into());
	}
	Ok(initial.samples[0].map(|body| body.direction))
}

#[cfg(test)]
mod tests
{
	use super::*;

	#[test]
	fn initial_record_matches_complete_series_and_rejects_bad_data()
	{
		let bytes = include_bytes!("../assets/data/moon-ephemeris.initial.bin");
		assert_eq!(bytes.len(), 56);
		let full = Ephemeris::decode(include_bytes!("../assets/data/moon-ephemeris.bin")).unwrap();
		assert_eq!(
			directions(bytes).unwrap(),
			full.samples[0].map(|body| body.direction)
		);
		assert!(directions(&[]).is_err());
		assert!(directions(include_bytes!("../assets/data/moon-ephemeris.bin")).is_err());
		let mut wrong_epoch = *bytes;
		wrong_epoch[8..16].copy_from_slice(&(full.start_utc + 3600).to_le_bytes());
		assert!(directions(&wrong_epoch).is_err());
		let mut corrupt = *bytes;
		corrupt[24..28].copy_from_slice(&f32::NAN.to_le_bytes());
		assert!(directions(&corrupt).is_err());
	}
}
