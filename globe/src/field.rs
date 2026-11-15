// Sample equal-area Fibonacci directions on the sunlit hemisphere; C++ vector<array<float,3>>.
pub(crate) fn illuminated_points(sun: [f32; 3]) -> Vec<[f32; 3]>
{
	// Reject absent/invalid directions instead of inventing a light; C++ finite-vector guard.
	if sun.iter().any(|v| !v.is_finite()) || sun.iter().map(|v| v * v).sum::<f32>() < 1e-12
	{
		return Vec::new();
	}
	// Half-offset latitudes avoid singular pole samples; C++ indexed loop with a golden-angle longitude.
	(0..512)
		.filter_map(|i| {
			let y = 1.0 - 2.0 * (i as f32 + 0.5) / 512.0;
			let r = (1.0 - y * y).sqrt();
			let a = i as f32 * 2.3999631;
			let p = [r * a.cos(), y, r * a.sin()];
			(p.iter().zip(sun).map(|(p, s)| p * s).sum::<f32>() > 0.0).then_some(p)
		})
		.collect()
}

#[cfg(test)]
mod tests
{
	use super::*;
	#[test]
	fn field_is_finite_even_and_sunlit()
	{
		for sun in [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]]
		{
			let points = illuminated_points(sun);
			assert!((250..=262).contains(&points.len()));
			for p in points
			{
				assert!((p.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 1e-5);
				assert!(p.iter().zip(sun).map(|(v, s)| v * s).sum::<f32>() > 0.0);
			}
		}
		assert!(illuminated_points([0.0; 3]).is_empty());
		assert!(illuminated_points([f32::NAN, 0.0, 0.0]).is_empty());
	}
}
