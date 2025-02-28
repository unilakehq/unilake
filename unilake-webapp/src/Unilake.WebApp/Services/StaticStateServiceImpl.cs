using Unilake.WebApp.DesignSystem.Components;

namespace Unilake.WebApp.Services;

public class StaticStateServiceImpl : StateService
{
    public StaticStateServiceImpl()
    {
        StateHandler.SetInitialState(State.DarkMode, false);
        StateHandler.SetInitialState(State.SideNavCollapsed, false);
        StateHandler.SetInitialState(State.NavigationMenu, NavigationMenu);
    }

    public IEnumerable<NavItem> NavigationMenu { get; } =
    [
        new()
        {
            Label = "Dashboard",
            LeftIcon = EnronIcons.BookOpen,
            Uri = "/weather",
        },
        new()
        {
            Label = "Jobs",
            LeftIcon = EnronIcons.Dashboard4,
            Uri = "/counter",
        },
        new()
        {
            Label = "Catalog",
            LeftIcon = EnronIcons.BookOpen,
            Children =
            [
                new()
                {
                    Label = "Explore",
                },
                new()
                {
                    Label = "Chat",
                    RightIcon = EnronIcons.Ai
                },
                new()
                {
                    Label = "Classifications",
                },
                new()
                {
                    Label = "Data Products",
                },
                new()
                {
                    Label = "Security",
                },
            ]
        },
        new()
        {
            Label = "Integration",
            LeftIcon = EnronIcons.Gear3,
            Uri = "/pipelines"
        },
        new()
        {
            Label = "SQL Warehouse",
            LeftIcon = EnronIcons.Server2,
            Children =
            [
                new()
                {
                    Label = "SQL Workbench",
                },
                new()
                {
                    Label = "Saved Queries"
                },
                new()
                {
                    Label = "History",
                },
                new()
                {
                    Label = "Compute"
                }
            ]
        },
        new()
        {
            Label = "Data Science",
            LeftIcon = EnronIcons.Eye,
            Children =
            [
                new()
                {
                    Label = "Develop"
                },
                new()
                {
                    Label = "Compute"
                }
            ]
        }
    ];
}