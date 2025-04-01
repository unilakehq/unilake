//
// Original source: https://github.com/TabBlazor/TabBlazor/blob/master/src/TabBlazor/Components/Toasts/Services/ToastService.cs
// MIT License
// Copyright (c) 2020 Joakim Dangården
//

namespace Unilake.WebApp.DesignSystem.Components.Toasts.Services;

public class ToastService
{
    private readonly List<ToastModel> _toasts = new();
    private readonly ReaderWriterLockSlim _listLock = new();
    public IEnumerable<ToastModel> Toasts => _toasts;

    public async Task AddToastAsync(ToastModel toast)
    {
        AddToast(toast);
        await UpdateAsync();
    }

    public async Task AddToastAsync(string message, ToastOptions? options = null)
    {
        var toast = new ToastModel(message, options);
        await AddToastAsync(toast);
    }

    private void AddToast(ToastModel toast)
    {
        try
        {
            _listLock.EnterWriteLock();
            _toasts.Add(toast);

        }
        finally
        {
            _listLock.ExitWriteLock();
        }
    }

    public async Task RemoveAllAsync()
    {
        _toasts.Clear();
        await UpdateAsync();
    }

    public async Task RemoveToastAsync(ToastModel toast)
    {
        try
        {
            _listLock.EnterWriteLock();
            if (_toasts.Contains(toast))
            {
                _toasts.Remove(toast);
            }
        }
        finally
        {
            _listLock.ExitWriteLock();
        }

        await UpdateAsync();
    }

    public async Task UpdateAsync()
    {
        if (OnChanged != null)
            await OnChanged.Invoke();
    }

    public event Func<Task>? OnChanged;
}