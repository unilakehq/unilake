using Unilake.WebApp.DesignSystem.Components;

namespace Unilake.WebApp.Services;

public class StaticStateServiceImpl : StateService
{
    public StaticStateServiceImpl()
    {
        StateHandler.SetInitialState(State.DarkMode, true);
        StateHandler.SetInitialState(State.SideNavCollapsed, false);
        StateHandler.SetInitialState(State.NavigationMenu, NavigationMenu);
        // todo: see how we are going to handle this and how we adjust it
        StateHandler.SetInitialState(State.Culture, "en-UK");
    }

    public IEnumerable<NavItem> NavigationMenu { get; } =
    [
        new()
        {
            Label = "Dashboard",
            LeftIcon = AnronIcons.BookOpen,
            Uri = "/weather",
        },
        new()
        {
            Label = "Jobs",
            LeftIcon = AnronIcons.Dashboard4,
            Uri = "/counter",
        },
        new()
        {
            Label = "Catalog",
            LeftIcon = AnronIcons.BookOpen,
            Children =
            [
                new()
                {
                    Label = "Explore",
                    Uri = "/catalog/pipelines"
                },
                new()
                {
                    Label = "Chat",
                    RightIcon = AnronIcons.Ai,
                    RightIconColor = "text-brand-light-interaction"
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
            LeftIcon = AnronIcons.Gear3,
        },
        new()
        {
            Label = "SQL Warehouse",
            LeftIcon = AnronIcons.Server2,
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
            LeftIcon = AnronIcons.Eye,
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