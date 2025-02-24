namespace Unilake.WebApp.DesignSystem.Components;

public class OffcanvasResult
{
    public object Data { get; }
    public Type DataType { get; }
    public bool Cancelled { get; }

    internal OffcanvasResult(object data, Type resultType, bool cancelled)
    {
        Data = data;
        DataType = resultType;
        Cancelled = cancelled;
    }

    public static ModalResult Ok() => new(null!, typeof(object), false);
    public static ModalResult Ok<T>(T result) => new(result!, typeof(T), false);
    public static ModalResult Cancel() => new(null!, typeof(object), true);
}