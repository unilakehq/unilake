using FakeItEasy;
using FastEndpoints;
using FluentAssertions;
using Flurl.Http.Testing;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Diagnostics.HealthChecks;
using Microsoft.Extensions.Logging;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Endpoints;

namespace Unilake.Worker.Tests.Endpoints;

[TestClass]
public class HealthTests
{

    private Health Setup(string publicEndpoint = "")
    {
        var inMemorySettings = new Dictionary<string, string> {
            {"Environment:OrchestratorEndpoint", "http://orchestrator"},
            {"HealthChecks:PublicEndpoint", publicEndpoint},
        };

        IConfiguration configuration = new ConfigurationBuilder()
            .AddInMemoryCollection(inMemorySettings)
            .Build();

        var logger = A.Fake<ILogger<Health>>();

        return Factory.Create<Health>(logger, configuration);
    }

    [TestMethod]
    // NOTE: this test can fail if ping utilities are not installed
    public async Task HealthChecks_Success()
    {
        // arrange
        var endpoint = Setup("127.0.0.1");

        // act
        using var httpTest = new HttpTest();
        await endpoint.HandleAsync(default);
        var response = endpoint.Response;

        // assert
        response.Should().NotBeNull();
        response.Status.Should().Be(HealthStatus.Healthy);
        response.HealthChecks.Should().AllSatisfy(x => x.Status.Should().Be(HealthStatus.Healthy));
        response.HealthChecks.Should().AllSatisfy(x => x.Component.Should().NotBeEmpty());
        response.HealthChecks.Should().AllSatisfy(x => x.Description.Should().NotBeEmpty());
        response.HealthChecks.Should().OnlyHaveUniqueItems();
        endpoint.HttpContext.Response.StatusCode.Should().Be(200);
    }

    [TestMethod]
    public async Task CheckPublicConnectivity_Failed()
    {
        // arrange
        string component = "PublicConnectivity";
        var endpoint = Setup("0.0.0.0");

        // act
        using var httpTest = new HttpTest();
        await endpoint.HandleAsync(default);
        var response = endpoint.Response;

        // assert
        response.Should().NotBeNull();
        response.Status.Should().Be(HealthStatus.Unhealthy);
        response.HealthChecks.Should().ContainSingle(x => x.Component == component);
        response.HealthChecks.First(x => x.Component == component).Status.Should().Be(HealthStatus.Unhealthy);
        endpoint.HttpContext.Response.StatusCode.Should().Be(503);
    }

    [TestMethod]
    public async Task CheckOrchestratorConnectivity_Failed()
    {
        // arrange
        string component = "OrchestratorConnectivity";
        var endpoint = Setup("127.0.0.1");

        // act
        using var httpTest = new HttpTest();
        httpTest.RespondWith("", status: 500);
        await endpoint.HandleAsync(default);
        var response = endpoint.Response;

        // assert
        response.Should().NotBeNull();
        response.Status.Should().Be(HealthStatus.Unhealthy);
        response.HealthChecks.Should().ContainSingle(x => x.Component == component);
        response.HealthChecks.First(x => x.Component == component).Status.Should().Be(HealthStatus.Unhealthy);
        endpoint.HttpContext.Response.StatusCode.Should().Be(503);
    }

}