using Unilake.Worker.Services.Dbt;

namespace Unilake.Worker.Events.Dbt;

public abstract class DbtTaskEvent : ServiceTaskEvent<IDbtService>
{
}