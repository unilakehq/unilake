using Unilake.Worker.Services.Git;

namespace Unilake.Worker.Events.Git;

public abstract class GitTaskEvent : ServiceTaskEvent<IGitService>
{
    
}