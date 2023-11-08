﻿namespace Unilake.WebApp.Shared.Infrastructure;

/// <summary>
/// <see cref="ICascadeEnabledComponent"/> helper method.
/// </summary>
public static class CascadeEnabledComponent
{
	/// <summary>
	/// Effective value of Enabled. When Enabled is not set, receives value from FormState or defaults to true.
	/// </summary>
	public static bool EnabledEffective(ICascadeEnabledComponent component)
	{
		return component.Enabled ?? component.FormState?.Enabled ?? true;
	}
}
