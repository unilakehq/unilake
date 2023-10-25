namespace Unilake.Cli.Config;

public class Unilake
{
    public Webapp? Webapp { get; set; }
    public Webapi? Webapi { get; set; }
    public ProxyQuery? ProxyQuery { get; set; }
    public ProxyStorage? ProxyStorage { get; set; }
}