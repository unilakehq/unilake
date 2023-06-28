namespace Unilake.Worker.Models.Dbt;

public class Analysis : Node
{
    public Analysis(string label, string key, string url) : base(label, key, url)
    {
        DisplayInModelTree = false;
    }
}