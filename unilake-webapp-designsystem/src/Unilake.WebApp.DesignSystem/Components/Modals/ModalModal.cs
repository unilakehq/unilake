//
// Original source: https://github.com/TabBlazor/TabBlazor/blob/master/src/TabBlazor/Components/Modals/ModalModel.cs
// MIT License
// Copyright (c) 2020 Joakim Dangården
//

using Microsoft.AspNetCore.Components;

namespace Unilake.WebApp.DesignSystem.Components;

public class ModalModel(RenderFragment contents, string title, ModalOptions? options)
{
    internal TaskCompletionSource<ModalResult> TaskSource { get; } = new();
    public Task<ModalResult> Task => TaskSource.Task;
    public string Title { get; set; } = title;
    public RenderFragment ModalContents { get; private set; } = contents;
    public ModalOptions Options { get; } = options ?? new ModalOptions();
}