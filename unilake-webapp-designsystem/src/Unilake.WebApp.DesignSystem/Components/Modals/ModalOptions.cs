namespace Unilake.WebApp.DesignSystem.Components;

public class ModalOptions
{
    public bool CloseOnClickOutside { get; init; }
    public bool Backdrop { get; init; } = true;
    public bool CloseOnEsc { get; init; }
    public string ModalCssClass { get; init; } = string.Empty;
    public string ModalBodyCssClass { get; init; } = string.Empty;
}