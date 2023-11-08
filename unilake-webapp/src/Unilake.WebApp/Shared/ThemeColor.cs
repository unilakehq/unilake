﻿namespace Unilake.WebApp.Shared;

/// <summary>
/// Bootstrap theme colors <see href="https://getbootstrap.com/docs/5.2/customize/color/#theme-colors">https://getbootstrap.com/docs/5.2/customize/color/#theme-colors</see>
/// (+ Link from predefined button styles <see href="https://getbootstrap.com/docs/5.2/components/buttons/">https://getbootstrap.com/docs/5.2/components/buttons/</see>)
/// </summary>
public enum ThemeColor
{
	None = 0,
	Primary,
	Secondary,
	Success,
	Danger,
	Warning,
	Info,
	Light,
	Dark,
	/// <summary>
	/// To be used for buttons only.
	/// </summary>
	Link
}
