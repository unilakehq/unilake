//
// Original source: https://github.com/TabBlazor/TabBlazor/blob/master/src/TabBlazor/Components/Modals/Services/IModalService.cs
// MIT License
// Copyright (c) 2020 Joakim Dangården
//

using Microsoft.AspNetCore.Components;
using Unilake.WebApp.DesignSystem.Components;
using Unilake.WebApp.DesignSystem.Components.Standard;

namespace Unilake.WebApp.DesignSystem.Services;

public interface IModalService
{
    event Action OnChanged;
    IEnumerable<ModalModel> Modals { get; }
    Task<ModalResult> ShowAsync<TComponent>(string title, RenderComponent<TComponent> component, ModalOptions? modalOptions = null) where TComponent : IComponent;
    void Close(ModalResult modalResult);
    void Close();
    Task<bool> ShowDialogAsync(DialogOptions options);

    void UpdateTitle(string title);
    void Refresh();

    ModalViewSettings RegisterModalView(ModalView modalView);
    void UnRegisterModalView(ModalView modalView);
}
