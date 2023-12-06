namespace Unilake.Iac.Kubernetes.Deployment.Input;

public class UnilakeDocsInputArgs
{
    public required string Version { get; set; }
    public string? Url { get; internal set; }
}