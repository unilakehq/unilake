namespace Unilake.Cli.Config;

public class Postgresql
{
    public bool Enabled { get; set; }
    public string Username { get; set; }
    public string Password { get; set; }
    public string Host { get; set; }
    public int? Port { get; set; }
    public string Name { get; set; }
    public string Schema { get; set; }
}