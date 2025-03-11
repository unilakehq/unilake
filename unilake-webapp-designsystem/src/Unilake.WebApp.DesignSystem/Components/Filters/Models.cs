namespace Unilake.WebApp.DesignSystem.Components;

public class FilterSettings
{
    public string PlaceholderText { get; set; }
    public required string[] SortOptions { get; set; }
    public string? CurrentSortOption { get; set; }
    public IEnumerable<FilterOption> SelectedItems { get; set; }
}

public class FilterInstance
{
    public string LabelText { get; set; }
    public IQueryable<FilterOption> Items { get; set; }
}

public class FilterOption
{
    public string LabelText { get; set; }
    public bool IsSelected { get; set; }
    public string Value { get; set; }
    public IQueryable<FilterOption>? Children { get; set; }
    public string ColorOption { get; set; }
}