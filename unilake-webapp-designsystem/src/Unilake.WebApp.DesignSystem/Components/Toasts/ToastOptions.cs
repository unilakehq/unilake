//
// Original source: https://github.com/TabBlazor/TabBlazor/blob/master/src/TabBlazor/Components/Toasts/ToastOptions.cs
// MIT License
// Copyright (c) 2020 Joakim Dangården
//
namespace Unilake.WebApp.DesignSystem;

public class ToastOptions
{
    /// <summary>
    /// Delay in Seconds
    /// Set 0 to show it until manually removed
    /// </summary>
    public int Delay { get; set; } = 3;
    public bool AutoClose => Delay > 0;
    public ToastPosition Position { get; set; } = ToastPosition.TopEnd;
}

public enum ToastPosition
{
    TopEnd,
    TopStart,
    BottomEnd,
    BottomStart
}