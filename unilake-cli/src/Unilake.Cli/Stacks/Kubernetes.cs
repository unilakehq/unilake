using Pulumi;
using Pulumi.Automation;
using Unilake.Cli.Config;

namespace Unilake.Cli;

public class Kubernetes : UnilakeStack
{
    public Kubernetes(EnvironmentConfig config) : base(config)
    {
    }

    public override (string name, string version)[] Packages => 
        new [] {("", "")};

    public override Task Create()
    {
        throw new NotImplementedException();
    }
}
