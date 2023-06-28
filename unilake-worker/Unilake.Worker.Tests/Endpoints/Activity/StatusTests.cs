using FakeItEasy;
using FastEndpoints;
using FluentAssertions;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Contracts.Responses.Activity;
using Unilake.Worker.Endpoints.Activity;
using Unilake.Worker.Mappers.Activity;
using Unilake.Worker.Models.Activity;
using Unilake.Worker.Services.Activity;

namespace Unilake.Worker.Tests.Endpoints.Activity;

[TestClass]
public class StatusTests
{
    [TestMethod]
    public async Task StatusTests_ReturnsCorrectResult()
    {
        // Arrange
        IActivityTracker tracker = A.Fake<IActivityTracker>();
        var endpoint = Factory.Create<Status>(tracker);
        endpoint.Map = new ActivityStatusResponseMapper();
        var input = new ActivityStatus
        {
            InstanceState = InstanceState.Running,
            ShutdownTimeoutInSeconds = 1231,
            TimeLeftInSeconds = 1211,
            FirstActivityUnixTimestampUtc = 123111,
            LastActivityUnixTimestampUtc = 123110
        };
        A.CallTo(() => tracker.GetStatus()).Returns(input);
        
        // Act
        await endpoint.HandleAsync(CancellationToken.None);
        var result = endpoint.Response;
        
        // Assert
        A.CallTo(() => tracker.GetStatus()).MustHaveHappened();
        Assert.IsNotNull(result);
        Assert.IsInstanceOfType(result, typeof(ActivityStatusResponse));
        result.InstanceState.Should().Be(input.InstanceState.ToString());
        result.ShutdownTimeoutInSeconds.Should().Be(input.ShutdownTimeoutInSeconds);
        result.TimeLeftInSeconds.Should().Be(input.TimeLeftInSeconds);
        result.FirstActivityUnixTimestampUtc.Should().Be(input.FirstActivityUnixTimestampUtc);
        result.LastActivityUnixTimestampUtc.Should().Be(input.LastActivityUnixTimestampUtc);
    }
}