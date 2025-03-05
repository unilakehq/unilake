namespace Unilake.WebApp.DesignSystem.Components;

public enum DropdownHorizontalAlign
{
    Left,
    Right
}

public record DropdownItem(string Label, IIcon? LeftIcon);

public class DropdownFilterItem
{
    private bool? _isSelected = false;
    public string Label { get; }
    public string Value { get; }
    public int Count { get; }

    public bool? IsSelected
    {
        get => IsCategory
            ? Siblings.All(x => x._isSelected.GetValueOrDefault()) ? true :
            Siblings.All(x => !x._isSelected.GetValueOrDefault()) ? false : null
            : _isSelected.GetValueOrDefault();
        private set => _isSelected = value;
    }

    public List<DropdownFilterItem> Siblings { get; } = new();
    public string CategoryColor { get; }
    public bool IsCategory { get; }
    public bool IsCategoryOpen { get; set; }

    public static DropdownFilterItem CreateCategory(string label, string value, List<DropdownFilterItem> siblings,
        string categoryColor, bool isSelected = false) =>
        new(label, value, siblings, isSelected, true, categoryColor);

    public static DropdownFilterItem CreateFilterItem(string label, string value, int count = 0) =>
        new(label, value, count);

    public DropdownFilterItem(string label, string value, int count = 0, bool isSelected = false)
    {
        Label = label;
        Value = value;
        Count = count;
        IsSelected = isSelected;
        IsCategory = false;
        CategoryColor = String.Empty;
    }

    public DropdownFilterItem(string label, string value, List<DropdownFilterItem> siblings, bool isSelected = false,
        bool isCategory = false, string categoryColor = "")
    {
        Label = label;
        Value = value;
        Count = 0;
        IsSelected = isSelected;
        Siblings = siblings;
        IsCategory = isCategory;
        CategoryColor = categoryColor;
    }

    public void AddSibling(DropdownFilterItem sibling)
    {
        Siblings.Add(sibling);
    }

    public void ToggleSelection(int maxSelectionCount = int.MaxValue)
    {
        IsSelected = !IsSelected;
        int currentCount = 0;
        foreach (var sibling in Siblings)
        {
            sibling.IsSelected = _isSelected;
            currentCount += sibling.IsSelected.GetValueOrDefault() ? 1 : 0;
            if (currentCount > maxSelectionCount)
                sibling.IsSelected = false;
        }
    }

    public void Clear()
    {
        IsSelected = false;
        foreach (var sibling in Siblings)
            sibling.IsSelected = false;
    }
}