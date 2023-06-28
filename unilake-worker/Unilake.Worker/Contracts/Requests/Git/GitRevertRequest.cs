namespace Unilake.Worker.Contracts.Requests.Git;

public class GitRevertRequest : AsyncRequestOption
{
    public string[] Files { get; set; } = Array.Empty<string>();
}