//
// Original source: https://github.com/TabBlazor/TabBlazor/blob/master/src/TabBlazor/Components/Modals/ModalResult.cs
// MIT License
// Copyright (c) 2020 Joakim Dangården
//

namespace Unilake.WebApp.DesignSystem.Components;

public class ModalResult
{
    public object Data { get; }
    public Type DataType { get; }
    public bool Cancelled { get; }

    internal ModalResult(object data, Type resultType, bool cancelled)
    {
        Data = data;
        DataType = resultType;
        Cancelled = cancelled;
    }

    public static ModalResult Ok() => new(default, typeof(object), false);
    public static ModalResult Ok<T>(T result) => new(result, typeof(T), false);

    public static ModalResult Cancel() => new(default, typeof(object), true);
}