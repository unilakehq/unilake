namespace Unilake.Worker.Models.Dbt;

public class Exposure : Node
{
    public Exposure(string label, string key, string url) : base(label, key, url)
    {
        DisplayInModelTree = false;
    }
}