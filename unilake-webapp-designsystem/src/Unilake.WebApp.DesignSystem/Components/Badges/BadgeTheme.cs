namespace Unilake.WebApp.DesignSystem.Components;

public enum BadgeTheme
{
    Brand,
    Info,
    Error,
    Success,
    Warning,
    Neutral,
    UserInput,
    Filter,
    Custom
}

public static class BadgeUserInputTheme
{
    public static (string, string, string, string) FromUserInputName(string name) => name switch
    {
        _ => ("bg-feedback-brand-background", "border-feedback-info-background", "text-feedback-brand-contrast", "")
    };
    
    public static string GetBackgroundColor(string name) => FromUserInputName(name).Item1;
    public static string GetBorderColor(string name) => FromUserInputName(name).Item2;
    public static string GetTextColor(string name) => FromUserInputName(name).Item3;
    public static string GetTextInteraction(string name) => FromUserInputName(name).Item4;
}