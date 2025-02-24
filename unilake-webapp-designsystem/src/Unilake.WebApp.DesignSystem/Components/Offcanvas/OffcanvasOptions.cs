namespace Unilake.WebApp.DesignSystem.Components;

public class OffcanvasOptions
{
    public IIcon? TitleIcon { get; set; }
    public bool Backdrop { get; set; } = true;
    public bool CloseOnClickOutside { get; set; } = false;
    public string WrapperCssClass { get; set; } = string.Empty;
    public OffcanvasPosition Position { get; set; }
    public bool CloseOnEsc { get; set; } = false;
}

public enum OffcanvasPosition
{
    Start = 0,
    End = 1,
    Top = 2,
    Bottom = 3
}