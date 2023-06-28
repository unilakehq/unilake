namespace Unilake.Worker.Models.Dbt;

public class Seed : Node
{
    public Seed(string label, string key, string url) : base(label, key, url) { }
}