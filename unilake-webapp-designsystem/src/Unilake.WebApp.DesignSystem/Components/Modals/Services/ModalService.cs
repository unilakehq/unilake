//
// Original source: https://github.com/TabBlazor/TabBlazor/blob/master/src/TabBlazor/Components/Modals/Services/ModalService.cs
// MIT License
// Copyright (c) 2020 Joakim Dangården
//

using Microsoft.AspNetCore.Components;
using Microsoft.AspNetCore.Components.Routing;
using Unilake.WebApp.DesignSystem.Components;
using Unilake.WebApp.DesignSystem.Components.Standard;

namespace Unilake.WebApp.DesignSystem.Services;

public class ModalService : IModalService, IDisposable
{
        public ModalService(NavigationManager navigationManager)
        {
            _navigationManager = navigationManager;
            _navigationManager.LocationChanged += LocationChanged;
        }

        private int _zIndex = 1200;
        private const int ZIndexIncrement = 10;
        private int _topOffset;
        private const int TopOffsetIncrement = 20;

        public event Action? OnChanged;
        private readonly Stack<ModalModel> _modals = new();
        private ModalModel _modalModel;
        private readonly NavigationManager _navigationManager;

        public IEnumerable<ModalModel> Modals => _modals;

        public Task<ModalResult> ShowAsync<TComponent>(string title, RenderComponent<TComponent> component, ModalOptions? modalOptions = null) where TComponent : IComponent
        {
            _modalModel = new ModalModel(component.Contents, title, modalOptions);
            _modals.Push(_modalModel);
            OnChanged?.Invoke();
            return _modalModel.Task;
        }

        public async Task<bool> ShowDialogAsync(DialogOptions options)
        {
            var component = new RenderComponent<DialogModal>().
                Set(e=> e.Options, options);
            var result = await ShowAsync("", component, new ModalOptions
            {
                ModalBodyCssClass="p-0",
                Backdrop = true,
                CloseOnEsc = false,
                CloseOnClickOutside = false,
            });
            return !result.Cancelled;
        }

        private void LocationChanged(object sender, LocationChangedEventArgs e) =>
            CloseAll();

        private void CloseAll()
        {
            foreach (var x in _modals.ToList())
                Close();
        }

        public void Close(ModalResult modalResult)
        {
            if (_modals.Count != 0)
            {
                var modalToClose = _modals.Pop();
                modalToClose.TaskSource.SetResult(modalResult);
            }

            OnChanged?.Invoke();
        }

        public void Close()
        {
            Close(ModalResult.Cancel());
        }

        public void Dispose()
        {
            _navigationManager.LocationChanged -= LocationChanged;
        }

        public void UpdateTitle(string title)
        {
            var modal = Modals.LastOrDefault();
            if (modal == null) return;
            modal.Title = title;
            OnChanged?.Invoke();
        }

        public void Refresh()
        {
            var modal = Modals.LastOrDefault();
            if (modal != null)
                OnChanged?.Invoke();
        }

        public ModalViewSettings RegisterModalView(ModalView modalView)
        {
            var settings = new ModalViewSettings { TopOffset = _topOffset, ZIndex = _zIndex };
            _zIndex += ZIndexIncrement;
            _topOffset += TopOffsetIncrement;

            return settings;
        }

        public void UnRegisterModalView(ModalView modalView)
        {
            _zIndex -= ZIndexIncrement;
            _topOffset -= TopOffsetIncrement;
        }
}