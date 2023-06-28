using FakeItEasy;
using FastEndpoints;
using Microsoft.VisualStudio.TestTools.UnitTesting;
using Unilake.Worker.Contracts.Requests.Activity;
using Unilake.Worker.Endpoints.Activity;
using Unilake.Worker.Mappers.Activity;
using Unilake.Worker.Models.Activity;
using Unilake.Worker.Services.Activity;

namespace Unilake.Worker.Tests.Endpoints.Activity;

[TestClass]
public class UpdateTimeoutTests
{
    [TestMethod]
    public async Task UpdateTimeoutTests_TimeoutAdjustmentMade()
    {
        // Arrange
        IActivityTracker tracker = A.Fake<IActivityTracker>();
        var endpoint = Factory.Create<UpdateShutdownTime>(tracker);
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
        await endpoint.HandleAsync(new UpdateShutdownTimeRequest
        {
            AdaptTimeout = 1
        }, CancellationToken.None);
        
        // Assert
        A.CallTo(() => tracker.AdjustTimeout(A<long>.That.Matches((i) => i == 1))).MustHaveHappenedOnceExactly();
    }
}