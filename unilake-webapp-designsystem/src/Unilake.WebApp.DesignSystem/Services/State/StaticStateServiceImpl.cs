using Unilake.WebApp.DesignSystem.Components;

namespace Unilake.WebApp.DesignSystem.Services.State;

public class StaticStateServiceImpl : StateService
{
    public StaticStateServiceImpl()
    {
        StateHandler.SetInitialState(StateKey.DarkMode, true);
        StateHandler.SetInitialState(StateKey.SideNavCollapsed, false);
        StateHandler.SetInitialState(StateKey.NavigationMenu, NavigationMenu);
        // todo: see how we are going to handle this and how we adjust it
        StateHandler.SetInitialState(StateKey.Culture, "en-UK");
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
                    Label = "Develop",
                    Uri = "/workspace/datascience/develop"
                },
                new()
                {
                    Label = "Compute"
                }
            ]
        }
    ];

    public override Task InitializeAsync()
    {
        Console.WriteLine("Static state service initialized. No need to connect to any external services.");
        return Task.CompletedTask;
    }
}