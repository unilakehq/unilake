using FluentValidation.Results;
using Unilake.Worker.Models.Activity;
using Unilake.Worker.Services.Activity;

namespace Unilake.Worker.Processors.PreProcessor;

public class RequestActivityTracker<TRequest> : IPreProcessor<TRequest>
{
    public Task PreProcessAsync(TRequest req, HttpContext ctx, List<ValidationFailure> failures, CancellationToken ct)
    {
        var tracker = ctx.Resolve<IActivityTracker>();
        if (tracker.GetStatus().InstanceState == InstanceState.Running)
            tracker.TrackActivity();
        else
        {
            failures.Add(new ValidationFailure("InvalidState", "Worker is not running"));
            return ctx.Response.SendErrorsAsync(failures);
        }
        return Task.CompletedTask;
    }
}