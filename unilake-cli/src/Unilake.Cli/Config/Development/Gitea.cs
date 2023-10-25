namespace Unilake.Cli.Config;

public class Gitea
{
    public bool Enabled { get; set; }
    public string? AdminUsername { get; set; }
    public string? AdminPassword { get; set; }
    public string? AdminEmail { get; set; }
    public Postgresql? Postgresql { get; set; }
    public Redis? Redis { get; set; }
}