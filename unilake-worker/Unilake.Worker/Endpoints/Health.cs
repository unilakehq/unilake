using System.Diagnostics;
using System.Net.NetworkInformation;
using Flurl.Http;
using Microsoft.Extensions.Diagnostics.HealthChecks;
using Prometheus;
using Unilake.Worker.Contracts.Responses;

namespace Unilake.Worker.Endpoints;

public class Health : EndpointWithoutRequest<HealthResponse>
{
    private readonly ILogger _logger;
    private readonly IConfiguration _configuration;
    private readonly Gauge _checkStatus = Metrics.CreateGauge(
        WorkerMetrics.AspnetcoreHealthcheckStatus,
        WorkerMetrics.AspnetcoreHealthcheckStatusDesc,
        labelNames: new[] { "name" }
        );

    public Health(ILogger<Health> logger, IConfiguration configuration)
    {
        _logger = logger;
        _configuration = configuration;
    }

    public override void Configure()
    {
        Get("/health");
        AllowAnonymous();
    }

    public override async Task HandleAsync(CancellationToken ct)
    {
        var ts = Stopwatch.StartNew();
        var checks = new List<IndividualHealthResponse>
        {
            CheckPublicConnectivity(),
            await CheckOrchestratorConnectivity(),
        };

        foreach (var check in checks)
            _checkStatus.WithLabels(check.Component).Set(check.Status == HealthStatus.Healthy ? 1 : 0);

        var status = checks.Any(x => x.Status != HealthStatus.Healthy) ? HealthStatus.Unhealthy : HealthStatus.Healthy;
        await SendAsync(new HealthResponse
        {
            HealthChecks = checks,
            Status = status,
            HealthCheckDuration = ts.Elapsed
        }, status == HealthStatus.Healthy ? 200 : 503, ct);
    }

    private async Task<IndividualHealthResponse> CheckOrchestratorConnectivity()
    {
        string endpoint = _configuration.GetValue<string>("Environment:OrchestratorEndpoint");
        string apiKey = _configuration.GetValue<string>("Environment:OrchestratorApiKey");
        try
        {
            var response = await endpoint.WithHeader("X-Api-Key", apiKey).GetAsync();
            if (response.StatusCode == 200)
            {
                return new IndividualHealthResponse
                {
                    Status = HealthStatus.Healthy,
                    Component = "OrchestratorConnectivity",
                    Description = $"Can connect to Orchestrator at {endpoint}"
                };
            }
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Failed: CheckOrchestratorConnectivity");
        }

        return new IndividualHealthResponse
        {
            Status = HealthStatus.Unhealthy,
            Component = "OrchestratorConnectivity",
            Description = $"Cannot connect to Orchestrator at {endpoint}"
        };
    }

    private IndividualHealthResponse CheckPublicConnectivity()
    {
        string endpoint = _configuration.GetValue<string>("HealthChecks:PublicEndpoint");
        try
        {
            Ping ping = new Ping();
            PingReply reply = ping.Send(endpoint);

            if (reply?.Status == IPStatus.Success)
            {
                return new IndividualHealthResponse
                {
                    Status = HealthStatus.Healthy,
                    Component = "PublicConnectivity",
                    Description = $"Can ping {endpoint}"
                };
            }
        }
        catch (Exception e)
        {
            _logger.LogError(e, "Failed: CheckPublicConnectivity");
        }

        return new IndividualHealthResponse
        {
            Status = HealthStatus.Unhealthy,
            Component = "PublicConnectivity",
            Description = $"Cannot ping {endpoint}"
        };
    }
}