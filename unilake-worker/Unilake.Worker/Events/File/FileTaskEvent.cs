using Unilake.Worker.Services.File;

namespace Unilake.Worker.Events.File;

public abstract class FileTaskEvent : ServiceTaskEvent<IFileService>
{
}