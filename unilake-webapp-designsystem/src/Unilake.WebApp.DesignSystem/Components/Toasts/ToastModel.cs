//
// Original source: https://github.com/TabBlazor/TabBlazor/blob/master/src/TabBlazor/Components/Toasts/ToastModel.cs
// MIT License
// Copyright (c) 2020 Joakim Dangården
//
using Microsoft.AspNetCore.Components;

namespace Unilake.WebApp.DesignSystem.Components;

public class ToastModel
{
    public ToastModel()
    {
    }

    public ToastModel(string message, ToastOptions? options = null)
    {
        Message = message;
        Options = options ?? new ToastOptions();
    }

    public ToastModel(RenderFragment contents, ToastOptions? options = null)
    {
        Contents = contents;
        Variant = Toast.ToastVariant.Interactive;
        Options = options ?? new ToastOptions();
    }

    public string Message { get; set; } = string.Empty;
    public string LeftIconTextColor { get; set; } = "text-blue-600";
    public string LeftIconBgColor { get; set; } = "bg-brand-bravo-50";
    public IIcon LeftIcon { get; set; } = EnronIcons.Send;
    public Toast.ToastVariant Variant { get; set; } = Toast.ToastVariant.Default;
    public ToastOptions Options { get; set; } = new();
    public RenderFragment? Contents { get; internal set; }
    public string Css { get; set; } = string.Empty;
}