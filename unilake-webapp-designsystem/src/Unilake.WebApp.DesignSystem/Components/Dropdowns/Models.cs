namespace Unilake.WebApp.DesignSystem.Components;

public record DropdownItem(string Label, IIcon LeftIcon);
public record DropdownFilterItem(string Label, int Count);
public record DropdownSearchCategory(string Label, List<DropdownSearchItem> Items, string Color = "");
public record DropdownSearchItem(string Label);