//
// Original source:
// MIT License
// Copyright (c) 2020 Joakim Dangården
//

namespace Unilake.WebApp.DesignSystem.Components.Standard;

public class DialogOptions
{
    public required string MainText { get; init; }
    public IIcon IconType { get; init; } = AnronIcons.AlertBadge;
    public string CancelText { get; init; } = "Cancel";
    public string OkText { get; init; } = "Ok";
}