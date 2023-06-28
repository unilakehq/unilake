using Microsoft.Extensions.Diagnostics.HealthChecks;

namespace Unilake.Worker.Contracts.Responses;

public class HealthResponse
{
    public HealthStatus Status { get; set; }
    public IEnumerable<IndividualHealthResponse> HealthChecks { get; set; }
    public TimeSpan HealthCheckDuration { get; set; }
}

public class IndividualHealthResponse
{
    public HealthStatus Status { get; set; }
    public string Component { get; set; }
    public string Description { get; set; }
}